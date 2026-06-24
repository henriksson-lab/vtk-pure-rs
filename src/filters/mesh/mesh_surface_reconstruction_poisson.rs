//! Simplified Poisson surface reconstruction from point cloud.
use crate::data::{CellArray, Points, PolyData};
pub fn poisson_reconstruct_simple(points: &PolyData, grid_res: usize) -> PolyData {
    let n = points.points.len();
    if n < 4 {
        return points.clone();
    }
    let gr = grid_res.max(4);
    // Compute bounds
    let mut mn = [f64::INFINITY; 3];
    let mut mx = [f64::NEG_INFINITY; 3];
    for i in 0..n {
        let p = points.points.get(i);
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    let pad = 0.1;
    for j in 0..3 {
        mn[j] -= pad * (mx[j] - mn[j]);
        mx[j] += pad * (mx[j] - mn[j]);
    }
    let dx = [
        (mx[0] - mn[0]) / gr as f64,
        (mx[1] - mn[1]) / gr as f64,
        (mx[2] - mn[2]) / gr as f64,
    ];
    // Compute indicator function (distance to nearest point)
    let total = gr * gr * gr;
    let mut field = vec![1.0f64; total];
    for iz in 0..gr {
        for iy in 0..gr {
            for ix in 0..gr {
                let x = mn[0] + (ix as f64 + 0.5) * dx[0];
                let y = mn[1] + (iy as f64 + 0.5) * dx[1];
                let z = mn[2] + (iz as f64 + 0.5) * dx[2];
                let mut min_d = f64::INFINITY;
                for i in 0..n {
                    let p = points.points.get(i);
                    let d = (x - p[0]).powi(2) + (y - p[1]).powi(2) + (z - p[2]).powi(2);
                    min_d = min_d.min(d);
                }
                field[ix + iy * gr + iz * gr * gr] = min_d.sqrt();
            }
        }
    }
    // Extract iso-surface at average distance using marching-cubes-like
    let avg_d = field.iter().sum::<f64>() / total as f64;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for iz in 0..gr - 1 {
        for iy in 0..gr - 1 {
            for ix in 0..gr - 1 {
                let v000 = field[ix + iy * gr + iz * gr * gr];
                let v100 = field[ix + 1 + iy * gr + iz * gr * gr];
                let v010 = field[ix + (iy + 1) * gr + iz * gr * gr];
                let v001 = field[ix + iy * gr + (iz + 1) * gr * gr];
                // Simple: if cell straddles iso, add a triangle
                let signs = [v000 < avg_d, v100 < avg_d, v010 < avg_d, v001 < avg_d];
                let inside = signs.iter().filter(|&&s| s).count();
                if inside > 0 && inside < 4 {
                    let cx = mn[0] + (ix as f64 + 0.5) * dx[0];
                    let cy = mn[1] + (iy as f64 + 0.5) * dx[1];
                    let cz = mn[2] + (iz as f64 + 0.5) * dx[2];
                    let b = pts.len();
                    pts.push([cx, cy, cz]);
                    pts.push([cx + dx[0], cy, cz]);
                    pts.push([cx, cy + dx[1], cz]);
                    polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64]);
                }
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut pc = PolyData::new();
        for i in 0..8 {
            let a = i as f64 * std::f64::consts::FRAC_PI_4;
            pc.points.push([a.cos(), a.sin(), 0.0]);
        }
        let r = poisson_reconstruct_simple(&pc, 8);
        assert!(r.polys.num_cells() > 0);
    }
}
