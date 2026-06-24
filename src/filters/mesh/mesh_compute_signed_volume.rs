//! Compute signed and unsigned volume of closed meshes.
use crate::data::PolyData;
pub fn signed_volume(mesh: &PolyData) -> f64 {
    let mut vol = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            vol += a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]);
        }
    }
    vol / 6.0
}
pub fn unsigned_volume(mesh: &PolyData) -> f64 {
    signed_volume(mesh).abs()
}
pub fn is_volume_positive(mesh: &PolyData) -> bool {
    signed_volume(mesh) > 0.0
}
pub fn surface_area(mesh: &PolyData) -> f64 {
    let mut area = 0.0;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            area += 0.5 * (cx * cx + cy * cy + cz * cz).sqrt();
        }
    }
    area
}
pub fn compactness(mesh: &PolyData) -> f64 {
    let v = unsigned_volume(mesh);
    let a = surface_area(mesh);
    if a < 1e-30 {
        return 0.0;
    }
    36.0 * std::f64::consts::PI * v * v / (a * a * a)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vol() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let v = unsigned_volume(&m);
        assert!((v - 1.0 / 6.0).abs() < 0.05);
    }
    #[test]
    fn test_area() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((surface_area(&m) - 2.0).abs() < 1e-10);
    }
}
