use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

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
    let mut point_ids = Vec::new();
    let mut points_flat = Vec::new();
    let mut kept_cells = Vec::new();
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
    let tol2 = tolerance.max(0.0) * tolerance.max(0.0);
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
    let mut out = CellArray::new();

    for cell in cells.iter() {
        let in_cell_id = *global_cell_id;
        *global_cell_id += 1;
        let mut remapped: Vec<i64> = Vec::with_capacity(cell.len());

        for &id in cell {
            let rep = point_reps[id as usize];
            if point_map[rep] < 0 {
                point_map[rep] = (points_flat.len() / 3) as isize;
                let p = input.points.get(rep);
                points_flat.extend_from_slice(&p);
                point_ids.push(rep);
            }
            remapped.push(point_map[rep] as i64);
        }

        if remove_degenerate {
            // Remove consecutive duplicates
            let mut deduped: Vec<i64> = Vec::with_capacity(remapped.len());
            for &id in &remapped {
                if deduped.last() != Some(&id) {
                    deduped.push(id);
                }
            }
            // Also check if first == last for closed cells
            if closed_cell && deduped.len() > 1 && deduped.first() == deduped.last() {
                deduped.pop();
            }

            let unique = count_unique_ids(&deduped);
            if unique >= min_unique {
                out.push_cell(&deduped);
                kept_cells.push(in_cell_id);
            }
        } else {
            out.push_cell(&remapped);
            kept_cells.push(in_cell_id);
        }
    }

    out
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
