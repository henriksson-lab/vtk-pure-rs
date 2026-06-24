//! Compute discrete Laplacian of a scalar field on mesh.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn scalar_laplacian(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
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
    let lap: Vec<f64> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let k = nb[i].len() as f64;
            nb[i].iter().map(|&j| vals[j] - vals[i]).sum::<f64>() / k
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Laplacian", lap, 1)));
    r.point_data_mut().set_active_scalars("Laplacian");
    r
}
pub fn scalar_bilaplacian(mesh: &PolyData, array_name: &str) -> PolyData {
    let first = scalar_laplacian(mesh, array_name);
    scalar_laplacian(&first, "Laplacian")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lap() {
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
                "s",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));
        let r = scalar_laplacian(&m, "s");
        assert!(r.point_data().get_array("Laplacian").is_some());
    }
    #[test]
    fn test_bilap() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = scalar_bilaplacian(&m, "s");
        assert!(r.point_data().get_array("Laplacian").is_some());
    }
}
