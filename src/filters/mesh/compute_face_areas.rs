//! Compute per-face areas and attach as cell data.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute area of each polygon and store as cell data "Area".
pub fn compute_face_areas(mesh: &PolyData) -> PolyData {
    let mut areas = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            areas.push(0.0);
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        // Triangulate polygon and sum triangle areas
        let mut total = 0.0;
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            total += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }
        areas.push(total);
    }
    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Area", areas, 1)));
    result
}

/// Compute total surface area.
pub fn total_surface_area(mesh: &PolyData) -> f64 {
    let result = compute_face_areas(mesh);
    let arr = result.cell_data().get_array("Area").unwrap();
    let mut buf = [0.0f64];
    (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .sum()
}

/// Compute min, max, average face area.
pub fn face_area_stats(mesh: &PolyData) -> (f64, f64, f64) {
    let result = compute_face_areas(mesh);
    let arr = result.cell_data().get_array("Area").unwrap();
    let n = arr.num_tuples();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mut buf = [0.0f64];
    let mut mn = f64::INFINITY;
    let mut mx = f64::NEG_INFINITY;
    let mut sum = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        mn = mn.min(buf[0]);
        mx = mx.max(buf[0]);
        sum += buf[0];
    }
    (mn, mx, sum / n as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_areas() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = compute_face_areas(&mesh);
        let arr = r.cell_data().get_array("Area").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10); // triangle area = 2
    }
    #[test]
    fn test_total() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let total = total_surface_area(&mesh);
        assert!((total - 1.0).abs() < 1e-10); // unit square = 2 * 0.5
    }
    #[test]
    fn test_stats() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let (mn, mx, avg) = face_area_stats(&mesh);
        assert!((mn - 0.5).abs() < 1e-10);
        assert!((mx - 2.0).abs() < 1e-10);
        assert!((avg - 1.25).abs() < 1e-10);
    }
}
