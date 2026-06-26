use std::f64::consts::PI;

use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Parameters for generating a circular arc.
pub struct ArcParams {
    /// First endpoint of the arc. Default: [0, 0.5, 0]
    pub point1: [f64; 3],
    /// Second endpoint of the arc. Default: [0.5, 0, 0]
    pub point2: [f64; 3],
    /// Center of the circle that defines the arc. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Normal vector to the arc plane. Used when `use_normal_and_angle` is true.
    pub normal: [f64; 3],
    /// Starting point vector from `center`. Used when `use_normal_and_angle` is true.
    pub polar_vector: [f64; 3],
    /// Arc length in degrees. Used when `use_normal_and_angle` is true.
    pub angle: f64,
    /// Number of line segments along the arc. Default: 1
    pub resolution: usize,
    /// Use the longest angular sector between `point1` and `point2`.
    pub negative: bool,
    /// Use the normal/polar-vector/angle API instead of endpoints plus center.
    pub use_normal_and_angle: bool,
}

impl Default for ArcParams {
    fn default() -> Self {
        Self {
            point1: [0.0, 0.5, 0.0],
            point2: [0.5, 0.0, 0.0],
            center: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            polar_vector: [1.0, 0.0, 0.0],
            angle: 90.0,
            resolution: 1,
            negative: false,
            use_normal_and_angle: false,
        }
    }
}

/// Generate a circular arc as a polyline.
pub fn arc(params: &ArcParams) -> PolyData {
    let resolution = params.resolution.max(1);
    let (angle, radius, v1, perpendicular) = if params.use_normal_and_angle {
        let mut v1 = params.polar_vector;
        let radius = normalize_in_place(&mut v1);
        let mut perpendicular = cross(params.normal, params.polar_vector);
        normalize_in_place(&mut perpendicular);
        (params.angle * PI / 180.0, radius, v1, perpendicular)
    } else {
        let mut v1 = [
            params.point1[0] - params.center[0],
            params.point1[1] - params.center[1],
            params.point1[2] - params.center[2],
        ];
        let v2 = [
            params.point2[0] - params.center[0],
            params.point2[1] - params.center[1],
            params.point2[2] - params.center[2],
        ];
        let normal = cross(v1, v2);
        let mut perpendicular = cross(normal, v1);
        let dotprod = dot(v1, v2) / (norm(v1) * norm(v2));
        let mut angle = dotprod.acos();
        if params.negative {
            angle -= 2.0 * PI;
        }
        let radius = normalize_in_place(&mut v1);
        normalize_in_place(&mut perpendicular);
        (angle, radius, v1, perpendicular)
    };
    let angle_inc = angle / resolution as f64;

    let mut points = Points::new();
    let mut tcoords = Vec::with_capacity((resolution + 1) * 2);
    let mut ids = Vec::with_capacity(resolution + 1);

    for i in 0..=resolution {
        let theta = i as f64 * angle_inc;
        let cosine = theta.cos();
        let sine = theta.sin();

        let x = params.center[0] + cosine * radius * v1[0] + sine * radius * perpendicular[0];
        let y = params.center[1] + cosine * radius * v1[1] + sine * radius * perpendicular[1];
        let z = params.center[2] + cosine * radius * v1[2] + sine * radius * perpendicular[2];

        points.push([x, y, z]);
        tcoords.push(i as f32 / resolution as f32);
        tcoords.push(0.0);
        ids.push(i as i64);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines.push_cell(&ids);
    pd.point_data_mut()
        .add_array(AnyDataArray::F32(DataArray::from_vec(
            "Texture Coordinates",
            tcoords,
            2,
        )));
    pd.point_data_mut()
        .set_active_tcoords("Texture Coordinates");
    pd
}

fn normalize_in_place(v: &mut [f64; 3]) -> f64 {
    let len = norm(*v);
    if len != 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
    len
}

fn norm(v: [f64; 3]) -> f64 {
    dot(v, v).sqrt()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_arc() {
        let pd = arc(&ArcParams::default());
        assert_eq!(pd.points.len(), 2);
        assert_eq!(pd.lines.num_cells(), 1);
        assert_eq!(pd.lines.cell(0).len(), 2);

        // All points should be at distance 0.5 from center
        for i in 0..pd.points.len() {
            let p = pd.points.get(i);
            let dist = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((dist - 0.5).abs() < 1e-10, "point {} dist = {}", i, dist);
        }

        let p0 = pd.points.get(0);
        let pn = pd.points.get(1);
        assert!(p0[0].abs() < 1e-10);
        assert!((p0[1] - 0.5).abs() < 1e-10);
        assert!((pn[0] - 0.5).abs() < 1e-10);
        assert!(pn[1].abs() < 1e-10);
    }

    #[test]
    fn normal_and_angle_full_circle() {
        let pd = arc(&ArcParams {
            angle: 360.0,
            resolution: 37,
            polar_vector: [1.0, 0.0, 0.0],
            use_normal_and_angle: true,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 38);
        // First and last points should be nearly the same
        let p0 = pd.points.get(0);
        let pn = pd.points.get(37);
        assert!((p0[0] - pn[0]).abs() < 1e-10);
        assert!((p0[1] - pn[1]).abs() < 1e-10);
    }

    #[test]
    fn arc_outputs_tcoords_like_vtk_source() {
        let pd = arc(&ArcParams {
            resolution: 2,
            ..Default::default()
        });
        let tcoords = pd.point_data().tcoords().unwrap();
        assert_eq!(tcoords.name(), "Texture Coordinates");
        assert_eq!(tcoords.num_tuples(), 3);
        assert_eq!(
            tcoords.to_f64_vec_flat(),
            vec![0.0, 0.0, 0.5, 0.0, 1.0, 0.0]
        );
    }

    #[test]
    fn single_segment_resolution() {
        let pd = arc(&ArcParams {
            resolution: 0,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 2);
        assert_eq!(pd.lines.cell(0), &[0, 1]);
    }
}
