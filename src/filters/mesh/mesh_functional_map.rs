//! Functional map between two meshes (spectral correspondence).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn compute_functional_map(
    mesh_a: &PolyData,
    mesh_b: &PolyData,
    num_basis: usize,
    iterations: usize,
) -> Vec<Vec<f64>> {
    let basis_a = compute_basis(mesh_a, num_basis, iterations);
    let basis_b = compute_basis(mesh_b, num_basis, iterations);
    // Functional map C: f_B = C * f_A
    // Simplified: identity map (assuming similar shapes)
    let mut c = vec![vec![0.0f64; num_basis]; num_basis];
    for i in 0..num_basis {
        c[i][i] = 1.0;
    }
    c
}
pub fn transfer_function(
    mesh_a: &PolyData,
    mesh_b: &PolyData,
    array_name: &str,
    num_basis: usize,
    iterations: usize,
) -> PolyData {
    let arr = match mesh_a.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh_b.clone(),
    };
    let na = mesh_a.points.len();
    let nb = mesh_b.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let basis_a = compute_basis(mesh_a, num_basis, iterations);
    let basis_b = compute_basis(mesh_b, num_basis, iterations);
    // Project function onto basis A
    let coeffs: Vec<f64> = (0..num_basis)
        .map(|k| {
            (0..na.min(vals.len()))
                .map(|i| vals[i] * basis_a[k][i])
                .sum::<f64>()
        })
        .collect();
    // Reconstruct on mesh B (using identity functional map)
    let transferred: Vec<f64> = (0..nb)
        .map(|i| {
            (0..num_basis)
                .map(|k| {
                    coeffs[k]
                        * if i < basis_b[k].len() {
                            basis_b[k][i]
                        } else {
                            0.0
                        }
                })
                .sum()
        })
        .collect();
    let mut r = mesh_b.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name,
            transferred,
            1,
        )));
    r
}
fn compute_basis(mesh: &PolyData, num_basis: usize, iterations: usize) -> Vec<Vec<f64>> {
    let n = mesh.points.len();
    let ne = num_basis.min(n).max(1);
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
    let mut eigvecs: Vec<Vec<f64>> = Vec::new();
    for ei in 0..ne {
        let mut v: Vec<f64> = (0..n)
            .map(|i| (i as f64 * 0.73 + ei as f64 * 1.37).sin())
            .collect();
        for _ in 0..iterations {
            let mut lv = vec![0.0f64; n];
            for i in 0..n {
                if nb[i].is_empty() {
                    continue;
                }
                lv[i] = nb[i].len() as f64 * v[i];
                for &j in &nb[i] {
                    lv[i] -= v[j];
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
    eigvecs
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fmap() {
        let a = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let c = compute_functional_map(&a, &a, 3, 20);
        assert_eq!(c.len(), 3);
    }
    #[test]
    fn test_transfer() {
        let mut a = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        let b = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = transfer_function(&a, &b, "s", 3, 20);
        assert!(r.point_data().get_array("s").is_some());
    }
}
