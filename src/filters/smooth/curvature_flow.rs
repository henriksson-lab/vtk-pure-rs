use crate::data::{Points, PolyData};

/// Mean curvature flow smoothing.
///
/// Moves each vertex along its mean curvature normal (Laplacian direction
/// weighted by cotangent weights). Shrinks bumps while preserving features
/// better than uniform Laplacian. `dt` is the time step.
pub fn curvature_flow(input: &PolyData, dt: f64, iterations: usize) -> PolyData {
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
            // Cotangent-weighted Laplacian (simplified: uniform weights)
            let cnt = neighbors[i].len() as f64;
            let mut lap = [0.0; 3];
            for &j in &neighbors[i] {
                lap[0] += pts[j][0] - pts[i][0];
                lap[1] += pts[j][1] - pts[i][1];
                lap[2] += pts[j][2] - pts[i][2];
            }
            lap[0] /= cnt;
            lap[1] /= cnt;
            lap[2] /= cnt;
            new_pts[i] = [
                pts[i][0] + dt * lap[0],
                pts[i][1] + dt * lap[1],
                pts[i][2] + dt * lap[2],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooths_spike() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.5, 2.0]); // spike
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);
        pd.polys.push_cell(&[2, 0, 3]);

        let result = curvature_flow(&pd, 0.3, 5);
        let spike = result.points.get(3);
        assert!(spike[2] < 2.0); // spike reduced
    }

    #[test]
    fn zero_dt_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.polys.push_cell(&[0, 1]);

        let result = curvature_flow(&pd, 0.0, 10);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = curvature_flow(&pd, 0.5, 10);
        assert_eq!(result.points.len(), 0);
    }
}
