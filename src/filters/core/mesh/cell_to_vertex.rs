//! Convert between cell and vertex representations.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::Scalar;
use std::collections::HashSet;

/// Convert cells to independent vertices (no sharing).
///
/// Each input cell gets its own copied points. VTK PolyData cell ordering
/// (verts, lines, polys, strips) and point-data values are preserved.
pub fn cells_to_independent_vertices(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut old_point_ids = Vec::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();

    copy_cells(&mesh.verts, mesh, &mut pts, &mut old_point_ids, &mut verts);
    copy_cells(&mesh.lines, mesh, &mut pts, &mut old_point_ids, &mut lines);
    copy_cells(&mesh.polys, mesh, &mut pts, &mut old_point_ids, &mut polys);
    copy_cells(
        &mesh.strips,
        mesh,
        &mut pts,
        &mut old_point_ids,
        &mut strips,
    );

    let mut result = mesh.clone();
    result.points = pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    copy_point_data(mesh, &old_point_ids, &mut result);
    result
}

/// Merge vertices that are at the same position (within tolerance).
pub fn merge_coincident_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    let tol2 = tolerance * tolerance;
    let mut mapping = vec![0usize; n];
    let mut representative_point_ids = Vec::new();
    let mut new_pts = Points::<f64>::new();
    let mut used = vec![false; n];

    for i in 0..n {
        if used[i] {
            continue;
        }
        let pi = mesh.points.get(i);
        let new_idx = new_pts.len();
        new_pts.push(pi);
        representative_point_ids.push(i);
        mapping[i] = new_idx;
        for j in i + 1..n {
            if used[j] {
                continue;
            }
            let pj = mesh.points.get(j);
            if (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2) < tol2 {
                mapping[j] = new_idx;
                used[j] = true;
            }
        }
    }

    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    let mut kept_cell_ids = Vec::new();
    let mut cell_idx = 0;

    remap_cells(
        &mesh.verts,
        &mapping,
        1,
        &mut cell_idx,
        &mut verts,
        &mut kept_cell_ids,
    );
    remap_cells(
        &mesh.lines,
        &mapping,
        2,
        &mut cell_idx,
        &mut lines,
        &mut kept_cell_ids,
    );
    remap_cells(
        &mesh.polys,
        &mapping,
        3,
        &mut cell_idx,
        &mut polys,
        &mut kept_cell_ids,
    );
    remap_cells(
        &mesh.strips,
        &mapping,
        3,
        &mut cell_idx,
        &mut strips,
        &mut kept_cell_ids,
    );

    let mut result = mesh.clone();
    result.points = new_pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    copy_point_data(mesh, &representative_point_ids, &mut result);
    copy_cell_data(mesh, &kept_cell_ids, &mut result);
    result
}

/// Create per-face vertex colors from cell data (by duplicating vertices).
pub fn cell_data_to_flat_vertex_colors(mesh: &PolyData, cell_array: &str) -> PolyData {
    let arr = match mesh.cell_data().get_array(cell_array) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let mut pts = Points::<f64>::new();
    let mut old_point_ids = Vec::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    let mut vert_data = Vec::new();
    let mut buf = vec![0.0f64; nc];
    let mut cell_idx = 0;

    copy_cells_with_cell_data(
        &mesh.verts,
        mesh,
        arr,
        &mut cell_idx,
        &mut pts,
        &mut old_point_ids,
        &mut verts,
        &mut vert_data,
        &mut buf,
    );
    copy_cells_with_cell_data(
        &mesh.lines,
        mesh,
        arr,
        &mut cell_idx,
        &mut pts,
        &mut old_point_ids,
        &mut lines,
        &mut vert_data,
        &mut buf,
    );
    copy_cells_with_cell_data(
        &mesh.polys,
        mesh,
        arr,
        &mut cell_idx,
        &mut pts,
        &mut old_point_ids,
        &mut polys,
        &mut vert_data,
        &mut buf,
    );
    copy_cells_with_cell_data(
        &mesh.strips,
        mesh,
        arr,
        &mut cell_idx,
        &mut pts,
        &mut old_point_ids,
        &mut strips,
        &mut vert_data,
        &mut buf,
    );

    let mut result = mesh.clone();
    result.points = pts;
    result.verts = verts;
    result.lines = lines;
    result.polys = polys;
    result.strips = strips;
    copy_point_data(mesh, &old_point_ids, &mut result);
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            cell_array, vert_data, nc,
        )));
    result
}

fn copy_cells(
    source: &CellArray,
    mesh: &PolyData,
    points: &mut Points<f64>,
    old_point_ids: &mut Vec<usize>,
    target: &mut CellArray,
) {
    for cell in source.iter() {
        let base = points.len() as i64;
        let ids: Vec<i64> = cell
            .iter()
            .enumerate()
            .map(|(i, &pid)| {
                points.push(mesh.points.get(pid as usize));
                old_point_ids.push(pid as usize);
                base + i as i64
            })
            .collect();
        target.push_cell(&ids);
    }
}

fn remap_cells(
    source: &CellArray,
    mapping: &[usize],
    min_unique_points: usize,
    cell_idx: &mut usize,
    target: &mut CellArray,
    kept_cell_ids: &mut Vec<usize>,
) {
    for cell in source.iter() {
        let ids: Vec<i64> = cell
            .iter()
            .map(|&pid| mapping[pid as usize] as i64)
            .collect();
        let unique: HashSet<i64> = ids.iter().copied().collect();
        if unique.len() >= min_unique_points {
            target.push_cell(&ids);
            kept_cell_ids.push(*cell_idx);
        }
        *cell_idx += 1;
    }
}

fn copy_cells_with_cell_data(
    source: &CellArray,
    mesh: &PolyData,
    array: &AnyDataArray,
    cell_idx: &mut usize,
    points: &mut Points<f64>,
    old_point_ids: &mut Vec<usize>,
    target: &mut CellArray,
    vertex_data: &mut Vec<f64>,
    tuple: &mut [f64],
) {
    for cell in source.iter() {
        let base = points.len() as i64;
        if *cell_idx < array.num_tuples() {
            array.tuple_as_f64(*cell_idx, tuple);
        } else {
            tuple.fill(0.0);
        }
        let ids: Vec<i64> = cell
            .iter()
            .enumerate()
            .map(|(i, &pid)| {
                points.push(mesh.points.get(pid as usize));
                old_point_ids.push(pid as usize);
                vertex_data.extend_from_slice(tuple);
                base + i as i64
            })
            .collect();
        target.push_cell(&ids);
        *cell_idx += 1;
    }
}

fn copy_point_data(input: &PolyData, old_point_ids: &[usize], output: &mut PolyData) {
    output.point_data_mut().clear();
    for i in 0..input.point_data().num_arrays() {
        let Some(array) = input.point_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() != input.points.len() {
            continue;
        }
        output
            .point_data_mut()
            .add_array(remap_array(array, old_point_ids));
    }
}

fn copy_cell_data(input: &PolyData, old_cell_ids: &[usize], output: &mut PolyData) {
    output.cell_data_mut().clear();
    for i in 0..input.cell_data().num_arrays() {
        let Some(array) = input.cell_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() != input.total_cells() {
            continue;
        }
        output
            .cell_data_mut()
            .add_array(remap_array(array, old_cell_ids));
    }
}

fn remap_array(array: &AnyDataArray, old_point_ids: &[usize]) -> AnyDataArray {
    macro_rules! remap {
        ($arr:expr, $variant:ident) => {
            AnyDataArray::$variant(remap_typed_array($arr, old_point_ids))
        };
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

fn remap_typed_array<T: Scalar>(array: &DataArray<T>, old_point_ids: &[usize]) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(old_point_ids.len() * nc);
    for &old_id in old_point_ids {
        data.extend_from_slice(array.tuple(old_id));
    }
    DataArray::from_vec(array.name(), data, nc)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = cells_to_independent_vertices(&mesh);
        assert_eq!(result.points.len(), 6); // 2 tris × 3 verts each
    }

    #[test]
    fn independent_preserves_polydata_cell_kinds_and_point_data() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "pid",
                vec![10, 20, 30],
                1,
            )));
        let result = cells_to_independent_vertices(&mesh);
        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 6);
        let arr = result.point_data().get_array("pid").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 10.0);
    }
    #[test]
    fn merge() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.001, 0.0, 0.0],
                [1.001, 0.0, 0.0],
                [0.501, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = merge_coincident_vertices(&mesh, 0.01);
        assert!(result.points.len() <= 4);
    }

    #[test]
    fn merge_preserves_cell_kinds_and_remaps_data() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 2]);
        mesh.polys.push_cell(&[0, 2, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "pid",
                vec![10, 11, 12, 13],
                1,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cid",
                vec![20, 21, 22],
                1,
            )));
        let result = merge_coincident_vertices(&mesh, 0.01);
        assert_eq!(result.verts.num_cells(), 1);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
        assert_eq!(
            result.point_data().get_array("pid").unwrap().num_tuples(),
            3
        );
        assert_eq!(result.cell_data().get_array("cid").unwrap().num_tuples(), 3);
    }
    #[test]
    fn flat_colors() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "color",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));
        let result = cell_data_to_flat_vertex_colors(&mesh, "color");
        assert_eq!(result.points.len(), 6);
        assert!(result.point_data().get_array("color").is_some());
    }

    #[test]
    fn flat_colors_use_vtk_cell_order() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_id",
                vec![7.0, 8.0, 9.0],
                1,
            )));
        let result = cell_data_to_flat_vertex_colors(&mesh, "cell_id");
        let arr = result.point_data().get_array("cell_id").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 7.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 8.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 9.0);
    }
}
