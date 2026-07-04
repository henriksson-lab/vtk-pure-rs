//! Bilateral mesh smoothing (preserves sharp features).
use crate::data::{Points, PolyData};

pub fn bilateral_smooth(
    mesh: &PolyData,
    iterations: usize,
    sigma_s: f64,
    sigma_r: f64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || iterations == 0 || sigma_s <= 0.0 || sigma_r <= 0.0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc == 0 {
            continue;
        }
        if !valid_cell(cell, n) {
            continue;
        }
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
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
        let vnorm = compute_vertex_normals(mesh, &positions);
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
    let mut result = mesh.clone();
    result.points = pts;
    result
}

fn compute_vertex_normals(mesh: &PolyData, positions: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let n = positions.len();
    let mut vnorm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 || !valid_cell(cell, n) {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        for &vi in cell {
            let vi = vi as usize;
            vnorm[vi][0] += nx;
            vnorm[vi][1] += ny;
            vnorm[vi][2] += nz;
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
    vnorm
}

fn valid_cell(cell: &[i64], npoints: usize) -> bool {
    cell.iter().all(|&id| id >= 0 && (id as usize) < npoints)
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
