//! Remove degenerate (zero-area) triangles and duplicate vertices.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

/// Remove degenerate triangles with area below threshold.
pub fn remove_degenerate_triangles(mesh: &PolyData, min_area: f64) -> PolyData {
    let mut new_polys = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !valid_cell_points(cell, mesh.points.len()) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        let area = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        if area >= min_area {
            new_polys.push_cell(cell);
        }
    }
    let mut result = mesh.clone();
    result.polys = new_polys;
    result
}

/// Merge duplicate vertices within tolerance distance.
pub fn merge_close_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    let tol2 = tolerance * tolerance;
    let mut remap = vec![0usize; n];
    let mut new_pts = Points::<f64>::new();
    let mut representative_ids: Vec<usize> = Vec::new();

    for i in 0..n {
        let p = mesh.points.get(i);
        let mut found = false;
        for (j, &ni) in representative_ids.iter().enumerate() {
            let q = mesh.points.get(ni);
            let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
            if d2 < tol2 {
                remap[i] = j;
                found = true;
                break;
            }
        }
        if !found {
            remap[i] = new_pts.len();
            new_pts.push(p);
            representative_ids.push(i);
        }
    }

    let mut result = mesh.clone();
    result.points = new_pts;
    result.verts = remap_cell_array(&mesh.verts, &remap, 1, false);
    result.lines = remap_cell_array(&mesh.lines, &remap, 2, false);
    result.polys = remap_cell_array(&mesh.polys, &remap, 3, true);
    result.strips = remap_cell_array(&mesh.strips, &remap, 3, true);
    replace_point_data(
        result.point_data_mut(),
        mesh.point_data(),
        &representative_ids,
    );
    result
}

fn remap_cell_array(
    cells: &CellArray,
    remap: &[usize],
    min_unique_points: usize,
    close_loop: bool,
) -> CellArray {
    let mut new_cells = CellArray::new();
    for cell in cells.iter() {
        if !valid_cell_points(cell, remap.len()) {
            continue;
        }
        let mut mapped = Vec::with_capacity(cell.len());
        for &v in cell {
            let pt_id = remap[v as usize] as i64;
            if mapped.last().copied() != Some(pt_id) {
                mapped.push(pt_id);
            }
        }

        if close_loop && mapped.len() > 1 && mapped.first() == mapped.last() {
            mapped.pop();
        }

        let unique: std::collections::HashSet<i64> = mapped.iter().copied().collect();
        if unique.len() >= min_unique_points {
            new_cells.push_cell(&mapped);
        }
    }
    new_cells
}

fn replace_point_data(
    output: &mut DataSetAttributes,
    input: &DataSetAttributes,
    representative_ids: &[usize],
) {
    let active_scalars = input.scalars().map(|a| a.name().to_string());
    let active_vectors = input.vectors().map(|a| a.name().to_string());
    let active_normals = input.normals().map(|a| a.name().to_string());
    let active_tcoords = input.tcoords().map(|a| a.name().to_string());
    let active_tensors = input.tensors().map(|a| a.name().to_string());
    let active_global_ids = input.global_ids().map(|a| a.name().to_string());
    let active_pedigree_ids = input.pedigree_ids().map(|a| a.name().to_string());
    let active_edge_flags = input.edge_flags().map(|a| a.name().to_string());
    let active_tangents = input.tangents().map(|a| a.name().to_string());
    let active_rational_weights = input.rational_weights().map(|a| a.name().to_string());
    let active_higher_order_degrees = input.higher_order_degrees().map(|a| a.name().to_string());
    let active_process_ids = input.process_ids().map(|a| a.name().to_string());

    output.clear();
    for array in input.field_data().iter() {
        output.add_array(compact_array(array, representative_ids));
    }

    if let Some(name) = active_scalars.as_deref() {
        output.set_active_scalars(name);
    }
    if let Some(name) = active_vectors.as_deref() {
        output.set_active_vectors(name);
    }
    if let Some(name) = active_normals.as_deref() {
        output.set_active_normals(name);
    }
    if let Some(name) = active_tcoords.as_deref() {
        output.set_active_tcoords(name);
    }
    if let Some(name) = active_tensors.as_deref() {
        output.set_active_tensors(name);
    }
    if let Some(name) = active_global_ids.as_deref() {
        output.set_active_global_ids(name);
    }
    if let Some(name) = active_pedigree_ids.as_deref() {
        output.set_active_pedigree_ids(name);
    }
    if let Some(name) = active_edge_flags.as_deref() {
        output.set_active_edge_flags(name);
    }
    if let Some(name) = active_tangents.as_deref() {
        output.set_active_tangents(name);
    }
    if let Some(name) = active_rational_weights.as_deref() {
        output.set_active_rational_weights(name);
    }
    if let Some(name) = active_higher_order_degrees.as_deref() {
        output.set_active_higher_order_degrees(name);
    }
    if let Some(name) = active_process_ids.as_deref() {
        output.set_active_process_ids(name);
    }
}

fn compact_array(array: &AnyDataArray, representative_ids: &[usize]) -> AnyDataArray {
    macro_rules! compact {
        ($array:expr, $variant:ident) => {{
            let num_components = $array.num_components();
            let mut data = Vec::with_capacity(representative_ids.len() * num_components);
            for &source_id in representative_ids {
                data.extend_from_slice($array.tuple(source_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($array.name(), data, num_components))
        }};
    }

    match array {
        AnyDataArray::F32(a) => compact!(a, F32),
        AnyDataArray::F64(a) => compact!(a, F64),
        AnyDataArray::I8(a) => compact!(a, I8),
        AnyDataArray::I16(a) => compact!(a, I16),
        AnyDataArray::I32(a) => compact!(a, I32),
        AnyDataArray::I64(a) => compact!(a, I64),
        AnyDataArray::U8(a) => compact!(a, U8),
        AnyDataArray::U16(a) => compact!(a, U16),
        AnyDataArray::U32(a) => compact!(a, U32),
        AnyDataArray::U64(a) => compact!(a, U64),
    }
}

/// Remove isolated vertices (not referenced by any cell).
pub fn remove_isolated_vertices(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let mut used = vec![false; n];
    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        for cell in cells.iter() {
            for &v in cell {
                let Some(v) = valid_point_id(v, n) else {
                    continue;
                };
                used[v as usize] = true;
            }
        }
    }

    let mut pt_map = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    let mut representative_ids = Vec::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = pts.len();
            pts.push(mesh.points.get(i));
            representative_ids.push(i);
        }
    }

    let remap_cells = |ca: &crate::data::CellArray| -> CellArray {
        let mut new = CellArray::new();
        for cell in ca.iter() {
            if !valid_cell_points(cell, pt_map.len()) {
                continue;
            }
            let mapped: Vec<i64> = cell.iter().map(|&v| pt_map[v as usize] as i64).collect();
            new.push_cell(&mapped);
        }
        new
    };

    let mut result = mesh.clone();
    result.points = pts;
    result.verts = remap_cells(&mesh.verts);
    result.lines = remap_cells(&mesh.lines);
    result.polys = remap_cells(&mesh.polys);
    result.strips = remap_cells(&mesh.strips);
    replace_point_data(
        result.point_data_mut(),
        mesh.point_data(),
        &representative_ids,
    );
    result
}

fn valid_cell_points(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&id| valid_point_id(id, num_points).is_some())
}

fn valid_point_id(id: i64, num_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_remove_degenerate() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = remove_degenerate_triangles(&mesh, 1e-10);
        assert_eq!(r.polys.num_cells(), 1); // degenerate removed
    }
    #[test]
    fn test_merge() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.001, 0.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = merge_close_vertices(&mesh, 0.01);
        assert!(r.points.len() < 4);
    }

    #[test]
    fn test_merge_preserves_and_remaps_topology_and_point_data() {
        let mut mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [0.001, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        mesh.lines.push_cell(&[0, 2]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![10.0, 11.0, 12.0, 13.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("id");

        let r = merge_close_vertices(&mesh, 0.01);
        assert_eq!(r.points.len(), 3);
        assert_eq!(r.lines.num_cells(), 1);
        assert_eq!(r.polys.num_cells(), 1);
        assert_eq!(r.polys.cell(0), &[0, 1, 2]);

        let arr = r.point_data().get_array("id").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut value = [0.0];
        arr.tuple_as_f64(0, &mut value);
        assert_eq!(value[0], 10.0);
        assert!(r.point_data().scalars().is_some());
    }
    #[test]
    fn test_isolated() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [99.0, 99.0, 99.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = remove_isolated_vertices(&mesh);
        assert_eq!(r.points.len(), 3);
    }

    #[test]
    fn test_isolated_preserves_all_referenced_cell_arrays() {
        let mut mesh = PolyData::from_points(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [99.0, 99.0, 99.0],
        ]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[1, 2]);
        mesh.strips.push_cell(&[2, 3, 4]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "id",
                vec![10.0, 11.0, 12.0, 13.0, 14.0, 99.0],
                1,
            )));

        let r = remove_isolated_vertices(&mesh);

        assert_eq!(r.points.len(), 5);
        assert_eq!(r.verts.cell(0), &[0]);
        assert_eq!(r.lines.cell(0), &[1, 2]);
        assert_eq!(r.strips.cell(0), &[2, 3, 4]);
        assert_eq!(r.point_data().get_array("id").unwrap().num_tuples(), 5);
    }
}
