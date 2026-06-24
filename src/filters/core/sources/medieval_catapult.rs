//! Detailed medieval catapult with wheels and frame.
use crate::data::{CellArray, Points, PolyData};
pub fn medieval_catapult(frame_w: f64, frame_l: f64, arm_l: f64, wheel_r: f64) -> PolyData {
    let hw = frame_w / 2.0;
    let hl = frame_l / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let ab = |pts: &mut Points<f64>,
              polys: &mut CellArray,
              x0: f64,
              y0: f64,
              z0: f64,
              x1: f64,
              y1: f64,
              z1: f64| {
        let b = pts.len();
        pts.push([x0, y0, z0]);
        pts.push([x1, y0, z0]);
        pts.push([x1, y1, z0]);
        pts.push([x0, y1, z0]);
        pts.push([x0, y0, z1]);
        pts.push([x1, y0, z1]);
        pts.push([x1, y1, z1]);
        pts.push([x0, y1, z1]);
        let f = |i: usize| (b + i) as i64;
        polys.push_cell(&[f(0), f(3), f(2), f(1)]);
        polys.push_cell(&[f(4), f(5), f(6), f(7)]);
        polys.push_cell(&[f(0), f(1), f(5), f(4)]);
        polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    };
    // Frame base
    let beam_t = frame_w * 0.06;
    let axle_h = wheel_r;
    ab(
        &mut pts,
        &mut polys,
        -hl,
        -hw,
        axle_h,
        hl,
        hw,
        axle_h + beam_t,
    );
    // Side beams
    ab(
        &mut pts,
        &mut polys,
        -hl,
        -hw,
        axle_h,
        hl,
        -hw + beam_t,
        axle_h + frame_w * 0.5,
    );
    ab(
        &mut pts,
        &mut polys,
        -hl,
        hw - beam_t,
        axle_h,
        hl,
        hw,
        axle_h + frame_w * 0.5,
    );
    // A-frame uprights
    let tower_h = frame_w * 0.8;
    for &y in &[-hw * 0.5, hw * 0.5] {
        let ub = pts.len();
        pts.push([-beam_t, y - beam_t, axle_h]);
        pts.push([beam_t, y + beam_t, axle_h]);
        pts.push([0.0, y, axle_h + tower_h]);
        lines.push_cell(&[ub as i64, (ub + 2) as i64]);
        lines.push_cell(&[(ub + 1) as i64, (ub + 2) as i64]);
    }
    // Axle
    let axb = pts.len();
    pts.push([0.0, -hw * 0.5, axle_h + tower_h]);
    pts.push([0.0, hw * 0.5, axle_h + tower_h]);
    lines.push_cell(&[axb as i64, (axb + 1) as i64]);
    // Arm
    let arm_short = arm_l * 0.3;
    let arm_long = arm_l * 0.7;
    let armb = pts.len();
    pts.push([-arm_short, 0.0, axle_h + tower_h]);
    pts.push([arm_long, 0.0, axle_h + tower_h]);
    lines.push_cell(&[armb as i64, (armb + 1) as i64]);
    // Counterweight
    let cw = arm_l * 0.12;
    ab(
        &mut pts,
        &mut polys,
        -arm_short - cw,
        -cw,
        axle_h + tower_h - cw * 3.0,
        -arm_short + cw,
        cw,
        axle_h + tower_h,
    );
    // Sling
    let slb = pts.len();
    pts.push([arm_long, 0.0, axle_h + tower_h]);
    pts.push([arm_long + arm_l * 0.15, 0.0, axle_h + tower_h - arm_l * 0.2]);
    lines.push_cell(&[slb as i64, (slb + 1) as i64]);
    // Wheels
    let wres = 12;
    for &x in &[-hl * 0.7, hl * 0.7] {
        for &y in &[-hw, hw] {
            let wc = pts.len();
            pts.push([x, y, wheel_r]); // wheel center
            for i in 0..wres {
                let a = 2.0 * std::f64::consts::PI * i as f64 / wres as f64;
                let wi = pts.len();
                pts.push([x, y + wheel_r * a.cos(), wheel_r + wheel_r * a.sin()]);
                lines.push_cell(&[wc as i64, wi as i64]);
            }
            // Rim
            for i in 0..wres {
                let j = (i + 1) % wres;
                lines.push_cell(&[(wc + 1 + i) as i64, (wc + 1 + j) as i64]);
            }
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
        let c = medieval_catapult(2.0, 4.0, 5.0, 0.6);
        assert!(c.polys.num_cells() > 10);
        assert!(c.lines.num_cells() > 20);
    }
}
