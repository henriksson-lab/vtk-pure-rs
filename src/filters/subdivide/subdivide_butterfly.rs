use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashMap;

/// Modified butterfly subdivision.
///
/// Each triangle is split into 4 by inserting edge midpoints, similar to
/// Loop subdivision but using the butterfly stencil for smoother interpolation
/// on irregular meshes. Uses the VTK boundary stencil for boundary edges.
pub fn subdivide_butterfly(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if input.verts.num_cells() != 0 || input.lines.num_cells() != 0 || input.strips.num_cells() != 0
    {
        return input.clone();
    }
    let mut points = input.points.clone();
    let mut new_tris: Vec<[i64; 3]> = Vec::new();
    let mut point_stencils: Vec<PointStencil> = (0..n).map(PointStencil::single).collect();
    let mut source_cell_ids = Vec::new();
    let poly_cell_offset = input.verts.num_cells() + input.lines.num_cells();

    // Build edge-to-face adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut point_faces: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut tris: Vec<[i64; 3]> = Vec::new();

    for cell in input.polys.iter() {
        let Some(tri) = valid_triangle_point_ids(cell, n) else {
            return input.clone();
        };
        let fi = tris.len();
        for &pt_id in &tri {
            point_faces[pt_id as usize].push(fi);
        }
        tris.push(tri);
    }

    for (fi, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }
    if edge_faces.values().any(|faces| faces.len() > 2) {
        return input.clone();
    }

    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

    for (cell_index, tri) in tris.iter().enumerate() {
        let a = tri[0];
        let b = tri[1];
        let c = tri[2];

        let ab = get_butterfly_midpoint(
            &mut points,
            &mut midpoint_cache,
            &mut point_stencils,
            &edge_faces,
            &point_faces,
            &tris,
            a,
            b,
        );
        let bc = get_butterfly_midpoint(
            &mut points,
            &mut midpoint_cache,
            &mut point_stencils,
            &edge_faces,
            &point_faces,
            &tris,
            b,
            c,
        );
        let ca = get_butterfly_midpoint(
            &mut points,
            &mut midpoint_cache,
            &mut point_stencils,
            &edge_faces,
            &point_faces,
            &tris,
            c,
            a,
        );

        new_tris.push([a, ab, ca]);
        new_tris.push([ab, b, bc]);
        new_tris.push([bc, c, ca]);
        new_tris.push([ab, bc, ca]);
        source_cell_ids.extend(std::iter::repeat(poly_cell_offset + cell_index).take(4));
    }

    let mut polys = CellArray::new();
    for tri in &new_tris {
        polys.push_cell(&[tri[0], tri[1], tri[2]]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    *pd.point_data_mut() = interpolate_point_data(input.point_data(), n, &point_stencils);
    *pd.cell_data_mut() = copy_cell_data(input.cell_data(), &source_cell_ids);
    *pd.field_data_mut() = input.field_data().clone();
    pd
}

fn get_butterfly_midpoint(
    points: &mut Points<f64>,
    cache: &mut HashMap<(i64, i64), i64>,
    point_stencils: &mut Vec<PointStencil>,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    point_faces: &[Vec<usize>],
    tris: &[[i64; 3]],
    a: i64,
    b: i64,
) -> i64 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&mid) = cache.get(&key) {
        return mid;
    }

    let pa = points.get(a as usize);
    let pb = points.get(b as usize);

    let faces = edge_faces.get(&key);
    let is_boundary = faces.map(|f| f.len()).unwrap_or(0) < 2;

    let (mid_pt, stencil) = if is_boundary {
        match boundary_stencil(a, b, edge_faces, point_faces, tris) {
            Some(ids) => {
                let weights = [-0.0625, 0.5625, 0.5625, -0.0625];
                (
                    weighted_point(points, &ids, &weights),
                    PointStencil::from_i64_ids(&ids, &weights),
                )
            }
            None => (
                [
                    (pa[0] + pb[0]) * 0.5,
                    (pa[1] + pb[1]) * 0.5,
                    (pa[2] + pb[2]) * 0.5,
                ],
                PointStencil {
                    sources: vec![a as usize, b as usize],
                    weights: vec![0.5, 0.5],
                },
            ),
        }
    } else {
        let faces = faces.unwrap();
        let valence_a = point_faces[a as usize].len();
        let valence_b = point_faces[b as usize].len();

        if valence_a == 6 && valence_b == 6 {
            let ids = butterfly_stencil(a, b, faces[0], faces[1], edge_faces, tris);
            let weights = [0.5, 0.5, 0.125, 0.125, -0.0625, -0.0625, -0.0625, -0.0625];
            (
                weighted_point(points, &ids, &weights),
                PointStencil::from_i64_ids(&ids, &weights),
            )
        } else if valence_a == 6 {
            let stencil = loop_stencil(b, a, edge_faces, tris);
            (
                weighted_stencil_point(points, &stencil),
                PointStencil::from_weighted_i64(&stencil),
            )
        } else if valence_b == 6 {
            let stencil = loop_stencil(a, b, edge_faces, tris);
            (
                weighted_stencil_point(points, &stencil),
                PointStencil::from_weighted_i64(&stencil),
            )
        } else {
            let mut stencil = loop_stencil(b, a, edge_faces, tris);
            stencil.extend(loop_stencil(a, b, edge_faces, tris));
            for (_, weight) in &mut stencil {
                *weight *= 0.5;
            }
            (
                weighted_stencil_point(points, &stencil),
                PointStencil::from_weighted_i64(&stencil),
            )
        }
    };

    let idx = points.len() as i64;
    points.push(mid_pt);
    point_stencils.push(stencil);
    cache.insert(key, idx);
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

    fn from_i64_ids(ids: &[i64], weights: &[f64]) -> Self {
        Self {
            sources: ids.iter().map(|&id| id as usize).collect(),
            weights: weights.iter().copied().take(ids.len()).collect(),
        }
    }

    fn from_weighted_i64(stencil: &[(i64, f64)]) -> Self {
        Self {
            sources: stencil.iter().map(|&(id, _)| id as usize).collect(),
            weights: stencil.iter().map(|&(_, weight)| weight).collect(),
        }
    }
}

fn valid_triangle_point_ids(cell: &[i64], n_points: usize) -> Option<[i64; 3]> {
    if cell.len() != 3 {
        return None;
    }
    for &id in cell {
        usize::try_from(id).ok().filter(|&idx| idx < n_points)?;
    }
    Some([cell[0], cell[1], cell[2]])
}

fn boundary_stencil(
    a: i64,
    b: i64,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    point_faces: &[Vec<usize>],
    tris: &[[i64; 3]],
) -> Option<Vec<i64>> {
    let p0 = other_boundary_neighbor(a, b, None, edge_faces, point_faces, tris)?;
    let mut ids = vec![p0, a, b];
    if let Some(p3) = other_boundary_neighbor(b, a, Some(p0), edge_faces, point_faces, tris) {
        ids.push(p3);
    }
    Some(ids)
}

fn other_boundary_neighbor(
    point: i64,
    edge_other: i64,
    skip: Option<i64>,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    point_faces: &[Vec<usize>],
    tris: &[[i64; 3]],
) -> Option<i64> {
    for &fi in &point_faces[point as usize] {
        for &candidate in &tris[fi] {
            if candidate == point || candidate == edge_other || Some(candidate) == skip {
                continue;
            }
            let key = edge_key_i64(point, candidate);
            if edge_faces.get(&key).map(|faces| faces.len()) == Some(1) {
                return Some(candidate);
            }
        }
    }
    None
}

fn butterfly_stencil(
    a: i64,
    b: i64,
    cell0: usize,
    cell1: usize,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    tris: &[[i64; 3]],
) -> [i64; 8] {
    let p3 = opposite_vertex(&tris[cell0], a, b);
    let p4 = opposite_vertex(&tris[cell1], a, b);
    let p5 = opposite_across(cell0, a, p3, edge_faces, tris).unwrap_or(p4);
    let p6 = opposite_across(cell0, b, p3, edge_faces, tris).unwrap_or(p4);
    let p7 = opposite_across(cell1, a, p4, edge_faces, tris).unwrap_or(p3);
    let p8 = opposite_across(cell1, b, p4, edge_faces, tris).unwrap_or(p3);
    [a, b, p3, p4, p5, p6, p7, p8]
}

fn loop_stencil(
    p1: i64,
    p2: i64,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    tris: &[[i64; 3]],
) -> Vec<(i64, f64)> {
    let Some(cell_ids) = edge_faces.get(&edge_key_i64(p1, p2)) else {
        return vec![(p2, 0.25), (p1, 0.75)];
    };
    if cell_ids.len() < 2 {
        return vec![(p2, 0.25), (p1, 0.75)];
    }

    let start_cell = cell_ids[0];
    let mut next_cell = cell_ids[1];
    let mut tp2 = p2;
    let mut stencil_ids = vec![p2];
    let mut shifts = vec![0i32];
    let mut processed = 0i32;
    let mut boundary = false;

    while next_cell != start_cell {
        let p = opposite_vertex(&tris[next_cell], p1, tp2);
        tp2 = p;
        stencil_ids.push(tp2);
        processed += 1;
        shifts.push(processed);

        let neighbors = edge_faces
            .get(&edge_key_i64(p1, tp2))
            .map(|faces| {
                faces
                    .iter()
                    .copied()
                    .filter(|&face| face != next_cell)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if neighbors.len() != 1 {
            boundary = true;
            break;
        }
        next_cell = neighbors[0];
    }

    if boundary {
        let ids = butterfly_stencil(p1, p2, start_cell, cell_ids[1], edge_faces, tris);
        return ids
            .into_iter()
            .zip([0.5, 0.5, 0.125, 0.125, -0.0625, -0.0625, -0.0625, -0.0625])
            .collect();
    }

    let k = stencil_ids.len();
    let mut weights = vec![0.0; k];
    if k >= 5 {
        for (j, weight) in weights.iter_mut().enumerate() {
            let shift = shifts[j] as f64;
            *weight = (0.25
                + (2.0 * std::f64::consts::PI * shift / k as f64).cos()
                + 0.5 * (4.0 * std::f64::consts::PI * shift / k as f64).cos())
                / k as f64;
        }
    } else if k == 4 {
        let weights4 = [3.0 / 8.0, 0.0, -1.0 / 8.0, 0.0];
        for (j, weight) in weights.iter_mut().enumerate() {
            *weight = weights4[shifts[j].unsigned_abs() as usize];
        }
    } else if k == 3 {
        weights.copy_from_slice(&[5.0 / 12.0, -1.0 / 12.0, -1.0 / 12.0]);
    } else {
        let p = opposite_vertex(&tris[start_cell], p1, p2);
        stencil_ids.push(p);
        weights = vec![5.0 / 12.0, -1.0 / 12.0, -1.0 / 12.0];
    }

    let mut stencil: Vec<(i64, f64)> = stencil_ids.into_iter().zip(weights).collect();
    stencil.push((p1, 0.75));
    stencil
}

fn opposite_across(
    excluded_cell: usize,
    a: i64,
    b: i64,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    tris: &[[i64; 3]],
) -> Option<i64> {
    let key = edge_key_i64(a, b);
    edge_faces.get(&key).and_then(|faces| {
        faces
            .iter()
            .copied()
            .find(|&fi| fi != excluded_cell)
            .map(|fi| opposite_vertex(&tris[fi], a, b))
    })
}

fn opposite_vertex(tri: &[i64; 3], a: i64, b: i64) -> i64 {
    tri.iter()
        .copied()
        .find(|&v| v != a && v != b)
        .unwrap_or(tri[0])
}

fn weighted_point(points: &Points<f64>, ids: &[i64], weights: &[f64]) -> [f64; 3] {
    let mut out = [0.0; 3];
    for (&id, &weight) in ids.iter().zip(weights) {
        let p = points.get(id as usize);
        out[0] += weight * p[0];
        out[1] += weight * p[1];
        out[2] += weight * p[2];
    }
    out
}

fn weighted_stencil_point(points: &Points<f64>, stencil: &[(i64, f64)]) -> [f64; 3] {
    let mut out = [0.0; 3];
    for &(id, weight) in stencil {
        let p = points.get(id as usize);
        out[0] += weight * p[0];
        out[1] += weight * p[1];
        out[2] += weight * p[2];
    }
    out
}

fn edge_key_i64(a: i64, b: i64) -> (i64, i64) {
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
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_butterfly(&pd);
        assert_eq!(result.points.len(), 6); // 3 + 3 midpoints
        assert_eq!(result.polys.num_cells(), 4); // 1 -> 4
    }

    #[test]
    fn subdivide_two_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = subdivide_butterfly(&pd);
        assert_eq!(result.polys.num_cells(), 8); // 2 -> 8
                                                 // Shared edge midpoint should be reused
        assert!(result.points.len() < 12); // would be 12 if no sharing
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = subdivide_butterfly(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
