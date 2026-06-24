//! Test if points are inside a closed mesh using ray casting.
use crate::data::PolyData;

pub fn point_inside_mesh(mesh: &PolyData, point: [f64; 3]) -> bool {
    let n = mesh.points.len();
    let tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    // Cast ray in +X direction, count intersections
    let dir = [1.0, 0.0, 0.0];
    let origin = [point[0] + 1e-7, point[1] + 1e-7, point[2]]; // small jitter
    let mut crossings = 0;
    for &[a, b, c] in &tris {
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let e1 = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let e2 = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let h = [
            dir[1] * e2[2] - dir[2] * e2[1],
            dir[2] * e2[0] - dir[0] * e2[2],
            dir[0] * e2[1] - dir[1] * e2[0],
        ];
        let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
        if det.abs() < 1e-12 {
            continue;
        }
        let inv = 1.0 / det;
        let s = [origin[0] - pa[0], origin[1] - pa[1], origin[2] - pa[2]];
        let u = inv * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
        if u < 0.0 || u > 1.0 {
            continue;
        }
        let q = [
            s[1] * e1[2] - s[2] * e1[1],
            s[2] * e1[0] - s[0] * e1[2],
            s[0] * e1[1] - s[1] * e1[0],
        ];
        let v = inv * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        let t = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
        if t > 1e-8 {
            crossings += 1;
        }
    }
    crossings % 2 == 1
}

pub fn classify_points(mesh: &PolyData, points: &[[f64; 3]]) -> Vec<bool> {
    points.iter().map(|&p| point_inside_mesh(mesh, p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_inside() {
        // Closed tetrahedron
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.7, 2.0],
            ],
            vec![[0, 1, 2], [0, 3, 1], [0, 2, 3], [1, 3, 2]],
        );
        assert!(point_inside_mesh(&mesh, [1.0, 0.5, 0.5]));
        assert!(!point_inside_mesh(&mesh, [5.0, 5.0, 5.0]));
    }
}
