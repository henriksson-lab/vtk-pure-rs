//! Mars rover geometry (body + wheels + arm + camera mast).
use crate::data::{CellArray, Points, PolyData};
pub fn mars_rover(body_l: f64, body_w: f64, body_h: f64, wheel_r: f64, arm_l: f64) -> PolyData {
    let hl = body_l / 2.0;
    let hw = body_w / 2.0;
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
        polys.push_cell(&[f(0), f(4), f(7), f(3)]);
        polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    };
    // Body
    ab(
        &mut pts,
        &mut polys,
        -hl,
        -hw,
        wheel_r * 1.2,
        hl,
        hw,
        wheel_r * 1.2 + body_h,
    );
    // Wheels (6 wheels, 3 per side)
    let wres = 8;
    for &x in &[-hl * 0.7, 0.0, hl * 0.7] {
        for &y in &[-hw - wheel_r * 0.3, hw + wheel_r * 0.3] {
            let wc = pts.len();
            pts.push([x, y, wheel_r]);
            for i in 0..wres {
                let a = 2.0 * std::f64::consts::PI * i as f64 / wres as f64;
                pts.push([x, y + wheel_r * a.cos() * 0.3, wheel_r + wheel_r * a.sin()]);
            }
            for i in 0..wres {
                let j = (i + 1) % wres;
                lines.push_cell(&[(wc + 1 + i) as i64, (wc + 1 + j) as i64]);
            }
            // Spokes
            for i in 0..wres {
                lines.push_cell(&[wc as i64, (wc + 1 + i) as i64]);
            }
            // Rocker-bogie suspension arm
            let sb = pts.len();
            pts.push([x, y.signum() * hw, wheel_r * 1.2]);
            pts.push([x, y, wheel_r]);
            lines.push_cell(&[sb as i64, (sb + 1) as i64]);
        }
    }
    // Camera mast
    let mast_h = body_h * 2.0;
    let mz = wheel_r * 1.2 + body_h;
    let mb = pts.len();
    pts.push([-hl * 0.3, 0.0, mz]);
    pts.push([-hl * 0.3, 0.0, mz + mast_h]);
    lines.push_cell(&[mb as i64, (mb + 1) as i64]);
    // Camera head
    let ch = body_h * 0.3;
    ab(
        &mut pts,
        &mut polys,
        -hl * 0.3 - ch / 2.0,
        -ch / 2.0,
        mz + mast_h,
        -hl * 0.3 + ch / 2.0,
        ch / 2.0,
        mz + mast_h + ch,
    );
    // Robot arm
    let armb = pts.len();
    pts.push([hl * 0.5, 0.0, mz]);
    pts.push([hl * 0.5 + arm_l * 0.5, 0.0, mz + arm_l * 0.3]);
    pts.push([hl * 0.5 + arm_l, 0.0, mz]);
    lines.push_cell(&[armb as i64, (armb + 1) as i64]);
    lines.push_cell(&[(armb + 1) as i64, (armb + 2) as i64]);
    // Solar panel on top
    let sp = pts.len();
    let pw = body_w * 0.9;
    let pl = body_l * 0.8;
    pts.push([-pl / 2.0, -pw / 2.0, mz + 0.01]);
    pts.push([pl / 2.0, -pw / 2.0, mz + 0.01]);
    pts.push([pl / 2.0, pw / 2.0, mz + 0.01]);
    pts.push([-pl / 2.0, pw / 2.0, mz + 0.01]);
    polys.push_cell(&[sp as i64, (sp + 1) as i64, (sp + 2) as i64, (sp + 3) as i64]);
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
        let r = mars_rover(2.0, 1.5, 0.5, 0.25, 1.0);
        assert!(r.polys.num_cells() > 10);
        assert!(r.lines.num_cells() > 15);
    }
}
