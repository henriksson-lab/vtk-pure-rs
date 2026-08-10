use crate::data::{CellArray, ImageData, Points, PolyData};
use crate::filters::core::marching_cubes::{EDGE_TABLE, TRI_TABLE};
use rayon::prelude::*;
use std::sync::OnceLock;

const FE_PAR_MIN_ROW_EDGES: usize = 20_000;

#[derive(Clone, Copy)]
struct UnsafeF64Writer {
    ptr: *mut f64,
}

unsafe impl Send for UnsafeF64Writer {}
unsafe impl Sync for UnsafeF64Writer {}

impl UnsafeF64Writer {
    fn new(slice: &mut [f64]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
        }
    }

    #[inline(always)]
    unsafe fn write(&self, idx: usize, value: f64) {
        *self.ptr.add(idx) = value;
    }
}

#[derive(Clone, Copy)]
struct UnsafeI64Writer {
    ptr: *mut i64,
}

unsafe impl Send for UnsafeI64Writer {}
unsafe impl Sync for UnsafeI64Writer {}

impl UnsafeI64Writer {
    fn new(slice: &mut [i64]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
        }
    }

    #[inline(always)]
    unsafe fn write(&self, idx: usize, value: i64) {
        *self.ptr.add(idx) = value;
    }
}

/// Flying Edges 3D — 4-pass algorithm with adaptive parallelism.
///
/// Uses rayon for parallel passes on large grids (>100K voxels),
/// falls back to serial for small grids to avoid thread overhead.
pub fn flying_edges_3d(image: &ImageData, scalars: &[f64], isovalue: f64) -> PolyData {
    let dims = image.dimensions();
    if dims[0] < 2 || dims[1] < 2 || dims[2] < 2 {
        return PolyData::new();
    }

    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let sp = image.spacing();
    let org = image.origin();
    let nxy = nx * ny;
    let nxm1 = nx - 1;
    let n_rows = ny * nz;
    let n_voxel_rows = (ny - 1) * (nz - 1);
    let case_table = fe_case_table();
    // Match VTK's preference for threaded image-data contouring once row work
    // is large enough to amortize rayon scheduling.
    let use_par = nxm1 * n_rows >= FE_PAR_MIN_ROW_EDGES;

    // ====== PASS 1: Classify x-edges + trim ranges ======
    let mut x_cases: Vec<u8> = vec![0; nxm1 * n_rows];
    let mut meta: Vec<[u32; 6]> = vec![[0, 0, 0, 0, nxm1 as u32, 0]; n_rows];

    let pass1_row = |row: usize, row_cases: &mut [u8], row_meta: &mut [u32; 6]| {
        let k = row / ny;
        let j = row % ny;
        let rs = k * nxy + j * nx;
        let mut s1 = scalars[rs];
        let mut x_count = 0u32;
        let mut tmin = nxm1 as u32;
        let mut tmax = 0u32;
        for i in 0..nxm1 {
            let s0 = s1;
            s1 = unsafe { *scalars.get_unchecked(rs + i + 1) };
            let mut ec = 0u8;
            if s0 >= isovalue {
                ec |= 1;
            }
            if s1 >= isovalue {
                ec |= 2;
            }
            row_cases[i] = ec;
            if ec == 1 || ec == 2 {
                x_count += 1;
                if (i as u32) < tmin {
                    tmin = i as u32;
                }
                tmax = (i + 1) as u32;
            }
        }
        row_meta[0] = x_count;
        row_meta[4] = tmin;
        row_meta[5] = tmax;
    };

    if use_par {
        x_cases
            .par_chunks_mut(nxm1)
            .zip(meta.par_iter_mut())
            .enumerate()
            .for_each(|(row, (rc, rm))| pass1_row(row, rc, rm));
    } else {
        for row in 0..n_rows {
            let rc = &mut x_cases[row * nxm1..(row + 1) * nxm1];
            pass1_row(row, rc, &mut meta[row]);
        }
    }

    // ====== PASS 2: Count y/z intersections and triangles ======
    let count_voxel_row = |vr: usize| -> ([[u32; 2]; 4], u32, u32, u32, bool) {
        let j = vr % (ny - 1);
        let k = vr / (ny - 1);
        let rows = [
            k * ny + j,
            k * ny + j + 1,
            (k + 1) * ny + j,
            (k + 1) * ny + j + 1,
        ];
        let mut xl = nxm1;
        let mut xr = 0;
        for &r in &rows {
            xl = xl.min(meta[r][4] as usize);
            xr = xr.max(meta[r][5] as usize);
        }

        let xc = [
            rows[0] * nxm1,
            rows[1] * nxm1,
            rows[2] * nxm1,
            rows[3] * nxm1,
        ];
        let has_x_intersections = rows.iter().any(|&r| meta[r][0] != 0);
        if !has_x_intersections {
            let same_initial_case = x_cases[xc[0]] == x_cases[xc[1]]
                && x_cases[xc[1]] == x_cases[xc[2]]
                && x_cases[xc[2]] == x_cases[xc[3]];
            if same_initial_case {
                return ([[0, 0]; 4], 0, 0, 0, false);
            }
            xl = 0;
            xr = nxm1;
        } else {
            if xl >= xr {
                return ([[0, 0]; 4], 0, 0, 0, false);
            }
            if xl > 0 {
                let same_left_state = (x_cases[xc[0] + xl] & 1) == (x_cases[xc[1] + xl] & 1)
                    && (x_cases[xc[1] + xl] & 1) == (x_cases[xc[2] + xl] & 1)
                    && (x_cases[xc[2] + xl] & 1) == (x_cases[xc[3] + xl] & 1);
                if !same_left_state {
                    xl = 0;
                }
            }

            if xr < nxm1 {
                let same_right_state = (x_cases[xc[0] + xr] & 2) == (x_cases[xc[1] + xr] & 2)
                    && (x_cases[xc[1] + xr] & 2) == (x_cases[xc[2] + xr] & 2)
                    && (x_cases[xc[2] + xr] & 2) == (x_cases[xc[3] + xr] & 2);
                if !same_right_state {
                    xr = nxm1;
                }
            }
        }

        let y_loc = if j >= ny - 2 { 2u8 } else { 0u8 };
        let z_loc = if k >= nz - 2 { 2u8 } else { 0u8 };
        let yz_loc = (y_loc << 2) | (z_loc << 4);
        let dim0_wall = nx - 2;
        let mut deltas = [[0u32, 0u32]; 4];
        let mut tc = 0u32;
        for i in xl..xr {
            let fe_case = edge_case(
                x_cases[xc[0] + i],
                x_cases[xc[1] + i],
                x_cases[xc[2] + i],
                x_cases[xc[3] + i],
            );
            let case = &case_table[fe_case as usize];
            if case.tri_count == 0 {
                continue;
            }
            let uses = case.uses_mask;
            deltas[0][0] += edge_used(uses, 4) as u32;
            deltas[0][1] += edge_used(uses, 8) as u32;
            count_boundary_yz(
                yz_loc | if i >= dim0_wall { 2 } else { 0 },
                uses,
                &mut deltas,
            );
            tc += case.tri_count as u32;
        }
        (deltas, tc, xl as u32, xr as u32, true)
    };

    let pass2: Vec<([[u32; 2]; 4], u32, u32, u32, bool)> = if use_par {
        (0..n_voxel_rows)
            .into_par_iter()
            .map(count_voxel_row)
            .collect()
    } else {
        (0..n_voxel_rows).map(count_voxel_row).collect()
    };

    for vr in 0..n_voxel_rows {
        let j = vr % (ny - 1);
        let k = vr / (ny - 1);
        let rows = [
            k * ny + j,
            k * ny + j + 1,
            (k + 1) * ny + j,
            (k + 1) * ny + j + 1,
        ];
        let row0 = k * ny + j;
        for n in 0..4 {
            meta[rows[n]][1] += pass2[vr].0[n][0];
            meta[rows[n]][2] += pass2[vr].0[n][1];
        }
        meta[row0][3] += pass2[vr].1;
        if pass2[vr].4 {
            meta[row0][4] = pass2[vr].2;
            meta[row0][5] = pass2[vr].3;
        }
    }

    // ====== PASS 3: Prefix sum ======
    let mut total_x: u32 = 0;
    let mut total_y: u32 = 0;
    let mut total_z: u32 = 0;
    let mut total_t: u32 = 0;

    for row in 0..n_rows {
        let (nx_p, ny_p, nz_p, nt) = (meta[row][0], meta[row][1], meta[row][2], meta[row][3]);
        meta[row][0] = total_x;
        meta[row][1] = total_y;
        meta[row][2] = total_z;
        meta[row][3] = total_t;
        total_x += nx_p;
        total_y += ny_p;
        total_z += nz_p;
        total_t += nt;
    }

    let total_pts = (total_x + total_y + total_z) as usize;
    let total_tris = total_t as usize;
    if total_tris == 0 {
        return PolyData::new();
    }

    let mut pts_flat = vec![0.0f64; total_pts * 3];
    let mut conn = vec![0i64; total_tris * 3];
    let y_offset = total_x;
    let z_offset = total_x + total_y;

    const C: [[usize; 3]; 8] = [
        [0, 0, 0],
        [1, 0, 0],
        [0, 1, 0],
        [1, 1, 0],
        [0, 0, 1],
        [1, 0, 1],
        [0, 1, 1],
        [1, 1, 1],
    ];
    const EV: [[usize; 2]; 12] = [
        [0, 1],
        [2, 3],
        [4, 5],
        [6, 7],
        [0, 2],
        [1, 3],
        [4, 6],
        [5, 7],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];

    // ====== PASS 4: Generate output ======
    let pts_writer = UnsafeF64Writer::new(&mut pts_flat);
    let conn_writer = UnsafeI64Writer::new(&mut conn);

    let gen_voxel_row = |vr: usize| {
        let j = vr % (ny - 1);
        let k = vr / (ny - 1);
        let rows = [
            k * ny + j,
            k * ny + j + 1,
            (k + 1) * ny + j,
            (k + 1) * ny + j + 1,
        ];
        let mut xl = nxm1;
        let mut xr = 0;
        for &r in &rows {
            xl = xl.min(meta[r][4] as usize);
            xr = xr.max(meta[r][5] as usize);
        }
        if xl >= xr {
            return;
        }

        let xc = [
            rows[0] * nxm1,
            rows[1] * nxm1,
            rows[2] * nxm1,
            rows[3] * nxm1,
        ];
        let row0 = rows[0];
        let first_fe_case = edge_case(
            x_cases[xc[0] + xl],
            x_cases[xc[1] + xl],
            x_cases[xc[2] + xl],
            x_cases[xc[3] + xl],
        );
        let first_uses = case_table[first_fe_case as usize].uses_mask;
        let mut eids = [
            meta[rows[0]][0],
            meta[rows[1]][0],
            meta[rows[2]][0],
            meta[rows[3]][0],
            y_offset + meta[row0][1],
            y_offset + meta[row0][1] + edge_used(first_uses, 4) as u32,
            y_offset + meta[rows[2]][1],
            y_offset + meta[rows[2]][1] + edge_used(first_uses, 6) as u32,
            z_offset + meta[row0][2],
            z_offset + meta[row0][2] + edge_used(first_uses, 8) as u32,
            z_offset + meta[rows[1]][2],
            z_offset + meta[rows[1]][2] + edge_used(first_uses, 10) as u32,
        ];
        let mut tid = meta[row0][3] as usize;

        for i in xl..xr {
            let fe_case = edge_case(
                x_cases[xc[0] + i],
                x_cases[xc[1] + i],
                x_cases[xc[2] + i],
                x_cases[xc[3] + i],
            );
            let case = &case_table[fe_case as usize];
            if case.tri_count == 0 {
                continue;
            }
            let uses = case.uses_mask;

            let base = k * nxy + j * nx + i;
            let v = unsafe {
                [
                    *scalars.get_unchecked(base),
                    *scalars.get_unchecked(base + 1),
                    *scalars.get_unchecked(base + nx),
                    *scalars.get_unchecked(base + nx + 1),
                    *scalars.get_unchecked(base + nxy),
                    *scalars.get_unchecked(base + nxy + 1),
                    *scalars.get_unchecked(base + nxy + nx),
                    *scalars.get_unchecked(base + nxy + nx + 1),
                ]
            };

            let mut x_loc = 0u8;
            if i < 1 {
                x_loc |= 1;
            }
            if i >= nx - 2 {
                x_loc |= 2;
            }
            let mut y_loc = 0u8;
            if j < 1 {
                y_loc |= 1;
            }
            if j >= ny - 2 {
                y_loc |= 2;
            }
            let mut z_loc = 0u8;
            if k < 1 {
                z_loc |= 1;
            }
            if k >= nz - 2 {
                z_loc |= 2;
            }
            let loc = x_loc | (y_loc << 2) | (z_loc << 4);

            for e in 0..12usize {
                if !edge_used(uses, e) || !generates_point_for_edge(e, loc) {
                    continue;
                }
                let [c0, c1] = EV[e];
                let d = v[c1] - v[c0];
                let t = if d.abs() > 1e-30 {
                    (isovalue - v[c0]) / d
                } else {
                    0.5
                };
                let (g0, g1) = (C[c0], C[c1]);

                let pid = eids[e];

                let p = pid as usize * 3;
                unsafe {
                    pts_writer.write(
                        p,
                        org[0] + ((i + g0[0]) as f64 + t * (g1[0] as f64 - g0[0] as f64)) * sp[0],
                    );
                    pts_writer.write(
                        p + 1,
                        org[1] + ((j + g0[1]) as f64 + t * (g1[1] as f64 - g0[1] as f64)) * sp[1],
                    );
                    pts_writer.write(
                        p + 2,
                        org[2] + ((k + g0[2]) as f64 + t * (g1[2] as f64 - g0[2] as f64)) * sp[2],
                    );
                }
            }

            let tri_edges = &case.tri_edges;
            for ti in (0..case.tri_count as usize * 3).step_by(3) {
                let c = tid * 3;
                unsafe {
                    conn_writer.write(c, eids[tri_edges[ti] as usize] as i64);
                    conn_writer.write(c + 1, eids[tri_edges[ti + 1] as usize] as i64);
                    conn_writer.write(c + 2, eids[tri_edges[ti + 2] as usize] as i64);
                }
                tid += 1;
            }

            eids[0] += edge_used(uses, 0) as u32;
            eids[1] += edge_used(uses, 1) as u32;
            eids[2] += edge_used(uses, 2) as u32;
            eids[3] += edge_used(uses, 3) as u32;
            eids[4] += edge_used(uses, 4) as u32;
            eids[5] = eids[4] + edge_used(uses, 5) as u32;
            eids[6] += edge_used(uses, 6) as u32;
            eids[7] = eids[6] + edge_used(uses, 7) as u32;
            eids[8] += edge_used(uses, 8) as u32;
            eids[9] = eids[8] + edge_used(uses, 9) as u32;
            eids[10] += edge_used(uses, 10) as u32;
            eids[11] = eids[10] + edge_used(uses, 11) as u32;
        }
    };

    if use_par {
        (0..n_voxel_rows).into_par_iter().for_each(gen_voxel_row);
    } else {
        (0..n_voxel_rows).for_each(gen_voxel_row);
    }

    let points = Points::from_flat_vec(pts_flat);
    let nt = conn.len() / 3;
    let offsets: Vec<i64> = (0..=nt).map(|i| (i * 3) as i64).collect();
    let polys = CellArray::from_raw(offsets, conn);
    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

const MC_TO_FE_EDGE: [u8; 12] = [0, 5, 1, 4, 2, 7, 3, 6, 8, 9, 10, 11];

#[derive(Clone, Copy)]
struct FeCase {
    uses_mask: u16,
    tri_edges: [u8; 15],
    tri_count: u8,
}

fn fe_case_table() -> &'static [FeCase; 256] {
    static TABLE: OnceLock<[FeCase; 256]> = OnceLock::new();
    TABLE.get_or_init(build_fe_case_table)
}

fn build_fe_case_table() -> [FeCase; 256] {
    std::array::from_fn(|fe_case| {
        let mc_case = edge_case_to_mc(fe_case as u8);
        let uses_mask = edge_uses_fe(EDGE_TABLE[mc_case as usize]);
        let tr = &TRI_TABLE[mc_case as usize];
        let mut tri_edges = [0u8; 15];
        let mut ti = 0usize;
        while ti < 15 && tr[ti] != -1 {
            tri_edges[ti] = MC_TO_FE_EDGE[tr[ti] as usize];
            tri_edges[ti + 1] = MC_TO_FE_EDGE[tr[ti + 1] as usize];
            tri_edges[ti + 2] = MC_TO_FE_EDGE[tr[ti + 2] as usize];
            ti += 3;
        }
        FeCase {
            uses_mask,
            tri_edges,
            tri_count: (ti / 3) as u8,
        }
    })
}

#[inline(always)]
fn edge_case(e00: u8, e10: u8, e01: u8, e11: u8) -> u8 {
    e00 | (e10 << 2) | (e01 << 4) | (e11 << 6)
}

#[inline(always)]
fn edge_case_to_mc(ec: u8) -> u8 {
    (ec & 1)
        | ((ec >> 1) & 1) << 1
        | ((ec >> 3) & 1) << 2
        | ((ec >> 2) & 1) << 3
        | ((ec >> 4) & 1) << 4
        | ((ec >> 5) & 1) << 5
        | ((ec >> 7) & 1) << 6
        | ((ec >> 6) & 1) << 7
}

#[inline(always)]
fn edge_uses_fe(mc_edge_flags: u16) -> u16 {
    let mut uses = 0u16;
    for (mc_edge, &fe_edge) in MC_TO_FE_EDGE.iter().enumerate() {
        if mc_edge_flags & (1 << mc_edge) != 0 {
            uses |= 1 << fe_edge;
        }
    }
    uses
}

#[inline(always)]
fn edge_used(uses_mask: u16, edge: usize) -> bool {
    uses_mask & (1 << edge) != 0
}

#[inline(always)]
fn generates_point_for_edge(edge: usize, loc: u8) -> bool {
    let plus_x = loc & 2 != 0;
    let plus_y = loc & 8 != 0;
    let plus_z = loc & 32 != 0;
    match edge {
        0 | 4 | 8 => true,
        1 | 10 => plus_y,
        2 | 6 => plus_z,
        3 => plus_y && plus_z,
        5 | 9 => plus_x,
        7 => plus_x && plus_z,
        11 => plus_x && plus_y,
        _ => false,
    }
}

#[inline(always)]
fn count_boundary_yz(loc: u8, edge_uses: u16, deltas: &mut [[u32; 2]; 4]) {
    match loc {
        2 => {
            deltas[0][0] += edge_used(edge_uses, 5) as u32;
            deltas[0][1] += edge_used(edge_uses, 9) as u32;
        }
        8 => {
            deltas[1][1] += edge_used(edge_uses, 10) as u32;
        }
        10 => {
            deltas[0][0] += edge_used(edge_uses, 5) as u32;
            deltas[0][1] += edge_used(edge_uses, 9) as u32;
            deltas[1][1] += edge_used(edge_uses, 10) as u32 + edge_used(edge_uses, 11) as u32;
        }
        32 => {
            deltas[2][0] += edge_used(edge_uses, 6) as u32;
        }
        34 => {
            deltas[0][0] += edge_used(edge_uses, 5) as u32;
            deltas[0][1] += edge_used(edge_uses, 9) as u32;
            deltas[2][0] += edge_used(edge_uses, 6) as u32 + edge_used(edge_uses, 7) as u32;
        }
        40 => {
            deltas[2][0] += edge_used(edge_uses, 6) as u32;
            deltas[1][1] += edge_used(edge_uses, 10) as u32;
        }
        42 => {
            deltas[0][0] += edge_used(edge_uses, 5) as u32;
            deltas[0][1] += edge_used(edge_uses, 9) as u32;
            deltas[1][1] += edge_used(edge_uses, 10) as u32 + edge_used(edge_uses, 11) as u32;
            deltas[2][0] += edge_used(edge_uses, 6) as u32 + edge_used(edge_uses, 7) as u32;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_isosurface() {
        let img = ImageData::with_dimensions(32, 32, 32);
        let mut v = Vec::with_capacity(32 * 32 * 32);
        for k in 0..32 {
            for j in 0..32 {
                for i in 0..32 {
                    let (x, y, z) = ((i as f64 - 16.0), (j as f64 - 16.0), (k as f64 - 16.0));
                    v.push(x * x + y * y + z * z);
                }
            }
        }
        let r = flying_edges_3d(&img, &v, 100.0);
        assert!(r.polys.num_cells() > 100);
    }

    #[test]
    fn empty_field() {
        let img = ImageData::with_dimensions(4, 4, 4);
        let r = flying_edges_3d(&img, &vec![0.0; 64], 1.0);
        assert_eq!(r.polys.num_cells(), 0);
    }

    #[test]
    fn fe_matches_cell_count() {
        let img = ImageData::with_dimensions(8, 8, 8);
        let mut v = Vec::with_capacity(512);
        for k in 0..8 {
            for j in 0..8 {
                for i in 0..8 {
                    v.push(
                        (i as f64 - 4.0).powi(2)
                            + (j as f64 - 4.0).powi(2)
                            + (k as f64 - 4.0).powi(2),
                    );
                }
            }
        }
        let r = flying_edges_3d(&img, &v, 5.0);
        assert!(r.polys.num_cells() > 10);
    }

    #[test]
    fn y_only_plane_matches_marching_cubes_count() {
        let img = ImageData::with_dimensions(4, 4, 4);
        let mut v = Vec::with_capacity(64);
        for _k in 0..4 {
            for j in 0..4 {
                for _i in 0..4 {
                    v.push(j as f64);
                }
            }
        }
        let fe = flying_edges_3d(&img, &v, 1.5);
        let mc = crate::filters::core::marching_cubes::marching_cubes(&img, &v, 1.5);
        assert_eq!(fe.polys.num_cells(), mc.polys.num_cells());
    }

    #[test]
    fn z_only_plane_matches_marching_cubes_count() {
        let img = ImageData::with_dimensions(4, 4, 4);
        let mut v = Vec::with_capacity(64);
        for k in 0..4 {
            for _j in 0..4 {
                for _i in 0..4 {
                    v.push(k as f64);
                }
            }
        }
        let fe = flying_edges_3d(&img, &v, 1.5);
        let mc = crate::filters::core::marching_cubes::marching_cubes(&img, &v, 1.5);
        assert_eq!(fe.polys.num_cells(), mc.polys.num_cells());
    }
}
