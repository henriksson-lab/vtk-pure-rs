use std::collections::{BTreeMap, BTreeSet};

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Subdivide a triangle mesh using Loop subdivision.
///
/// Each triangle is split into 4 sub-triangles. Edge midpoints are computed as
/// weighted averages of the edge endpoints and opposite vertices (for interior edges)
/// or simple midpoints (for boundary edges). Existing vertices are repositioned
/// using their valence-weighted neighbors.
///
/// Input must be a triangle mesh. Run `triangulate` first if needed.
pub fn subdivide(input: &PolyData, iterations: usize) -> PolyData {
    let mut current = input.clone();

    for _ in 0..iterations {
        current = subdivide_once(&current);
    }

    current
}

fn subdivide_once(input: &PolyData) -> PolyData {
    let n_pts = input.points.len();
    if input.verts.num_cells() != 0 || input.lines.num_cells() != 0 || input.strips.num_cells() != 0
    {
        return input.clone();
    }
    if input.polys.iter().any(|cell| cell.len() != 3) {
        return input.clone();
    }

    let mut neighbors: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n_pts];
    let mut boundary_neighbors: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n_pts];
    let mut edge_opposite: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    let mut triangles = Vec::new();
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();
    for cell in input.polys.iter() {
        let Some(t) = valid_triangle_point_ids(cell, n_pts) else {
            return input.clone();
        };
        for j in 0..3 {
            let a = t[j];
            let b = t[(j + 1) % 3];
            let opp = t[(j + 2) % 3];
            neighbors[a].insert(b);
            neighbors[b].insert(a);
            let edge = if a < b { (a, b) } else { (b, a) };
            edge_opposite.entry(edge).or_default().push(opp);
        }
        triangles.push(t);
    }
    if edge_opposite.values().any(|opposites| opposites.len() > 2) {
        return input.clone();
    }
    for (&(a, b), opposites) in &edge_opposite {
        if opposites.len() == 1 {
            boundary_neighbors[a].insert(b);
            boundary_neighbors[b].insert(a);
        }
    }

    let mut new_points = Points::<f64>::new();
    let mut point_stencils = Vec::new();

    for i in 0..n_pts {
        let p = input.points.get(i);
        let n = neighbors[i].len();
        if n == 0 {
            new_points.push(p);
            point_stencils.push(PointStencil::single(i));
            continue;
        }

        if boundary_neighbors[i].len() >= 2 {
            let mut iter = boundary_neighbors[i].iter();
            let b0_id = *iter.next().unwrap();
            let b1_id = *iter.next().unwrap();
            let b0 = input.points.get(b0_id);
            let b1 = input.points.get(b1_id);
            new_points.push([
                0.75 * p[0] + 0.125 * (b0[0] + b1[0]),
                0.75 * p[1] + 0.125 * (b0[1] + b1[1]),
                0.75 * p[2] + 0.125 * (b0[2] + b1[2]),
            ]);
            point_stencils.push(PointStencil {
                sources: vec![b0_id, b1_id, i],
                weights: vec![0.125, 0.125, 0.75],
            });
            continue;
        }

        let beta = if n == 3 {
            3.0 / 16.0
        } else {
            let cos_sq = 0.375 + 0.25 * (2.0 * std::f64::consts::PI / n as f64).cos();
            (0.625 - cos_sq * cos_sq) / n as f64
        };

        let mut avg = [0.0f64; 3];
        for &nb in &neighbors[i] {
            let q = input.points.get(nb);
            avg[0] += q[0];
            avg[1] += q[1];
            avg[2] += q[2];
        }

        let new_pos = [
            (1.0 - n as f64 * beta) * p[0] + beta * avg[0],
            (1.0 - n as f64 * beta) * p[1] + beta * avg[1],
            (1.0 - n as f64 * beta) * p[2] + beta * avg[2],
        ];
        new_points.push(new_pos);
        let mut sources = Vec::with_capacity(n + 1);
        let mut weights = Vec::with_capacity(n + 1);
        for &nb in &neighbors[i] {
            sources.push(nb);
            weights.push(beta);
        }
        sources.push(i);
        weights.push(1.0 - n as f64 * beta);
        point_stencils.push(PointStencil { sources, weights });
    }

    let mut edge_midpoints: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for (&(a, b), opposites) in &edge_opposite {
        let pa = input.points.get(a);
        let pb = input.points.get(b);

        let mid = if opposites.len() == 2 {
            let pc = input.points.get(opposites[0]);
            let pd = input.points.get(opposites[1]);
            [
                3.0 / 8.0 * (pa[0] + pb[0]) + 1.0 / 8.0 * (pc[0] + pd[0]),
                3.0 / 8.0 * (pa[1] + pb[1]) + 1.0 / 8.0 * (pc[1] + pd[1]),
                3.0 / 8.0 * (pa[2] + pb[2]) + 1.0 / 8.0 * (pc[2] + pd[2]),
            ]
        } else {
            [
                0.5 * (pa[0] + pb[0]),
                0.5 * (pa[1] + pb[1]),
                0.5 * (pa[2] + pb[2]),
            ]
        };
        let stencil = if opposites.len() == 2 {
            PointStencil {
                sources: vec![a, b, opposites[0], opposites[1]],
                weights: vec![3.0 / 8.0, 3.0 / 8.0, 1.0 / 8.0, 1.0 / 8.0],
            }
        } else {
            PointStencil {
                sources: vec![a, b],
                weights: vec![0.5, 0.5],
            }
        };

        let idx = new_points.len();
        new_points.push(mid);
        point_stencils.push(stencil);
        edge_midpoints.insert((a, b), idx);
    }

    // Generate 4 sub-triangles per original triangle
    let mut polys = CellArray::new();
    let mut source_cell_ids = Vec::with_capacity(triangles.len() * 4);
    for (cell_index, tri) in triangles.iter().enumerate() {
        let [v0, v1, v2] = *tri;

        let m01 = *edge_midpoints.get(&edge_key(v0, v1)).unwrap();
        let m12 = *edge_midpoints.get(&edge_key(v1, v2)).unwrap();
        let m20 = *edge_midpoints.get(&edge_key(v2, v0)).unwrap();

        polys.push_cell(&[v0 as i64, m01 as i64, m20 as i64]);
        polys.push_cell(&[m01 as i64, v1 as i64, m12 as i64]);
        polys.push_cell(&[m12 as i64, v2 as i64, m20 as i64]);
        polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
        source_cell_ids.extend(std::iter::repeat(poly_cell_offset + cell_index).take(4));
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.polys = polys;
    *output.point_data_mut() = interpolate_point_data(input.point_data(), n_pts, &point_stencils);
    *output.cell_data_mut() = copy_cell_data(input.cell_data(), &source_cell_ids);
    *output.field_data_mut() = input.field_data().clone();
    output
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

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn valid_triangle_point_ids(cell: &[i64], n_points: usize) -> Option<[usize; 3]> {
    if cell.len() != 3 {
        return None;
    }
    Some([
        usize::try_from(cell[0])
            .ok()
            .filter(|&idx| idx < n_points)?,
        usize::try_from(cell[1])
            .ok()
            .filter(|&idx| idx < n_points)?,
        usize::try_from(cell[2])
            .ok()
            .filter(|&idx| idx < n_points)?,
    ])
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
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = subdivide(&pd, 1);
        // 1 triangle → 4 triangles, 3 original + 3 midpoints = 6 points
        assert_eq!(result.polys.num_cells(), 4);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn subdivide_two_iterations() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = subdivide(&pd, 2);
        // 1 → 4 → 16 triangles
        assert_eq!(result.polys.num_cells(), 16);
    }

    #[test]
    fn subdivide_preserves_manifold() {
        // Two triangles sharing an edge
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );

        let result = subdivide(&pd, 1);
        // 2 triangles → 8
        assert_eq!(result.polys.num_cells(), 8);
    }
}
