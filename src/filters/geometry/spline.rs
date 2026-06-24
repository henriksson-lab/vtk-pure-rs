use crate::data::{Points, PolyData};

/// Fit a Catmull-Rom spline through the points of each polyline and
/// resample at a higher resolution.
///
/// Each line cell is replaced by a smooth polyline with `resolution`
/// points per original segment.
pub fn spline(input: &PolyData, resolution: usize) -> PolyData {
    let res = resolution.max(1);
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

        let mut spline_pts: Vec<i64> = Vec::new();

        for seg in 0..n - 1 {
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

            let steps = if seg == n - 2 { res + 1 } else { res };
            for s in 0..steps {
                let t = s as f64 / res as f64;
                let pt = catmull_rom(p0, p1, p2, p3, t);
                let idx = out_points.len() as i64;
                out_points.push(pt);
                spline_pts.push(idx);
            }
        }

        out_lines.push_cell(&spline_pts);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
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
        // 1 segment * 4 + 1 = 5 points
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
        // 2 segments * 5 + 1 = 11 points
        assert_eq!(result.points.len(), 11);
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
        // 3 segments * 3 + 1 = 10
        assert_eq!(result.points.len(), 10);
    }
}
