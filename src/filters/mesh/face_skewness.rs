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

        let pts: Vec<[f64; 3]> = cell
            .iter()
            .map(|&id| input.points.get(id as usize))
            .collect();
        let angles = interior_angles(&pts);

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
}
