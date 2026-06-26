use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Weld (merge) vertices that are within a given distance tolerance.
///
/// Vertices closer than `tolerance` are merged into one. Cell connectivity is
/// updated to reference the merged vertices. Degenerate cells (with fewer than
/// 3 unique vertices for polygons) are discarded.
pub fn weld_vertices(input: &PolyData, tolerance: f64) -> PolyData {
    let n: usize = input.points.len();
    let tolerance = if tolerance.is_finite() && tolerance > 0.0 {
        tolerance
    } else {
        0.0
    };
    let tol2: f64 = tolerance * tolerance;

    // Spatial hashing for efficient neighbor lookup
    let inv_tol: f64 = if tolerance > 0.0 {
        1.0 / tolerance
    } else {
        1.0
    };

    let mut new_points: Points<f64> = Points::new();
    let mut point_map: Vec<usize> = vec![0; n];
    let mut representative_points: Vec<usize> = Vec::new();
    let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();

    for i in 0..n {
        let p = input.points.get(i);
        let key = (
            (p[0] * inv_tol).round() as i64,
            (p[1] * inv_tol).round() as i64,
            (p[2] * inv_tol).round() as i64,
        );

        let mut found: Option<usize> = None;

        // Check 3x3x3 neighborhood
        'search: for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                for dz in -1i64..=1 {
                    let Some(nkey) = offset_key(key, dx, dy, dz) else {
                        continue;
                    };
                    if let Some(indices) = grid.get(&nkey) {
                        for &idx in indices {
                            let q = new_points.get(idx);
                            let d2: f64 = (p[0] - q[0]) * (p[0] - q[0])
                                + (p[1] - q[1]) * (p[1] - q[1])
                                + (p[2] - q[2]) * (p[2] - q[2]);
                            if d2 <= tol2 {
                                found = Some(idx);
                                break 'search;
                            }
                        }
                    }
                }
            }
        }

        match found {
            Some(idx) => {
                point_map[i] = idx;
            }
            None => {
                let new_idx: usize = new_points.len();
                new_points.push(p);
                point_map[i] = new_idx;
                representative_points.push(i);
                grid.entry(key).or_default().push(new_idx);
            }
        }
    }

    let mut output = input.clone();
    output.points = new_points;

    // Remap cell arrays
    output.polys = remap_cell_array(&input.polys, &point_map, 3);
    output.verts = remap_cell_array(&input.verts, &point_map, 1);
    output.lines = remap_cell_array(&input.lines, &point_map, 2);
    output.strips = remap_cell_array(&input.strips, &point_map, 3);
    *output.point_data_mut() = remap_point_data(input.point_data(), &representative_points, n);

    output
}

fn offset_key(key: (i64, i64, i64), dx: i64, dy: i64, dz: i64) -> Option<(i64, i64, i64)> {
    Some((
        key.0.checked_add(dx)?,
        key.1.checked_add(dy)?,
        key.2.checked_add(dz)?,
    ))
}

/// Remap cell indices and discard degenerate cells.
fn remap_cell_array(cells: &CellArray, point_map: &[usize], min_unique: usize) -> CellArray {
    let mut out = CellArray::new();
    for cell in cells.iter() {
        if !cell.iter().all(|&id| valid_point_id(id, point_map.len())) {
            continue;
        }

        let mapped: Vec<i64> = cell
            .iter()
            .map(|&id| point_map[id as usize] as i64)
            .collect();

        // Check for enough unique vertices
        let mut unique = mapped.clone();
        unique.sort();
        unique.dedup();
        if unique.len() >= min_unique {
            out.push_cell(&mapped);
        }
    }
    out
}

fn valid_point_id(id: i64, number_of_points: usize) -> bool {
    id >= 0 && (id as usize) < number_of_points
}

fn remap_point_data(
    input: &DataSetAttributes,
    representative_points: &[usize],
    number_of_input_points: usize,
) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();

    for array in input.iter() {
        if let Some(remapped) =
            remap_point_array(array, representative_points, number_of_input_points)
        {
            output.add_array(remapped);
        }
    }

    if let Some(array) = input.scalars() {
        output.set_active_scalars(array.name());
    }
    if let Some(array) = input.vectors() {
        output.set_active_vectors(array.name());
    }
    if let Some(array) = input.normals() {
        output.set_active_normals(array.name());
    }
    if let Some(array) = input.tcoords() {
        output.set_active_tcoords(array.name());
    }
    if let Some(array) = input.tensors() {
        output.set_active_tensors(array.name());
    }
    if let Some(array) = input.global_ids() {
        output.set_active_global_ids(array.name());
    }
    if let Some(array) = input.pedigree_ids() {
        output.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = input.edge_flags() {
        output.set_active_edge_flags(array.name());
    }
    if let Some(array) = input.tangents() {
        output.set_active_tangents(array.name());
    }
    if let Some(array) = input.rational_weights() {
        output.set_active_rational_weights(array.name());
    }
    if let Some(array) = input.higher_order_degrees() {
        output.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = input.process_ids() {
        output.set_active_process_ids(array.name());
    }

    output
}

fn remap_point_array(
    array: &AnyDataArray,
    representative_points: &[usize],
    number_of_input_points: usize,
) -> Option<AnyDataArray> {
    if array.num_tuples() != number_of_input_points {
        return None;
    }

    macro_rules! remap {
        ($array:expr, $variant:ident) => {{
            let number_of_components = $array.num_components();
            let mut values = Vec::with_capacity(representative_points.len() * number_of_components);
            for &point_id in representative_points {
                values.extend_from_slice($array.tuple(point_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $array.name(),
                values,
                number_of_components,
            )))
        }};
    }

    match array {
        AnyDataArray::F32(a) => remap!(a, F32),
        AnyDataArray::F64(a) => remap!(a, F64),
        AnyDataArray::I8(a) => remap!(a, I8),
        AnyDataArray::I16(a) => remap!(a, I16),
        AnyDataArray::I32(a) => remap!(a, I32),
        AnyDataArray::I64(a) => remap!(a, I64),
        AnyDataArray::U8(a) => remap!(a, U8),
        AnyDataArray::U16(a) => remap!(a, U16),
        AnyDataArray::U32(a) => remap!(a, U32),
        AnyDataArray::U64(a) => remap!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn weld_duplicate_vertices() {
        // Two triangles sharing an edge, but with duplicated vertices
        let pts: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [0.5, 1.0, 0.0], // 2
            [1.0, 0.0, 0.0], // 3 = duplicate of 1
            [2.0, 0.0, 0.0], // 4
            [1.5, 1.0, 0.0], // 5
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2], [3, 4, 5]];
        let pd = PolyData::from_triangles(pts, tris);

        let result = weld_vertices(&pd, 1e-6);
        // Should have 5 unique points (vertex 3 merged with vertex 1)
        assert_eq!(result.points.len(), 5);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn weld_with_tolerance() {
        // Points that are close but not identical
        let pts: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.001, 0.0, 0.0], // close to point 1
            [2.0, 0.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2], [3, 4, 5]];
        let pd = PolyData::from_triangles(pts, tris);

        // With a large enough tolerance, point 3 merges with point 1
        let result = weld_vertices(&pd, 0.01);
        assert_eq!(result.points.len(), 5);

        // With a tiny tolerance, nothing merges
        let result2 = weld_vertices(&pd, 1e-6);
        assert_eq!(result2.points.len(), 6);
    }

    #[test]
    fn zero_tolerance_welds_exact_large_coordinate_duplicates() {
        let pts: Vec<[f64; 3]> = vec![
            [1.0e20, 2.0e20, 3.0e20],
            [1.0e20, 2.0e20, 3.0e20],
            [1.0e20 + 1.0e6, 2.0e20, 3.0e20],
        ];
        let pd = PolyData::from_points(pts);

        let result = weld_vertices(&pd, 0.0);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn point_data_remapped_to_representative_points() {
        let mut pd = PolyData::from_points(vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "value",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("value");

        let result = weld_vertices(&pd, 0.0);

        let values = result.point_data().scalars().unwrap();
        let mut buf = [0.0f64];
        values.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0);
        values.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 30.0);
    }

    #[test]
    fn degenerate_cells_removed() {
        // A triangle where two vertices are the same after welding
        let pts: Vec<[f64; 3]> = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], // duplicate of 0
            [1.0, 0.0, 0.0],
        ];
        let tris: Vec<[i64; 3]> = vec![[0, 1, 2]];
        let pd = PolyData::from_triangles(pts, tris);

        let result = weld_vertices(&pd, 1e-6);
        // The triangle becomes degenerate (only 2 unique vertices)
        assert_eq!(result.polys.num_cells(), 0);
    }
}
