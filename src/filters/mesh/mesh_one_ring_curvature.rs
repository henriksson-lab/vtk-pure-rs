//! Estimate curvature at each vertex from the one-ring neighborhood.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn one_ring_curvature(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !adj[a].contains(&b) {
                    adj[a].push(b);
                }
                if !adj[b].contains(&a) {
                    adj[b].push(a);
                }
            }
        }
    }
    // Vertex normals
    let mut vnorm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let a = cell[0] as usize;
        let b = cell[1] as usize;
        let c = cell[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = mesh.points.get(a);
        let pb = mesh.points.get(b);
        let pc = mesh.points.get(c);
        let u = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let v = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let nx = u[1] * v[2] - u[2] * v[1];
        let ny = u[2] * v[0] - u[0] * v[2];
        let nz = u[0] * v[1] - u[1] * v[0];
        for &vi in &cell[..] {
            let vi = vi as usize;
            if vi < n {
                vnorm[vi][0] += nx;
                vnorm[vi][1] += ny;
                vnorm[vi][2] += nz;
            }
        }
    }
    for vn in &mut vnorm {
        let l = (vn[0] * vn[0] + vn[1] * vn[1] + vn[2] * vn[2]).sqrt();
        if l > 1e-15 {
            vn[0] /= l;
            vn[1] /= l;
            vn[2] /= l;
        }
    }
    let mut curvature = vec![0.0f64; n];
    for i in 0..n {
        if adj[i].is_empty() {
            continue;
        }
        let p = mesh.points.get(i);
        let nn = vnorm[i];
        let k = adj[i].len() as f64;
        let mut lap = [0.0f64; 3];
        for &j in &adj[i] {
            let q = mesh.points.get(j);
            lap[0] += q[0] - p[0];
            lap[1] += q[1] - p[1];
            lap[2] += q[2] - p[2];
        }
        lap[0] /= k;
        lap[1] /= k;
        lap[2] /= k;
        // Mean curvature ~ projection of Laplacian onto normal
        curvature[i] = (lap[0] * nn[0] + lap[1] * nn[1] + lap[2] * nn[2]).abs();
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "OneRingCurvature",
            curvature,
            1,
        )));
    result
        .point_data_mut()
        .set_active_scalars("OneRingCurvature");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_one_ring() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        let r = one_ring_curvature(&mesh);
        assert!(r.point_data().get_array("OneRingCurvature").is_some());
    }
}
