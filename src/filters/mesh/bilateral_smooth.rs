use crate::data::{Points, PolyData};

/// Bilateral mesh smoothing: edge-preserving vertex position smoothing.
///
/// Combines spatial proximity and normal similarity weights.
/// Vertices move toward neighbors with similar normals but resist
/// crossing sharp edges. Preserves features better than Laplacian.
pub fn bilateral_mesh_smooth(
    input: &PolyData,
    sigma_spatial: f64,
    sigma_normal: f64,
    iterations: usize,
) -> PolyData {
    let n = input.points.len();
    if n == 0 || iterations == 0 || sigma_spatial <= 0.0 || sigma_normal <= 0.0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        let num_ids = cell.len();
        for i in 0..num_ids {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % num_ids] as usize;
            if a >= n || b >= n {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let inv_2ss = 1.0 / (2.0 * sigma_spatial * sigma_spatial);
    let inv_2sn = 1.0 / (2.0 * sigma_normal * sigma_normal);

    for _ in 0..iterations {
        // Compute normals
        let mut vnormals = vec![[0.0f64; 3]; n];
        for cell in input.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let a = cell[0] as usize;
            let b = cell[1] as usize;
            let c = cell[2] as usize;
            if a >= n || b >= n || c >= n {
                continue;
            }
            let v0 = pts[a];
            let v1 = pts[b];
            let v2 = pts[c];
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for &id in cell.iter() {
                let i = id as usize;
                if i >= n {
                    continue;
                }
                vnormals[i][0] += fn_[0];
                vnormals[i][1] += fn_[1];
                vnormals[i][2] += fn_[2];
            }
        }
        for nm in &mut vnormals {
            let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
            if l > 1e-15 {
                nm[0] /= l;
                nm[1] /= l;
                nm[2] /= l;
            }
        }

        let mut new_pts = pts.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let p = pts[i];
            let ni = vnormals[i];
            let mut sum = 0.0;
            let mut sw = 0.0;

            for &j in &neighbors[i] {
                let q = pts[j];
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                let h = d[0] * ni[0] + d[1] * ni[1] + d[2] * ni[2];
                let w = (-d2 * inv_2ss - h * h * inv_2sn).exp();
                sum += w * h;
                sw += w;
            }

            if sw > 1e-15 {
                let offset = sum / sw;
                new_pts[i] = [
                    p[0] + offset * ni[0],
                    p[1] + offset * ni[1],
                    p[2] + offset * ni[2],
                ];
            }
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooths_noise() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.3]); // slightly noisy
        pd.polys.push_cell(&[0, 1, 2]);

        let result = bilateral_mesh_smooth(&pd, 1.0, 1.0, 3);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn zero_iterations() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = bilateral_mesh_smooth(&pd, 1.0, 1.0, 0);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = bilateral_mesh_smooth(&pd, 1.0, 1.0, 5);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn invalid_connectivity_is_ignored() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 99]);

        let result = bilateral_mesh_smooth(&pd, 1.0, 1.0, 1);
        assert_eq!(result.points.len(), 2);
    }
}
