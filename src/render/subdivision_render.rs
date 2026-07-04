//! Pre-render mesh refinement via midpoint subdivision.
//!
//! Each subdivision level splits every triangle into 4 triangles by
//! inserting midpoints along edges.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashMap;

/// Configuration for subdivision surface rendering.
#[derive(Debug, Clone)]
pub struct SubdivisionConfig {
    /// Whether subdivision is enabled.
    pub enabled: bool,
    /// Number of subdivision levels to apply.
    pub level: u32,
}

impl Default for SubdivisionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 1,
        }
    }
}

/// Apply midpoint subdivision to a PolyData mesh.
///
/// Each triangle is split into 4 triangles by inserting midpoints at
/// edge centers. This is repeated `level` times.
pub fn subdivide_for_render(poly_data: &PolyData, level: u32) -> PolyData {
    if level == 0 {
        return poly_data.clone();
    }

    let mut current = poly_data.clone();

    for _ in 0..level {
        current = subdivide_once(&current);
    }

    current
}

fn subdivide_once(pd: &PolyData) -> PolyData {
    let mut new_points = Points::<f64>::new();
    let mut new_polys = CellArray::new();

    // Copy existing points
    for i in 0..pd.points.len() {
        new_points.push(pd.points.get(i));
    }

    // Map from edge (min_idx, max_idx) to midpoint index
    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();
    let mut midpoint_sources: Vec<(usize, usize)> = Vec::new();
    let mut source_cell_ids: Vec<usize> = Vec::new();
    let poly_cell_offset = pd.verts.num_cells() + pd.lines.num_cells();

    let get_midpoint = |a: usize,
                        b: usize,
                        points: &mut Points<f64>,
                        cache: &mut HashMap<(usize, usize), usize>,
                        sources: &mut Vec<(usize, usize)>|
     -> usize {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = cache.get(&key) {
            return idx;
        }
        let pa = pd.points.get(a);
        let pb = pd.points.get(b);
        let mid = [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ];
        let idx = points.len();
        points.push(mid);
        cache.insert(key, idx);
        sources.push(key);
        idx
    };

    for (ci, cell) in pd.polys.iter().enumerate() {
        if cell.len() == 3 {
            let a = cell[0] as usize;
            let b = cell[1] as usize;
            let c = cell[2] as usize;

            let ab = get_midpoint(
                a,
                b,
                &mut new_points,
                &mut midpoint_cache,
                &mut midpoint_sources,
            );
            let bc = get_midpoint(
                b,
                c,
                &mut new_points,
                &mut midpoint_cache,
                &mut midpoint_sources,
            );
            let ca = get_midpoint(
                c,
                a,
                &mut new_points,
                &mut midpoint_cache,
                &mut midpoint_sources,
            );

            // 4 sub-triangles
            new_polys.push_cell(&[a as i64, ab as i64, ca as i64]);
            new_polys.push_cell(&[ab as i64, b as i64, bc as i64]);
            new_polys.push_cell(&[ca as i64, bc as i64, c as i64]);
            new_polys.push_cell(&[ab as i64, bc as i64, ca as i64]);
            source_cell_ids.extend(std::iter::repeat(poly_cell_offset + ci).take(4));
        } else {
            // Non-triangle cells are passed through unchanged
            new_polys.push_cell(&cell.iter().copied().collect::<Vec<_>>());
            source_cell_ids.push(poly_cell_offset + ci);
        }
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;
    *result.point_data_mut() =
        interpolate_point_data(pd.point_data(), pd.points.len(), &midpoint_sources);
    *result.cell_data_mut() = copy_cell_data(pd.cell_data(), &source_cell_ids);
    *result.field_data_mut() = pd.field_data().clone();
    result
}

fn interpolate_point_data(
    input: &DataSetAttributes,
    input_point_count: usize,
    midpoint_sources: &[(usize, usize)],
) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();
    for array in input.iter() {
        if array.num_tuples() < input_point_count {
            continue;
        }
        output.add_array(interpolate_point_array(
            array,
            input_point_count,
            midpoint_sources,
        ));
    }
    copy_active_attributes(input, &mut output);
    output
}

fn interpolate_point_array(
    array: &AnyDataArray,
    input_point_count: usize,
    midpoint_sources: &[(usize, usize)],
) -> AnyDataArray {
    macro_rules! interpolate {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(interpolate_typed_point_array(
                $array,
                input_point_count,
                midpoint_sources,
            ))
        };
    }
    match array {
        AnyDataArray::F32(a) => interpolate!(a, F32),
        AnyDataArray::F64(a) => interpolate!(a, F64),
        AnyDataArray::I8(a) => interpolate!(a, I8),
        AnyDataArray::I16(a) => interpolate!(a, I16),
        AnyDataArray::I32(a) => interpolate!(a, I32),
        AnyDataArray::I64(a) => interpolate!(a, I64),
        AnyDataArray::U8(a) => interpolate!(a, U8),
        AnyDataArray::U16(a) => interpolate!(a, U16),
        AnyDataArray::U32(a) => interpolate!(a, U32),
        AnyDataArray::U64(a) => interpolate!(a, U64),
    }
}

fn interpolate_typed_point_array<T: Scalar>(
    array: &DataArray<T>,
    input_point_count: usize,
    midpoint_sources: &[(usize, usize)],
) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity((input_point_count + midpoint_sources.len()) * nc);
    for i in 0..input_point_count {
        data.extend_from_slice(array.tuple(i));
    }
    for &(a, b) in midpoint_sources {
        let ta = array.tuple(a);
        let tb = array.tuple(b);
        for c in 0..nc {
            data.push(T::from_f64((ta[c].to_f64() + tb[c].to_f64()) * 0.5));
        }
    }
    DataArray::from_vec(array.name(), data, nc)
}

fn copy_cell_data(input: &DataSetAttributes, source_cell_ids: &[usize]) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();
    for array in input.iter() {
        if source_cell_ids.iter().any(|&id| id >= array.num_tuples()) {
            continue;
        }
        output.add_array(copy_cell_array(array, source_cell_ids));
    }
    copy_active_attributes(input, &mut output);
    output
}

fn copy_cell_array(array: &AnyDataArray, source_cell_ids: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(copy_typed_cell_array($array, source_cell_ids))
        };
    }
    match array {
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

fn copy_typed_cell_array<T: Scalar>(
    array: &DataArray<T>,
    source_cell_ids: &[usize],
) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(source_cell_ids.len() * nc);
    for &id in source_cell_ids {
        data.extend_from_slice(array.tuple(id));
    }
    DataArray::from_vec(array.name(), data, nc)
}

fn copy_active_attributes(input: &DataSetAttributes, output: &mut DataSetAttributes) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray, PolyData};

    #[test]
    fn test_subdivide_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = subdivide_for_render(&pd, 1);
        // 1 triangle -> 4 triangles, 3 original + 3 midpoints = 6 points
        assert_eq!(result.polys.num_cells(), 4);
        assert_eq!(result.points.len(), 6);

        // Level 2: 4 triangles -> 16 triangles
        let result2 = subdivide_for_render(&pd, 2);
        assert_eq!(result2.polys.num_cells(), 16);
    }

    #[test]
    fn subdivide_interpolates_point_data_at_midpoints() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "uv",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                2,
            )));
        assert!(pd.point_data_mut().set_active_tcoords("uv"));

        let result = subdivide_for_render(&pd, 1);
        let tcoords = result.point_data().tcoords().unwrap();
        assert_eq!(tcoords.num_tuples(), 6);

        let mut tuple = [0.0; 2];
        tcoords.tuple_as_f64(3, &mut tuple);
        assert_eq!(tuple, [0.5, 0.0]);
    }

    #[test]
    fn subdivide_copies_cell_data_to_child_triangles() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("cid", vec![7], 1)));
        assert!(pd.cell_data_mut().set_active_scalars("cid"));

        let result = subdivide_for_render(&pd, 1);
        let scalars = result.cell_data().scalars().unwrap();
        assert_eq!(scalars.num_tuples(), 4);
        for i in 0..4 {
            let mut tuple = [0.0; 1];
            scalars.tuple_as_f64(i, &mut tuple);
            assert_eq!(tuple[0], 7.0);
        }
    }
}
