use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute per-face skewness for each polygon in a PolyData.
///
/// Skewness measures how far each face deviates from an ideal shape:
/// - For triangles: deviation from equilateral (based on angle deviation)
/// - For quads and higher: deviation from a regular polygon
///
/// The result is a value in [0, 1] where 0 = ideal and 1 = degenerate.
/// The skewness is added as a "Skewness" cell data array.
pub fn compute_face_skewness(input: &PolyData) -> PolyData {
    let mut skewness_values: Vec<f64> = Vec::new();

    for cell in input.polys.iter() {
        let n: usize = cell.len();
        if n < 3 {
            skewness_values.push(1.0);
            continue;
        }

        let mut pts: Vec<[f64; 3]> = Vec::with_capacity(n);
        let mut valid_cell = true;
        for &id in cell {
            let Some(point_id) = valid_point_id(id, input.points.len()) else {
                valid_cell = false;
                break;
            };
            pts.push(input.points.get(point_id));
        }
        if !valid_cell {
            skewness_values.push(1.0);
            continue;
        }
        let angles = interior_angles(&pts);

        if n == 3 {
            skewness_values.push(triangle_equiangle_skew(&angles));
            continue;
        }
        if n == 4 {
            skewness_values.push(quad_equiangle_skew(&angles, &pts));
            continue;
        }

        // Ideal interior angle for a regular n-gon
        let ideal_angle: f64 = std::f64::consts::PI * (n as f64 - 2.0) / n as f64;

        // Maximum possible deviation from ideal angle
        // (angle can range from 0 to PI, so max deviation is max(ideal, PI - ideal))
        let max_deviation: f64 = ideal_angle.max(std::f64::consts::PI - ideal_angle);

        if max_deviation < 1e-15 {
            skewness_values.push(0.0);
            continue;
        }

        // Skewness = max deviation of any angle from ideal / max possible deviation
        let mut max_angle_dev: f64 = 0.0;
        for &angle in &angles {
            let dev: f64 = (angle - ideal_angle).abs();
            if dev > max_angle_dev {
                max_angle_dev = dev;
            }
        }

        let skew: f64 = (max_angle_dev / max_deviation).clamp(0.0, 1.0);
        skewness_values.push(skew);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Skewness",
            skewness_values,
            1,
        )));
    pd
}

fn triangle_equiangle_skew(angles: &[f64]) -> f64 {
    let min_angle = angles.iter().copied().fold(f64::INFINITY, f64::min);
    let max_angle = angles.iter().copied().fold(0.0f64, f64::max);
    let ideal = std::f64::consts::FRAC_PI_3;
    let skew_max = (max_angle - ideal) / (std::f64::consts::PI - ideal);
    let skew_min = (ideal - min_angle) / ideal;
    skew_max.max(skew_min).clamp(0.0, 1.0)
}

fn quad_equiangle_skew(angles: &[f64], pts: &[[f64; 3]]) -> f64 {
    if is_collapsed_quad(pts) {
        return triangle_equiangle_skew(&angles[..3]);
    }

    let min_angle = angles.iter().copied().fold(f64::INFINITY, f64::min);
    let mut max_angle = angles.iter().copied().fold(0.0f64, f64::max);

    let areas = signed_corner_areas(pts);
    if areas.iter().any(|&area| area < 0.0) {
        max_angle = std::f64::consts::TAU - max_angle;
    }

    let ideal = std::f64::consts::FRAC_PI_2;
    let skew_max = (max_angle - ideal) / ideal;
    let skew_min = (ideal - min_angle) / ideal;
    skew_max.max(skew_min).clamp(0.0, 1.0)
}

fn interior_angles(pts: &[[f64; 3]]) -> Vec<f64> {
    let n: usize = pts.len();
    (0..n)
        .map(|i| {
            let prev: usize = if i == 0 { n - 1 } else { i - 1 };
            let next: usize = (i + 1) % n;
            let a = [
                pts[prev][0] - pts[i][0],
                pts[prev][1] - pts[i][1],
                pts[prev][2] - pts[i][2],
            ];
            let b = [
                pts[next][0] - pts[i][0],
                pts[next][1] - pts[i][1],
                pts[next][2] - pts[i][2],
            ];
            let la: f64 = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            let lb: f64 = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
            if la < 1e-20 || lb < 1e-20 {
                return 0.0;
            }
            let cos_angle: f64 =
                ((a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (la * lb)).clamp(-1.0, 1.0);
            cos_angle.acos()
        })
        .collect()
}

fn signed_corner_areas(pts: &[[f64; 3]]) -> [f64; 4] {
    let edges = [
        subtract(pts[1], pts[0]),
        subtract(pts[2], pts[1]),
        subtract(pts[3], pts[2]),
        subtract(pts[0], pts[3]),
    ];

    let principal_axes = [subtract(edges[0], edges[2]), subtract(edges[1], edges[3])];
    let unit_center_normal = normalize(cross(principal_axes[0], principal_axes[1]));

    [
        dot(unit_center_normal, cross(edges[3], edges[0])),
        dot(unit_center_normal, cross(edges[0], edges[1])),
        dot(unit_center_normal, cross(edges[1], edges[2])),
        dot(unit_center_normal, cross(edges[2], edges[3])),
    ]
}

fn is_collapsed_quad(pts: &[[f64; 3]]) -> bool {
    pts[3][0] == pts[2][0] && pts[3][1] == pts[2][1] && pts[3][2] == pts[2][2]
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn subtract(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let length = dot(v, v).sqrt();
    if length > 0.0 {
        scale(v, 1.0 / length)
    } else {
        [0.0, 0.0, 0.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CellArray, Points};

    fn make_equilateral_triangle() -> PolyData {
        let mut points = Points::<f64>::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.5, (3.0f64).sqrt() / 2.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd
    }

    fn make_degenerate_triangle() -> PolyData {
        let mut points = Points::<f64>::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.5, 1e-10, 0.0]); // nearly collinear

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd
    }

    #[test]
    fn test_equilateral_has_zero_skewness() {
        let pd = make_equilateral_triangle();
        let result = compute_face_skewness(&pd);
        let arr = result.cell_data().get_array("Skewness").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] < 0.01,
            "equilateral triangle skewness should be near 0, got {}",
            buf[0]
        );
    }

    #[test]
    fn test_degenerate_has_high_skewness() {
        let pd = make_degenerate_triangle();
        let result = compute_face_skewness(&pd);
        let arr = result.cell_data().get_array("Skewness").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 0.9,
            "degenerate triangle skewness should be near 1, got {}",
            buf[0]
        );
    }

    #[test]
    fn test_right_triangle_moderate_skewness() {
        let mut points = Points::<f64>::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.0, 1.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;

        let result = compute_face_skewness(&pd);
        let arr = result.cell_data().get_array("Skewness").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        // Right triangle: 45-45-90, ideal is 60. Max deviation = 30 degrees
        assert!(
            buf[0] > 0.1 && buf[0] < 0.9,
            "right triangle should have moderate skewness, got {}",
            buf[0]
        );
    }

    #[test]
    fn test_skinny_triangle_uses_vtk_equiangle_normalization() {
        let mut points = Points::<f64>::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);
        points.push([0.01, 0.1, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;

        let result = compute_face_skewness(&pd);
        let arr = result.cell_data().get_array("Skewness").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 0.8,
            "small triangle angles should be normalized by the ideal angle, got {}",
            buf[0]
        );
    }

    #[test]
    fn malformed_cell_ids_produce_degenerate_skewness() {
        let mut points = Points::<f64>::new();
        points.push([0.0, 0.0, 0.0]);
        points.push([1.0, 0.0, 0.0]);

        let mut polys = CellArray::new();
        polys.push_cell(&[0, -1, 99]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;

        let result = compute_face_skewness(&pd);
        let arr = result.cell_data().get_array("Skewness").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
