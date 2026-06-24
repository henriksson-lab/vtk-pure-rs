//! Compute Laplacian magnitude at each vertex (smoothness measure).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn vertex_laplacian_mag(mesh: &PolyData) -> PolyData {
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
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if nb[i].is_empty() {
                return 0.0;
            }
            let p = mesh.points.get(i);
            let k = nb[i].len() as f64;
            let mut lap = [0.0, 0.0, 0.0];
            for &j in &nb[i] {
                let q = mesh.points.get(j);
                lap[0] += q[0] - p[0];
                lap[1] += q[1] - p[1];
                lap[2] += q[2] - p[2];
            }
            (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt() / k
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LaplacianMag",
            data,
            1,
        )));
    r.point_data_mut().set_active_scalars("LaplacianMag");
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
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = vertex_laplacian_mag(&m);
        assert!(r.point_data().get_array("LaplacianMag").is_some());
    }
}
