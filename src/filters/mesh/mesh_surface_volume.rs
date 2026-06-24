//! Compute total surface area and signed volume of a closed mesh.
use crate::data::PolyData;

pub fn surface_area(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    let mut total = 0.0f64;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        for i in 1..(cell.len() - 1) {
            let b = cell[i] as usize;
            let c = cell[i + 1] as usize;
            if a >= n || b >= n || c >= n {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            total += 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
        }
    }
    total
}

pub fn signed_volume(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    let mut total = 0.0f64;
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        for i in 1..(cell.len() - 1) {
            let b = cell[i] as usize;
            let c = cell[i + 1] as usize;
            if a >= n || b >= n || c >= n {
                continue;
            }
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            // Signed volume of tetrahedron with origin
            total += (pa[0] * (pb[1] * pc[2] - pb[2] * pc[1])
                + pa[1] * (pb[2] * pc[0] - pb[0] * pc[2])
                + pa[2] * (pb[0] * pc[1] - pb[1] * pc[0]))
                / 6.0;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!((surface_area(&mesh) - 0.5).abs() < 1e-9);
    }
    #[test]
    fn test_volume() {
        // Tetrahedron: V = 1/6 for unit
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]],
        );
        let v = signed_volume(&mesh).abs();
        assert!((v - 1.0 / 6.0).abs() < 0.02);
    }
}
