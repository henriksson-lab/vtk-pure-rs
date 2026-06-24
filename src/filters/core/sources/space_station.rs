//! Space station geometry (modules + solar panels + truss).
use crate::data::{CellArray, Points, PolyData};
pub fn space_station(
    module_r: f64,
    module_l: f64,
    num_modules: usize,
    truss_l: f64,
    panel_w: f64,
    panel_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nm = num_modules.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Central truss
    let tb = pts.len();
    pts.push([-truss_l / 2.0, 0.0, 0.0]);
    pts.push([truss_l / 2.0, 0.0, 0.0]);
    lines.push_cell(&[tb as i64, (tb + 1) as i64]);
    // Modules along truss
    let spacing = truss_l / (nm + 1) as f64;
    for mi in 0..nm {
        let cx = -truss_l / 2.0 + spacing * (mi + 1) as f64;
        let mb = pts.len();
        for ring in 0..=1 {
            let y = if ring == 0 {
                -module_l / 2.0
            } else {
                module_l / 2.0
            };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([cx + module_r * a.cos(), y, module_r * a.sin()]);
            }
        }
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (mb + i) as i64,
                (mb + j) as i64,
                (mb + res + j) as i64,
                (mb + res + i) as i64,
            ]);
        }
    }
    // Solar panels (flat quads at truss ends)
    for &side in &[-1.0f64, 1.0] {
        let px = side * truss_l / 2.0;
        for &pz in &[panel_h * 0.7, -panel_h * 0.7] {
            let pb = pts.len();
            pts.push([px, -panel_w / 2.0, pz - panel_h / 2.0]);
            pts.push([px, panel_w / 2.0, pz - panel_h / 2.0]);
            pts.push([px, panel_w / 2.0, pz + panel_h / 2.0]);
            pts.push([px, -panel_w / 2.0, pz + panel_h / 2.0]);
            polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
            // Panel strut
            let sb = pts.len();
            pts.push([px, 0.0, 0.0]);
            pts.push([px, 0.0, pz]);
            lines.push_cell(&[sb as i64, (sb + 1) as i64]);
        }
    }
    // Docking port (small cylinder at one end)
    let db = pts.len();
    let dr = module_r * 0.4;
    for ring in 0..=1 {
        let y = if ring == 0 { -module_l * 0.3 } else { 0.0 };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([0.0 + dr * a.cos(), y, dr * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (db + i) as i64,
            (db + j) as i64,
            (db + res + j) as i64,
            (db + res + i) as i64,
        ]);
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
        let s = space_station(1.5, 4.0, 3, 30.0, 8.0, 3.0, 8);
        assert!(s.polys.num_cells() > 20);
        assert!(s.lines.num_cells() > 3);
    }
}
