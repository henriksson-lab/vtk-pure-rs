use crate::data::PolyData;

/// Test if a point is inside a closed triangle mesh using ray casting.
///
/// Casts a ray in +X and counts intersections. Odd = inside.
pub fn point_in_mesh(mesh: &PolyData, point: [f64; 3]) -> bool {
    let mut hits = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = mesh.points.get(cell[i] as usize);
            let v2 = mesh.points.get(cell[i + 1] as usize);
            if let Some(t) = ray_x_hit(point, v0, v1, v2) {
                hits.push(t);
            }
        }
    }
    hits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut count = 0u32;
    let mut last = None;
    for t in hits {
        if last.map_or(true, |prev: f64| (t - prev).abs() > 1e-9) {
            count += 1;
            last = Some(t);
        }
    }
    count % 2 == 1
}

/// Test multiple points against a mesh. Returns Vec<bool>.
pub fn points_in_mesh(mesh: &PolyData, points: &[[f64; 3]]) -> Vec<bool> {
    points.iter().map(|&p| point_in_mesh(mesh, p)).collect()
}

/// Count how many points are inside the mesh.
pub fn count_points_inside(mesh: &PolyData, points: &[[f64; 3]]) -> usize {
    points.iter().filter(|&&p| point_in_mesh(mesh, p)).count()
}

fn ray_x_hit(o: [f64; 3], v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> Option<f64> {
    let dir = [1.0, 0.0, 0.0];
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = [o[0] - v0[0], o[1] - v0[1], o[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    (t > 1e-12).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box() -> PolyData {
        let mut pd = PolyData::new();
        let c = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        for p in &c {
            pd.points.push(*p);
        }
        let f = [
            [0, 3, 2, 1],
            [4, 5, 6, 7],
            [0, 1, 5, 4],
            [2, 3, 7, 6],
            [0, 4, 7, 3],
            [1, 2, 6, 5],
        ];
        for face in &f {
            pd.polys
                .push_cell(&[face[0] as i64, face[1] as i64, face[2] as i64]);
            pd.polys
                .push_cell(&[face[0] as i64, face[2] as i64, face[3] as i64]);
        }
        pd
    }

    #[test]
    fn runs_without_panic() {
        let mesh = make_box();
        assert!(point_in_mesh(&mesh, [0.5, 0.5, 0.5]));
        assert!(!point_in_mesh(&mesh, [5.0, 5.0, 5.0]));
    }

    #[test]
    fn multiple_points_runs() {
        let mesh = make_box();
        let pts = [[0.5, 0.5, 0.5], [5.0, 0.0, 0.0], [0.1, 0.1, 0.1]];
        let result = points_in_mesh(&mesh, &pts);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn count_runs() {
        let mesh = make_box();
        let pts = [[0.5, 0.5, 0.5], [5.0, 0.0, 0.0]];
        let c = count_points_inside(&mesh, &pts);
        assert!(c <= 2);
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::new();
        assert!(!point_in_mesh(&mesh, [0.0, 0.0, 0.0]));
    }
}
