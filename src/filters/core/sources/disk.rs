use std::f64::consts::PI;

use crate::data::{CellArray, Points, PolyData};

/// Parameters for generating a disk (annulus).
pub struct DiskParams {
    /// Inner radius. Default: 0.25
    pub inner_radius: f64,
    /// Outer radius. Default: 0.5
    pub outer_radius: f64,
    /// Number of sides around the circumference. Default: 6
    pub circumferential_resolution: usize,
    /// Number of rings between inner and outer radius. Default: 1
    pub radial_resolution: usize,
    /// Center of the disk. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Plane normal. Default: [0, 0, 1]
    pub normal: [f64; 3],
}

impl Default for DiskParams {
    fn default() -> Self {
        Self {
            inner_radius: 0.25,
            outer_radius: 0.5,
            circumferential_resolution: 6,
            radial_resolution: 1,
            center: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }
}

/// Generate a flat disk (or annulus) as PolyData.
pub fn disk(params: &DiskParams) -> PolyData {
    let circumferential_resolution = params.circumferential_resolution.max(3);
    let radial_resolution = params.radial_resolution.max(1);
    let inner_radius = params.inner_radius.max(0.0);
    let outer_radius = params.outer_radius.max(0.0);

    let Some(transform) = DiskTransform::new(params.center, params.normal) else {
        return PolyData::new();
    };

    let mut new_points = Points::new();
    let mut new_polys = CellArray::new();

    let theta = 2.0 * PI / circumferential_resolution as f64;
    let delta_radius = (outer_radius - inner_radius) / radial_resolution as f64;

    for i in 0..circumferential_resolution {
        let cos_theta = (i as f64 * theta).cos();
        let sin_theta = (i as f64 * theta).sin();
        for j in 0..=radial_resolution {
            let radius = inner_radius + j as f64 * delta_radius;
            let x = [
                params.center[0] + radius * cos_theta,
                params.center[1] + radius * sin_theta,
                params.center[2],
            ];
            new_points.push(transform.transform_point(x));
        }
    }

    for i in 0..circumferential_resolution {
        for j in 0..radial_resolution {
            let pts0 = i * (radial_resolution + 1) + j;
            let pts1 = pts0 + 1;
            let pts2 = if i < circumferential_resolution - 1 {
                pts1 + radial_resolution + 1
            } else {
                j + 1
            };
            let pts3 = pts2 - 1;
            new_polys.push_cell(&[pts0 as i64, pts1 as i64, pts2 as i64, pts3 as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = new_points;
    pd.polys = new_polys;
    pd
}

struct DiskTransform {
    center: [f64; 3],
    axis: [f64; 3],
    cos_angle: f64,
    sin_angle: f64,
}

impl DiskTransform {
    fn new(center: [f64; 3], normal: [f64; 3]) -> Option<Self> {
        let n = normalize(normal)?;
        let default_normal = [0.0, 0.0, 1.0];
        let mut axis = cross(default_normal, n);
        let axis_len = length(axis);
        if axis_len > 0.0 {
            axis = [axis[0] / axis_len, axis[1] / axis_len, axis[2] / axis_len];
        } else {
            axis = [1.0, 0.0, 0.0];
        }
        let cos_angle = dot(default_normal, n).clamp(-1.0, 1.0);
        let sin_angle = (1.0 - cos_angle * cos_angle).sqrt();
        Some(Self {
            center,
            axis,
            cos_angle,
            sin_angle,
        })
    }

    fn transform_point(&self, x: [f64; 3]) -> [f64; 3] {
        let p = [
            x[0] - self.center[0],
            x[1] - self.center[1],
            x[2] - self.center[2],
        ];
        let k_cross_p = cross(self.axis, p);
        let k_dot_p = dot(self.axis, p);
        [
            self.center[0]
                + p[0] * self.cos_angle
                + k_cross_p[0] * self.sin_angle
                + self.axis[0] * k_dot_p * (1.0 - self.cos_angle),
            self.center[1]
                + p[1] * self.cos_angle
                + k_cross_p[1] * self.sin_angle
                + self.axis[1] * k_dot_p * (1.0 - self.cos_angle),
            self.center[2]
                + p[2] * self.cos_angle
                + k_cross_p[2] * self.sin_angle
                + self.axis[2] * k_dot_p * (1.0 - self.cos_angle),
        ]
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn length(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
}

fn normalize(v: [f64; 3]) -> Option<[f64; 3]> {
    let len = length(v);
    if len == 0.0 {
        None
    } else {
        Some([v[0] / len, v[1] / len, v[2] / len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_disk() {
        let pd = disk(&DiskParams::default());
        assert_eq!(pd.points.len(), 12);
        assert_eq!(pd.polys.num_cells(), 6);
    }

    #[test]
    fn annulus() {
        let pd = disk(&DiskParams {
            inner_radius: 0.25,
            outer_radius: 0.5,
            circumferential_resolution: 8,
            radial_resolution: 2,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 24);
        assert_eq!(pd.polys.num_cells(), 16);
    }

    #[test]
    fn multi_ring_disk() {
        let pd = disk(&DiskParams {
            inner_radius: 0.0,
            outer_radius: 1.0,
            circumferential_resolution: 6,
            radial_resolution: 3,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 24);
        assert_eq!(pd.polys.num_cells(), 18);
    }

    #[test]
    fn oriented_disk() {
        let pd = disk(&DiskParams {
            inner_radius: 0.0,
            outer_radius: 1.0,
            circumferential_resolution: 4,
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        });
        for p in &pd.points {
            assert!(p[1].abs() < 1e-10);
        }
    }
}
