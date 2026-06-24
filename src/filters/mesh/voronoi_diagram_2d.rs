use crate::data::{CellArray, Points, PolyData};

/// 2D Voronoi diagram approximation from seed points.
///
/// Rasterizes the domain defined by `bounds` at the given `resolution`,
/// assigns each grid cell to its nearest seed using brute force, then
/// extracts boundary edges between differently-labeled cells as line
/// segments.
pub fn voronoi_diagram(seeds: &[[f64; 2]], bounds: [[f64; 2]; 2], resolution: usize) -> PolyData {
    if seeds.is_empty() || resolution < 2 {
        return PolyData::new();
    }

    let res: usize = resolution;
    let dx: f64 = (bounds[0][1] - bounds[0][0]) / res as f64;
    let dy: f64 = (bounds[1][1] - bounds[1][0]) / res as f64;

    // Assign each grid cell to nearest seed
    let mut labels: Vec<usize> = vec![0; res * res];
    for jj in 0..res {
        let y: f64 = bounds[1][0] + (jj as f64 + 0.5) * dy;
        for ii in 0..res {
            let x: f64 = bounds[0][0] + (ii as f64 + 0.5) * dx;
            let mut best_dist: f64 = f64::MAX;
            let mut best_seed: usize = 0;
            for (si, seed) in seeds.iter().enumerate() {
                let d: f64 = (x - seed[0]) * (x - seed[0]) + (y - seed[1]) * (y - seed[1]);
                if d < best_dist {
                    best_dist = d;
                    best_seed = si;
                }
            }
            labels[jj * res + ii] = best_seed;
        }
    }

    // Extract boundary edges
    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    // Horizontal boundaries (between rows j and j+1)
    for j in 0..res - 1 {
        for i in 0..res {
            if labels[j * res + i] != labels[(j + 1) * res + i] {
                let y: f64 = bounds[1][0] + (j as f64 + 1.0) * dy;
                let x0: f64 = bounds[0][0] + i as f64 * dx;
                let x1: f64 = x0 + dx;
                let id: i64 = out_points.len() as i64;
                out_points.push([x0, y, 0.0]);
                out_points.push([x1, y, 0.0]);
                out_lines.push_cell(&[id, id + 1]);
            }
        }
    }

    // Vertical boundaries (between columns i and i+1)
    for j in 0..res {
        for i in 0..res - 1 {
            if labels[j * res + i] != labels[j * res + i + 1] {
                let x: f64 = bounds[0][0] + (i as f64 + 1.0) * dx;
                let y0: f64 = bounds[1][0] + j as f64 * dy;
                let y1: f64 = y0 + dy;
                let id: i64 = out_points.len() as i64;
                out_points.push([x, y0, 0.0]);
                out_points.push([x, y1, 0.0]);
                out_lines.push_cell(&[id, id + 1]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_seed_no_boundaries() {
        let seeds: Vec<[f64; 2]> = vec![[0.5, 0.5]];
        let result = voronoi_diagram(&seeds, [[0.0, 1.0], [0.0, 1.0]], 10);
        // Single seed means all cells belong to same region: no boundaries
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn two_seeds_produces_boundary() {
        let seeds: Vec<[f64; 2]> = vec![[0.25, 0.5], [0.75, 0.5]];
        let result = voronoi_diagram(&seeds, [[0.0, 1.0], [0.0, 1.0]], 20);
        // Should produce boundary lines near x=0.5
        assert!(result.lines.num_cells() > 0);
        assert!(result.points.len() > 0);
    }

    #[test]
    fn empty_seeds_returns_empty() {
        let seeds: Vec<[f64; 2]> = vec![];
        let result = voronoi_diagram(&seeds, [[0.0, 1.0], [0.0, 1.0]], 10);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
