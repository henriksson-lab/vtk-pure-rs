//! Particle physics detector (barrel + endcaps + calorimeters).
use crate::data::{CellArray, Points, PolyData};
pub fn particle_detector(
    barrel_r: f64,
    barrel_l: f64,
    endcap_r: f64,
    num_layers: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let nl = num_layers.max(2);
    let hl = barrel_l / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let lines = CellArray::new();
    // Barrel layers (concentric cylinders)
    for li in 0..nl {
        let r = barrel_r * (li + 1) as f64 / nl as f64;
        for ring in 0..=1 {
            let x = if ring == 0 { -hl } else { hl };
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([x, r * a.cos(), r * a.sin()]);
            }
        }
        let base = pts.len() - res * 2;
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (base + i) as i64,
                (base + j) as i64,
                (base + res + j) as i64,
                (base + res + i) as i64,
            ]);
        }
    }
    // Endcaps (disks at each end)
    for &side in &[-1.0f64, 1.0] {
        let x = side * hl;
        let ec = pts.len();
        pts.push([x, 0.0, 0.0]);
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([x, endcap_r * a.cos(), endcap_r * a.sin()]);
        }
        for i in 0..res {
            let j = if i + 1 < res { ec + 2 + i } else { ec + 1 };
            polys.push_cell(&[ec as i64, (ec + 1 + i) as i64, j as i64]);
        }
    }
    // Beam pipe (thin cylinder along axis)
    let bp_r = barrel_r * 0.05;
    let bpb = pts.len();
    for ring in 0..=1 {
        let x = if ring == 0 { -hl * 1.5 } else { hl * 1.5 };
        for i in 0..res / 2 {
            let a = 2.0 * std::f64::consts::PI * i as f64 / (res / 2) as f64;
            pts.push([x, bp_r * a.cos(), bp_r * a.sin()]);
        }
    }
    let hr = res / 2;
    for i in 0..hr {
        let j = (i + 1) % hr;
        polys.push_cell(&[
            (bpb + i) as i64,
            (bpb + j) as i64,
            (bpb + hr + j) as i64,
            (bpb + hr + i) as i64,
        ]);
    }
    // Muon chambers (outermost layer segments)
    let mu_r = barrel_r * 1.2;
    for si in 0..8 {
        let a0 = std::f64::consts::FRAC_PI_4 * si as f64;
        let a1 = a0 + std::f64::consts::FRAC_PI_4 * 0.8;
        let mb = pts.len();
        pts.push([-hl, mu_r * a0.cos(), mu_r * a0.sin()]);
        pts.push([hl, mu_r * a0.cos(), mu_r * a0.sin()]);
        pts.push([hl, mu_r * a1.cos(), mu_r * a1.sin()]);
        pts.push([-hl, mu_r * a1.cos(), mu_r * a1.sin()]);
        polys.push_cell(&[mb as i64, (mb + 1) as i64, (mb + 2) as i64, (mb + 3) as i64]);
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
        let d = particle_detector(3.0, 8.0, 4.0, 3, 12);
        assert!(d.polys.num_cells() > 40);
    }
}
