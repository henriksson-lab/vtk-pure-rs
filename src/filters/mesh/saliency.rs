use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute mesh saliency: multi-scale mean curvature difference.
///
/// At each vertex, computes Gaussian-weighted average curvature at
/// two scales (fine and coarse) and takes the absolute difference.
/// High saliency = visually prominent feature. Adds "Saliency" scalar.
pub fn mesh_saliency(input: &PolyData, fine_sigma: f64, coarse_sigma: f64) -> PolyData {
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

    // Compute raw curvature (Laplacian magnitude)
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut curvature = vec![0.0f64; n];
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let p = pts[i];
        let cnt = neighbors[i].len() as f64;
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for &j in &neighbors[i] {
            lx += pts[j][0] - p[0];
            ly += pts[j][1] - p[1];
            lz += pts[j][2] - p[2];
        }
        curvature[i] = (lx * lx + ly * ly + lz * lz).sqrt() / cnt;
    }

    let smooth = |sigma: f64| -> Vec<f64> {
        let inv_2s2 = 1.0 / (2.0 * sigma * sigma);
        let mut result = curvature.clone();
        for _ in 0..3 {
            let mut new = result.clone();
            for i in 0..n {
                if neighbors[i].is_empty() {
                    continue;
                }
                let mut sum = result[i];
                let mut sw = 1.0;
                for &j in &neighbors[i] {
                    let d2 = (pts[i][0] - pts[j][0]).powi(2)
                        + (pts[i][1] - pts[j][1]).powi(2)
                        + (pts[i][2] - pts[j][2]).powi(2);
                    let w = (-d2 * inv_2s2).exp();
                    sum += w * result[j];
                    sw += w;
                }
                new[i] = sum / sw;
            }
            result = new;
        }
        result
    };

    let fine = smooth(fine_sigma);
    let coarse = smooth(coarse_sigma);
    let saliency: Vec<f64> = (0..n).map(|i| (fine[i] - coarse[i]).abs()).collect();

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Saliency", saliency, 1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saliency_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 2.0]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[2, 0, 3]);

        let result = mesh_saliency(&pd, 0.5, 2.0);
        assert!(result.point_data().get_array("Saliency").is_some());
    }

    #[test]
    fn flat_low_saliency() {
        let mut pd = PolyData::new();
        for j in 0..3 {
            for i in 0..3 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..2 {
            for i in 0..2 {
                let a = (j * 3 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 4]);
                pd.polys.push_cell(&[a, a + 4, a + 3]);
            }
        }

        let result = mesh_saliency(&pd, 0.5, 2.0);
        let arr = result.point_data().get_array("Saliency").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] < 0.5); // flat -> low saliency
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = mesh_saliency(&pd, 0.5, 2.0);
        assert_eq!(result.points.len(), 0);
    }
}
