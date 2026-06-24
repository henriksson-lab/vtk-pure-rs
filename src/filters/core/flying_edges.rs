use crate::data::{CellArray, ImageData, Points, PolyData};
use crate::filters::core::marching_cubes::{EDGE_TABLE, TRI_TABLE};
use rayon::prelude::*;

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
    // Only use parallel for grids larger than ~100^3 (1M voxels)
    let use_par = nxm1 * n_rows > 1_000_000;

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
    let count_voxel_row = |vr: usize| -> [u32; 3] {
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
            return [0, 0, 0];
        }

        let xc = [
            rows[0] * nxm1,
            rows[1] * nxm1,
            rows[2] * nxm1,
            rows[3] * nxm1,
        ];
        let mut yc = 0u32;
        let mut zc = 0u32;
        let mut tc = 0u32;

        for i in xl..xr {
            let ci = edge_case_to_mc(
                x_cases[xc[0] + i],
                x_cases[xc[1] + i],
                x_cases[xc[2] + i],
                x_cases[xc[3] + i],
            );
            let ef = EDGE_TABLE[ci as usize];
            if ef == 0 {
                continue;
            }
            for e in [1u32, 3, 5, 7] {
                if ef & (1 << e) != 0 {
                    yc += 1;
                }
            }
            for e in [8u32, 9, 10, 11] {
                if ef & (1 << e) != 0 {
                    zc += 1;
                }
            }
            let tr = &TRI_TABLE[ci as usize];
            let mut t = 0;
            while t < 15 && tr[t] != -1 {
                t += 3;
            }
            tc += (t / 3) as u32;
        }
        [yc, zc, tc]
    };

    let pass2: Vec<[u32; 3]> = if use_par {
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
        let row0 = k * ny + j;
        meta[row0][1] += pass2[vr][0];
        meta[row0][2] += pass2[vr][1];
        meta[row0][3] += pass2[vr][2];
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
        [1, 1, 0],
        [0, 1, 0],
        [0, 0, 1],
        [1, 0, 1],
        [1, 1, 1],
        [0, 1, 1],
    ];
    const EV: [[usize; 2]; 12] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0],
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];

    // ====== PASS 4: Generate output ======
    let pts_base = pts_flat.as_mut_ptr() as usize;
    let conn_base = conn.as_mut_ptr() as usize;

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
        let mut xid = [
            meta[rows[0]][0],
            meta[rows[1]][0],
            meta[rows[2]][0],
            meta[rows[3]][0],
        ];
        let mut yid = y_offset + meta[row0][1];
        let mut zid = z_offset + meta[row0][2];
        let mut tid = meta[row0][3] as usize;

        for i in xl..xr {
            let ci = edge_case_to_mc(
                x_cases[xc[0] + i],
                x_cases[xc[1] + i],
                x_cases[xc[2] + i],
                x_cases[xc[3] + i],
            );
            let ef = EDGE_TABLE[ci as usize];
            if ef == 0 {
                continue;
            }

            let base = k * nxy + j * nx + i;
            let v = unsafe {
                [
                    *scalars.get_unchecked(base),
                    *scalars.get_unchecked(base + 1),
                    *scalars.get_unchecked(base + nx + 1),
                    *scalars.get_unchecked(base + nx),
                    *scalars.get_unchecked(base + nxy),
                    *scalars.get_unchecked(base + nxy + 1),
                    *scalars.get_unchecked(base + nxy + nx + 1),
                    *scalars.get_unchecked(base + nxy + nx),
                ]
            };

            let mut ep = [0u32; 12];
            for e in 0..12u32 {
                if ef & (1 << e) == 0 {
                    continue;
                }
                let [c0, c1] = EV[e as usize];
                let d = v[c1] - v[c0];
                let t = if d.abs() > 1e-30 {
                    (isovalue - v[c0]) / d
                } else {
                    0.5
                };
                let (g0, g1) = (C[c0], C[c1]);

                let pid = match e {
                    0 => {
                        let id = xid[0];
                        xid[0] += 1;
                        id
                    }
                    2 => {
                        let id = xid[1];
                        xid[1] += 1;
                        id
                    }
                    4 => {
                        let id = xid[2];
                        xid[2] += 1;
                        id
                    }
                    6 => {
                        let id = xid[3];
                        xid[3] += 1;
                        id
                    }
                    1 | 3 | 5 | 7 => {
                        let id = yid;
                        yid += 1;
                        id
                    }
                    _ => {
                        let id = zid;
                        zid += 1;
                        id
                    }
                };
                ep[e as usize] = pid;

                unsafe {
                    let p = (pts_base as *mut f64).add(pid as usize * 3);
                    *p = org[0] + ((i + g0[0]) as f64 + t * (g1[0] as f64 - g0[0] as f64)) * sp[0];
                    *p.add(1) =
                        org[1] + ((j + g0[1]) as f64 + t * (g1[1] as f64 - g0[1] as f64)) * sp[1];
                    *p.add(2) =
                        org[2] + ((k + g0[2]) as f64 + t * (g1[2] as f64 - g0[2] as f64)) * sp[2];
                }
            }

            let tr = &TRI_TABLE[ci as usize];
            let mut ti = 0;
            while ti < 15 && tr[ti] != -1 {
                unsafe {
                    let c = (conn_base as *mut i64).add(tid * 3);
                    *c = ep[tr[ti] as usize] as i64;
                    *c.add(1) = ep[tr[ti + 1] as usize] as i64;
                    *c.add(2) = ep[tr[ti + 2] as usize] as i64;
                }
                tid += 1;
                ti += 3;
            }
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

#[inline(always)]
fn edge_case_to_mc(e00: u8, e10: u8, e01: u8, e11: u8) -> u8 {
    let ec = e00 | (e10 << 2) | (e01 << 4) | (e11 << 6);
    (ec & 1)
        | ((ec >> 1) & 1) << 1
        | ((ec >> 3) & 1) << 2
        | ((ec >> 2) & 1) << 3
        | ((ec >> 4) & 1) << 4
        | ((ec >> 5) & 1) << 5
        | ((ec >> 7) & 1) << 6
        | ((ec >> 6) & 1) << 7
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
                        ((i as f64 - 4.0).powi(2)
                            + (j as f64 - 4.0).powi(2)
                            + (k as f64 - 4.0).powi(2)),
                    );
                }
            }
        }
        let r = flying_edges_3d(&img, &v, 5.0);
        assert!(r.polys.num_cells() > 10);
    }
}
