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
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
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
            let v0 = pts[cell[0] as usize];
            let v1 = pts[cell[1] as usize];
            let v2 = pts[cell[2] as usize];
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let fn_ = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            for &id in cell.iter() {
                let i = id as usize;
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
            let mut sx = 0.0;
            let mut sy = 0.0;
            let mut sz = 0.0;
            let mut sw = 0.0;

            for &j in &neighbors[i] {
                let q = pts[j];
                let nj = vnormals[j];
                let d2 = (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2);
                let ndiff = 1.0 - (ni[0] * nj[0] + ni[1] * nj[1] + ni[2] * nj[2]).max(0.0);
                let w = (-d2 * inv_2ss - ndiff * ndiff * inv_2sn).exp();
                sx += w * q[0];
                sy += w * q[1];
                sz += w * q[2];
                sw += w;
            }

            if sw > 1e-15 {
                let alpha = 0.5;
                new_pts[i] = [
                    p[0] * (1.0 - alpha) + sx / sw * alpha,
                    p[1] * (1.0 - alpha) + sy / sw * alpha,
                    p[2] * (1.0 - alpha) + sz / sw * alpha,
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
}
