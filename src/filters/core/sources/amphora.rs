//! Ancient Greek amphora vase.
use crate::data::{CellArray, Points, PolyData};

pub fn amphora(height: f64, na: usize) -> PolyData {
    let na = na.max(12);
    // Profile: (radius_fraction, height_fraction)
    let profile = [
        (0.15, 0.0),
        (0.25, 0.03),
        (0.3, 0.08),
        (0.35, 0.15),
        (0.4, 0.25),
        (0.42, 0.35),
        (0.4, 0.45),
        (0.35, 0.55),
        (0.25, 0.65),
        (0.15, 0.72),
        (0.1, 0.78),
        (0.08, 0.82),
        (0.12, 0.85),
        (0.15, 0.88),
        (0.12, 0.92),
        (0.08, 0.95),
        (0.0, 1.0),
    ];
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for (_pi, &(rf, hf)) in profile.iter().enumerate() {
        let r = rf * height;
        let z = hf * height;
        if r < 1e-10 {
            pts.push([0.0, 0.0, z]); // apex
        } else {
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([r * a.cos(), r * a.sin(), z]);
            }
        }
    }
    // Connect rings
    let mut ring_start = 0usize;
    for pi in 0..profile.len() - 1 {
        let r0 = profile[pi].0;
        let r1 = profile[pi + 1].0;
        if r0 < 1e-10 && r1 >= 1e-10 {
            let apex = ring_start;
            let next = ring_start + 1;
            for j in 0..na {
                polys.push_cell(&[apex as i64, (next + j) as i64, (next + (j + 1) % na) as i64]);
            }
            ring_start += 1;
        } else if r0 >= 1e-10 && r1 < 1e-10 {
            let apex = ring_start + na;
            for j in 0..na {
                polys.push_cell(&[
                    (ring_start + j) as i64,
                    apex as i64,
                    (ring_start + (j + 1) % na) as i64,
                ]);
            }
            ring_start += na;
        } else if r0 >= 1e-10 && r1 >= 1e-10 {
            let b0 = ring_start;
            let b1 = ring_start + na;
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
            }
            ring_start += na;
        }
    }
    // Handles (simplified as arcs on each side)
    let mut lines = CellArray::new();
    for &side in &[0.0f64, std::f64::consts::PI] {
        let handle_na = 8;
        let hb = pts.len();
        for j in 0..=handle_na {
            let t = j as f64 / handle_na as f64;
            let angle = -0.3 + 0.6 * t;
            let hr = height * 0.15;
            let hx = (height * 0.42 + hr * angle.sin()) * side.cos();
            let hy = (height * 0.42 + hr * angle.sin()) * side.sin();
            let hz = height * 0.4 + hr * angle.cos();
            pts.push([hx, hy, hz]);
        }
        for j in 0..handle_na {
            lines.push_cell(&[(hb + j) as i64, (hb + j + 1) as i64]);
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_amphora() {
        let m = amphora(10.0, 16);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 80);
    }
}
