//! Detailed DNA double helix with nucleotide pairs.
use crate::data::{CellArray, Points, PolyData};
pub fn dna_detailed(
    radius: f64,
    pitch: f64,
    turns: f64,
    base_pair_count: usize,
    tube_r: f64,
    tube_res: usize,
) -> PolyData {
    let tres = tube_res.max(4);
    let nbp = base_pair_count.max(1);
    let total_h = turns * pitch;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Two backbone helices
    let helix_steps = nbp * 4;
    for strand in 0..2 {
        let phase = strand as f64 * std::f64::consts::PI;
        for i in 0..=helix_steps {
            let t = i as f64 / helix_steps as f64;
            let a = 2.0 * std::f64::consts::PI * turns * t + phase;
            let cx = radius * a.cos();
            let cy = radius * a.sin();
            let cz = t * total_h;
            // Tangent for tube cross-section
            let dt = 0.001;
            let a2 = 2.0 * std::f64::consts::PI * turns * (t + dt) + phase;
            let tx = radius * a2.cos() - cx;
            let ty = radius * a2.sin() - cy;
            let tz = dt * total_h;
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
                    cx + tube_r * (phi.cos() * n1[0] + phi.sin() * n2[0]),
                    cy + tube_r * (phi.cos() * n1[1] + phi.sin() * n2[1]),
                    cz + tube_r * (phi.cos() * n1[2] + phi.sin() * n2[2]),
                ]);
            }
        }
        let base = strand * (helix_steps + 1) * tres;
        for i in 0..helix_steps {
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
    // Base pairs (rungs connecting the two strands)
    for bp in 0..nbp {
        let t = (bp as f64 + 0.5) / nbp as f64;
        let a1 = 2.0 * std::f64::consts::PI * turns * t;
        let a2 = a1 + std::f64::consts::PI;
        let z = t * total_h;
        let b = pts.len();
        pts.push([radius * 0.9 * a1.cos(), radius * 0.9 * a1.sin(), z]);
        pts.push([radius * 0.9 * a2.cos(), radius * 0.9 * a2.sin(), z]);
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
        let d = dna_detailed(1.0, 3.4, 2.0, 20, 0.1, 4);
        assert!(d.polys.num_cells() > 50);
        assert!(d.lines.num_cells() >= 20);
    }
}
