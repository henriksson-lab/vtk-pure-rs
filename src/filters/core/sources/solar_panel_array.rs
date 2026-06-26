//! Deployable solar panel array (accordion fold).
use crate::data::{CellArray, Points, PolyData};

pub fn solar_panel_array(panel_width: f64, panel_height: f64, n_panels: usize) -> PolyData {
    let np = n_panels.max(2);
    let hh = panel_height / 2.0;
    let gap = panel_width * 0.02;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    for p in 0..np {
        let x_off = (panel_width + gap) * p as f64;
        let fold_angle = if p % 2 == 0 { 0.0 } else { 0.05 }; // slight alternating fold
        let z_off = fold_angle * panel_width;
        let pb = pts.len();
        pts.push([x_off, -hh, z_off]);
        pts.push([x_off + panel_width, -hh, z_off]);
        pts.push([x_off + panel_width, hh, z_off]);
        pts.push([x_off, hh, z_off]);
        polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
        // Grid lines on panel surface
        let n_grid = 4;
        for g in 1..n_grid {
            let gx = x_off + panel_width * g as f64 / n_grid as f64;
            let g0 = pts.len();
            pts.push([gx, -hh, z_off + 0.001]);
            let g1 = pts.len();
            pts.push([gx, hh, z_off + 0.001]);
            lines.push_cell(&[g0 as i64, g1 as i64]);
        }
        for g in 1..3 {
            let gy = -hh + panel_height * g as f64 / 3.0;
            let g0 = pts.len();
            pts.push([x_off, gy, z_off + 0.001]);
            let g1 = pts.len();
            pts.push([x_off + panel_width, gy, z_off + 0.001]);
            lines.push_cell(&[g0 as i64, g1 as i64]);
        }
        // Hinge between panels
        if p + 1 < np {
            let h0 = pts.len();
            pts.push([x_off + panel_width, -hh * 0.3, z_off]);
            let h1 = pts.len();
            pts.push([x_off + panel_width, hh * 0.3, z_off]);
            lines.push_cell(&[h0 as i64, h1 as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_solar_array() {
        let m = solar_panel_array(2.0, 3.0, 4);
        assert!(m.points.len() > 40);
        assert_eq!(m.polys.num_cells(), 4);
        assert!(m.lines.num_cells() > 15);
    }
}
