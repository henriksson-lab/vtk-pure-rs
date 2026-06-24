use crate::data::{CellArray, KdTree, Points, PolyData};

/// Poisson-disk subsampling of a point set.
///
/// Greedily selects points such that no two selected points are closer
/// than `min_distance`. Produces a well-spaced subset.
pub fn poisson_disk_sample(input: &PolyData, min_distance: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let d2 = min_distance * min_distance;

    let mut selected: Vec<[f64; 3]> = Vec::new();
    let mut rejected = vec![false; n];

    // Process in random-ish order (use point index shuffled by hash)
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by_key(|&i| {
        let p = pts[i];
        ((p[0] * 73856093.0) as i64 ^ (p[1] * 19349663.0) as i64 ^ (p[2] * 83492791.0) as i64)
            .wrapping_abs()
    });

    for &idx in &order {
        if rejected[idx] {
            continue;
        }
        let p = pts[idx];

        // Check against all selected points
        let too_close = selected
            .iter()
            .any(|&s| (p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2) < d2);

        if !too_close {
            selected.push(p);
        }
    }

    let mut out_pts = Points::<f64>::new();
    let mut out_verts = CellArray::new();
    for p in &selected {
        let idx = out_pts.len() as i64;
        out_pts.push(*p);
        out_verts.push_cell(&[idx]);
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.verts = out_verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_maintained() {
        let mut pd = PolyData::new();
        for i in 0..50 {
            pd.points.push([(i as f64) * 0.1, 0.0, 0.0]);
        }

        let result = poisson_disk_sample(&pd, 0.5);
        assert!(result.points.len() < 50);
        assert!(result.points.len() > 3);

        // Verify min distance
        for i in 0..result.points.len() {
            for j in i + 1..result.points.len() {
                let a = result.points.get(i);
                let b = result.points.get(j);
                let d =
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
                assert!(d >= 0.49, "d={} between {} and {}", d, i, j);
            }
        }
    }

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = poisson_disk_sample(&pd, 1.0);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = poisson_disk_sample(&pd, 1.0);
        assert_eq!(result.points.len(), 0);
    }
}
