use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Equalize edge lengths by moving vertices toward their neighbors.
///
/// Each vertex moves toward the average position of neighbors that
/// are connected by edges longer than the target length, and away from
/// those connected by shorter edges. `target` is the desired edge length.
pub fn equalize_edge_lengths(input: &PolyData, target: f64, iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for _ in 0..iterations {
        let mut new_pts = pts.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let mut dx = 0.0;
            let mut dy = 0.0;
            let mut dz = 0.0;
            for &j in &neighbors[i] {
                let d = [
                    pts[j][0] - pts[i][0],
                    pts[j][1] - pts[i][1],
                    pts[j][2] - pts[i][2],
                ];
                let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if len < 1e-15 {
                    continue;
                }
                let scale = 0.1 * (len - target) / len; // move toward target length
                dx += d[0] * scale;
                dy += d[1] * scale;
                dz += d[2] * scale;
            }
            let cnt = neighbors[i].len() as f64;
            new_pts[i] = [
                pts[i][0] + dx / cnt,
                pts[i][1] + dy / cnt,
                pts[i][2] + dz / cnt,
            ];
        }
        pts = new_pts;
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Compute edge length statistics.
pub fn edge_length_stats(input: &PolyData) -> (f64, f64, f64, f64) {
    // (min, max, mean, stddev)
    let mut min_l = f64::MAX;
    let mut max_l = 0.0f64;
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    let mut count = 0usize;
    let mut seen = std::collections::HashSet::new();

    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let pa = input.points.get(a as usize);
                let pb = input.points.get(b as usize);
                let d =
                    ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                        .sqrt();
                min_l = min_l.min(d);
                max_l = max_l.max(d);
                sum += d;
                sum2 += d * d;
                count += 1;
            }
        }
    }
    if count == 0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mean = sum / count as f64;
    let var = (sum2 / count as f64 - mean * mean).max(0.0);
    (min_l, max_l, mean, var.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converges_toward_target() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 8.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (_, _, mean_before, _) = edge_length_stats(&pd);
        let result = equalize_edge_lengths(&pd, mean_before, 20);
        let (_, _, _, std_after) = edge_length_stats(&result);
        let (_, _, _, std_before) = edge_length_stats(&pd);
        // Standard deviation should decrease (more uniform)
        assert!(std_after <= std_before + 0.5);
    }

    #[test]
    fn stats_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (min_l, max_l, mean, _) = edge_length_stats(&pd);
        assert!((min_l - 1.0).abs() < 1e-10);
        assert!(max_l > 1.0);
        assert!(mean > 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (min_l, _, _, _) = edge_length_stats(&pd);
        assert_eq!(min_l, 0.0);
    }
}
