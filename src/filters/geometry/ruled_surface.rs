use crate::data::{CellArray, Points, PolyData};

/// Create a ruled surface between polylines.
///
/// Connects corresponding arc-length samples on adjacent polyline cells with
/// quads to create surfaces "ruled" between them. If paired polylines have
/// different numbers of points, the shorter one is resampled to match.
pub fn ruled_surface(input: &PolyData) -> PolyData {
    let lines: Vec<Vec<i64>> = input.lines.iter().map(|c| c.to_vec()).collect();
    if lines.len() < 2 {
        return PolyData::new();
    }

    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for pair in lines.windows(2) {
        let line_a = &pair[0];
        let line_b = &pair[1];

        if line_a.len() < 2 || line_b.len() < 2 {
            continue;
        }

        // Resample both lines to the same number of points (the max of the two).
        let n = line_a.len().max(line_b.len());
        let pts_a = resample_line(input, line_a, n);
        let pts_b = resample_line(input, line_b, n);
        let offset = out_points.len() as i64;

        for p in &pts_a {
            out_points.push(*p);
        }
        for p in &pts_b {
            out_points.push(*p);
        }

        // Connect with quads.
        for i in 0..n - 1 {
            let a0 = offset + i as i64;
            let a1 = offset + (i + 1) as i64;
            let b0 = offset + (n + i) as i64;
            let b1 = offset + (n + i + 1) as i64;
            out_polys.push_cell(&[a0, a1, b1, b0]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Resample a polyline to `target_n` evenly-spaced points.
fn resample_line(input: &PolyData, cell: &[i64], target_n: usize) -> Vec<[f64; 3]> {
    let pts: Vec<[f64; 3]> = cell
        .iter()
        .map(|&id| input.points.get(id as usize))
        .collect();

    if pts.len() == target_n {
        return pts;
    }

    // Compute cumulative arc length
    let mut arc = vec![0.0f64; pts.len()];
    for i in 1..pts.len() {
        let d = dist(&pts[i - 1], &pts[i]);
        arc[i] = arc[i - 1] + d;
    }
    let total = arc[pts.len() - 1];
    if total < 1e-15 {
        return vec![pts[0]; target_n];
    }

    // Resample at uniform parameter values
    let mut result = Vec::with_capacity(target_n);
    for i in 0..target_n {
        let t = if target_n > 1 {
            total * (i as f64) / ((target_n - 1) as f64)
        } else {
            0.0
        };

        // Find segment
        let mut seg = 0;
        for j in 1..arc.len() {
            if arc[j] >= t {
                seg = j - 1;
                break;
            }
            seg = j - 1;
        }

        let seg_len = arc[seg + 1] - arc[seg];
        let local_t = if seg_len > 1e-15 {
            (t - arc[seg]) / seg_len
        } else {
            0.0
        };

        result.push(lerp3(pts[seg], pts[seg + 1], local_t));
    }
    result
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ruled_between_two_lines() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);
        pd.lines.push_cell(&[3, 4, 5]);

        let result = ruled_surface(&pd);
        assert_eq!(result.points.len(), 6); // 3 + 3
        assert_eq!(result.polys.num_cells(), 2); // 2 quads
    }

    #[test]
    fn ruled_different_lengths() {
        let mut pd = PolyData::new();
        // Line A: 2 points
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        // Line B: 4 points
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.33, 1.0, 0.0]);
        pd.points.push([0.66, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[2, 3, 4, 5]);

        let result = ruled_surface(&pd);
        // Resampled to max(2,4) = 4 points each
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.polys.num_cells(), 3); // 3 quads
    }

    #[test]
    fn single_line_returns_empty() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = ruled_surface(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn ruled_between_adjacent_line_pairs() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[2, 3]);
        pd.lines.push_cell(&[4, 5]);

        let result = ruled_surface(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = ruled_surface(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
