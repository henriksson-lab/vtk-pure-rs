//! Normal-guided mesh filtering (anisotropic smoothing based on face normals).
use crate::data::PolyData;
pub fn normal_guided_smooth(mesh: &PolyData, iterations: usize, sigma_n: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    // Compute face normals
    let fnormals: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.len() < 3 {
                return [0.0, 0.0, 1.0];
            }
            let a = mesh.points.get(c[0] as usize);
            let b = mesh.points.get(c[1] as usize);
            let cc = mesh.points.get(c[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [cc[0] - a[0], cc[1] - a[1], cc[2] - a[2]];
            let nn = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
            if l < 1e-15 {
                [0.0, 0.0, 1.0]
            } else {
                [nn[0] / l, nn[1] / l, nn[2] / l]
            }
        })
        .collect();
    // Vertex-face adjacency
    let mut vf: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (ci, c) in cells.iter().enumerate() {
        for &v in c {
            vf[v as usize].push(ci);
        }
    }
    let sn2 = 2.0 * sigma_n * sigma_n;
    // Step 1: Filter face normals
    let mut filtered_normals = fnormals.clone();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let nc = c.len();
        for i in 0..nc {
            let a = c[i] as usize;
            let b = c[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut fadj: Vec<Vec<usize>> = vec![Vec::new(); cells.len()];
    for (_, faces) in &ef {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                fadj[faces[i]].push(faces[j]);
                fadj[faces[j]].push(faces[i]);
            }
        }
    }
    for _ in 0..iterations {
        let prev = filtered_normals.clone();
        for ci in 0..cells.len() {
            let mut avg = [0.0, 0.0, 0.0];
            let mut wsum = 0.0;
            let n0 = prev[ci];
            avg[0] += n0[0];
            avg[1] += n0[1];
            avg[2] += n0[2];
            wsum += 1.0;
            for &ni in &fadj[ci] {
                let nn = prev[ni];
                let dot = n0[0] * nn[0] + n0[1] * nn[1] + n0[2] * nn[2];
                let w = (-(1.0 - dot).max(0.0) / sn2).exp();
                avg[0] += nn[0] * w;
                avg[1] += nn[1] * w;
                avg[2] += nn[2] * w;
                wsum += w;
            }
            let l = (avg[0] * avg[0] + avg[1] * avg[1] + avg[2] * avg[2])
                .sqrt()
                .max(1e-15);
            filtered_normals[ci] = [avg[0] / l, avg[1] / l, avg[2] / l];
        }
    }
    // Step 2: Update vertex positions to match filtered normals
    let mut pos: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for _ in 0..iterations {
        let mut new_pos = pos.clone();
        for i in 0..n {
            if vf[i].is_empty() {
                continue;
            }
            let p = pos[i];
            let mut disp = [0.0, 0.0, 0.0];
            let mut count = 0.0;
            for &fi in &vf[i] {
                let fn_ = filtered_normals[fi];
                // Project centroid displacement onto normal
                let c = &cells[fi];
                let mut cx = 0.0;
                let mut cy = 0.0;
                let mut cz = 0.0;
                for &v in c {
                    cx += pos[v as usize][0];
                    cy += pos[v as usize][1];
                    cz += pos[v as usize][2];
                }
                let cn = c.len() as f64;
                cx /= cn;
                cy /= cn;
                cz /= cn;
                let d = (cx - p[0]) * fn_[0] + (cy - p[1]) * fn_[1] + (cz - p[2]) * fn_[2];
                disp[0] += d * fn_[0];
                disp[1] += d * fn_[1];
                disp[2] += d * fn_[2];
                count += 1.0;
            }
            if count > 0.0 {
                new_pos[i] = [
                    p[0] + disp[0] / count,
                    p[1] + disp[1] / count,
                    p[2] + disp[2] / count,
                ];
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
                [1.0, 0.5, 0.3],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let r = normal_guided_smooth(&m, 2, 0.5);
        assert_eq!(r.points.len(), 4);
    }
}
