//! Space shuttle geometry (orbiter + external tank + boosters).
use crate::data::{CellArray, Points, PolyData};
pub fn space_shuttle(
    orbiter_l: f64,
    orbiter_r: f64,
    tank_r: f64,
    tank_l: f64,
    booster_r: f64,
    booster_l: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Orbiter fuselage
    let nseg = 8;
    let ol2 = orbiter_l / 2.0;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let x = -ol2 + orbiter_l * t;
        let taper = (1.0 - (2.0 * t - 1.0).powi(4)).max(0.05);
        let r = orbiter_r * taper.sqrt();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, r * a.cos(), r * a.sin() + tank_r * 1.3]);
        }
    }
    for is in 0..nseg {
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (is * res + i) as i64,
                (is * res + j) as i64,
                ((is + 1) * res + j) as i64,
                ((is + 1) * res + i) as i64,
            ]);
        }
    }
    // Wings (delta)
    let wing_span = orbiter_l * 0.6;
    let wing_chord = orbiter_l * 0.4;
    let wb = pts.len();
    pts.push([0.0, -wing_span / 2.0, tank_r * 1.2]);
    pts.push([wing_chord, 0.0, tank_r * 1.2]);
    pts.push([0.0, wing_span / 2.0, tank_r * 1.2]);
    pts.push([-wing_chord * 0.2, 0.0, tank_r * 1.2]);
    polys.push_cell(&[wb as i64, (wb + 1) as i64, (wb + 2) as i64, (wb + 3) as i64]);
    // Vertical stabilizer
    let vb = pts.len();
    pts.push([ol2 * 0.3, 0.0, tank_r * 1.3]);
    pts.push([ol2 * 0.7, 0.0, tank_r * 1.3]);
    pts.push([ol2 * 0.5, 0.0, tank_r * 1.3 + orbiter_r * 2.0]);
    polys.push_cell(&[vb as i64, (vb + 1) as i64, (vb + 2) as i64]);
    // External tank
    let et_offset = 0.0;
    let etb = pts.len();
    for ring in 0..=1 {
        let x = -tank_l / 2.0 + if ring == 0 { 0.0 } else { tank_l };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x + et_offset, tank_r * a.cos(), tank_r * a.sin()]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (etb + i) as i64,
            (etb + j) as i64,
            (etb + res + j) as i64,
            (etb + res + i) as i64,
        ]);
    }
    // Nose cone
    let etc = pts.len();
    pts.push([-tank_l / 2.0 - tank_r * 0.5 + et_offset, 0.0, 0.0]);
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[etc as i64, (etb + j) as i64, (etb + i) as i64]);
    }
    // Solid rocket boosters (two)
    for &by in &[-tank_r * 1.3, tank_r * 1.3] {
        let sbb = pts.len();
        for ring in 0..=1 {
            let x = -booster_l / 2.0 + if ring == 0 { 0.0 } else { booster_l };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([x, by + booster_r * a.cos(), booster_r * a.sin()]);
            }
        }
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (sbb + i) as i64,
                (sbb + j) as i64,
                (sbb + res + j) as i64,
                (sbb + res + i) as i64,
            ]);
        }
        // Nose
        let sbc = pts.len();
        pts.push([-booster_l / 2.0 - booster_r * 0.3, by, 0.0]);
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[sbc as i64, (sbb + j) as i64, (sbb + i) as i64]);
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
        let s = space_shuttle(15.0, 1.5, 3.0, 20.0, 1.0, 18.0, 8);
        assert!(s.polys.num_cells() > 60);
    }
}
