//! Equatorial telescope mount (tripod + polar axis + declination axis).
use crate::data::{CellArray, Points, PolyData};

pub fn telescope_mount(height: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    // Tripod legs
    let top = pts.len();
    pts.push([0.0, 0.0, height]);
    for i in 0..3 {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0;
        let foot = pts.len();
        pts.push([height * 0.5 * a.cos(), height * 0.5 * a.sin(), 0.0]);
        lines.push_cell(&[top as i64, foot as i64]);
    }
    // Polar axis (tilted at ~45 degrees toward north)
    let polar_base = top;
    let polar_top = pts.len();
    pts.push([0.0, -height * 0.3, height + height * 0.3]);
    lines.push_cell(&[polar_base as i64, polar_top as i64]);
    // Declination axis (perpendicular to polar)
    let dec_a = pts.len();
    pts.push([-height * 0.15, -height * 0.3, height + height * 0.3]);
    let dec_b = pts.len();
    pts.push([height * 0.15, -height * 0.3, height + height * 0.3]);
    lines.push_cell(&[dec_a as i64, dec_b as i64]);
    // Counterweight shaft
    let cw_end = pts.len();
    pts.push([height * 0.3, -height * 0.15, height + height * 0.15]);
    lines.push_cell(&[polar_top as i64, cw_end as i64]);
    // Telescope tube (from dec axis)
    let tube_end = pts.len();
    pts.push([-height * 0.15, -height * 0.6, height + height * 0.5]);
    lines.push_cell(&[dec_a as i64, tube_end as i64]);
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mount() {
        let m = telescope_mount(3.0);
        assert!(m.points.len() >= 8);
        assert!(m.lines.num_cells() >= 6);
    }
}
