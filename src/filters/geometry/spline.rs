use crate::data::{Points, PolyData};

/// Fit a Catmull-Rom spline through the points of each polyline and
/// resample at a higher resolution.
///
/// Each line cell is replaced by a smooth polyline with `resolution`
/// subdivisions. Like vtkSplineFilter's specified-subdivision mode, the
/// spline parameter is based on normalized arc length along the input line.
pub fn spline(input: &PolyData, resolution: usize) -> PolyData {
    let num_divs = resolution.max(1);
    let mut out_points = Points::<f64>::new();
    let mut out_lines = crate::data::CellArray::new();

    for cell in input.lines.iter() {
        if cell.len() < 2 {
            continue;
        }

        let control: Vec<[f64; 3]> = cell
            .iter()
            .map(|&id| input.points.get(id as usize))
            .collect();
        let n = control.len();

        let mut params = vec![0.0; n];
        let mut total_length = 0.0;
        for i in 1..n {
            total_length += distance(control[i - 1], control[i]);
            params[i] = total_length;
        }
        if total_length <= 0.0 {
            continue;
        }
        for t in &mut params {
            *t /= total_length;
        }

        let mut spline_pts: Vec<i64> = Vec::with_capacity(num_divs + 1);
        let mut seg = 0;
        for i in 0..=num_divs {
            let t = i as f64 / num_divs as f64;
            while seg + 2 < n && t > params[seg + 1] {
                seg += 1;
            }

            let t0 = params[seg];
            let t1 = params[seg + 1];
            let local_t = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
            let p0 = if seg > 0 {
                control[seg - 1]
            } else {
                control[seg]
            };
            let p1 = control[seg];
            let p2 = control[seg + 1];
            let p3 = if seg + 2 < n {
                control[seg + 2]
            } else {
                control[seg + 1]
            };

            let idx = out_points.len() as i64;
            out_points.push(catmull_rom(p0, p1, p2, p3, local_t));
            spline_pts.push(idx);
        }

        out_lines.push_cell(&spline_pts);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn catmull_rom(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], t: f64) -> [f64; 3] {
    let t2 = t * t;
    let t3 = t2 * t;

    let mut result = [0.0f64; 3];
    for i in 0..3 {
        result[i] = 0.5
            * ((2.0 * p1[i])
                + (-p0[i] + p2[i]) * t
                + (2.0 * p0[i] - 5.0 * p1[i] + 4.0 * p2[i] - p3[i]) * t2
                + (-p0[i] + 3.0 * p1[i] - 3.0 * p2[i] + p3[i]) * t3);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_straight_line() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = spline(&pd, 4);
        // 4 subdivisions + 1 endpoint = 5 points
        assert_eq!(result.points.len(), 5);
        assert_eq!(result.lines.num_cells(), 1);

        // Endpoints should match
        let p0 = result.points.get(0);
        let pn = result.points.get(4);
        assert!((p0[0]).abs() < 1e-10);
        assert!((pn[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn spline_polyline() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = spline(&pd, 5);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn spline_preserves_topology() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2, 3]);

        let result = spline(&pd, 3);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn spline_uses_arc_length_parameterization() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([10.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let result = spline(&pd, 11);
        let mid = result.points.get(9);
        assert!(
            mid[0] > 9.0,
            "normalized arc-length sampling should spend most samples on the long segment"
        );
    }
}
