//! Optical microscope (base, arm, stage, tube, eyepiece).
use crate::data::{CellArray, Points, PolyData};

pub fn microscope(height: f64, na: usize) -> PolyData {
    let na = na.max(10);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Base (horseshoe shape approximated as disk)
    let base_r = height * 0.2;
    let bc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let bb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([base_r * a.cos(), base_r * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[bc as i64, (bb + j) as i64, (bb + (j + 1) % na) as i64]);
    }
    // Arm (vertical then curved)
    let arm_w = height * 0.04;
    let a0 = pts.len();
    pts.push([0.0, -arm_w, 0.0]);
    let a1 = pts.len();
    pts.push([0.0, -arm_w, height * 0.7]);
    let a2 = pts.len();
    pts.push([0.0, -arm_w - height * 0.1, height * 0.8]);
    let a3 = pts.len();
    pts.push([0.0, -arm_w - height * 0.1, height]);
    lines.push_cell(&[a0 as i64, a1 as i64]);
    lines.push_cell(&[a1 as i64, a2 as i64]);
    lines.push_cell(&[a2 as i64, a3 as i64]);
    // Stage (small platform)
    let stage_z = height * 0.35;
    let stage_r = height * 0.12;
    let sc = pts.len();
    pts.push([0.0, 0.0, stage_z]);
    let sb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([stage_r * a.cos(), stage_r * a.sin(), stage_z]);
    }
    for j in 0..na {
        polys.push_cell(&[sc as i64, (sb + j) as i64, (sb + (j + 1) % na) as i64]);
    }
    // Objective tube (cylinder below arm top)
    let tube_r = height * 0.025;
    let tb = pts.len();
    for s in 0..=2 {
        let z = height * 0.5 + height * 0.15 * s as f64 / 2.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                tube_r * a.cos(),
                -arm_w - height * 0.1 + tube_r * a.sin(),
                z,
            ]);
        }
    }
    for s in 0..2 {
        let b0 = tb + s * na;
        let b1 = tb + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (b0 + j) as i64,
                (b1 + j) as i64,
                (b1 + j1) as i64,
                (b0 + j1) as i64,
            ]);
        }
    }
    // Eyepiece (wider cylinder at top)
    let ep_r = tube_r * 2.0;
    let eb = pts.len();
    for s in 0..=1 {
        let z = height * 0.85 + height * 0.1 * s as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([ep_r * a.cos(), -arm_w - height * 0.1 + ep_r * a.sin(), z]);
        }
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (eb + j) as i64,
            (eb + na + j) as i64,
            (eb + na + j1) as i64,
            (eb + j1) as i64,
        ]);
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
    fn test_microscope() {
        let m = microscope(5.0, 12);
        assert!(m.points.len() > 60);
        assert!(m.polys.num_cells() > 20);
    }
}
