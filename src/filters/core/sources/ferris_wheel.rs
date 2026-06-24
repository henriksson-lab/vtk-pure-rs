//! Ferris wheel geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn ferris_wheel(
    radius: f64,
    num_gondolas: usize,
    gondola_size: f64,
    hub_height: f64,
    _resolution: usize,
) -> PolyData {
    let ng = num_gondolas.max(4);
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    // Support structure (A-frame)
    let spread = radius * 0.3;
    let b = pts.len();
    pts.push([-spread, 0.0, 0.0]);
    pts.push([0.0, 0.0, hub_height]);
    pts.push([spread, 0.0, 0.0]);
    pts.push([0.0, 0.0, hub_height]);
    lines.push_cell(&[b as i64, (b + 1) as i64]);
    lines.push_cell(&[(b + 2) as i64, (b + 3) as i64]);
    // Wheel rim (spokes)
    let hub = pts.len();
    pts.push([0.0, 0.0, hub_height]);
    for i in 0..ng {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ng as f64;
        let x = radius * a.cos();
        let z = hub_height + radius * a.sin();
        let si = pts.len();
        pts.push([x, 0.0, z]);
        lines.push_cell(&[hub as i64, si as i64]);
    }
    // Rim circle
    let rim_start = hub + 1;
    for i in 0..ng {
        let j = (i + 1) % ng;
        lines.push_cell(&[(rim_start + i) as i64, (rim_start + j) as i64]);
    }
    // Gondolas (small boxes)
    let gs = gondola_size / 2.0;
    for i in 0..ng {
        let a = 2.0 * std::f64::consts::PI * i as f64 / ng as f64;
        let cx = radius * a.cos();
        let cz = hub_height + radius * a.sin() - gs * 2.0;
        let gb = pts.len();
        pts.push([cx - gs, -gs, cz]);
        pts.push([cx + gs, -gs, cz]);
        pts.push([cx + gs, gs, cz]);
        pts.push([cx - gs, gs, cz]);
        pts.push([cx - gs, -gs, cz + gs * 2.0]);
        pts.push([cx + gs, -gs, cz + gs * 2.0]);
        pts.push([cx + gs, gs, cz + gs * 2.0]);
        pts.push([cx - gs, gs, cz + gs * 2.0]);
        let f = |i: usize| (gb + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let f = ferris_wheel(10.0, 8, 1.0, 12.0, 16);
        assert!(f.lines.num_cells() > 10);
        assert!(f.polys.num_cells() >= 16);
    }
}
