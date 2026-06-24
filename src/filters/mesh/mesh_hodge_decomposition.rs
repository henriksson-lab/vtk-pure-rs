//! Hodge decomposition of vector field on mesh (exact + coexact + harmonic).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn hodge_decompose(mesh: &PolyData, vector_array: &str, iterations: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(vector_array) {
        Some(a) if a.num_components() == 3 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64; 3];
    let vecs: Vec<[f64; 3]> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1], buf[2]]
        })
        .collect();
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
    // Compute divergence (scalar potential source)
    let mut div = vec![0.0f64; n];
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let k = nb[i].len() as f64;
        for &j in &nb[i] {
            let p = mesh.points.get(j);
            let pi = mesh.points.get(i);
            let e = [p[0] - pi[0], p[1] - pi[1], p[2] - pi[2]];
            let el = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt().max(1e-15);
            div[i] += (vecs[i][0] * e[0] + vecs[i][1] * e[1] + vecs[i][2] * e[2]) / el;
        }
        div[i] /= k;
    }
    // Solve Poisson for scalar potential phi: Lap(phi) = div
    let mut phi = vec![0.0f64; n];
    for _ in 0..iterations {
        let prev = phi.clone();
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let k = nb[i].len() as f64;
            phi[i] = (nb[i].iter().map(|&j| prev[j]).sum::<f64>() + div[i]) / k;
        }
    }
    // Exact part (gradient of phi)
    let mut exact = vec![[0.0f64; 3]; n];
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        for &j in &nb[i] {
            let p = mesh.points.get(j);
            let pi = mesh.points.get(i);
            let e = [p[0] - pi[0], p[1] - pi[1], p[2] - pi[2]];
            let el = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt().max(1e-15);
            let dphi = (phi[j] - phi[i]) / el;
            exact[i][0] += dphi * e[0] / el;
            exact[i][1] += dphi * e[1] / el;
            exact[i][2] += dphi * e[2] / el;
        }
        let k = nb[i].len() as f64;
        exact[i][0] /= k;
        exact[i][1] /= k;
        exact[i][2] /= k;
    }
    // Harmonic = original - exact (simplified: coexact lumped into harmonic)
    let harmonic: Vec<f64> = (0..n)
        .flat_map(|i| {
            vec![
                vecs[i][0] - exact[i][0],
                vecs[i][1] - exact[i][1],
                vecs[i][2] - exact[i][2],
            ]
        })
        .collect();
    let exact_data: Vec<f64> = exact.iter().flat_map(|e| e.iter().copied()).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ExactPart",
            exact_data,
            3,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "HarmonicPart",
            harmonic,
            3,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ScalarPotential",
            phi,
            1,
        )));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "V",
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));
        let r = hodge_decompose(&m, "V", 30);
        assert!(r.point_data().get_array("ExactPart").is_some());
        assert!(r.point_data().get_array("HarmonicPart").is_some());
        assert!(r.point_data().get_array("ScalarPotential").is_some());
    }
}
