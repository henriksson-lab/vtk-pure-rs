//! Loop subdivision (approximating smooth subdivision).
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use std::collections::{BTreeMap, HashMap};
pub fn loop_subdivide(mesh: &PolyData) -> PolyData {
    loop_subdivide_n(mesh, 1)
}
pub fn loop_subdivide_n(mesh: &PolyData, n: usize) -> PolyData {
    let mut current = mesh.clone();
    for _ in 0..n {
        current = loop_once(&current);
    }
    current
}
fn loop_once(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for (ci, c) in cells.iter().enumerate() {
        if !valid_triangle(c, n) {
            continue;
        }
        let nc = c.len();
        for i in 0..nc {
            let a = c[i] as usize;
            let b = c[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    // Edge points
    let mut em: HashMap<(usize, usize), usize> = HashMap::new();
    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let mut point_stencils: Vec<Vec<(usize, f64)>> = (0..n).map(|i| vec![(i, 1.0)]).collect();
    for (&(a, b), faces) in &ef {
        let pa = pts[a];
        let pb = pts[b];
        let (mid, stencil) = if faces.len() == 2 {
            // Get opposite vertices
            let opp: Vec<usize> = faces
                .iter()
                .filter_map(|&fi| {
                    cells[fi]
                        .iter()
                        .find(|&&v| v as usize != a && v as usize != b)
                        .map(|&v| v as usize)
                })
                .collect();
            if opp.len() == 2 {
                let po0 = pts[opp[0]];
                let po1 = pts[opp[1]];
                (
                    [
                        (pa[0] + pb[0]) * 3.0 / 8.0 + (po0[0] + po1[0]) / 8.0,
                        (pa[1] + pb[1]) * 3.0 / 8.0 + (po0[1] + po1[1]) / 8.0,
                        (pa[2] + pb[2]) * 3.0 / 8.0 + (po0[2] + po1[2]) / 8.0,
                    ],
                    vec![
                        (a, 3.0 / 8.0),
                        (b, 3.0 / 8.0),
                        (opp[0], 1.0 / 8.0),
                        (opp[1], 1.0 / 8.0),
                    ],
                )
            } else {
                (
                    [
                        (pa[0] + pb[0]) / 2.0,
                        (pa[1] + pb[1]) / 2.0,
                        (pa[2] + pb[2]) / 2.0,
                    ],
                    vec![(a, 0.5), (b, 0.5)],
                )
            }
        } else {
            (
                [
                    (pa[0] + pb[0]) / 2.0,
                    (pa[1] + pb[1]) / 2.0,
                    (pa[2] + pb[2]) / 2.0,
                ],
                vec![(a, 0.5), (b, 0.5)],
            )
        };
        let idx = pts.len();
        pts.push(mid);
        point_stencils.push(stencil);
        em.insert((a, b), idx);
    }
    // Update original vertices
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in &cells {
        if !valid_triangle(cell, n) {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if !nb[a].contains(&b) {
                nb[a].push(b);
            }
            if !nb[b].contains(&a) {
                nb[b].push(a);
            }
        }
    }
    let mut new_pts = pts.clone();
    for i in 0..n {
        let k = nb[i].len();
        let boundary_neighbors: Vec<usize> = ef
            .iter()
            .filter_map(|(&(a, b), faces)| {
                if faces.len() == 1 {
                    if a == i {
                        Some(b)
                    } else if b == i {
                        Some(a)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        if boundary_neighbors.len() == 2 {
            let b0 = boundary_neighbors[0];
            let b1 = boundary_neighbors[1];
            new_pts[i] = [
                0.75 * pts[i][0] + 0.125 * (pts[b0][0] + pts[b1][0]),
                0.75 * pts[i][1] + 0.125 * (pts[b0][1] + pts[b1][1]),
                0.75 * pts[i][2] + 0.125 * (pts[b0][2] + pts[b1][2]),
            ];
            point_stencils[i] = vec![(b0, 0.125), (b1, 0.125), (i, 0.75)];
            continue;
        }
        if k < 3 {
            continue;
        }
        let beta = if k == 3 {
            3.0 / 16.0
        } else {
            let cos_sq = 0.375 + 0.25 * (2.0 * std::f64::consts::PI / k as f64).cos();
            (0.625 - cos_sq * cos_sq) / k as f64
        };
        let mut avg = [0.0, 0.0, 0.0];
        for &j in &nb[i] {
            avg[0] += pts[j][0];
            avg[1] += pts[j][1];
            avg[2] += pts[j][2];
        }
        new_pts[i] = [
            (1.0 - k as f64 * beta) * pts[i][0] + beta * avg[0],
            (1.0 - k as f64 * beta) * pts[i][1] + beta * avg[1],
            (1.0 - k as f64 * beta) * pts[i][2] + beta * avg[2],
        ];
        point_stencils[i] = nb[i]
            .iter()
            .map(|&j| (j, beta))
            .chain(std::iter::once((i, 1.0 - k as f64 * beta)))
            .collect();
    }
    // Build new triangles
    let mut new_polys = CellArray::new();
    for c in &cells {
        if !valid_triangle(c, n) {
            continue;
        }
        let v = [c[0] as usize, c[1] as usize, c[2] as usize];
        let m01 = em[&(v[0].min(v[1]), v[0].max(v[1]))];
        let m12 = em[&(v[1].min(v[2]), v[1].max(v[2]))];
        let m20 = em[&(v[2].min(v[0]), v[2].max(v[0]))];
        new_polys.push_cell(&[v[0] as i64, m01 as i64, m20 as i64]);
        new_polys.push_cell(&[v[1] as i64, m12 as i64, m01 as i64]);
        new_polys.push_cell(&[v[2] as i64, m20 as i64, m12 as i64]);
        new_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
    }
    let mut mesh_pts = Points::<f64>::new();
    for p in &new_pts {
        mesh_pts.push(*p);
    }
    let mut r = PolyData::new();
    r.points = mesh_pts;
    r.polys = new_polys;
    *r.field_data_mut() = mesh.field_data().clone();
    interpolate_point_data(mesh.point_data(), r.point_data_mut(), &point_stencils);
    r
}

fn valid_triangle(cell: &[i64], n: usize) -> bool {
    cell.len() == 3 && cell.iter().all(|&v| v >= 0 && (v as usize) < n)
}

fn interpolate_point_data(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    stencils: &[Vec<(usize, f64)>],
) {
    for array in source.iter() {
        if stencils
            .iter()
            .flatten()
            .all(|&(idx, _)| idx < array.num_tuples())
        {
            target.add_array(interpolate_array(array, stencils));
        }
    }
    copy_active_attributes(source, target);
}

fn interpolate_array(array: &AnyDataArray, stencils: &[Vec<(usize, f64)>]) -> AnyDataArray {
    macro_rules! interp {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(interpolate_typed_array($array, stencils))
        };
    }
    match array {
        AnyDataArray::F32(a) => interp!(a, F32),
        AnyDataArray::F64(a) => interp!(a, F64),
        AnyDataArray::I8(a) => interp!(a, I8),
        AnyDataArray::I16(a) => interp!(a, I16),
        AnyDataArray::I32(a) => interp!(a, I32),
        AnyDataArray::I64(a) => interp!(a, I64),
        AnyDataArray::U8(a) => interp!(a, U8),
        AnyDataArray::U16(a) => interp!(a, U16),
        AnyDataArray::U32(a) => interp!(a, U32),
        AnyDataArray::U64(a) => interp!(a, U64),
    }
}

fn interpolate_typed_array<T: Scalar>(
    array: &DataArray<T>,
    stencils: &[Vec<(usize, f64)>],
) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(stencils.len() * num_components);
    for stencil in stencils {
        for component in 0..num_components {
            let mut value = 0.0;
            for &(idx, weight) in stencil {
                value += array.tuple(idx)[component].to_f64() * weight;
            }
            data.push(T::from_f64(value));
        }
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn copy_active_attributes(source: &DataSetAttributes, target: &mut DataSetAttributes) {
    if let Some(array) = source.scalars() {
        target.set_active_scalars(array.name());
    }
    if let Some(array) = source.vectors() {
        target.set_active_vectors(array.name());
    }
    if let Some(array) = source.normals() {
        target.set_active_normals(array.name());
    }
    if let Some(array) = source.tcoords() {
        target.set_active_tcoords(array.name());
    }
    if let Some(array) = source.tensors() {
        target.set_active_tensors(array.name());
    }
    if let Some(array) = source.global_ids() {
        target.set_active_global_ids(array.name());
    }
    if let Some(array) = source.pedigree_ids() {
        target.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = source.edge_flags() {
        target.set_active_edge_flags(array.name());
    }
    if let Some(array) = source.tangents() {
        target.set_active_tangents(array.name());
    }
    if let Some(array) = source.rational_weights() {
        target.set_active_rational_weights(array.name());
    }
    if let Some(array) = source.higher_order_degrees() {
        target.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = source.process_ids() {
        target.set_active_process_ids(array.name());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::AnyDataArray;
    #[test]
    fn test_once() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = loop_subdivide(&m);
        assert_eq!(r.polys.num_cells(), 4);
    }
    #[test]
    fn test_twice() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = loop_subdivide_n(&m, 2);
        assert_eq!(r.polys.num_cells(), 16);
    }

    #[test]
    fn point_data_is_interpolated_with_loop_stencils() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "value",
                vec![0.0, 2.0, 4.0],
                1,
            )));
        m.point_data_mut().set_active_scalars("value");

        let r = loop_subdivide(&m);
        let array = r.point_data().scalars().unwrap();
        assert_eq!(array.num_tuples(), r.points.len());
        let mut tuple = [0.0];
        array.tuple_as_f64(3, &mut tuple);
        assert!((tuple[0] - 1.0).abs() < 1e-12);
    }
}
