//! Biharmonic (4th order) smoothing.
use crate::data::PolyData;
pub fn biharmonic_smooth(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
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
    for _ in 0..iterations {
        // First Laplacian
        let lap1: Vec<[f64; 3]> = (0..n)
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
                    avg[0] / k - pos[i][0],
                    avg[1] / k - pos[i][1],
                    avg[2] / k - pos[i][2],
                ]
            })
            .collect();
        // Second Laplacian (Laplacian of Laplacian)
        let lap2: Vec<[f64; 3]> = (0..n)
            .map(|i| {
                if nb[i].is_empty() {
                    return [0.0, 0.0, 0.0];
                }
                let k = nb[i].len() as f64;
                let mut avg = [0.0, 0.0, 0.0];
                for &j in &nb[i] {
                    avg[0] += lap1[j][0];
                    avg[1] += lap1[j][1];
                    avg[2] += lap1[j][2];
                }
                [
                    avg[0] / k - lap1[i][0],
                    avg[1] / k - lap1[i][1],
                    avg[2] / k - lap1[i][2],
                ]
            })
            .collect();
        for i in 0..n {
            pos[i][0] -= lambda * lap2[i][0];
            pos[i][1] -= lambda * lap2[i][1];
            pos[i][2] -= lambda * lap2[i][2];
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
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [1.0, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = biharmonic_smooth(&m, 3, 0.1);
        assert_eq!(r.points.len(), 4);
    }
}
