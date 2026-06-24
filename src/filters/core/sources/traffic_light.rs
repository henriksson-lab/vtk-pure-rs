//! Traffic light geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn traffic_light(
    pole_height: f64,
    pole_radius: f64,
    housing_width: f64,
    housing_height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Pole
    for ring in 0..=1 {
        let z = if ring == 0 { 0.0 } else { pole_height };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([pole_radius * a.cos(), pole_radius * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Housing (box)
    let hw = housing_width / 2.0;
    let hh = housing_height / 2.0;
    let hd = housing_width * 0.4;
    let hb = pts.len();
    let hz = pole_height;
    pts.push([-hw, -hd, hz - hh]);
    pts.push([hw, -hd, hz - hh]);
    pts.push([hw, hd, hz - hh]);
    pts.push([-hw, hd, hz - hh]);
    pts.push([-hw, -hd, hz + hh]);
    pts.push([hw, -hd, hz + hh]);
    pts.push([hw, hd, hz + hh]);
    pts.push([-hw, hd, hz + hh]);
    let f = |i: usize| (hb + i) as i64;
    polys.push_cell(&[f(0), f(3), f(2), f(1)]);
    polys.push_cell(&[f(4), f(5), f(6), f(7)]);
    polys.push_cell(&[f(0), f(1), f(5), f(4)]);
    polys.push_cell(&[f(2), f(3), f(7), f(6)]);
    polys.push_cell(&[f(0), f(4), f(7), f(3)]);
    polys.push_cell(&[f(1), f(2), f(6), f(5)]);
    // Light circles (3 circles on front face)
    let light_r = housing_width * 0.15;
    let light_res = 8;
    for li in 0..3 {
        let lz = hz - hh * 0.6 + li as f64 * hh * 0.6;
        let lc = pts.len();
        pts.push([0.0, -hd - 0.01, lz]);
        for i in 0..light_res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / light_res as f64;
            pts.push([light_r * a.cos(), -hd - 0.01, lz + light_r * a.sin()]);
        }
        for i in 0..light_res {
            let j = if i + 1 < light_res {
                lc + 2 + i
            } else {
                lc + 1
            };
            polys.push_cell(&[lc as i64, (lc + 1 + i) as i64, j as i64]);
        }
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
        let t = traffic_light(3.0, 0.1, 0.4, 1.2, 8);
        assert!(t.polys.num_cells() > 20);
    }
}
