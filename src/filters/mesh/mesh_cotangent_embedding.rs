//! Cotangent-weighted spectral embedding for mesh flattening.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn spectral_embedding_2d(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n < 3 {
        return mesh.clone();
    }
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    // Cotangent weights
    for c in &cells {
        if c.len() != 3 {
            continue;
        }
        let ids = [c[0] as usize, c[1] as usize, c[2] as usize];
        let p = [
            mesh.points.get(ids[0]),
            mesh.points.get(ids[1]),
            mesh.points.get(ids[2]),
        ];
        for i in 0..3 {
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;
            let eij = [p[j][0] - p[i][0], p[j][1] - p[i][1], p[j][2] - p[i][2]];
            let eik = [p[k][0] - p[i][0], p[k][1] - p[i][1], p[k][2] - p[i][2]];
            let dot = eij[0] * eik[0] + eij[1] * eik[1] + eij[2] * eik[2];
            let cl = ((eij[1] * eik[2] - eij[2] * eik[1]).powi(2)
                + (eij[2] * eik[0] - eij[0] * eik[2]).powi(2)
                + (eij[0] * eik[1] - eij[1] * eik[0]).powi(2))
            .sqrt();
            let cot = if cl > 1e-15 {
                (dot / cl).abs() * 0.5
            } else {
                0.01
            };
            nb[ids[j]].push((ids[k], cot));
            nb[ids[k]].push((ids[j], cot));
        }
    }
    // Compute two smallest non-trivial eigenvectors
    let mut eigvecs: Vec<Vec<f64>> = Vec::new();
    for ei in 0..2 {
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 * 0.73 + ei as f64 * 1.37).sin())
            .collect();
        for _ in 0..iterations {
            let mut lv = vec![0.0f64; n];
            for i in 0..n {
                let mut wsum = 0.0;
                for &(j, w) in &nb[i] {
                    lv[i] += w * (v[i] - v[j]);
                    wsum += w;
                }
                if wsum > 0.0 {
                    lv[i] /= wsum;
                } else {
                    lv[i] = 0.0;
                }
            }
            let mean = lv.iter().sum::<f64>() / n as f64;
            for x in &mut lv {
                *x -= mean;
            }
            for prev in &eigvecs {
                let dot: f64 = lv.iter().zip(prev).map(|(a, b)| a * b).sum();
                for j in 0..n {
                    lv[j] -= dot * prev[j];
                }
            }
            let norm = lv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-15);
            v = lv.iter().map(|x| x / norm).collect();
        }
        eigvecs.push(v);
    }
    let data: Vec<f64> = (0..n)
        .flat_map(|i| vec![eigvecs[0][i], eigvecs[1.min(eigvecs.len() - 1)][i]])
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SpectralUV",
            data,
            2,
        )));
    r.point_data_mut().set_active_tcoords("SpectralUV");
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
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = spectral_embedding_2d(&m, 50);
        assert!(r.point_data().get_array("SpectralUV").is_some());
        assert_eq!(
            r.point_data()
                .get_array("SpectralUV")
                .unwrap()
                .num_components(),
            2
        );
    }
}
