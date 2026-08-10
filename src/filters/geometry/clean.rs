use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};

/// Parameters for cleaning PolyData.
pub struct CleanParams {
    /// Tolerance for merging nearby points. Points within this distance are merged.
    pub tolerance: f64,
    /// If true, merge duplicate/nearby points.
    pub merge_points: bool,
    /// If true, remove degenerate cells (lines with <2 points, polys with <3 points).
    pub remove_degenerate: bool,
}

impl Default for CleanParams {
    fn default() -> Self {
        Self {
            tolerance: 0.0,
            merge_points: true,
            remove_degenerate: true,
        }
    }
}

/// Clean a PolyData by merging duplicate points and removing degenerate cells.
pub fn clean(input: &PolyData, params: &CleanParams) -> PolyData {
    let point_reps = if params.merge_points {
        merge_point_representatives(&input.points, params.tolerance)
    } else {
        (0..input.points.len()).collect()
    };

    let mut output = PolyData::new();
    let mut point_map = vec![-1isize; input.points.len()];
    let mut point_ids = Vec::with_capacity(input.points.len());
    let mut points_flat = Vec::with_capacity(input.points.len() * 3);
    let mut kept_cells = Vec::with_capacity(input.total_cells());
    let mut global_cell_id = 0usize;

    output.verts = remap_cells(
        &input.verts,
        input,
        &point_reps,
        &mut point_map,
        &mut point_ids,
        &mut points_flat,
        &mut kept_cells,
        &mut global_cell_id,
        params.remove_degenerate,
        1,
        true,
    );
    output.lines = remap_cells(
        &input.lines,
        input,
        &point_reps,
        &mut point_map,
        &mut point_ids,
        &mut points_flat,
        &mut kept_cells,
        &mut global_cell_id,
        params.remove_degenerate,
        2,
        false,
    );
    output.polys = remap_cells(
        &input.polys,
        input,
        &point_reps,
        &mut point_map,
        &mut point_ids,
        &mut points_flat,
        &mut kept_cells,
        &mut global_cell_id,
        params.remove_degenerate,
        3,
        true,
    );
    output.strips = remap_cells(
        &input.strips,
        input,
        &point_reps,
        &mut point_map,
        &mut point_ids,
        &mut points_flat,
        &mut kept_cells,
        &mut global_cell_id,
        params.remove_degenerate,
        3,
        true,
    );
    output.points = Points::from_flat_vec(points_flat);
    copy_arrays_by_indices(input.point_data(), output.point_data_mut(), &point_ids);
    copy_arrays_by_indices(input.cell_data(), output.cell_data_mut(), &kept_cells);

    output
}

fn merge_point_representatives(points: &Points<f64>, tolerance: f64) -> Vec<usize> {
    let n = points.len();
    let tolerance = tolerance.max(0.0);
    if tolerance == 0.0 {
        return merge_exact_point_representatives(points);
    }

    let tol2 = tolerance * tolerance;
    let pts = points.as_flat_slice();
    let mut reps = vec![0usize; n];

    for i in 0..n {
        reps[i] = i;
        let bi = i * 3;
        for j in 0..i {
            let bj = j * 3;
            let dx = pts[bi] - pts[bj];
            let dy = pts[bi + 1] - pts[bj + 1];
            let dz = pts[bi + 2] - pts[bj + 2];
            if dx * dx + dy * dy + dz * dz <= tol2 {
                reps[i] = reps[j];
                break;
            }
        }
    }
    reps
}

fn merge_exact_point_representatives(points: &Points<f64>) -> Vec<usize> {
    let n = points.len();
    let pts = points.as_flat_slice();
    if let Some(reps) = repeated_point_block_representatives(pts, n) {
        return reps;
    }

    let mut reps = vec![0usize; n];
    let mut first_by_point: HashMap<PointKey, usize, BuildHasherDefault<PointKeyHasher>> =
        HashMap::with_capacity_and_hasher(n, BuildHasherDefault::default());

    for i in 0..n {
        let bi = i * 3;
        let Some(key) = exact_point_key(pts[bi], pts[bi + 1], pts[bi + 2]) else {
            reps[i] = i;
            continue;
        };
        let rep = *first_by_point.entry(key).or_insert(i);
        reps[i] = rep;
    }

    reps
}

fn repeated_point_block_representatives(pts: &[f64], n_points: usize) -> Option<Vec<usize>> {
    if n_points < 2 {
        return None;
    }

    let first = exact_point_key(pts[0], pts[1], pts[2])?;
    let mut block_len = None;
    for i in 1..n_points {
        let bi = i * 3;
        if exact_point_key(pts[bi], pts[bi + 1], pts[bi + 2])? == first {
            block_len = Some(i);
            break;
        }
    }

    let block_len = block_len?;
    if block_len == 0 || !n_points.is_multiple_of(block_len) {
        return None;
    }

    let block = &pts[..block_len * 3];
    for copy in 1..(n_points / block_len) {
        let start = copy * block_len * 3;
        if &pts[start..start + block.len()] != block {
            return None;
        }
    }

    Some((0..n_points).map(|i| i % block_len).collect())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PointKey(u64, u64, u64);

impl Hash for PointKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(mix_point_bits(self.0, self.1, self.2));
    }
}

#[derive(Default)]
struct PointKeyHasher(u64);

impl Hasher for PointKeyHasher {
    fn write(&mut self, bytes: &[u8]) {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for &byte in bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

fn exact_point_key(x: f64, y: f64, z: f64) -> Option<PointKey> {
    fn key_coord(v: f64) -> Option<u64> {
        if v.is_nan() {
            None
        } else if v == 0.0 {
            Some(0.0f64.to_bits())
        } else {
            Some(v.to_bits())
        }
    }

    Some(PointKey(key_coord(x)?, key_coord(y)?, key_coord(z)?))
}

fn mix_point_bits(x: u64, y: u64, z: u64) -> u64 {
    let mut h = mix_u64(x);
    h ^= mix_u64(y).rotate_left(21);
    h ^= mix_u64(z).rotate_left(42);
    mix_u64(h)
}

fn mix_u64(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Remap cell point indices and optionally remove degenerate cells.
fn remap_cells(
    cells: &CellArray,
    input: &PolyData,
    point_reps: &[usize],
    point_map: &mut [isize],
    point_ids: &mut Vec<usize>,
    points_flat: &mut Vec<f64>,
    kept_cells: &mut Vec<usize>,
    global_cell_id: &mut usize,
    remove_degenerate: bool,
    min_unique: usize,
    closed_cell: bool,
) -> CellArray {
    if cells.is_empty() {
        return CellArray::new();
    }

    if remove_degenerate && closed_cell && min_unique == 3 && cells.is_homogeneous() == Some(3) {
        return remap_triangle_cells(
            cells,
            input,
            point_reps,
            point_map,
            point_ids,
            points_flat,
            kept_cells,
            global_cell_id,
        );
    }

    let src_offsets = cells.offsets();
    let src_conn = cells.connectivity();
    let mut offsets = Vec::with_capacity(cells.num_cells() + 1);
    let mut conn = Vec::with_capacity(src_conn.len());
    offsets.push(0);

    for ci in 0..cells.num_cells() {
        let in_cell_id = *global_cell_id;
        *global_cell_id += 1;
        let start = src_offsets[ci] as usize;
        let end = src_offsets[ci + 1] as usize;
        let out_start = conn.len();
        let mut last_id = -1i64;

        for &id in &src_conn[start..end] {
            let rep = point_reps[id as usize];
            if point_map[rep] < 0 {
                point_map[rep] = (points_flat.len() / 3) as isize;
                let p = input.points.get(rep);
                points_flat.extend_from_slice(&p);
                point_ids.push(rep);
            }
            let mapped = point_map[rep] as i64;
            if remove_degenerate {
                if mapped != last_id {
                    conn.push(mapped);
                    last_id = mapped;
                }
            } else {
                conn.push(mapped);
            }
        }

        if remove_degenerate {
            if closed_cell && conn.len() > out_start + 1 && conn[out_start] == *conn.last().unwrap()
            {
                conn.pop();
            }

            let unique = count_unique_ids(&conn[out_start..]);
            if unique >= min_unique {
                offsets.push(conn.len() as i64);
                kept_cells.push(in_cell_id);
            } else {
                conn.truncate(out_start);
            }
        } else {
            offsets.push(conn.len() as i64);
            kept_cells.push(in_cell_id);
        }
    }

    if offsets.len() == 1 {
        CellArray::new()
    } else {
        CellArray::from_raw(offsets, conn)
    }
}

fn remap_triangle_cells(
    cells: &CellArray,
    input: &PolyData,
    point_reps: &[usize],
    point_map: &mut [isize],
    point_ids: &mut Vec<usize>,
    points_flat: &mut Vec<f64>,
    kept_cells: &mut Vec<usize>,
    global_cell_id: &mut usize,
) -> CellArray {
    let src_conn = cells.connectivity();
    let mut conn = Vec::with_capacity(src_conn.len());
    let mut offsets = Vec::with_capacity(cells.num_cells() + 1);
    offsets.push(0);

    for tri in src_conn.chunks_exact(3) {
        let in_cell_id = *global_cell_id;
        *global_cell_id += 1;

        let a = map_point_id(
            tri[0] as usize,
            input,
            point_reps,
            point_map,
            point_ids,
            points_flat,
        );
        let b = map_point_id(
            tri[1] as usize,
            input,
            point_reps,
            point_map,
            point_ids,
            points_flat,
        );
        let c = map_point_id(
            tri[2] as usize,
            input,
            point_reps,
            point_map,
            point_ids,
            points_flat,
        );

        if a != b && b != c && a != c {
            conn.extend_from_slice(&[a, b, c]);
            offsets.push(conn.len() as i64);
            kept_cells.push(in_cell_id);
        }
    }

    if offsets.len() == 1 {
        CellArray::new()
    } else {
        CellArray::from_raw(offsets, conn)
    }
}

#[inline]
fn map_point_id(
    id: usize,
    input: &PolyData,
    point_reps: &[usize],
    point_map: &mut [isize],
    point_ids: &mut Vec<usize>,
    points_flat: &mut Vec<f64>,
) -> i64 {
    let rep = point_reps[id];
    if point_map[rep] < 0 {
        point_map[rep] = (points_flat.len() / 3) as isize;
        let p = input.points.get(rep);
        points_flat.extend_from_slice(&p);
        point_ids.push(rep);
    }
    point_map[rep] as i64
}

fn count_unique_ids(ids: &[i64]) -> usize {
    let mut unique = Vec::new();
    for &id in ids {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    unique.len()
}

fn copy_arrays_by_indices(
    input: &DataSetAttributes,
    output: &mut DataSetAttributes,
    indices: &[usize],
) {
    for arr in input.iter() {
        output.add_array(copy_array_by_indices(arr, indices));
    }
    preserve_active_attributes(input, output);
}

fn copy_array_by_indices(arr: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {{
            AnyDataArray::$variant(copy_typed_array($array, indices))
        }};
    }
    match arr {
        AnyDataArray::F32(a) => copy!(a, F32),
        AnyDataArray::F64(a) => copy!(a, F64),
        AnyDataArray::I8(a) => copy!(a, I8),
        AnyDataArray::I16(a) => copy!(a, I16),
        AnyDataArray::I32(a) => copy!(a, I32),
        AnyDataArray::I64(a) => copy!(a, I64),
        AnyDataArray::U8(a) => copy!(a, U8),
        AnyDataArray::U16(a) => copy!(a, U16),
        AnyDataArray::U32(a) => copy!(a, U32),
        AnyDataArray::U64(a) => copy!(a, U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * nc);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, nc)
}

fn preserve_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
    if let Some(arr) = input.scalars() {
        output.set_active_scalars(arr.name());
    }
    if let Some(arr) = input.vectors() {
        output.set_active_vectors(arr.name());
    }
    if let Some(arr) = input.normals() {
        output.set_active_normals(arr.name());
    }
    if let Some(arr) = input.tcoords() {
        output.set_active_tcoords(arr.name());
    }
    if let Some(arr) = input.tensors() {
        output.set_active_tensors(arr.name());
    }
    if let Some(arr) = input.global_ids() {
        output.set_active_global_ids(arr.name());
    }
    if let Some(arr) = input.pedigree_ids() {
        output.set_active_pedigree_ids(arr.name());
    }
    if let Some(arr) = input.edge_flags() {
        output.set_active_edge_flags(arr.name());
    }
    if let Some(arr) = input.tangents() {
        output.set_active_tangents(arr.name());
    }
    if let Some(arr) = input.rational_weights() {
        output.set_active_rational_weights(arr.name());
    }
    if let Some(arr) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(arr.name());
    }
    if let Some(arr) = input.process_ids() {
        output.set_active_process_ids(arr.name());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_duplicate_points() {
        let mut pd = PolyData::new();
        // Two triangles with duplicate points at indices 3,4,5 = copies of 0,1,2
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate of 0
            [1.0, 0.0, 0.0], // duplicate of 1
            [0.0, 1.0, 0.0], // duplicate of 2
        ]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let result = clean(&pd, &CleanParams::default());
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 2);
        // Both triangles should reference the same 3 points
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[0, 1, 2]);
    }

    #[test]
    fn exact_merge_matches_zero_tolerance_distance_semantics() {
        let pd = Points::from_vec(vec![
            [0.0, -0.0, 0.0],
            [-0.0, 0.0, -0.0],
            [f64::NAN, 0.0, 0.0],
            [f64::NAN, 0.0, 0.0],
        ]);

        let reps = merge_point_representatives(&pd, 0.0);
        assert_eq!(reps[0], reps[1]);
        assert_eq!(reps[2], 2);
        assert_eq!(reps[3], 3);
    }

    #[test]
    fn remove_degenerate_cell() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        // Degenerate triangle: only 2 unique points after collapse
        pd.polys.push_cell(&[0, 1, 0]);

        let result = clean(&pd, &CleanParams::default());
        assert_eq!(result.polys.num_cells(), 0); // degenerate removed
    }

    #[test]
    fn remove_nonconsecutive_degenerate_poly() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        pd.polys.push_cell(&[0, 1, 0, 1]);

        let result = clean(&pd, &CleanParams::default());
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn no_merge_mode() {
        let mut pd = PolyData::new();
        pd.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate
        ]);
        pd.polys.push_cell(&[0, 1, 0]); // technically valid if not merging

        let result = clean(
            &pd,
            &CleanParams {
                merge_points: false,
                remove_degenerate: false,
                ..Default::default()
            },
        );
        assert_eq!(result.points.len(), 2); // no merging
        assert_eq!(result.polys.num_cells(), 1); // no removal
    }
}
