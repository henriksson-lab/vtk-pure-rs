use std::f64::consts::PI;

use crate::data::{Points, PolyData};

/// Parameters for generating a regular polygon.
pub struct RegularPolygonParams {
    /// Number of sides. Default: 6 (hexagon)
    pub number_of_sides: usize,
    /// Radius of the circumscribed circle. Default: 0.5
    pub radius: f64,
    /// Center of the polygon. Default: [0, 0, 0]
    pub center: [f64; 3],
    /// Normal to the polygon. Default: [0, 0, 1]
    pub normal: [f64; 3],
    /// If true, generate a filled polygon. If false, generate just the outline.
    /// Default: true
    pub generate_polygon: bool,
    /// If true, generate the closed outline polyline. Default: true
    pub generate_polyline: bool,
}

impl Default for RegularPolygonParams {
    fn default() -> Self {
        Self {
            number_of_sides: 6,
            radius: 0.5,
            center: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            generate_polygon: true,
            generate_polyline: true,
        }
    }
}

/// Generate a regular polygon.
pub fn regular_polygon(params: &RegularPolygonParams) -> PolyData {
    let n = params.number_of_sides.max(3);

    let mut points = Points::new();

    let mut normal = params.normal;
    if normalize(&mut normal) == 0.0 {
        normal = [0.0, 0.0, 1.0];
    }

    let mut axis = [1.0, 0.0, 0.0];
    let mut px = cross(normal, axis);
    let mut found_plane_vector = normalize(&mut px) > 1.0e-3;

    if !found_plane_vector {
        axis = [0.0, 1.0, 0.0];
        px = cross(normal, axis);
        found_plane_vector = normalize(&mut px) > 1.0e-3;
    }

    if !found_plane_vector {
        axis = [0.0, 0.0, 1.0];
        px = cross(normal, axis);
        normalize(&mut px);
    }

    let py = cross(px, normal);
    let theta = 2.0 * PI / n as f64;

    for i in 0..n {
        let angle = i as f64 * theta;
        let r = [
            px[0] * angle.cos() + py[0] * angle.sin(),
            px[1] * angle.cos() + py[1] * angle.sin(),
            px[2] * angle.cos() + py[2] * angle.sin(),
        ];
        points.push([
            params.center[0] + params.radius * r[0],
            params.center[1] + params.radius * r[1],
            params.center[2] + params.radius * r[2],
        ]);
    }

    let mut pd = PolyData::new();
    pd.points = points;

    if params.generate_polyline {
        let mut ids: Vec<i64> = (0..n as i64).collect();
        ids.push(0);
        pd.lines.push_cell(&ids);
    }

    if params.generate_polygon {
        let ids: Vec<i64> = (0..n as i64).collect();
        pd.polys.push_cell(&ids);
    }

    pd
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: &mut [f64; 3]) -> f64 {
    let norm = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if norm != 0.0 {
        v[0] /= norm;
        v[1] /= norm;
        v[2] /= norm;
    }
    norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hexagon() {
        let pd = regular_polygon(&RegularPolygonParams::default());
        assert_eq!(pd.points.len(), 6);
        assert_eq!(pd.polys.num_cells(), 1);
        assert_eq!(pd.lines.num_cells(), 1);
        assert_eq!(pd.polys.cell(0).len(), 6);
    }

    #[test]
    fn triangle() {
        let pd = regular_polygon(&RegularPolygonParams {
            number_of_sides: 3,
            radius: 1.0,
            ..Default::default()
        });
        assert_eq!(pd.points.len(), 3);
        // Matches vtkRegularPolygonSource orientation for normal (0, 0, 1).
        let p = pd.points.get(0);
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn outline_mode() {
        let pd = regular_polygon(&RegularPolygonParams {
            number_of_sides: 4,
            generate_polygon: false,
            generate_polyline: true,
            ..Default::default()
        });
        assert_eq!(pd.polys.num_cells(), 0);
        assert_eq!(pd.lines.num_cells(), 1);
        // Closed polyline: 4 + 1 = 5 point references
        assert_eq!(pd.lines.cell(0).len(), 5);
    }

    #[test]
    fn polygon_only_mode() {
        let pd = regular_polygon(&RegularPolygonParams {
            number_of_sides: 4,
            generate_polygon: true,
            generate_polyline: false,
            ..Default::default()
        });
        assert_eq!(pd.polys.num_cells(), 1);
        assert_eq!(pd.lines.num_cells(), 0);
    }
}
