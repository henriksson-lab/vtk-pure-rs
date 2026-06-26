//! UV mapping quality metrics: distortion, coverage, seam analysis.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// UV quality metrics for a mesh.
#[derive(Debug, Clone)]
pub struct UvQualityMetrics {
    pub num_uv_points: usize,
    pub area_distortion_mean: f64,
    pub area_distortion_max: f64,
    pub angle_distortion_mean: f64,
    pub angle_distortion_max: f64,
    pub uv_coverage: f64, // fraction of [0,1]² used
    pub num_overlapping_triangles: usize,
}

impl std::fmt::Display for UvQualityMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UV Quality: area_distortion(mean={:.4},max={:.4}), \
               angle_distortion(mean={:.2}°,max={:.2}°), coverage={:.1}%, overlaps={}",
            self.area_distortion_mean,
            self.area_distortion_max,
            self.angle_distortion_mean,
            self.angle_distortion_max,
            self.uv_coverage * 100.0,
            self.num_overlapping_triangles
        )
    }
}

/// Compute UV quality metrics for a mesh with texture coordinates.
pub fn uv_quality(mesh: &PolyData) -> Option<UvQualityMetrics> {
    let tcoords = mesh.point_data().tcoords()?;
    if tcoords.num_components() != 2 {
        return None;
    }

    let n = mesh.points.len();
    if tcoords.num_tuples() < n {
        return None;
    }
    let mut tc_buf = [0.0f64; 2];

    // Read all UVs
    let uvs: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            tcoords.tuple_as_f64(i, &mut tc_buf);
            [tc_buf[0], tc_buf[1]]
        })
        .collect();

    let mut area_dist = Vec::new();
    let mut angle_dist = Vec::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let i0 = cell[0] as usize;
        let i1 = cell[1] as usize;
        let i2 = cell[2] as usize;
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }

        // 3D area
        let a3d = tri_area_3d(mesh, i0, i1, i2);
        // UV area
        let a_uv = tri_area_2d(uvs[i0], uvs[i1], uvs[i2]);

        if a3d > 1e-15 {
            area_dist.push(a_uv / a3d);
        }

        // Angle distortion
        let angles_3d = tri_angles_3d(mesh, i0, i1, i2);
        let angles_uv = tri_angles_2d(uvs[i0], uvs[i1], uvs[i2]);
        let max_angle_diff = (0..3)
            .map(|j| (angles_3d[j] - angles_uv[j]).abs().to_degrees())
            .fold(0.0f64, f64::max);
        angle_dist.push(max_angle_diff);
    }

    // UV coverage: fraction of [0,1]² bounding box used
    let mut u_min = f64::MAX;
    let mut u_max = f64::MIN;
    let mut v_min = f64::MAX;
    let mut v_max = f64::MIN;
    for uv in &uvs {
        u_min = u_min.min(uv[0]);
        u_max = u_max.max(uv[0]);
        v_min = v_min.min(uv[1]);
        v_max = v_max.max(uv[1]);
    }
    let coverage = if uvs.is_empty() {
        0.0
    } else {
        (u_max - u_min) * (v_max - v_min)
    };

    let mean_area = if !area_dist.is_empty() {
        area_dist.iter().sum::<f64>() / area_dist.len() as f64
    } else {
        0.0
    };
    let max_area = area_dist.iter().cloned().fold(0.0f64, f64::max);
    let mean_angle = if !angle_dist.is_empty() {
        angle_dist.iter().sum::<f64>() / angle_dist.len() as f64
    } else {
        0.0
    };
    let max_angle = angle_dist.iter().cloned().fold(0.0f64, f64::max);

    Some(UvQualityMetrics {
        num_uv_points: n,
        area_distortion_mean: mean_area,
        area_distortion_max: max_area,
        angle_distortion_mean: mean_angle,
        angle_distortion_max: max_angle,
        uv_coverage: coverage.min(1.0),
        num_overlapping_triangles: count_overlapping_triangles(mesh, &uvs),
    })
}

/// Add per-triangle UV distortion as cell data.
pub fn add_uv_distortion_data(mesh: &PolyData) -> PolyData {
    let tcoords = match mesh.point_data().tcoords() {
        Some(tc) if tc.num_components() == 2 => tc,
        _ => return mesh.clone(),
    };

    let n = mesh.points.len();
    if tcoords.num_tuples() < n {
        return mesh.clone();
    }
    let mut tc_buf = [0.0f64; 2];
    let uvs: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            tcoords.tuple_as_f64(i, &mut tc_buf);
            [tc_buf[0], tc_buf[1]]
        })
        .collect();

    let mut area_data = Vec::new();
    let mut angle_data = Vec::new();

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            area_data.push(1.0);
            angle_data.push(0.0);
            continue;
        }
        let (i0, i1, i2) = (cell[0] as usize, cell[1] as usize, cell[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            area_data.push(1.0);
            angle_data.push(0.0);
            continue;
        }
        let a3d = tri_area_3d(mesh, i0, i1, i2);
        let auv = tri_area_2d(uvs[i0], uvs[i1], uvs[i2]);
        area_data.push(if a3d > 1e-15 { auv / a3d } else { 1.0 });

        let a3 = tri_angles_3d(mesh, i0, i1, i2);
        let au = tri_angles_2d(uvs[i0], uvs[i1], uvs[i2]);
        angle_data.push(
            (0..3)
                .map(|j| (a3[j] - au[j]).abs().to_degrees())
                .fold(0.0f64, f64::max),
        );
    }

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "UVAreaDistortion",
            area_data,
            1,
        )));
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "UVAngleDistortion",
            angle_data,
            1,
        )));
    result
}

fn tri_area_3d(mesh: &PolyData, i0: usize, i1: usize, i2: usize) -> f64 {
    let a = mesh.points.get(i0);
    let b = mesh.points.get(i1);
    let c = mesh.points.get(i2);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    0.5 * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
    .sqrt()
}
fn tri_area_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs()
}
fn tri_angles_3d(mesh: &PolyData, i0: usize, i1: usize, i2: usize) -> [f64; 3] {
    let a = mesh.points.get(i0);
    let b = mesh.points.get(i1);
    let c = mesh.points.get(i2);
    let angle = |u: [f64; 3], v: [f64; 3]| {
        let d = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        let l = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt()
            * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            (d / l).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        }
    };
    [
        angle(
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
        ),
        angle(
            [a[0] - b[0], a[1] - b[1], a[2] - b[2]],
            [c[0] - b[0], c[1] - b[1], c[2] - b[2]],
        ),
        angle(
            [a[0] - c[0], a[1] - c[1], a[2] - c[2]],
            [b[0] - c[0], b[1] - c[1], b[2] - c[2]],
        ),
    ]
}
fn tri_angles_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> [f64; 3] {
    let angle = |u: [f64; 2], v: [f64; 2]| {
        let d = u[0] * v[0] + u[1] * v[1];
        let l = (u[0] * u[0] + u[1] * u[1]).sqrt() * (v[0] * v[0] + v[1] * v[1]).sqrt();
        if l > 1e-15 {
            (d / l).clamp(-1.0, 1.0).acos()
        } else {
            0.0
        }
    };
    [
        angle([b[0] - a[0], b[1] - a[1]], [c[0] - a[0], c[1] - a[1]]),
        angle([a[0] - b[0], a[1] - b[1]], [c[0] - b[0], c[1] - b[1]]),
        angle([a[0] - c[0], a[1] - c[1]], [b[0] - c[0], b[1] - c[1]]),
    ]
}

fn count_overlapping_triangles(mesh: &PolyData, uvs: &[[f64; 2]]) -> usize {
    let mut triangles = Vec::new();
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let tri = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        if tri.iter().all(|&i| i < uvs.len()) {
            let uv_tri = [uvs[tri[0]], uvs[tri[1]], uvs[tri[2]]];
            if tri_area_2d(uv_tri[0], uv_tri[1], uv_tri[2]) > 1e-15 {
                triangles.push((tri, uv_tri));
            }
        }
    }

    let mut overlaps = 0;
    for i in 0..triangles.len() {
        for j in i + 1..triangles.len() {
            if triangles[i].0.iter().any(|a| triangles[j].0.contains(a)) {
                continue;
            }
            if triangles_overlap_2d(triangles[i].1, triangles[j].1) {
                overlaps += 1;
            }
        }
    }
    overlaps
}

fn triangles_overlap_2d(a: [[f64; 2]; 3], b: [[f64; 2]; 3]) -> bool {
    if !bbox_overlap(a, b) {
        return false;
    }

    for i in 0..3 {
        let a0 = a[i];
        let a1 = a[(i + 1) % 3];
        for j in 0..3 {
            let b0 = b[j];
            let b1 = b[(j + 1) % 3];
            if segments_cross_2d(a0, a1, b0, b1) {
                return true;
            }
        }
    }

    point_strictly_in_triangle(a[0], b) || point_strictly_in_triangle(b[0], a)
}

fn bbox_overlap(a: [[f64; 2]; 3], b: [[f64; 2]; 3]) -> bool {
    let (a_min_u, a_max_u, a_min_v, a_max_v) = triangle_bounds(a);
    let (b_min_u, b_max_u, b_min_v, b_max_v) = triangle_bounds(b);
    a_min_u < b_max_u && b_min_u < a_max_u && a_min_v < b_max_v && b_min_v < a_max_v
}

fn triangle_bounds(tri: [[f64; 2]; 3]) -> (f64, f64, f64, f64) {
    let mut min_u = tri[0][0];
    let mut max_u = tri[0][0];
    let mut min_v = tri[0][1];
    let mut max_v = tri[0][1];
    for p in &tri[1..] {
        min_u = min_u.min(p[0]);
        max_u = max_u.max(p[0]);
        min_v = min_v.min(p[1]);
        max_v = max_v.max(p[1]);
    }
    (min_u, max_u, min_v, max_v)
}

fn segments_cross_2d(a0: [f64; 2], a1: [f64; 2], b0: [f64; 2], b1: [f64; 2]) -> bool {
    let o1 = orient_2d(a0, a1, b0);
    let o2 = orient_2d(a0, a1, b1);
    let o3 = orient_2d(b0, b1, a0);
    let o4 = orient_2d(b0, b1, a1);
    o1 * o2 < -1e-15 && o3 * o4 < -1e-15
}

fn point_strictly_in_triangle(p: [f64; 2], tri: [[f64; 2]; 3]) -> bool {
    let o0 = orient_2d(tri[0], tri[1], p);
    let o1 = orient_2d(tri[1], tri[2], p);
    let o2 = orient_2d(tri[2], tri[0], p);
    (o0 > 1e-15 && o1 > 1e-15 && o2 > 1e-15) || (o0 < -1e-15 && o1 < -1e-15 && o2 < -1e-15)
}

fn orient_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mesh_with_uv() -> PolyData {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                2,
            )));
        m.point_data_mut().set_active_tcoords("TCoords");
        m
    }

    #[test]
    fn perfect_mapping() {
        let metrics = uv_quality(&make_mesh_with_uv()).unwrap();
        assert!(metrics.area_distortion_mean > 0.0);
        assert!(metrics.angle_distortion_mean < 1.0); // near-zero for identity mapping
    }

    #[test]
    fn distortion_data() {
        let result = add_uv_distortion_data(&make_mesh_with_uv());
        assert!(result.cell_data().get_array("UVAreaDistortion").is_some());
        assert!(result.cell_data().get_array("UVAngleDistortion").is_some());
    }

    #[test]
    fn no_uv() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert!(uv_quality(&mesh).is_none());
    }

    #[test]
    fn display() {
        let metrics = uv_quality(&make_mesh_with_uv()).unwrap();
        let s = format!("{metrics}");
        assert!(s.contains("UV Quality"));
    }
}
