//! Chess pawn piece (lathe profile).
use crate::data::{CellArray, Points, PolyData};

pub fn chess_pawn(height: f64, n_angular: usize, n_profile: usize) -> PolyData {
    let na = n_angular.max(8);
    let _np = n_profile.max(10);
    // Profile points: (radius, height_fraction)
    let profile = vec![
        (0.4, 0.0),
        (0.45, 0.02),
        (0.42, 0.05),
        (0.2, 0.1),
        (0.18, 0.15),
        (0.15, 0.4),
        (0.18, 0.55),
        (0.22, 0.6),
        (0.18, 0.65),
        (0.15, 0.7),
        (0.2, 0.8),
        (0.18, 0.85),
        (0.12, 0.9),
        (0.0, 1.0),
    ];
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for (_pi, &(r, h)) in profile.iter().enumerate() {
        let z = h * height;
        let radius = r * height;
        if radius < 1e-10 {
            // Apex point
            pts.push([0.0, 0.0, z]);
        } else {
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([radius * a.cos(), radius * a.sin(), z]);
            }
        }
    }
    // Connect rings
    let mut ring_start = 0usize;
    for pi in 0..profile.len() - 1 {
        let r0 = profile[pi].0;
        let r1 = profile[pi + 1].0;
        if r0 < 1e-10 && r1 < 1e-10 {
            continue;
        }
        if r0 < 1e-10 {
            // Fan from apex to next ring
            let apex = ring_start;
            let next_base = ring_start + 1;
            for j in 0..na {
                polys.push_cell(&[
                    apex as i64,
                    (next_base + j) as i64,
                    (next_base + (j + 1) % na) as i64,
                ]);
            }
            ring_start += 1;
        } else if r1 < 1e-10 {
            // Fan from ring to apex
            let apex = ring_start + na;
            for j in 0..na {
                polys.push_cell(&[
                    (ring_start + j) as i64,
                    apex as i64,
                    (ring_start + (j + 1) % na) as i64,
                ]);
            }
            ring_start += na;
        } else {
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
    let mut m = PolyData::new();
    m.points = pts;
    m.polys = polys;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pawn() {
        let m = chess_pawn(3.0, 12, 14);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 30);
    }
}
