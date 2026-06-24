//! Virus capsid (icosahedral shell with spikes).
use crate::data::{CellArray, Points, PolyData};
pub fn virus_capsid(
    radius: f64,
    spike_height: f64,
    num_spikes: usize,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Icosahedral capsid (sphere approximation)
    let vres = res;
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            pts.push([radius * sv * u.cos(), radius * sv * u.sin(), radius * cv]);
        }
    }
    let w = res + 1;
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Spikes (small cones protruding from surface)
    let ns = num_spikes.min(vres * res);
    let _spike_r = radius * 0.05;
    let mut rng = 42u64;
    for _ in 0..ns {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let theta = std::f64::consts::PI * ((rng >> 33) as f64 / u32::MAX as f64);
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let phi = 2.0 * std::f64::consts::PI * ((rng >> 33) as f64 / u32::MAX as f64);
        let st = theta.sin();
        let ct = theta.cos();
        let nx = st * phi.cos();
        let ny = st * phi.sin();
        let nz = ct;
        let base_x = radius * nx;
        let base_y = radius * ny;
        let base_z = radius * nz;
        let tip_x = (radius + spike_height) * nx;
        let tip_y = (radius + spike_height) * ny;
        let tip_z = (radius + spike_height) * nz;
        // Spike as line + small triangle base
        let sb = pts.len();
        pts.push([base_x, base_y, base_z]);
        pts.push([tip_x, tip_y, tip_z]);
        lines.push_cell(&[sb as i64, (sb + 1) as i64]);
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
        let v = virus_capsid(5.0, 1.0, 20, 8);
        assert!(v.polys.num_cells() > 40);
        assert!(v.lines.num_cells() >= 20);
    }
}
