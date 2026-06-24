//! 2D maze generator as wall geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn maze(cols: usize, rows: usize, cell_size: f64, wall_height: f64, seed: u64) -> PolyData {
    let cols = cols.max(2);
    let rows = rows.max(2);
    let n = cols * rows;
    let mut walls_h = vec![true; cols * (rows + 1)]; // horizontal walls
    let mut walls_v = vec![true; (cols + 1) * rows]; // vertical walls
                                                     // DFS maze generation
    let mut visited = vec![false; n];
    let mut stack = Vec::new();
    let mut rng = seed;
    visited[0] = true;
    stack.push(0);
    while let Some(&current) = stack.last() {
        let cx = current % cols;
        let cy = current / cols;
        let mut neighbors = Vec::new();
        if cx > 0 && !visited[current - 1] {
            neighbors.push((current - 1, 0));
        }
        if cx + 1 < cols && !visited[current + 1] {
            neighbors.push((current + 1, 1));
        }
        if cy > 0 && !visited[current - cols] {
            neighbors.push((current - cols, 2));
        }
        if cy + 1 < rows && !visited[current + cols] {
            neighbors.push((current + cols, 3));
        }
        if neighbors.is_empty() {
            stack.pop();
            continue;
        }
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (rng >> 33) as usize % neighbors.len();
        let (next, dir) = neighbors[idx];
        visited[next] = true;
        match dir {
            0 => walls_v[cx + cy * (cols + 1)] = false,     // left
            1 => walls_v[cx + 1 + cy * (cols + 1)] = false, // right
            2 => walls_h[cx + cy * cols] = false,           // up
            3 => walls_h[cx + (cy + 1) * cols] = false,     // down
            _ => {}
        }
        stack.push(next);
    }
    // Build wall geometry
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let cs = cell_size;
    let wh = wall_height;
    let wt = cs * 0.1;
    let mut add_wall = |x0: f64, y0: f64, x1: f64, y1: f64| {
        let b = pts.len();
        let dx = x1 - x0;
        let dy = y1 - y0;
        let dl = (dx * dx + dy * dy).sqrt().max(1e-15);
        let nx = -dy / dl * wt * 0.5;
        let ny = dx / dl * wt * 0.5;
        pts.push([x0 + nx, y0 + ny, 0.0]);
        pts.push([x1 + nx, y1 + ny, 0.0]);
        pts.push([x1 - nx, y1 - ny, 0.0]);
        pts.push([x0 - nx, y0 - ny, 0.0]);
        pts.push([x0 + nx, y0 + ny, wh]);
        pts.push([x1 + nx, y1 + ny, wh]);
        pts.push([x1 - nx, y1 - ny, wh]);
        pts.push([x0 - nx, y0 - ny, wh]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    };
    for y in 0..=rows {
        for x in 0..cols {
            if walls_h[x + y * cols] {
                add_wall(
                    x as f64 * cs,
                    y as f64 * cs,
                    (x + 1) as f64 * cs,
                    y as f64 * cs,
                );
            }
        }
    }
    for y in 0..rows {
        for x in 0..=cols {
            if walls_v[x + y * (cols + 1)] {
                add_wall(
                    x as f64 * cs,
                    y as f64 * cs,
                    x as f64 * cs,
                    (y + 1) as f64 * cs,
                );
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
        let m = maze(5, 5, 1.0, 0.5, 42);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 10);
    }
}
