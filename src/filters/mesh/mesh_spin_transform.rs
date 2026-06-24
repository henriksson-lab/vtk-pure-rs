//! Spin transformation for conformal mesh deformation.
use crate::data::PolyData;
pub fn spin_deform(
    mesh: &PolyData,
    handle: usize,
    target: [f64; 3],
    iterations: usize,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || handle >= n {
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
    // Compute original Laplacian coordinates
    let lap: Vec<[f64; 3]> = (0..n)
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
    // Apply handle constraint
    pos[handle] = target;
    // Iterative reconstruction preserving Laplacian coordinates
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if i == handle || nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let mut avg = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                avg[0] += prev[j][0];
                avg[1] += prev[j][1];
                avg[2] += prev[j][2];
            }
            pos[i][0] = avg[0] / k + lap[i][0];
            pos[i][1] = avg[1] / k + lap[i][1];
            pos[i][2] = avg[2] / k + lap[i][2];
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}
pub fn as_rigid_as_possible(
    mesh: &PolyData,
    handles: &[(usize, [f64; 3])],
    iterations: usize,
) -> PolyData {
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
    let handle_map: std::collections::HashMap<usize, [f64; 3]> = handles.iter().cloned().collect();
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for (&vi, &target) in &handle_map {
        if vi < n {
            pos[vi] = target;
        }
    }
    let orig: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let prev = pos.clone();
        for i in 0..n {
            if handle_map.contains_key(&i) || nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            let mut sum = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                let orig_diff = [
                    orig[i][0] - orig[j][0],
                    orig[i][1] - orig[j][1],
                    orig[i][2] - orig[j][2],
                ];
                sum[0] += prev[j][0] + orig_diff[0];
                sum[1] += prev[j][1] + orig_diff[1];
                sum[2] += prev[j][2] + orig_diff[2];
            }
            pos[i] = [sum[0] / k, sum[1] / k, sum[2] / k];
        }
    }
    let mut r = mesh.clone();
    for i in 0..n {
        r.points.set(i, pos[i]);
    }
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spin() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = spin_deform(&m, 3, [1.0, 0.5, 2.0], 20);
        let p = r.points.get(3);
        assert!((p[2] - 2.0).abs() < 0.1);
    }
    #[test]
    fn test_arap() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = as_rigid_as_possible(&m, &[(0, [0.0, 0.0, 0.0]), (3, [1.0, 0.5, 2.0])], 30);
        let p0 = r.points.get(0);
        assert!((p0[0]).abs() < 0.1);
    }
}
