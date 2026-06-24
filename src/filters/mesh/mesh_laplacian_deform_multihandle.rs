//! Multi-handle Laplacian deformation with soft constraints.
use crate::data::PolyData;
pub fn soft_laplacian_deform(
    mesh: &PolyData,
    handles: &[(usize, [f64; 3], f64)],
    iterations: usize,
) -> PolyData {
    // handles: (vertex_id, target_position, weight)
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    let lap_orig: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return [0.0, 0.0, 0.0];
            }
            let k = nb[i].len() as f64;
            let mut avg = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                avg[0] += pos[j][0];
                avg[1] += pos[j][1];
                avg[2] += pos[j][2];
            }
            [
                pos[i][0] - avg[0] / k,
                pos[i][1] - avg[1] / k,
                pos[i][2] - avg[2] / k,
            ]
        })
        .collect();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let mut avg = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                avg[0] += prev[j][0];
                avg[1] += prev[j][1];
                avg[2] += prev[j][2];
            }
            pos[i][0] = avg[0] / k + lap_orig[i][0];
            pos[i][1] = avg[1] / k + lap_orig[i][1];
            pos[i][2] = avg[2] / k + lap_orig[i][2];
        }
        // Apply soft handle constraints
        for &(vi, target, weight) in handles {
            if vi < n {
                let w = weight.clamp(0.0, 1.0);
                pos[vi][0] = pos[vi][0] * (1.0 - w) + target[0] * w;
                pos[vi][1] = pos[vi][1] * (1.0 - w) + target[1] * w;
                pos[vi][2] = pos[vi][2] * (1.0 - w) + target[2] * w;
            }
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}
pub fn rigid_handle_deform(
    mesh: &PolyData,
    fixed: &[usize],
    moved: &[(usize, [f64; 3])],
    iterations: usize,
) -> PolyData {
    let mut handles: Vec<(usize, [f64; 3], f64)> = Vec::new();
    for &fi in fixed {
        handles.push((fi, mesh.points.get(fi), 1.0));
    }
    for &(mi, target) in moved {
        handles.push((mi, target, 1.0));
    }
    soft_laplacian_deform(mesh, &handles, iterations)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_soft() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = soft_laplacian_deform(&m, &[(3, [1.0, 0.5, 2.0], 0.9)], 20);
        let p = r.points.get(3);
        assert!((p[2] - 2.0).abs() < 0.5);
    }
    #[test]
    fn test_rigid() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = rigid_handle_deform(&m, &[0, 1], &[(3, [1.0, 0.5, 3.0])], 30);
        let p0 = r.points.get(0);
        assert!((p0[0]).abs() < 1e-5);
    }
}
