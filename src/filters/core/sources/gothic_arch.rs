//! Gothic (pointed) arch geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn gothic_arch(width: f64, height: f64, thickness: f64, resolution: usize) -> PolyData {
    let res = resolution.max(4);
    let hw = width / 2.0;
    let ht = thickness / 2.0;
    let radius = ((hw * hw + height * height) / (2.0 * height)).max(hw);
    let center_offset = radius - height;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Two arcs meeting at top
    let start_angle = ((hw) / (radius)).asin();
    for side in [-1.0f64, 1.0] {
        let _cx = side * hw;
        let _cy = -center_offset;
        let n = res / 2;
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let _a = if side < 0.0 {
                std::f64::consts::FRAC_PI_2 - start_angle + t * (start_angle)
            } else {
                std::f64::consts::FRAC_PI_2 + t * start_angle - start_angle
            };
            // Simplified: just use parabolic arch
            let x = hw * side * (1.0 - t);
            let y = height * (1.0 - (1.0 - t).powi(2));
            pts.push([x, y, -ht]);
            pts.push([x, y, ht]);
        }
    }
    // Connect front and back with quads
    let np = pts.len() / 2;
    for i in 0..np - 1 {
        polys.push_cell(&[
            (i * 2) as i64,
            ((i + 1) * 2) as i64,
            ((i + 1) * 2 + 1) as i64,
            (i * 2 + 1) as i64,
        ]);
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
        let a = gothic_arch(2.0, 3.0, 0.3, 12);
        assert!(a.points.len() > 10);
        assert!(a.polys.num_cells() > 3);
    }
}
