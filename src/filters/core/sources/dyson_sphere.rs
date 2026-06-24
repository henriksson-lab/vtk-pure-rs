//! Dyson sphere/swarm concept geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn dyson_sphere(radius: f64, panel_coverage: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Sparse shell panels
    let vres = res;
    let coverage = panel_coverage.clamp(0.01, 1.0);
    let mut rng = 42u64;
    for iv in 0..vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            if ((rng >> 33) as f64 / u32::MAX as f64) > coverage {
                continue;
            }
            let x = radius * sv * u.cos();
            let y = radius * sv * u.sin();
            let z = radius * cv;
            let du = 2.0 * std::f64::consts::PI / res as f64;
            let dv = std::f64::consts::PI / vres as f64;
            let panel_size = radius * du.min(dv) * 0.4;
            // Tangent vectors
            let tx = -u.sin();
            let ty = u.cos();
            let vx = cv * u.cos();
            let vy = cv * u.sin();
            let vz = -sv;
            let pb = pts.len();
            pts.push([
                x - panel_size * (tx + vx),
                y - panel_size * (ty + vy),
                z - panel_size * vz,
            ]);
            pts.push([
                x + panel_size * (tx - vx),
                y + panel_size * (ty - vy),
                z + panel_size * vz,
            ]);
            pts.push([
                x + panel_size * (tx + vx),
                y + panel_size * (ty + vy),
                z + panel_size * vz,
            ]);
            pts.push([
                x - panel_size * (tx - vx),
                y - panel_size * (ty - vy),
                z - panel_size * vz,
            ]);
            polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64, (pb + 3) as i64]);
        }
    }
    // Central star (small sphere representation)
    let star_r = radius * 0.01;
    let sc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for i in 0..6 {
        let a = std::f64::consts::FRAC_PI_3 * i as f64;
        pts.push([star_r * a.cos(), star_r * a.sin(), 0.0]);
    }
    for i in 0..6 {
        let j = if i + 1 < 6 { sc + 2 + i } else { sc + 1 };
        polys.push_cell(&[sc as i64, (sc + 1 + i) as i64, j as i64]);
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
        let d = dyson_sphere(100.0, 0.3, 8);
        assert!(d.polys.num_cells() > 10);
    }
}
