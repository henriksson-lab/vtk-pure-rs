//! Angle-weighted Laplacian smoothing.
use crate::data::PolyData;
pub fn smooth_angle_weighted(mesh: &PolyData, iterations: usize, lambda: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        for i in 0..3 {
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;
            let v1 = [p[j][0] - p[i][0], p[j][1] - p[i][1], p[j][2] - p[i][2]];
            let v2 = [p[k][0] - p[i][0], p[k][1] - p[i][1], p[k][2] - p[i][2]];
            let l1 = (v1[0] * v1[0] + v1[1] * v1[1] + v1[2] * v1[2]).sqrt();
            let l2 = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
            let angle = if l1 > 1e-15 && l2 > 1e-15 {
                ((v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]) / (l1 * l2))
                    .clamp(-1.0, 1.0)
                    .acos()
            } else {
                0.0
            };
            nb[ids[i]].push((ids[j], angle));
            nb[ids[i]].push((ids[k], angle));
        }
    }
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let mut new_pos = pos.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let mut avg = [0.0, 0.0, 0.0];
            let mut wsum = 0.0;
            for &(j, w) in &nb[i] {
                avg[0] += pos[j][0] * w;
                avg[1] += pos[j][1] * w;
                avg[2] += pos[j][2] * w;
                wsum += w;
            }
            if wsum > 1e-15 {
                avg[0] /= wsum;
                avg[1] /= wsum;
                avg[2] /= wsum;
                new_pos[i][0] = pos[i][0] + lambda * (avg[0] - pos[i][0]);
                new_pos[i][1] = pos[i][1] + lambda * (avg[1] - pos[i][1]);
                new_pos[i][2] = pos[i][2] + lambda * (avg[2] - pos[i][2]);
            }
        }
        pos = new_pos;
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
        let r = smooth_angle_weighted(&m, 3, 0.3);
        assert_eq!(r.points.len(), 4);
    }
}
