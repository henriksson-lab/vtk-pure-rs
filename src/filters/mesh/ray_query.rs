//! Ray query utilities: ray-mesh intersection, ray casting, visibility.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Result of a ray-mesh intersection.
#[derive(Debug, Clone)]
pub struct RayHit {
    pub point: [f64; 3],
    pub distance: f64,
    pub cell_index: usize,
    pub barycentric: [f64; 3],
}

/// Cast a ray and find all intersections with a triangle mesh.
///
/// Delegates to [`crate::filters::mesh::ray_cast_mesh::ray_cast_all`], which
/// holds the single ray/triangle implementation; only the hit record differs
/// (this one reports full barycentric coordinates).
pub fn ray_cast_all(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Vec<RayHit> {
    crate::filters::mesh::ray_cast_mesh::ray_cast_all(mesh, origin, direction)
        .into_iter()
        .map(|hit| RayHit {
            point: hit.point,
            distance: hit.t,
            cell_index: hit.cell_index,
            barycentric: [1.0 - hit.u - hit.v, hit.u, hit.v],
        })
        .collect()
}

/// Cast a ray and find the closest intersection.
pub fn ray_cast_closest(mesh: &PolyData, origin: [f64; 3], direction: [f64; 3]) -> Option<RayHit> {
    ray_cast_all(mesh, origin, direction).into_iter().next()
}

/// Cast multiple rays from a viewpoint and compute per-vertex visibility.
///
/// Returns the mesh with a "Visibility" point data array (0 or 1).
pub fn compute_visibility(mesh: &PolyData, viewpoint: [f64; 3], n_samples: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut visible = vec![0.0f64; n];

    for i in 0..n {
        let p = mesh.points.get(i);
        let dir = [
            p[0] - viewpoint[0],
            p[1] - viewpoint[1],
            p[2] - viewpoint[2],
        ];
        let dist = (dir[0].powi(2) + dir[1].powi(2) + dir[2].powi(2)).sqrt();
        if dist < 1e-15 {
            visible[i] = 1.0;
            continue;
        }

        let hits = ray_cast_all(mesh, viewpoint, dir);
        if let Some(first) = hits.first() {
            if (first.distance - dist).abs() < dist * 0.01 {
                visible[i] = 1.0;
            }
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Visibility",
            visible,
            1,
        )));
    result
}

/// Generate a depth map by casting rays on a regular grid.
pub fn depth_map(
    mesh: &PolyData,
    origin: [f64; 3],
    forward: [f64; 3],
    up: [f64; 3],
    width: usize,
    height: usize,
    fov: f64,
) -> Vec<f64> {
    let fl = (fov.to_radians() / 2.0).tan().recip();
    let flen = (forward[0].powi(2) + forward[1].powi(2) + forward[2].powi(2)).sqrt();
    let fw = [forward[0] / flen, forward[1] / flen, forward[2] / flen];
    let right = [
        fw[1] * up[2] - fw[2] * up[1],
        fw[2] * up[0] - fw[0] * up[2],
        fw[0] * up[1] - fw[1] * up[0],
    ];
    let rlen = (right[0].powi(2) + right[1].powi(2) + right[2].powi(2)).sqrt();
    let rt = [right[0] / rlen, right[1] / rlen, right[2] / rlen];
    let u = [
        fw[1] * rt[2] - fw[2] * rt[1],
        fw[2] * rt[0] - fw[0] * rt[2],
        fw[0] * rt[1] - fw[1] * rt[0],
    ];

    let mut depths = vec![f64::MAX; width * height];
    for py in 0..height {
        for px in 0..width {
            let sx = (2.0 * px as f64 / width as f64 - 1.0) * width as f64 / height as f64;
            let sy = 1.0 - 2.0 * py as f64 / height as f64;
            let dir = [
                fw[0] * fl + rt[0] * sx + u[0] * sy,
                fw[1] * fl + rt[1] * sx + u[1] * sy,
                fw[2] * fl + rt[2] * sx + u[2] * sy,
            ];
            if let Some(hit) = ray_cast_closest(mesh, origin, dir) {
                depths[py * width + px] = hit.distance;
            }
        }
    }
    depths
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cast_at_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let hits = ray_cast_all(&mesh, [1.0, 0.5, -1.0], [0.0, 0.0, 1.0]);
        assert_eq!(hits.len(), 1);
        assert!((hits[0].distance - 1.0).abs() < 0.01);
    }
    #[test]
    fn miss() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(ray_cast_closest(&mesh, [5.0, 5.0, -1.0], [0.0, 0.0, 1.0]).is_none());
    }

    #[test]
    fn ray_starting_on_triangle_is_not_a_forward_hit() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(ray_cast_closest(&mesh, [1.0, 0.5, 0.0], [0.0, 0.0, 1.0]).is_none());
    }

    #[test]
    fn visibility() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_visibility(&mesh, [0.5, 0.3, -1.0], 0);
        assert!(result.point_data().get_array("Visibility").is_some());
    }
    #[test]
    fn depth() {
        let mesh = PolyData::from_triangles(
            vec![[-2.0, -2.0, 5.0], [2.0, -2.0, 5.0], [0.0, 2.0, 5.0]],
            vec![[0, 1, 2]],
        );
        let depths = depth_map(
            &mesh,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            4,
            4,
            60.0,
        );
        assert_eq!(depths.len(), 16);
        // At least some pixels should hit (the triangle covers a large area)
        let hit_count = depths.iter().filter(|&&d| d < 100.0).count();
        assert!(hit_count > 0, "no ray hits");
    }

    #[test]
    fn polygon_fan_tests_all_triangles_once_per_cell() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);

        let second_fan_triangle_hit = ray_cast_all(&mesh, [0.25, 0.75, -1.0], [0.0, 0.0, 1.0]);
        assert_eq!(second_fan_triangle_hit.len(), 1);
        assert_eq!(second_fan_triangle_hit[0].cell_index, 0);

        let diagonal_hit = ray_cast_all(&mesh, [0.5, 0.5, -1.0], [0.0, 0.0, 1.0]);
        assert_eq!(diagonal_hit.len(), 1);
    }

    #[test]
    fn invalid_cell_ids_are_ignored() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2]);
        mesh.polys.push_cell(&[0, -1, 2]);
        mesh.polys.push_cell(&[0, 99, 2]);

        let hits = ray_cast_all(&mesh, [0.25, 0.25, -1.0], [0.0, 0.0, 1.0]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].cell_index, 0);
    }
}
