//! Generate 2D Voronoi diagram as PolyData from seed points.
use crate::data::{CellArray, Points, PolyData};
pub fn voronoi_grid(nx: usize, ny: usize, seeds: &[[f64; 2]], bounds: [f64; 4]) -> PolyData {
    let (x0, y0, x1, y1) = (bounds[0], bounds[1], bounds[2], bounds[3]);
    let dx = (x1 - x0) / nx as f64;
    let dy = (y1 - y0) / ny as f64;
    let mut labels = vec![0usize; nx * ny];
    for iy in 0..ny {
        for ix in 0..nx {
            let px = x0 + (ix as f64 + 0.5) * dx;
            let py = y0 + (iy as f64 + 0.5) * dy;
            let mut best = 0;
            let mut bd = f64::INFINITY;
            for (si, s) in seeds.iter().enumerate() {
                let d = (px - s[0]).powi(2) + (py - s[1]).powi(2);
                if d < bd {
                    bd = d;
                    best = si;
                }
            }
            labels[ix + iy * nx] = best;
        }
    }
    // Extract boundaries as line segments
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    for iy in 0..ny {
        for ix in 0..nx {
            let idx = ix + iy * nx;
            if ix + 1 < nx && labels[idx] != labels[idx + 1] {
                let x = x0 + (ix as f64 + 1.0) * dx;
                let y = y0 + iy as f64 * dy;
                let i = pts.len();
                pts.push([x, y, 0.0]);
                pts.push([x, y + dy, 0.0]);
                lines.push_cell(&[i as i64, (i + 1) as i64]);
            }
            if iy + 1 < ny && labels[idx] != labels[idx + nx] {
                let x = x0 + ix as f64 * dx;
                let y = y0 + (iy as f64 + 1.0) * dy;
                let i = pts.len();
                pts.push([x, y, 0.0]);
                pts.push([x + dx, y, 0.0]);
                lines.push_cell(&[i as i64, (i + 1) as i64]);
            }
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let v = voronoi_grid(
            20,
            20,
            &[[0.2, 0.2], [0.8, 0.8], [0.5, 0.5]],
            [0.0, 0.0, 1.0, 1.0],
        );
        assert!(v.lines.num_cells() > 5);
    }
}
