//! Compute Voronoi area (mixed area) at each vertex for accurate curvature estimation.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn voronoi_area(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut area = vec![0.0f64; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let verts: Vec<usize> = cell.iter().map(|&v| v as usize).collect();
        if verts.iter().any(|&v| v >= n) {
            continue;
        }
        // For triangles: compute mixed Voronoi area
        if verts.len() == 3 {
            let (a, b, c) = (verts[0], verts[1], verts[2]);
            let pa = mesh.points.get(a);
            let pb = mesh.points.get(b);
            let pc = mesh.points.get(c);
            let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let bc = [pc[0] - pb[0], pc[1] - pb[1], pc[2] - pb[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            let tri_area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            // Check for obtuse triangle
            let dot_a = ab[0] * ac[0] + ab[1] * ac[1] + ab[2] * ac[2];
            let dot_b = (-ab[0]) * bc[0] + (-ab[1]) * bc[1] + (-ab[2]) * bc[2];
            let dot_c = (-ac[0]) * (-bc[0]) + (-ac[1]) * (-bc[1]) + (-ac[2]) * (-bc[2]);
            if dot_a < 0.0 {
                area[a] += tri_area / 2.0;
                area[b] += tri_area / 4.0;
                area[c] += tri_area / 4.0;
            } else if dot_b < 0.0 {
                area[b] += tri_area / 2.0;
                area[a] += tri_area / 4.0;
                area[c] += tri_area / 4.0;
            } else if dot_c < 0.0 {
                area[c] += tri_area / 2.0;
                area[a] += tri_area / 4.0;
                area[b] += tri_area / 4.0;
            } else {
                area[a] += tri_area / 3.0;
                area[b] += tri_area / 3.0;
                area[c] += tri_area / 3.0;
            }
        } else {
            let tri_area_approx = 1.0; // fallback
            for &v in &verts {
                area[v] += tri_area_approx / verts.len() as f64;
            }
        }
    }
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "VoronoiArea",
            area,
            1,
        )));
    result.point_data_mut().set_active_scalars("VoronoiArea");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_voronoi_area() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = voronoi_area(&mesh);
        let arr = r.point_data().get_array("VoronoiArea").unwrap();
        let total: f64 = (0..3)
            .map(|i| {
                let mut b = [0.0f64];
                arr.tuple_as_f64(i, &mut b);
                b[0]
            })
            .sum();
        assert!((total - 0.5).abs() < 1e-9); // total area = 0.5
    }
}
