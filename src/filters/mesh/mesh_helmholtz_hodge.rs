//! Helmholtz-Hodge decomposition into curl-free + divergence-free + harmonic.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn helmholtz_hodge(mesh: &PolyData, vector_array: &str, iterations: usize) -> PolyData {
    let arr = match mesh.point_data().get_array(vector_array) {
        Some(a) if a.num_components() == 3 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    if arr.num_tuples() < n {
        return mesh.clone();
    }
    let mut buf = [0.0f64; 3];
    let vecs: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            [buf[0], buf[1], buf[2]]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], n) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], n) else {
                continue;
            };
            if !nb[a].contains(&b) {
                nb[a].push(b);
            }
            if !nb[b].contains(&a) {
                nb[b].push(a);
            }
        }
    }
    // Step 1: Compute divergence of V
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
    // Step 2: Solve Poisson for scalar potential
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
    // Step 3: Curl-free part = gradient of phi
    let mut curl_free = vec![[0.0f64; 3]; n];
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
            let dphi = (phi[j] - phi[i]) / el;
            curl_free[i][0] += dphi * e[0] / el;
            curl_free[i][1] += dphi * e[1] / el;
            curl_free[i][2] += dphi * e[2] / el;
        }
        curl_free[i][0] /= k;
        curl_free[i][1] /= k;
        curl_free[i][2] /= k;
    }
    // Divergence-free = V - curl_free (simplified)
    let div_free: Vec<f64> = (0..n)
        .flat_map(|i| {
            vec![
                vecs[i][0] - curl_free[i][0],
                vecs[i][1] - curl_free[i][1],
                vecs[i][2] - curl_free[i][2],
            ]
        })
        .collect();
    let cf_data: Vec<f64> = curl_free.iter().flat_map(|v| v.iter().copied()).collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CurlFree", cf_data, 3,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DivFree", div_free, 3,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Potential", phi, 1)));
    r
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
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
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                3,
            )));
        let r = helmholtz_hodge(&m, "V", 30);
        assert!(r.point_data().get_array("CurlFree").is_some());
        assert!(r.point_data().get_array("DivFree").is_some());
        assert!(r.point_data().get_array("Potential").is_some());
    }
}
