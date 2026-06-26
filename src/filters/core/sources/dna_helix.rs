//! DNA double helix geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn dna_helix(
    radius: f64,
    pitch: f64,
    turns: f64,
    tube_radius: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let tres = 6;
    let turns = turns.max(0.0);
    let total = ((res as f64 * turns).ceil() as usize).max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Two helical backbones
    for strand in 0..2 {
        let phase = strand as f64 * std::f64::consts::PI;
        for i in 0..=total {
            let t = i as f64 / total as f64 * turns;
            let a = 2.0 * std::f64::consts::PI * t + phase;
            let cx = radius * a.cos();
            let cy = radius * a.sin();
            let cz = t * pitch;
            // Tube cross-section
            let t2 = (i as f64 + 0.01) / total as f64 * turns;
            let a2 = 2.0 * std::f64::consts::PI * t2 + phase;
            let tx = radius * a2.cos() - cx;
            let ty = radius * a2.sin() - cy;
            let tz = t2 * pitch - cz;
            let tl = (tx * tx + ty * ty + tz * tz).sqrt().max(1e-15);
            let tang = [tx / tl, ty / tl, tz / tl];
            let up = if tang[0].abs() < 0.9 {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            let n1 = normalize(cross(tang, up));
            let n2 = cross(tang, n1);
            for it in 0..tres {
                let phi = 2.0 * std::f64::consts::PI * it as f64 / tres as f64;
                pts.push([
                    cx + tube_radius * (phi.cos() * n1[0] + phi.sin() * n2[0]),
                    cy + tube_radius * (phi.cos() * n1[1] + phi.sin() * n2[1]),
                    cz + tube_radius * (phi.cos() * n1[2] + phi.sin() * n2[2]),
                ]);
            }
        }
        let base = strand * (total + 1) * tres;
        for i in 0..total {
            let r0 = base + i * tres;
            let r1 = base + (i + 1) * tres;
            for it in 0..tres {
                let it1 = (it + 1) % tres;
                polys.push_cell(&[
                    (r0 + it) as i64,
                    (r0 + it1) as i64,
                    (r1 + it1) as i64,
                    (r1 + it) as i64,
                ]);
            }
        }
    }
    // Base pair rungs (every few steps)
    let rung_interval = total / ((turns * 2.0) as usize).max(1);
    for i in (0..=total).step_by(rung_interval.max(1)) {
        let t = i as f64 / total as f64 * turns;
        let a1 = 2.0 * std::f64::consts::PI * t;
        let a2 = a1 + std::f64::consts::PI;
        let z = t * pitch;
        let b = pts.len();
        pts.push([radius * a1.cos(), radius * a1.sin(), z]);
        pts.push([radius * a2.cos(), radius * a2.sin(), z]);
        lines.push_cell(&[b as i64, (b + 1) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r.lines = lines;
    r
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-15 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let d = dna_helix(1.0, 2.0, 3.0, 0.1, 16);
        assert!(d.points.len() > 100);
        assert!(d.polys.num_cells() > 50);
    }
}
