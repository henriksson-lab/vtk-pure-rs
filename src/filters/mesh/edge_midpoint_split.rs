use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashMap;

/// Split every triangle at edge midpoints, producing 4 triangles per input triangle.
///
/// This is a simple midpoint subdivision: each edge is split at its midpoint
/// and the original triangle is replaced by 4 smaller triangles. Original
/// vertex positions are not moved (unlike Loop subdivision).
///
/// Only polygon cells with exactly 3 vertices (triangles) are subdivided.
/// Non-triangle cells are dropped.
pub fn midpoint_split(input: &PolyData) -> PolyData {
    if input.polys.iter().any(|cell| cell.len() != 3)
        || input.verts.num_cells() != 0
        || input.lines.num_cells() != 0
        || input.strips.num_cells() != 0
        || input
            .polys
            .iter()
            .any(|cell| !valid_triangle(cell, input.points.len()))
        || has_non_manifold_edge(input)
    {
        return PolyData::new();
    }

    let mut out_points: Points<f64> = Points::new();

    // Copy original points
    for i in 0..input.points.len() {
        out_points.push(input.points.get(i));
    }

    // Cache for midpoints: (min_id, max_id) -> new point index
    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

    let get_midpoint = |points: &mut Points<f64>,
                        cache: &mut HashMap<(usize, usize), usize>,
                        a: usize,
                        b: usize|
     -> usize {
        let key: (usize, usize) = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = cache.get(&key) {
            return idx;
        }
        let pa = input.points.get(a);
        let pb = input.points.get(b);
        let mid: [f64; 3] = [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ];
        let idx: usize = points.len();
        points.push(mid);
        cache.insert(key, idx);
        idx
    };

    let mut out_polys: CellArray = CellArray::new();
    let mut source_cell_ids = Vec::with_capacity(input.polys.num_cells() * 4);

    for (cell_id, cell) in input.polys.iter().enumerate() {
        let v0 = cell[0] as usize;
        let v1 = cell[1] as usize;
        let v2 = cell[2] as usize;

        let m01: usize = get_midpoint(&mut out_points, &mut midpoint_cache, v0, v1);
        let m12: usize = get_midpoint(&mut out_points, &mut midpoint_cache, v1, v2);
        let m20: usize = get_midpoint(&mut out_points, &mut midpoint_cache, v2, v0);

        out_polys.push_cell(&[v0 as i64, m01 as i64, m20 as i64]);
        out_polys.push_cell(&[m01 as i64, v1 as i64, m12 as i64]);
        out_polys.push_cell(&[m12 as i64, v2 as i64, m20 as i64]);
        out_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
        source_cell_ids.extend([cell_id; 4]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    interpolate_point_data(input, &midpoint_cache, pd.point_data_mut());
    copy_cell_data(input, &source_cell_ids, pd.cell_data_mut());
    pd
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn valid_triangle(cell: &[i64], n_points: usize) -> bool {
    cell.len() == 3
        && valid_point_index(cell[0], n_points).is_some()
        && valid_point_index(cell[1], n_points).is_some()
        && valid_point_index(cell[2], n_points).is_some()
}

fn has_non_manifold_edge(input: &PolyData) -> bool {
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in input.polys.iter() {
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        for (a, b) in [(ids[2], ids[0]), (ids[0], ids[1]), (ids[1], ids[2])] {
            let key = if a < b { (a, b) } else { (b, a) };
            let count = edge_counts.entry(key).or_insert(0);
            *count += 1;
            if *count > 2 {
                return true;
            }
        }
    }
    false
}

fn interpolate_point_data(
    input: &PolyData,
    midpoint_cache: &HashMap<(usize, usize), usize>,
    output: &mut DataSetAttributes,
) {
    for array in input.point_data().field_data().iter() {
        if array.num_tuples() == input.points.len() {
            output.add_array(interpolate_array(array, input.points.len(), midpoint_cache));
        }
    }
}

fn interpolate_array(
    array: &AnyDataArray,
    n_input_points: usize,
    midpoint_cache: &HashMap<(usize, usize), usize>,
) -> AnyDataArray {
    macro_rules! interpolate {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(interpolate_typed_array(
                $array,
                n_input_points,
                midpoint_cache,
            ))
        };
    }

    match array {
        AnyDataArray::F32(array) => interpolate!(array, F32),
        AnyDataArray::F64(array) => interpolate!(array, F64),
        AnyDataArray::I8(array) => interpolate!(array, I8),
        AnyDataArray::I16(array) => interpolate!(array, I16),
        AnyDataArray::I32(array) => interpolate!(array, I32),
        AnyDataArray::I64(array) => interpolate!(array, I64),
        AnyDataArray::U8(array) => interpolate!(array, U8),
        AnyDataArray::U16(array) => interpolate!(array, U16),
        AnyDataArray::U32(array) => interpolate!(array, U32),
        AnyDataArray::U64(array) => interpolate!(array, U64),
    }
}

fn interpolate_typed_array<T: Scalar>(
    array: &DataArray<T>,
    n_input_points: usize,
    midpoint_cache: &HashMap<(usize, usize), usize>,
) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = vec![T::default(); (n_input_points + midpoint_cache.len()) * num_components];
    data[..n_input_points * num_components].copy_from_slice(array.as_slice());

    for (&(p1, p2), &new_id) in midpoint_cache {
        for component in 0..num_components {
            let value =
                (array.tuple(p1)[component].to_f64() + array.tuple(p2)[component].to_f64()) * 0.5;
            data[new_id * num_components + component] = T::from_f64(value);
        }
    }

    DataArray::from_vec(array.name(), data, num_components)
}

fn copy_cell_data(input: &PolyData, source_cell_ids: &[usize], output: &mut DataSetAttributes) {
    if source_cell_ids.is_empty() {
        return;
    }
    for array in input.cell_data().field_data().iter() {
        if array.num_tuples() == input.polys.num_cells() {
            output.add_array(copy_array_by_indices(array, source_cell_ids));
        }
    }
}

fn copy_array_by_indices(array: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(copy_typed_array($array, indices))
        };
    }

    match array {
        AnyDataArray::F32(array) => copy!(array, F32),
        AnyDataArray::F64(array) => copy!(array, F64),
        AnyDataArray::I8(array) => copy!(array, I8),
        AnyDataArray::I16(array) => copy!(array, I16),
        AnyDataArray::I32(array) => copy!(array, I32),
        AnyDataArray::I64(array) => copy!(array, I64),
        AnyDataArray::U8(array) => copy!(array, U8),
        AnyDataArray::U16(array) => copy!(array, U16),
        AnyDataArray::U32(array) => copy!(array, U32),
        AnyDataArray::U64(array) => copy!(array, U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * num_components);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, num_components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 4);
        // 3 original + 3 midpoints = 6 points
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn two_triangles_shared_edge() {
        // Two triangles sharing edge 1-2; midpoint on that edge should be shared
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],  // 0
                [2.0, 0.0, 0.0],  // 1
                [1.0, 2.0, 0.0],  // 2
                [1.0, -2.0, 0.0], // 3
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 8); // 4 per triangle
                                                 // 4 original + 5 unique midpoints (edge 0-1 shared) = 9
                                                 // edges: 0-1, 1-2, 2-0, 0-3, 3-1 => 5 midpoints
        assert_eq!(result.points.len(), 9);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn skips_invalid_triangle_point_ids() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, -1, 2]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = midpoint_split(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn interpolates_point_data_to_edge_points() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "value",
                vec![0.0, 2.0, 4.0],
                1,
            )));

        let result = midpoint_split(&pd);
        let values = result.point_data().get_array("value").unwrap();
        assert_eq!(values.num_tuples(), result.points.len());
        let mut midpoint_values = Vec::new();
        let mut buf = [0.0f64];
        for tuple in 3..values.num_tuples() {
            values.tuple_as_f64(tuple, &mut buf);
            midpoint_values.push(buf[0]);
        }
        midpoint_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(midpoint_values, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn rejects_non_triangle_cells() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = midpoint_split(&pd);
        assert_eq!(result.total_cells(), 0);
    }

    #[test]
    fn copies_cell_data_to_each_subtriangle() {
        let mut pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![10, 20],
                1,
            )));

        let result = midpoint_split(&pd);
        let values = result.cell_data().get_array("cell_id").unwrap();
        assert_eq!(
            values.to_f64_vec(),
            vec![10.0, 10.0, 10.0, 10.0, 20.0, 20.0, 20.0, 20.0]
        );
    }
}
