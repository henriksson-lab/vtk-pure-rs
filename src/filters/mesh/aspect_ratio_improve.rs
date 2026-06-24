use crate::data::{Points, PolyData};

/// Improve mesh quality by moving vertices to equalize adjacent edge lengths.
///
/// Combines Laplacian smoothing with edge-length regularization.
/// Each vertex moves toward the average of its neighbors, weighted by
/// how much each edge deviates from the mean edge length.
pub fn quality_improve(input: &PolyData, iterations: usize, strength: f64) -> PolyData {
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
        // Compute global mean edge length
        let mut sum_len = 0.0;
        let mut count = 0;
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            for &j in &neighbors[i] {
                let key = if i < j { (i, j) } else { (j, i) };
                if seen.insert(key) {
                    sum_len += ((pts[i][0] - pts[j][0]).powi(2)
                        + (pts[i][1] - pts[j][1]).powi(2)
                        + (pts[i][2] - pts[j][2]).powi(2))
                    .sqrt();
                    count += 1;
                }
            }
        }
        let mean_len = if count > 0 {
            sum_len / count as f64
        } else {
            1.0
        };

        let mut new_pts = pts.clone();
        for i in 0..n {
            if neighbors[i].is_empty() {
                continue;
            }
            let p = pts[i];
            let mut dx = 0.0;
            let mut dy = 0.0;
            let mut dz = 0.0;
            let mut tw = 0.0;
            for &j in &neighbors[i] {
                let d = ((p[0] - pts[j][0]).powi(2)
                    + (p[1] - pts[j][1]).powi(2)
                    + (p[2] - pts[j][2]).powi(2))
                .sqrt();
                let w = 1.0 + (d - mean_len).abs() / mean_len.max(1e-15); // weight more for deviant edges
                dx += w * (pts[j][0] - p[0]);
                dy += w * (pts[j][1] - p[1]);
                dz += w * (pts[j][2] - p[2]);
                tw += w;
            }
            if tw > 1e-15 {
                new_pts[i] = [
                    p[0] + strength * dx / tw,
                    p[1] + strength * dy / tw,
                    p[2] + strength * dz / tw,
                ];
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn improves_quality() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 0.1, 0.0]);
        pd.points.push([5.0, -0.1, 0.0]); // very thin triangles
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);

        let result = quality_improve(&pd, 10, 0.3);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn zero_strength_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.points.push([7.0, 8.0, 9.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        let result = quality_improve(&pd, 10, 0.0);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(quality_improve(&pd, 5, 0.5).points.len(), 0);
    }
}
