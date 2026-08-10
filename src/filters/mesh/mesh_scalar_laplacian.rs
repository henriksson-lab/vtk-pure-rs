//! Compute discrete Laplacian of a scalar field on mesh vertices.
use crate::data::{AnyDataArray, DataArray, PolyData};

/// Discrete (umbrella) Laplacian of a scalar field, written to
/// `"<scalar_name>_laplacian"`.
///
/// Renaming wrapper over
/// [`crate::filters::mesh::mesh_scalar_field_laplacian::scalar_laplacian`],
/// which holds the single implementation and writes to a fixed "Laplacian"
/// array.
pub fn scalar_laplacian(mesh: &PolyData, scalar_name: &str) -> PolyData {
    let n = mesh.points.len();
    match mesh.point_data().get_array(scalar_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == n => {}
        _ => return mesh.clone(),
    }

    let base =
        crate::filters::mesh::mesh_scalar_field_laplacian::scalar_laplacian(mesh, scalar_name);
    let Some(arr) = base.point_data().get_array("Laplacian") else {
        return mesh.clone();
    };
    let mut buf = [0.0f64];
    let lap: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let out = format!("{}_laplacian", scalar_name);
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&out, lap, 1)));
    result.point_data_mut().set_active_scalars(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lap() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 10.0, 0.0, 0.0],
                1,
            )));
        let r = scalar_laplacian(&mesh, "f");
        let arr = r.point_data().get_array("f_laplacian").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(1, &mut b);
        assert!(b[0] < 0.0); // vertex 1 is higher than neighbors → negative Laplacian
    }
}
