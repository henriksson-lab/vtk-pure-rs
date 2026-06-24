//! Edge flipping to improve triangle quality (maximize minimum angle).
use crate::data::{CellArray, PolyData};

pub fn edge_flip_improve(mesh: &PolyData, max_iterations: usize) -> PolyData {
    let n = mesh.points.len();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    if tris.len() < 2 {
        return mesh.clone();
    }
    for _ in 0..max_iterations {
        let mut flipped = false;
        let mut edge_tris: std::collections::HashMap<(usize, usize), Vec<usize>> =
            std::collections::HashMap::new();
        for (ti, &[a, b, c]) in tris.iter().enumerate() {
            for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
                let e = if e0 < e1 { (e0, e1) } else { (e1, e0) };
                edge_tris.entry(e).or_default().push(ti);
            }
        }
        for (&(e0, e1), faces) in &edge_tris {
            if faces.len() != 2 {
                continue;
            }
            let t0 = faces[0];
            let t1 = faces[1];
            let opp0 = tris[t0].iter().find(|&&v| v != e0 && v != e1).copied();
            let opp1 = tris[t1].iter().find(|&&v| v != e0 && v != e1).copied();
            if let (Some(o0), Some(o1)) = (opp0, opp1) {
                let min_before = min_angle_pair(&mesh, &tris[t0], &tris[t1]);
                let new_t0 = [e0, o0, o1];
                let new_t1 = [e1, o1, o0];
                let min_after = min_angle_pair(&mesh, &new_t0, &new_t1);
                if min_after > min_before + 0.01 {
                    tris[t0] = new_t0;
                    tris[t1] = new_t1;
                    flipped = true;
                }
            }
        }
        if !flipped {
            break;
        }
    }
    let mut polys = CellArray::new();
    for &[a, b, c] in &tris {
        polys.push_cell(&[a as i64, b as i64, c as i64]);
    }
    let mut result = PolyData::new();
    result.points = mesh.points.clone();
    result.polys = polys;
    result
}

fn min_angle_pair(mesh: &PolyData, t0: &[usize; 3], t1: &[usize; 3]) -> f64 {
    min_angle(mesh, t0).min(min_angle(mesh, t1))
}

fn min_angle(mesh: &PolyData, tri: &[usize; 3]) -> f64 {
    let n = mesh.points.len();
    if tri[0] >= n || tri[1] >= n || tri[2] >= n {
        return 0.0;
    }
    let pts: Vec<[f64; 3]> = tri.iter().map(|&v| mesh.points.get(v)).collect();
    let mut min_a = std::f64::consts::PI;
    for i in 0..3 {
        let p = pts[i];
        let a = pts[(i + 1) % 3];
        let b = pts[(i + 2) % 3];
        let u = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
        let v = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
        let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if lu < 1e-15 || lv < 1e-15 {
            return 0.0;
        }
        let cos_a = (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (lu * lv);
        min_a = min_a.min(cos_a.clamp(-1.0, 1.0).acos());
    }
    min_a
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flip() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 0.1, 0.0],
                [1.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let r = edge_flip_improve(&mesh, 5);
        assert_eq!(r.polys.num_cells(), 2);
    }
}
