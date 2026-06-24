//! Assign each vertex to nearest seed point (Voronoi partition on mesh).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn voronoi_partition(mesh: &PolyData, seeds: &[[f64; 3]]) -> PolyData {
    let n = mesh.points.len();
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            let mut best = 0usize;
            let mut best_d = f64::INFINITY;
            for (si, s) in seeds.iter().enumerate() {
                let d = (p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = si;
                }
            }
            best as f64
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiLabel",
            labels,
            1,
        )));
    r.point_data_mut().set_active_scalars("VoronoiLabel");
    r
}
pub fn voronoi_partition_by_vertices(mesh: &PolyData, seed_indices: &[usize]) -> PolyData {
    let seeds: Vec<[f64; 3]> = seed_indices.iter().map(|&i| mesh.points.get(i)).collect();
    voronoi_partition(mesh, &seeds)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_partition() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = voronoi_partition(&m, &[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
        let arr = r.point_data().get_array("VoronoiLabel").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
