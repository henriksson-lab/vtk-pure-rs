//! Campfire geometry (logs + flame shape).
use crate::data::{CellArray, Points, PolyData};
pub fn campfire(
    log_length: f64,
    log_radius: f64,
    num_logs: usize,
    flame_height: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nl = num_logs.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let log_ring_r = log_length * 0.3;
    // Logs arranged in tepee pattern
    for li in 0..nl {
        let angle = 2.0 * std::f64::consts::PI * li as f64 / nl as f64;
        let tilt = 0.4f64;
        let ca = angle.cos();
        let sa = angle.sin();
        // Log as simplified cylinder (2 rings)
        let base_x = log_ring_r * ca;
        let base_y = log_ring_r * sa;
        let tip_x = 0.0;
        let tip_y = 0.0;
        let tip_z = log_length * tilt.sin();
        for ring in 0..2 {
            let t = ring as f64;
            let cx = base_x * (1.0 - t) + tip_x * t;
            let cy = base_y * (1.0 - t) + tip_y * t;
            let cz = t * tip_z;
            let dir_x = tip_x - base_x;
            let dir_y = tip_y - base_y;
            let dir_z = tip_z;
            let dl = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z)
                .sqrt()
                .max(1e-15);
            let dx = dir_x / dl;
            let dy = dir_y / dl;
            let dz = dir_z / dl;
            let ux = if dx.abs() < 0.9 { 1.0 } else { 0.0 };
            let uy = if dx.abs() < 0.9 { 0.0 } else { 1.0 };
            let uz = 0.0;
            let n1x = dy * uz - dz * uy;
            let n1y = dz * ux - dx * uz;
            let n1z = dx * uy - dy * ux;
            let nl = (n1x * n1x + n1y * n1y + n1z * n1z).sqrt().max(1e-15);
            let n1x = n1x / nl;
            let n1y = n1y / nl;
            let n1z = n1z / nl;
            let n2x = dy * n1z - dz * n1y;
            let n2y = dz * n1x - dx * n1z;
            let n2z = dx * n1y - dy * n1x;
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([
                    cx + log_radius * (a.cos() * n1x + a.sin() * n2x),
                    cy + log_radius * (a.cos() * n1y + a.sin() * n2y),
                    cz + log_radius * (a.cos() * n1z + a.sin() * n2z),
                ]);
            }
        }
        let base_idx = pts.len() - res * 2;
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (base_idx + i) as i64,
                (base_idx + j) as i64,
                (base_idx + res + j) as i64,
                (base_idx + res + i) as i64,
            ]);
        }
    }
    // Flame (cone)
    let fc = pts.len();
    pts.push([0.0, 0.0, flame_height]);
    let fr = log_ring_r * 0.6;
    for i in 0..res {
        let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
        pts.push([fr * a.cos(), fr * a.sin(), log_length * 0.15]);
    }
    for i in 0..res {
        let j = if i + 1 < res { fc + 2 + i } else { fc + 1 };
        polys.push_cell(&[fc as i64, (fc + 1 + i) as i64, j as i64]);
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
        let c = campfire(1.0, 0.05, 5, 1.5, 6);
        assert!(c.polys.num_cells() > 10);
    }
}
