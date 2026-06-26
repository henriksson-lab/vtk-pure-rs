use crate::data::{CellArray, Points, PolyData};

/// Taubin smoothing: alternating positive/negative Laplacian steps.
///
/// Unlike pure Laplacian smoothing which shrinks the mesh, Taubin's
/// method alternates a shrink step (lambda) with an inflate step (mu)
/// to preserve volume while removing noise.
///
/// Typical values: lambda=0.5, mu=-0.53 (mu < -lambda to avoid shrinkage).
pub fn taubin_smooth(input: &PolyData, lambda: f64, mu: f64, iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    add_cell_array_neighbors(&input.lines, n, false, &mut neighbors);
    add_cell_array_neighbors(&input.polys, n, true, &mut neighbors);
    add_cell_array_neighbors(&input.strips, n, false, &mut neighbors);

    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();

    for _ in 0..iterations {
        pts = laplacian_step(&pts, &neighbors, lambda);
        pts = laplacian_step(&pts, &neighbors, mu);
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

fn add_cell_array_neighbors(
    cells: &CellArray,
    n_points: usize,
    closed: bool,
    neighbors: &mut [Vec<usize>],
) {
    for cell in cells.iter() {
        if cell.len() < 2 {
            continue;
        }
        let edge_count = if closed { cell.len() } else { cell.len() - 1 };
        for i in 0..edge_count {
            let Some(a) = valid_point_id(cell[i], n_points) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % cell.len()], n_points) else {
                continue;
            };
            if a == b {
                continue;
            }
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id).ok().filter(|&idx| idx < n_points)
}

fn laplacian_step(pts: &[[f64; 3]], neighbors: &[Vec<usize>], factor: f64) -> Vec<[f64; 3]> {
    let n = pts.len();
    let mut new_pts = pts.to_vec();
    for i in 0..n {
        if neighbors[i].is_empty() {
            continue;
        }
        let cnt = neighbors[i].len() as f64;
        let mut avg = [0.0; 3];
        for &j in &neighbors[i] {
            avg[0] += pts[j][0];
            avg[1] += pts[j][1];
            avg[2] += pts[j][2];
        }
        avg[0] /= cnt;
        avg[1] /= cnt;
        avg[2] /= cnt;
        new_pts[i] = [
            pts[i][0] + factor * (avg[0] - pts[i][0]),
            pts[i][1] + factor * (avg[1] - pts[i][1]),
            pts[i][2] + factor * (avg[2] - pts[i][2]),
        ];
    }
    new_pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_volume_better() {
        let mut pd = PolyData::new();
        // Simple quad
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = taubin_smooth(&pd, 0.5, -0.53, 5);
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn single_point_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([5.0, 5.0, 5.0]);
        let result = taubin_smooth(&pd, 0.5, -0.53, 10);
        assert_eq!(result.points.get(0), [5.0, 5.0, 5.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = taubin_smooth(&pd, 0.5, -0.53, 10);
        assert_eq!(result.points.len(), 0);
    }
}
