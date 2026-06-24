//! Airplane geometry (fuselage + wings + tail + engines).
use crate::data::{CellArray, Points, PolyData};
pub fn airplane(
    fuselage_l: f64,
    fuselage_r: f64,
    wing_span: f64,
    wing_chord: f64,
    tail_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let hl = fuselage_l / 2.0;
    let hws = wing_span / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Fuselage
    let nseg = 8;
    for is in 0..=nseg {
        let t = is as f64 / nseg as f64;
        let x = -hl + fuselage_l * t;
        let taper = (1.0 - (2.0 * t - 1.0).powi(4)).max(0.05);
        let r = fuselage_r * taper.sqrt();
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, r * a.cos(), r * a.sin()]);
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
    // Wings (flat quads)
    let wing_x = -fuselage_l * 0.05;
    let wing_z = -fuselage_r * 0.3;
    let wb = pts.len();
    pts.push([wing_x, -hws, wing_z]);
    pts.push([wing_x + wing_chord, -hws, wing_z]);
    pts.push([wing_x + wing_chord * 0.8, hws, wing_z]);
    pts.push([wing_x - wing_chord * 0.1, hws, wing_z]);
    polys.push_cell(&[wb as i64, (wb + 1) as i64, (wb + 2) as i64, (wb + 3) as i64]);
    // Horizontal stabilizer
    let stab_span = wing_span * 0.35;
    let stab_chord = wing_chord * 0.4;
    let stab_x = hl * 0.7;
    let sb = pts.len();
    pts.push([stab_x, -stab_span / 2.0, 0.0]);
    pts.push([stab_x + stab_chord, -stab_span / 2.0, 0.0]);
    pts.push([stab_x + stab_chord, stab_span / 2.0, 0.0]);
    pts.push([stab_x, stab_span / 2.0, 0.0]);
    polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64, (sb + 3) as i64]);
    // Vertical stabilizer
    let vb = pts.len();
    pts.push([stab_x, 0.0, 0.0]);
    pts.push([stab_x + stab_chord * 0.8, 0.0, 0.0]);
    pts.push([stab_x + stab_chord * 0.5, 0.0, tail_h]);
    pts.push([stab_x, 0.0, tail_h * 0.8]);
    polys.push_cell(&[vb as i64, (vb + 1) as i64, (vb + 2) as i64, (vb + 3) as i64]);
    // Engine nacelles (simplified cylinders under wings)
    for &ey in &[-hws * 0.35, hws * 0.35] {
        let ex = wing_x + wing_chord * 0.3;
        let er = fuselage_r * 0.3;
        let eb = pts.len();
        for ring in 0..=1 {
            let x = ex + if ring == 0 { -er * 2.0 } else { er * 2.0 };
            for i in 0..res / 2 {
                let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
                pts.push([x, ey + er * a.cos(), wing_z + er * a.sin()]);
            }
        }
        let hr = res / 2;
        for i in 0..hr {
            let j = (i + 1) % hr;
            polys.push_cell(&[
                (eb + i) as i64,
                (eb + j) as i64,
                (eb + hr + j) as i64,
                (eb + hr + i) as i64,
            ]);
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
        let a = airplane(20.0, 1.5, 15.0, 3.0, 2.5, 8);
        assert!(a.polys.num_cells() > 40);
    }
}
