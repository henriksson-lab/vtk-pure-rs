use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Midpoint subdivision: split each triangle into 4 by inserting
/// midpoints on each edge.
///
/// This is simpler than Loop subdivision — it doesn't reposition
/// existing vertices, just splits edges at their midpoints.
/// Good for increasing mesh resolution without smoothing.
pub fn subdivide_midpoint(input: &PolyData) -> PolyData {
    if input.verts.num_cells() != 0 || input.lines.num_cells() != 0 || input.strips.num_cells() != 0
    {
        return input.clone();
    }
    let mut out_points = input.points.clone();
    let mut out_polys = CellArray::new();
    let input_point_count = input.points.len();
    let mut point_stencils: Vec<PointStencil> =
        (0..input_point_count).map(PointStencil::single).collect();
    let mut source_cell_ids = Vec::new();
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();
    let mut edge_counts: HashMap<(i64, i64), usize> = HashMap::new();

    for cell in input.polys.iter() {
        let Some([a, b, c]) = valid_triangle_point_ids(cell, input_point_count) else {
            return input.clone();
        };
        for (u, v) in [(a, b), (b, c), (c, a)] {
            *edge_counts.entry(edge_key(u as i64, v as i64)).or_default() += 1;
        }
    }
    if edge_counts.values().any(|&count| count > 2) {
        return input.clone();
    }

    // Cache midpoints to avoid duplicates on shared edges
    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

    let get_midpoint = |a: i64,
                        b: i64,
                        pts: &mut Points<f64>,
                        cache: &mut HashMap<(i64, i64), i64>,
                        stencils: &mut Vec<PointStencil>|
     -> i64 {
        let key = edge_key(a, b);
        if let Some(&mid) = cache.get(&key) {
            return mid;
        }
        let pa = pts.get(a as usize);
        let pb = pts.get(b as usize);
        let idx = pts.len() as i64;
        pts.push([
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ]);
        cache.insert(key, idx);
        stencils.push(PointStencil {
            sources: vec![a as usize, b as usize],
            weights: vec![0.5, 0.5],
        });
        idx
    };

    for (cell_index, cell) in input.polys.iter().enumerate() {
        let [a, b, c] = valid_triangle_point_ids(cell, input_point_count).unwrap();
        let a = a as i64;
        let b = b as i64;
        let c = c as i64;

        let ab = get_midpoint(
            a,
            b,
            &mut out_points,
            &mut midpoint_cache,
            &mut point_stencils,
        );
        let bc = get_midpoint(
            b,
            c,
            &mut out_points,
            &mut midpoint_cache,
            &mut point_stencils,
        );
        let ca = get_midpoint(
            c,
            a,
            &mut out_points,
            &mut midpoint_cache,
            &mut point_stencils,
        );

        out_polys.push_cell(&[a, ab, ca]);
        out_polys.push_cell(&[ab, b, bc]);
        out_polys.push_cell(&[bc, c, ca]);
        out_polys.push_cell(&[ab, bc, ca]);
        source_cell_ids.extend(std::iter::repeat(poly_cell_offset + cell_index).take(4));
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    *pd.point_data_mut() =
        interpolate_point_data(input.point_data(), input_point_count, &point_stencils);
    *pd.cell_data_mut() = copy_cell_data(input.cell_data(), &source_cell_ids);
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

#[derive(Debug, Clone)]
struct PointStencil {
    sources: Vec<usize>,
    weights: Vec<f64>,
}

impl PointStencil {
    fn single(source: usize) -> Self {
        Self {
            sources: vec![source],
            weights: vec![1.0],
        }
    }
}

fn valid_triangle_point_ids(cell: &[i64], n_points: usize) -> Option<[usize; 3]> {
    if cell.len() != 3 {
        return None;
    }
    Some([
        valid_point_index(cell[0], n_points)?,
        valid_point_index(cell[1], n_points)?,
        valid_point_index(cell[2], n_points)?,
    ])
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < n_points)
}

fn edge_key(a: i64, b: i64) -> (i64, i64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn interpolate_point_data(
    input: &DataSetAttributes,
    input_point_count: usize,
    stencils: &[PointStencil],
) -> DataSetAttributes {
    let mut output = DataSetAttributes::new();
    for array in input.iter() {
        if array.num_tuples() < input_point_count {
            continue;
        }
        output.add_array(interpolate_point_array(array, stencils));
    }
    copy_active_attributes(input, &mut output);
    output
}

fn interpolate_point_array(array: &AnyDataArray, stencils: &[PointStencil]) -> AnyDataArray {
    macro_rules! interpolate {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(interpolate_typed_point_array($array, stencils))
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
    stencils: &[PointStencil],
) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(stencils.len() * nc);
    for stencil in stencils {
        for component in 0..nc {
            let mut value = 0.0;
            for (&source, &weight) in stencil.sources.iter().zip(stencil.weights.iter()) {
                value += array.tuple(source)[component].to_f64() * weight;
            }
            data.push(T::from_f64(value));
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

    #[test]
    fn subdivide_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_midpoint(&pd);
        // 3 original + 3 midpoints = 6 points
        assert_eq!(result.points.len(), 6);
        // 4 sub-triangles
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn shared_edge_reuses_midpoint() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = subdivide_midpoint(&pd);
        // 4 original + 5 midpoints (shared edge 1-2 produces 1 midpoint) = 9
        assert_eq!(result.points.len(), 9);
        assert_eq!(result.polys.num_cells(), 8);
    }

    #[test]
    fn midpoint_requires_triangle_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = subdivide_midpoint(&pd);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 1);
    }
}
