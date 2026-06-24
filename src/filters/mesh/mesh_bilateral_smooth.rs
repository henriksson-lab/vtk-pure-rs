//! Bilateral mesh smoothing (preserves sharp features).
use crate::data::{Points, PolyData};

pub fn bilateral_smooth(
    mesh: &PolyData,
    iterations: usize,
    sigma_s: f64,
    sigma_r: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    // Compute vertex normals
    let mut vnorm = vec![[0.0f64; 3]; n];
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
        for &vi in &cell[..] {
            let vi = vi as usize;
            if vi < n {
                vnorm[vi][0] += nx;
                vnorm[vi][1] += ny;
                vnorm[vi][2] += nz;
            }
        }
    }
    for vn in &mut vnorm {
        let l = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if l > 1e-15 {
            vn[0] /= l;
            vn[1] /= l;
            vn[2] /= l;
        }
    }
    let mut positions: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            [p[0], p[1], p[2]]
        })
        .collect();
    let ss2 = 2.0 * sigma_s * sigma_s;
    let sr2 = 2.0 * sigma_r * sigma_r;
    for _ in 0..iterations {
        let mut new_pos = positions.clone();
        for i in 0..n {
            if adj[i].is_empty() {
                continue;
            }
            let ni = vnorm[i];
            let mut sum = 0.0f64;
            let mut wsum = 0.0f64;
            for &j in &adj[i] {
                let d = [
                    positions[j][0] - positions[i][0],
                    positions[j][1] - positions[i][1],
                    positions[j][2] - positions[i][2],
                ];
                let dist2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                let h = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2]; // projection onto normal
                let ws = (-dist2 / ss2).exp();
                let wr = (-h * h / sr2).exp();
                let w = ws * wr;
                sum += w * h;
                wsum += w;
            }
            if wsum > 1e-15 {
                let offset = sum / wsum;
                new_pos[i][0] += offset * ni[0];
                new_pos[i][1] += offset * ni[1];
                new_pos[i][2] += offset * ni[2];
            }
        }
        positions = new_pos;
    }
    let mut pts = Points::<f64>::new();
    for p in &positions {
        pts.push(*p);
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = mesh.polys.clone();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bilateral() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.3],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = bilateral_smooth(&mesh, 3, 1.0, 0.5);
        assert_eq!(r.points.len(), 4);
    }
}
