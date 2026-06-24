//! Assign random colors to mesh vertices.
use crate::data::{AnyDataArray, DataArray, PolyData};
/// Assign random RGB colors to each vertex.
pub fn random_vertex_colors(mesh: &PolyData, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let mut rng = seed;
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        data.push(((rng >> 33) as f64 / u32::MAX as f64) * 255.0);
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        data.push(((rng >> 33) as f64 / u32::MAX as f64) * 255.0);
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        data.push(((rng >> 33) as f64 / u32::MAX as f64) * 255.0);
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", data, 3)));
    r
}
/// Assign random color per face (cell data).
pub fn random_face_colors(mesh: &PolyData, seed: u64) -> PolyData {
    let nc = mesh.polys.num_cells();
    let mut rng = seed;
    let mut data = Vec::with_capacity(nc * 3);
    for _ in 0..nc {
        for _ in 0..3 {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            data.push(((rng >> 33) as f64 / u32::MAX as f64) * 255.0);
        }
    }
    let mut r = mesh.clone();
    r.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", data, 3)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_vert_colors() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = random_vertex_colors(&mesh, 42);
        assert!(r.point_data().get_array("Colors").is_some());
        assert_eq!(
            r.point_data().get_array("Colors").unwrap().num_components(),
            3
        );
    }
    #[test]
    fn test_face_colors() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = random_face_colors(&mesh, 42);
        assert!(r.cell_data().get_array("Colors").is_some());
    }
}
