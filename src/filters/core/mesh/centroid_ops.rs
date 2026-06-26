//! Centroid-based operations: per-cell centroid, centroid-distance, centroid cloud.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Compute per-cell centroid as a point cloud.
pub fn cell_centroids_as_points(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut measure = Vec::new();

    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        push_cell_centroids(mesh, cells, &mut pts, &mut measure);
    }

    let mut result = PolyData::new();
    result.points = pts;
    copy_cell_data_to_point_data(mesh, &mut result);
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CellArea", measure, 1,
        )));
    result
}

/// Add distance from each vertex to the mesh centroid as point data.
pub fn distance_from_centroid(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;
    let data: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
        })
        .collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CentroidDistance",
            data,
            1,
        )));
    result
}

/// Center mesh at origin (translate so centroid is at [0,0,0]).
pub fn center_at_origin(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    let nf = n as f64;
    for j in 0..3 {
        c[j] /= nf;
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([p[0] - c[0], p[1] - c[1], p[2] - c[2]]);
    }
    let mut result = mesh.clone();
    result.points = pts;
    result
}

/// Normalize mesh to fit within a unit sphere centered at origin.
pub fn normalize_to_unit_sphere(mesh: &PolyData) -> PolyData {
    let centered = center_at_origin(mesh);
    let n = centered.points.len();
    if n == 0 {
        return centered;
    }
    let mut max_r = 0.0f64;
    for i in 0..n {
        let p = centered.points.get(i);
        max_r = max_r.max((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt());
    }
    if max_r < 1e-15 {
        return centered;
    }
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = centered.points.get(i);
        pts.push([p[0] / max_r, p[1] / max_r, p[2] / max_r]);
    }
    let mut result = centered;
    result.points = pts;
    result
}

fn copy_cell_data_to_point_data(input: &PolyData, output: &mut PolyData) {
    for i in 0..input.cell_data().num_arrays() {
        let Some(array) = input.cell_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() == input.total_cells() {
            output.point_data_mut().add_array(array.clone());
        }
    }
}

fn push_cell_centroids(
    mesh: &PolyData,
    cells: &CellArray,
    points: &mut Points<f64>,
    measure: &mut Vec<f64>,
) {
    for cell in cells.iter() {
        points.push(cell_centroid(mesh, cell));
        measure.push(cell_measure(mesh, cell));
    }
}

fn cell_centroid(mesh: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.is_empty() {
        return [0.0; 3];
    }
    let mut c = [0.0; 3];
    for &pid in cell {
        let p = mesh.points.get(pid as usize);
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = cell.len() as f64;
    [c[0] / n, c[1] / n, c[2] / n]
}

fn cell_measure(mesh: &PolyData, cell: &[i64]) -> f64 {
    match cell.len() {
        0 | 1 => 0.0,
        2 => {
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
        }
        _ => polygon_area(mesh, cell),
    }
}

fn polygon_area(mesh: &PolyData, cell: &[i64]) -> f64 {
    let p0 = mesh.points.get(cell[0] as usize);
    let mut area = 0.0;
    for i in 1..cell.len() - 1 {
        let p1 = mesh.points.get(cell[i] as usize);
        let p2 = mesh.points.get(cell[i + 1] as usize);
        let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let c = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        area += 0.5 * (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn centroids() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = cell_centroids_as_points(&mesh);
        assert_eq!(result.points.len(), 1);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn centroids_include_vtk_polydata_cell_order() {
        let mut mesh = PolyData::new();
        mesh.points = Points::from_vec(vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
        ]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.strips.push_cell(&[1, 3, 2]);
        let result = cell_centroids_as_points(&mesh);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
        assert_eq!(result.points.get(1), [1.0, 0.0, 0.0]);
        assert!((result.points.get(2)[0] - 2.0 / 3.0).abs() < 1e-10);
        assert!((result.points.get(3)[1] - 4.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn polygon_measure_uses_full_fan() {
        let mesh = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3]],
        );
        let result = cell_centroids_as_points(&mesh);
        let arr = result.point_data().get_array("CellArea").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn centroids_copy_cell_data_to_point_data() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_id",
                vec![10, 11],
                1,
            )));

        let result = cell_centroids_as_points(&mesh);
        let arr = result.point_data().get_array("cell_id").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 11.0);
    }

    #[test]
    fn dist() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let result = distance_from_centroid(&mesh);
        let arr = result.point_data().get_array("CentroidDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.01); // distance from centroid (1,0,0) to (0,0,0)
    }
    #[test]
    fn center() {
        let mesh = PolyData::from_points(vec![[2.0, 4.0, 6.0], [4.0, 6.0, 8.0]]);
        let result = center_at_origin(&mesh);
        let p = result.points.get(0);
        assert!((p[0] + 1.0).abs() < 0.01);
    }
    #[test]
    fn normalize() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
        let result = normalize_to_unit_sphere(&mesh);
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() <= 1.01);
        }
    }
}
