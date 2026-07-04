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
    let centroid = [cx, cy, cz];
    for point_id in 0..n {
        let point = mesh.points.get(point_id);
        if !segment_to_vertex_visible(mesh, centroid, point, point_id) {
            return false;
        }
    }
    true
}

fn segment_to_vertex_visible(
    mesh: &PolyData,
    origin: [f64; 3],
    target: [f64; 3],
    target_id: usize,
) -> bool {
    let dir = [
        target[0] - origin[0],
        target[1] - origin[1],
        target[2] - origin[2],
    ];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if cell
            .iter()
            .any(|&id| id < 0 || id as usize >= mesh.points.len())
        {
            continue;
        }
        if cell.iter().any(|&id| id as usize == target_id) {
            continue;
        }
        let a = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let b = mesh.points.get(cell[i] as usize);
            let c = mesh.points.get(cell[i + 1] as usize);
            if let Some(t) = segment_triangle_intersection(origin, dir, a, b, c) {
                if t > 1e-10 && t < 1.0 - 1e-10 {
                    return false;
                }
            }
        }
    }
    true
}

fn segment_triangle_intersection(
    origin: [f64; 3],
    dir: [f64; 3],
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
) -> Option<f64> {
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = cross(dir, edge2);
    let a = dot(edge1, h);
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * dot(s, h);
    if !(-1e-12..=1.0 + 1e-12).contains(&u) {
        return None;
    }
    let q = cross(s, edge1);
    let v = f * dot(dir, q);
    if v < -1e-12 || u + v > 1.0 + 1e-12 {
        return None;
    }
    let t = f * dot(edge2, q);
    if (0.0..=1.0).contains(&t) {
        Some(t)
    } else {
        None
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
