//! Protein backbone (alpha helix + beta sheet simplified).
use crate::data::{CellArray, Points, PolyData};
pub fn alpha_helix(
    radius: f64,
    pitch: f64,
    residues: usize,
    tube_r: f64,
    resolution: usize,
) -> PolyData {
    let tres = resolution.max(4);
    let total = residues * 4;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..=total {
        let t = i as f64 / total as f64;
        let a = 2.0 * std::f64::consts::PI * 3.6 * t * residues as f64 / 3.6; // 3.6 residues per turn
        let cx = radius * a.cos();
        let cy = radius * a.sin();
        let cz = t * pitch * residues as f64;
        let dt = 0.001;
        let a2 = 2.0 * std::f64::consts::PI * 3.6 * (t + dt) * residues as f64 / 3.6;
        let tx = radius * a2.cos() - cx;
        let ty = radius * a2.sin() - cy;
        let tz = dt * pitch * residues as f64;
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
    for i in 0..total {
        let r0 = i * tres;
        let r1 = (i + 1) * tres;
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
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
pub fn beta_sheet(
    strands: usize,
    strand_length: usize,
    spacing: f64,
    pleat_amplitude: f64,
) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let residue_dist = 3.5; // angstroms between residues
    for si in 0..strands {
        let y = si as f64 * spacing;
        let direction = if si % 2 == 0 { 1.0 } else { -1.0 }; // antiparallel
        for ri in 0..strand_length {
            let x = if direction > 0.0 {
                ri as f64 * residue_dist
            } else {
                (strand_length - 1 - ri) as f64 * residue_dist
            };
            let z = pleat_amplitude * (ri % 2) as f64 * if si % 2 == 0 { 1.0 } else { -1.0 };
            pts.push([x, y, z]);
        }
    }
    // Connect strands into sheet
    for si in 0..strands - 1 {
        for ri in 0..strand_length - 1 {
            let i00 = si * strand_length + ri;
            let i10 = si * strand_length + ri + 1;
            let i01 = (si + 1) * strand_length + ri;
            let i11 = (si + 1) * strand_length + ri + 1;
            polys.push_cell(&[i00 as i64, i10 as i64, i11 as i64, i01 as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
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
    fn test_helix() {
        let h = alpha_helix(2.3, 5.4, 10, 0.3, 6);
        assert!(h.polys.num_cells() > 100);
    }
    #[test]
    fn test_sheet() {
        let s = beta_sheet(4, 8, 4.7, 0.5);
        assert!(s.polys.num_cells() > 15);
    }
}
