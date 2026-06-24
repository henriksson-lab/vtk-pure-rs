//! Check if a mesh is star-shaped from its centroid (all vertices visible).
use crate::data::PolyData;

pub fn is_star_shaped(mesh: &PolyData) -> bool {
    let n = mesh.points.len();
    if n < 4 {
        return true;
    }
    // Compute centroid
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut cz = 0.0;
    for i in 0..n {
        let p = mesh.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    cx /= n as f64;
    cy /= n as f64;
    cz /= n as f64;
    // Check if centroid is on the inside of all face planes
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        let d = (cx - pa[0]) * nx + (cy - pa[1]) * ny + (cz - pa[2]) * nz;
        // If dot product is negative, centroid is behind this face
        if d < -1e-10 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_star() {
        // Tetrahedron is star-shaped
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 2, 1], [0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        assert!(is_star_shaped(&mesh));
    }
}
