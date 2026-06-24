//! Compute discrete Laplacian vector per vertex.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute Laplacian vector (difference from neighbor average) per vertex.
pub fn laplacian_vector(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
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
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = mesh.points.get(i);
        if nb[i].is_empty() {
            data.extend_from_slice(&[0.0, 0.0, 0.0]);
            continue;
        }
        let k = nb[i].len() as f64;
        let mut avg = [0.0, 0.0, 0.0];
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            avg[0] += q[0];
            avg[1] += q[1];
            avg[2] += q[2];
        }
        data.push(avg[0] / k - p[0]);
        data.push(avg[1] / k - p[1]);
        data.push(avg[2] / k - p[2]);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Laplacian", data, 3)));
    r
}

/// Compute Laplacian magnitude as scalar.
pub fn laplacian_magnitude(mesh: &PolyData) -> PolyData {
    let r = laplacian_vector(mesh);
    let arr = r.point_data().get_array("Laplacian").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let mag: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0] * buf[0] + buf[1] * buf[1] + buf[2] * buf[2]).sqrt()
        })
        .collect();
    let mut result = r;
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LaplacianMag",
            mag,
            1,
        )));
    result.point_data_mut().set_active_scalars("LaplacianMag");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lap_vec() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = laplacian_vector(&mesh);
        assert!(r.point_data().get_array("Laplacian").is_some());
    }
    #[test]
    fn test_lap_mag() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = laplacian_magnitude(&mesh);
        assert!(r.point_data().get_array("LaplacianMag").is_some());
    }
}
