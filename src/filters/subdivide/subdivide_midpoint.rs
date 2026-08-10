use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

type EdgeMap = HashMap<u64, EdgeInfo, BuildHasherDefault<U64FastHasher>>;

#[derive(Default)]
struct U64FastHasher(u64);

impl Hasher for U64FastHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        mix_u64(self.0)
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        let mut value = 0u64;
        for (shift, &byte) in bytes.iter().take(8).enumerate() {
            value |= (byte as u64) << (shift * 8);
        }
        self.0 = value;
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}

#[inline(always)]
fn mix_u64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Clone, Copy)]
struct EdgeInfo {
    count: u8,
    midpoint: i64,
}

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
    let input_point_count = input.points.len();
    let has_point_data = input.point_data().num_arrays() != 0;
    let has_cell_data = input.cell_data().num_arrays() != 0;
    let mut point_stencils: Option<Vec<PointStencil>> =
        has_point_data.then(|| (0..input_point_count).map(PointStencil::single).collect());
    let mut source_cell_ids = has_cell_data.then(Vec::new);
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();
    let mut edges: EdgeMap = HashMap::with_capacity_and_hasher(
        input.polys.num_cells() * 3,
        BuildHasherDefault::<U64FastHasher>::default(),
    );
    let mut triangles = Vec::with_capacity(input.polys.num_cells());

    for cell in input.polys.iter() {
        let Some([a, b, c]) = valid_triangle_point_ids(cell, input_point_count) else {
            return input.clone();
        };
        triangles.push([a, b, c]);
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let info = edges
                .entry(edge_key(u, v, input_point_count))
                .or_insert(EdgeInfo {
                    count: 0,
                    midpoint: -1,
                });
            info.count += 1;
            if info.count > 2 {
                return input.clone();
            }
        }
    }

    let mut offsets = Vec::with_capacity(triangles.len() * 4 + 1);
    let mut conn = Vec::with_capacity(triangles.len() * 12);
    offsets.push(0);

    for (cell_index, &[a, b, c]) in triangles.iter().enumerate() {
        let ab = midpoint_for_edge(
            a,
            b,
            input,
            &mut out_points,
            &mut edges,
            input_point_count,
            point_stencils.as_mut(),
        );
        let bc = midpoint_for_edge(
            b,
            c,
            input,
            &mut out_points,
            &mut edges,
            input_point_count,
            point_stencils.as_mut(),
        );
        let ca = midpoint_for_edge(
            c,
            a,
            input,
            &mut out_points,
            &mut edges,
            input_point_count,
            point_stencils.as_mut(),
        );

        let a = a as i64;
        let b = b as i64;
        let c = c as i64;
        conn.extend_from_slice(&[a, ab, ca, ab, b, bc, bc, c, ca, ab, bc, ca]);
        let base = conn.len() as i64 - 12;
        offsets.extend_from_slice(&[base + 3, base + 6, base + 9, base + 12]);
        if let Some(source_cell_ids) = &mut source_cell_ids {
            source_cell_ids.extend(std::iter::repeat(poly_cell_offset + cell_index).take(4));
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = CellArray::from_raw(offsets, conn);
    if let Some(point_stencils) = point_stencils {
        *pd.point_data_mut() =
            interpolate_point_data(input.point_data(), input_point_count, &point_stencils);
    }
    if let Some(source_cell_ids) = source_cell_ids {
        *pd.cell_data_mut() = copy_cell_data(input.cell_data(), &source_cell_ids);
    }
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

fn midpoint_for_edge(
    a: usize,
    b: usize,
    input: &PolyData,
    pts: &mut Points<f64>,
    edges: &mut EdgeMap,
    point_count: usize,
    stencils: Option<&mut Vec<PointStencil>>,
) -> i64 {
    let key = edge_key(a, b, point_count);
    let info = edges.get_mut(&key).unwrap();
    if info.midpoint >= 0 {
        return info.midpoint;
    }

    let pa = input.points.get(a);
    let pb = input.points.get(b);
    let idx = pts.len() as i64;
    pts.push([
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ]);
    info.midpoint = idx;
    if let Some(stencils) = stencils {
        stencils.push(PointStencil {
            sources: vec![a, b],
            weights: vec![0.5, 0.5],
        });
    }
    idx
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

fn edge_key(a: usize, b: usize, point_count: usize) -> u64 {
    if a < b {
        a as u64 * point_count as u64 + b as u64
    } else {
        b as u64 * point_count as u64 + a as u64
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
