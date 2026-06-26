//! Dock/pier geometry extending over water.
use crate::data::{CellArray, Points, PolyData};
pub fn dock(
    length: f64,
    width: f64,
    height: f64,
    num_pilings: usize,
    piling_radius: f64,
) -> PolyData {
    let hw = width / 2.0;
    let np = num_pilings.max(2);
    let piling_radius = piling_radius.abs();
    let piling_resolution = 12;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Deck
    let db = pts.len();
    pts.push([0.0, -hw, height]);
    pts.push([length, -hw, height]);
    pts.push([length, hw, height]);
    pts.push([0.0, hw, height]);
    polys.push_cell(&[db as i64, (db + 1) as i64, (db + 2) as i64, (db + 3) as i64]);
    // Pilings
    let spacing = length / (np - 1) as f64;
    for pi in 0..np {
        let x = pi as f64 * spacing;
        for &y in &[-hw * 0.8, hw * 0.8] {
            if piling_radius > 0.0 {
                let pb = pts.len();
                for iz in 0..2 {
                    let z = if iz == 0 { -height * 0.5 } else { height };
                    for i in 0..piling_resolution {
                        let a = 2.0 * std::f64::consts::PI * i as f64 / piling_resolution as f64;
                        pts.push([x + piling_radius * a.cos(), y + piling_radius * a.sin(), z]);
                    }
                }
                for i in 0..piling_resolution {
                    let i1 = (i + 1) % piling_resolution;
                    polys.push_cell(&[
                        (pb + i) as i64,
                        (pb + i1) as i64,
                        (pb + piling_resolution + i1) as i64,
                        (pb + piling_resolution + i) as i64,
                    ]);
                }
            }
            let lb = pts.len();
            pts.push([x, y, -height * 0.5]);
            pts.push([x, y, height]);
            lines.push_cell(&[lb as i64, (lb + 1) as i64]);
        }
    }
    // Cross bracing
    for pi in 0..np - 1 {
        let x0 = pi as f64 * spacing;
        let x1 = (pi + 1) as f64 * spacing;
        let cb = pts.len();
        pts.push([x0, -hw * 0.8, 0.0]);
        pts.push([x1, hw * 0.8, height * 0.5]);
        lines.push_cell(&[cb as i64, (cb + 1) as i64]);
    }
    // Railings
    let rh = height + width * 0.3;
    let rlb = pts.len();
    pts.push([0.0, -hw, rh]);
    pts.push([length, -hw, rh]);
    lines.push_cell(&[rlb as i64, (rlb + 1) as i64]);
    let rrb = pts.len();
    pts.push([0.0, hw, rh]);
    pts.push([length, hw, rh]);
    lines.push_cell(&[rrb as i64, (rrb + 1) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let d = dock(15.0, 3.0, 2.0, 6, 0.15);
        assert!(d.polys.num_cells() >= 1);
        assert!(d.lines.num_cells() > 10);
        assert!(d.polys.num_cells() > 1);
    }
}
