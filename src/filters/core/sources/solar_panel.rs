//! Solar panel array geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn solar_panel_array(
    panel_w: f64,
    panel_h: f64,
    rows: usize,
    cols: usize,
    tilt_angle: f64,
    gap: f64,
    mount_height: f64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let ca = tilt_angle.to_radians().cos();
    let sa = tilt_angle.to_radians().sin();
    for r in 0..rows {
        for c in 0..cols {
            let x = c as f64 * (panel_w + gap);
            let y = r as f64 * (panel_h * ca + gap);
            let b = pts.len();
            // Panel (tilted quad)
            pts.push([x, y, mount_height]);
            pts.push([x + panel_w, y, mount_height]);
            pts.push([x + panel_w, y + panel_h * ca, mount_height + panel_h * sa]);
            pts.push([x, y + panel_h * ca, mount_height + panel_h * sa]);
            polys.push_cell(&[b as i64, (b + 1) as i64, (b + 2) as i64, (b + 3) as i64]);
            // Support legs
            let lb = pts.len();
            pts.push([x + panel_w * 0.2, y, 0.0]);
            pts.push([x + panel_w * 0.2, y, mount_height]);
            pts.push([x + panel_w * 0.8, y, 0.0]);
            pts.push([x + panel_w * 0.8, y, mount_height]);
            lines.push_cell(&[lb as i64, (lb + 1) as i64]);
            lines.push_cell(&[(lb + 2) as i64, (lb + 3) as i64]);
        }
    }
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
        let s = solar_panel_array(2.0, 1.0, 2, 3, 30.0, 0.2, 1.5);
        assert_eq!(s.polys.num_cells(), 6);
        assert!(s.lines.num_cells() >= 12);
    }
}
