//! Extrude mesh surface along vertex normals to create a thickened shell.
use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

pub fn extrude_along_normals(mesh: &PolyData, distance: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let vnorm = extract_vertex_normals(mesh);
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0], p[1], p[2]]);
    }
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + vnorm[i][0] * distance,
            p[1] + vnorm[i][1] * distance,
            p[2] + vnorm[i][2] * distance,
        ]);
    }
    let offset = n as i64;
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();
    let mut line_cell_ids = Vec::new();
    let mut poly_cell_ids = Vec::new();
    let mut strip_cell_ids = Vec::new();

    for (cell_id, cell) in mesh.verts.iter().enumerate() {
        for &pt_id in cell {
            if valid_point_index(pt_id, n).is_some() {
                lines.push_cell(&[pt_id, pt_id + offset]);
                line_cell_ids.push(cell_id);
            }
        }
    }

    let line_offset = mesh.verts.num_cells();
    for (cell_id, cell) in mesh.lines.iter().enumerate() {
        for pair in cell.windows(2) {
            if valid_point_index(pair[0], n).is_some() && valid_point_index(pair[1], n).is_some() {
                strips.push_cell(&[pair[0], pair[1], pair[0] + offset, pair[1] + offset]);
                strip_cell_ids.push(line_offset + cell_id);
            }
        }
    }

    let poly_offset = line_offset + mesh.lines.num_cells();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if valid_polygon_cell(cell, n) {
            polys.push_cell(cell);
            poly_cell_ids.push(poly_offset + cell_id);
        }
    }
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if valid_polygon_cell(cell, n) {
            let top: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            polys.push_cell(&top);
            poly_cell_ids.push(poly_offset + cell_id);
        }
    }

    let strip_offset = poly_offset + mesh.polys.num_cells();
    for (cell_id, cell) in mesh.strips.iter().enumerate() {
        if valid_strip_cell(cell, n) {
            strips.push_cell(cell);
            strip_cell_ids.push(strip_offset + cell_id);
            let top: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            strips.push_cell(&top);
            strip_cell_ids.push(strip_offset + cell_id);
        }
    }

    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    let mut ordered_edges = Vec::new();
    for (cell_id, cell) in mesh.polys.iter().enumerate() {
        if !valid_polygon_cell(cell, n) {
            continue;
        }
        let nc = cell.len();
        for i in 0..nc {
            let a = valid_point_index(cell[i], n).unwrap();
            let b = valid_point_index(cell[(i + 1) % nc], n).unwrap();
            let e = if a < b { (a, b) } else { (b, a) };
            if !edge_count.contains_key(&e) {
                ordered_edges.push((a, b, poly_offset + cell_id));
            }
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    for (cell_id, strip) in mesh.strips.iter().enumerate() {
        for (i, tri) in strip.windows(3).enumerate() {
            if !valid_triangle(tri, n) {
                continue;
            }
            let tri = if i % 2 == 0 {
                [tri[0], tri[1], tri[2]]
            } else {
                [tri[1], tri[0], tri[2]]
            };
            for j in 0..3 {
                let a = valid_point_index(tri[j], n).unwrap();
                let b = valid_point_index(tri[(j + 1) % 3], n).unwrap();
                let e = if a < b { (a, b) } else { (b, a) };
                if !edge_count.contains_key(&e) {
                    ordered_edges.push((a, b, strip_offset + cell_id));
                }
                *edge_count.entry(e).or_insert(0) += 1;
            }
        }
    }
    for (a, b, cell_id) in ordered_edges {
        let e = if a < b { (a, b) } else { (b, a) };
        if edge_count[&e] == 1 {
            strips.push_cell(&[a as i64, b as i64, (a + n) as i64, (b + n) as i64]);
            strip_cell_ids.push(cell_id);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m.polys = polys;
    m.strips = strips;
    *m.field_data_mut() = mesh.field_data().clone();
    copy_duplicated_point_data(mesh.point_data(), m.point_data_mut(), n);
    let mut cell_ids =
        Vec::with_capacity(line_cell_ids.len() + poly_cell_ids.len() + strip_cell_ids.len());
    cell_ids.extend(line_cell_ids);
    cell_ids.extend(poly_cell_ids);
    cell_ids.extend(strip_cell_ids);
    copy_cell_data_by_ids(mesh.cell_data(), m.cell_data_mut(), &cell_ids);
    m
}

fn extract_vertex_normals(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    if let Some(normals) = mesh.point_data().normals() {
        if normals.num_components() == 3 && normals.num_tuples() == n {
            let mut result = Vec::with_capacity(n);
            let mut tuple = [0.0; 3];
            for i in 0..n {
                normals.tuple_as_f64(i, &mut tuple);
                result.push(tuple);
            }
            return result;
        }
    }

    let mut vnorm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(ids) = valid_cell_ids(cell, n) else {
            continue;
        };
        let mut nx = 0.0;
        let mut ny = 0.0;
        let mut nz = 0.0;
        for i in 0..ids.len() {
            let p = mesh.points.get(ids[i]);
            let q = mesh.points.get(ids[(i + 1) % ids.len()]);
            nx += (p[1] - q[1]) * (p[2] + q[2]);
            ny += (p[2] - q[2]) * (p[0] + q[0]);
            nz += (p[0] - q[0]) * (p[1] + q[1]);
        }
        for &vi in &ids {
            vnorm[vi][0] += nx;
            vnorm[vi][1] += ny;
            vnorm[vi][2] += nz;
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        let Some(ids) = valid_cell_ids(strip, n) else {
            continue;
        };
        for (i, tri) in ids.windows(3).enumerate() {
            let tri = if i % 2 == 0 {
                [tri[0], tri[1], tri[2]]
            } else {
                [tri[1], tri[0], tri[2]]
            };
            let p0 = mesh.points.get(tri[0]);
            let p1 = mesh.points.get(tri[1]);
            let p2 = mesh.points.get(tri[2]);
            let ux = p1[0] - p0[0];
            let uy = p1[1] - p0[1];
            let uz = p1[2] - p0[2];
            let vx = p2[0] - p0[0];
            let vy = p2[1] - p0[1];
            let vz = p2[2] - p0[2];
            let nx = uy * vz - uz * vy;
            let ny = uz * vx - ux * vz;
            let nz = ux * vy - uy * vx;
            for vi in tri {
                vnorm[vi][0] += nx;
                vnorm[vi][1] += ny;
                vnorm[vi][2] += nz;
            }
        }
    }
    for vn in &mut vnorm {
        let l = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if l > 1e-15 {
            vn[0] /= l;
            vn[1] /= l;
            vn[2] /= l;
        } else {
            *vn = [0.0, 0.0, 1.0];
        }
    }
    vnorm
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn valid_cell(cell: &[i64], n_points: usize) -> bool {
    cell.iter()
        .all(|&id| valid_point_index(id, n_points).is_some())
}

fn valid_polygon_cell(cell: &[i64], n_points: usize) -> bool {
    cell.len() >= 3 && valid_cell(cell, n_points)
}

fn valid_strip_cell(cell: &[i64], n_points: usize) -> bool {
    cell.windows(3).any(|tri| valid_triangle(tri, n_points))
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && valid_cell(tri, n_points)
}

fn valid_cell_ids(cell: &[i64], n_points: usize) -> Option<Vec<usize>> {
    cell.iter()
        .map(|&id| valid_point_index(id, n_points))
        .collect()
}

fn copy_duplicated_point_data(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    num_points: usize,
) {
    let normals_name = source.normals().map(|array| array.name());
    for array in source.iter() {
        if array.num_tuples() >= num_points && Some(array.name()) != normals_name {
            target.add_array(copy_duplicated_array(array, num_points));
        }
    }
    copy_active_attributes(source, target, false);
}

fn copy_cell_data_by_ids(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    ids: &[usize],
) {
    if ids.is_empty() {
        return;
    }
    let normals_name = source.normals().map(|array| array.name());
    for array in source.iter() {
        if ids.iter().all(|&id| id < array.num_tuples()) && Some(array.name()) != normals_name {
            target.add_array(copy_array_by_indices(array, ids));
        }
    }
    copy_active_attributes(source, target, false);
}

fn copy_duplicated_array(array: &AnyDataArray, num_points: usize) -> AnyDataArray {
    let mut ids = Vec::with_capacity(num_points * 2);
    ids.extend(0..num_points);
    ids.extend(0..num_points);
    copy_array_by_indices(array, &ids)
}

fn copy_array_by_indices(array: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {
            AnyDataArray::$variant(copy_typed_array($array, indices))
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

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let num_components = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * num_components);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, num_components)
}

fn copy_active_attributes(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    copy_normals: bool,
) {
    if let Some(name) = source.scalars().map(|array| array.name().to_string()) {
        target.set_active_scalars(&name);
    }
    if let Some(name) = source.vectors().map(|array| array.name().to_string()) {
        target.set_active_vectors(&name);
    }
    if copy_normals {
        if let Some(name) = source.normals().map(|array| array.name().to_string()) {
            target.set_active_normals(&name);
        }
    }
    if let Some(name) = source.tcoords().map(|array| array.name().to_string()) {
        target.set_active_tcoords(&name);
    }
    if let Some(name) = source.tensors().map(|array| array.name().to_string()) {
        target.set_active_tensors(&name);
    }
    if let Some(name) = source.global_ids().map(|array| array.name().to_string()) {
        target.set_active_global_ids(&name);
    }
    if let Some(name) = source.pedigree_ids().map(|array| array.name().to_string()) {
        target.set_active_pedigree_ids(&name);
    }
    if let Some(name) = source.edge_flags().map(|array| array.name().to_string()) {
        target.set_active_edge_flags(&name);
    }
    if let Some(name) = source.tangents().map(|array| array.name().to_string()) {
        target.set_active_tangents(&name);
    }
    if let Some(name) = source
        .rational_weights()
        .map(|array| array.name().to_string())
    {
        target.set_active_rational_weights(&name);
    }
    if let Some(name) = source
        .higher_order_degrees()
        .map(|array| array.name().to_string())
    {
        target.set_active_higher_order_degrees(&name);
    }
    if let Some(name) = source.process_ids().map(|array| array.name().to_string()) {
        target.set_active_process_ids(&name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extrude() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = extrude_along_normals(&mesh, 0.5);
        assert_eq!(r.points.len(), 6);
        assert!(r.polys.num_cells() >= 2); // top + bottom + sides
    }
}
