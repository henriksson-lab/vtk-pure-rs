//! Rotating space habitat (Stanford torus variant).
use crate::data::{CellArray, Points, PolyData};
pub fn stanford_torus(
    major_r: f64,
    minor_r: f64,
    spoke_count: usize,
    hub_r: f64,
    hub_l: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let ns = spoke_count.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Torus habitat ring
    for iv in 0..res {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / res as f64;
        for iu in 0..res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let r = major_r + minor_r * u.cos();
            pts.push([r * v.cos(), r * v.sin(), minor_r * u.sin()]);
        }
    }
    for iv in 0..res {
        let iv1 = (iv + 1) % res;
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (iv * res + iu) as i64,
                (iv * res + iu1) as i64,
                (iv1 * res + iu1) as i64,
                (iv1 * res + iu) as i64,
            ]);
        }
    }
    // Central hub (cylinder along Z)
    let hb = pts.len();
    for ring in 0..=1 {
        let z = if ring == 0 { -hub_l / 2.0 } else { hub_l / 2.0 };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([hub_r * a.cos(), hub_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (hb + i) as i64,
            (hb + j) as i64,
            (hb + res + j) as i64,
            (hb + res + i) as i64,
        ]);
    }
    // Spokes connecting hub to torus
    for si in 0..ns {
        let a = 2.0 * std::f64::consts::PI * si as f64 / ns as f64;
        let hub_pt_idx = pts.len();
        pts.push([hub_r * a.cos(), hub_r * a.sin(), 0.0]);
        let ring_pt_idx = pts.len();
        pts.push([major_r * a.cos(), major_r * a.sin(), 0.0]);
        lines.push_cell(&[hub_pt_idx as i64, ring_pt_idx as i64]);
    }
    // Mirror (flat disk at one end for sunlight)
    let mc = pts.len();
    let mirror_z = hub_l / 2.0 + major_r * 0.3;
    pts.push([0.0, 0.0, mirror_z]);
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([major_r * 1.2 * a.cos(), major_r * 1.2 * a.sin(), mirror_z]);
    }
    for i in 0..res {
        let j = if i + 1 < res { mc + 2 + i } else { mc + 1 };
        polys.push_cell(&[mc as i64, (mc + 1 + i) as i64, j as i64]);
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
        let t = stanford_torus(50.0, 10.0, 6, 5.0, 20.0, 12);
        assert!(t.polys.num_cells() > 100);
        assert!(t.lines.num_cells() >= 6);
    }
}
