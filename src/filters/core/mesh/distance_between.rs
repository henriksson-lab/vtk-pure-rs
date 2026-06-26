//! Compute distance between two meshes (Hausdorff and average).

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Distance metrics between two meshes.
pub struct MeshDistance {
    pub hausdorff: f64,
    pub mean: f64,
    pub rms: f64,
}

/// Compute distance from each vertex of mesh A to closest point on mesh B.
/// Attaches "Distance" point data to mesh A.
pub fn distance_to_mesh(mesh_a: &PolyData, mesh_b: &PolyData) -> PolyData {
    let n = mesh_a.points.len();
    let distances: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh_a.points.get(i);
            min_distance_to_surface(p, mesh_b)
        })
        .collect();
    let mut result = mesh_a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));
    result.point_data_mut().set_active_scalars("Distance");
    result
}

/// Compute distance metrics between two meshes.
pub fn mesh_distance_metrics(mesh_a: &PolyData, mesh_b: &PolyData) -> MeshDistance {
    let n = mesh_a.points.len();
    if n == 0 {
        return MeshDistance {
            hausdorff: 0.0,
            mean: 0.0,
            rms: 0.0,
        };
    }
    let distances: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh_a.points.get(i);
            min_distance_to_surface(p, mesh_b)
        })
        .collect();
    let hausdorff = distances.iter().cloned().fold(0.0f64, f64::max);
    let mean = distances.iter().sum::<f64>() / n as f64;
    let rms = (distances.iter().map(|d| d * d).sum::<f64>() / n as f64).sqrt();
    MeshDistance {
        hausdorff,
        mean,
        rms,
    }
}

fn min_distance_to_surface(p: [f64; 3], mesh: &PolyData) -> f64 {
    let mut best = f64::INFINITY;

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = mesh.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = mesh.points.get(cell[i] as usize);
            let v2 = mesh.points.get(cell[i + 1] as usize);
            let q = closest_point_on_triangle(&p, &v0, &v1, &v2);
            best = best.min(distance(p, q));
        }
    }

    if best.is_finite() {
        return best;
    }

    for i in 0..mesh.points.len() {
        let q = mesh.points.get(i);
        best = best.min(distance(p, q));
    }
    best
}

fn closest_point_on_triangle(
    p: &[f64; 3],
    v0: &[f64; 3],
    v1: &[f64; 3],
    v2: &[f64; 3],
) -> [f64; 3] {
    let ab = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let ac = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let ap = [p[0] - v0[0], p[1] - v0[1], p[2] - v0[2]];

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return *v0;
    }

    let bp = [p[0] - v1[0], p[1] - v1[1], p[2] - v1[2]];
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return *v1;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [v0[0] + v * ab[0], v0[1] + v * ab[1], v0[2] + v * ab[2]];
    }

    let cp = [p[0] - v2[0], p[1] - v2[1], p[2] - v2[2]];
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return *v2;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [v0[0] + w * ac[0], v0[1] + w * ac[1], v0[2] + w * ac[2]];
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [
            v1[0] + w * (v2[0] - v1[0]),
            v1[1] + w * (v2[1] - v1[1]),
            v1[2] + w * (v2[2] - v1[2]),
        ];
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        v0[0] + ab[0] * v + ac[0] * w,
        v0[1] + ab[1] * v + ac[1] * w,
        v0[2] + ab[2] * v + ac[2] * w,
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_distance() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.5, 1.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_mesh(&a, &b);
        let arr = r.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10);
    }
    #[test]
    fn test_metrics() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 3.0], [1.0, 0.0, 3.0], [0.5, 1.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let m = mesh_distance_metrics(&a, &b);
        assert!((m.hausdorff - 3.0).abs() < 1e-10);
        assert!((m.mean - 3.0).abs() < 1e-10);
    }

    #[test]
    fn distance_uses_triangle_surface_not_only_vertices() {
        let mut a = PolyData::new();
        a.points.push([0.25, 0.25, 2.0]);
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = distance_to_mesh(&a, &b);
        let arr = r.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10);
    }
}
