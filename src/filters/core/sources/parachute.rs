//! Parachute canopy with suspension lines.
use crate::data::{CellArray, Points, PolyData};

pub fn parachute(canopy_radius: f64, line_length: f64, n_gores: usize, n_rings: usize) -> PolyData {
    let ng = n_gores.max(6);
    let nr = n_rings.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Canopy (hemispherical cap)
    let apex = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for r in 1..=nr {
        let phi = std::f64::consts::PI / 3.0 * r as f64 / nr as f64; // 60 degree cap
        let ring_r = canopy_radius * phi.sin();
        let z = -canopy_radius * (1.0 - phi.cos());
        for g in 0..ng {
            let theta = 2.0 * std::f64::consts::PI * g as f64 / ng as f64;
            pts.push([ring_r * theta.cos(), ring_r * theta.sin(), z]);
        }
    }
    // Top cap
    for g in 0..ng {
        polys.push_cell(&[
            apex as i64,
            (apex + 1 + g) as i64,
            (apex + 1 + (g + 1) % ng) as i64,
        ]);
    }
    // Rings
    for r in 0..(nr - 1) {
        let b0 = apex + 1 + r * ng;
        let b1 = apex + 1 + (r + 1) * ng;
        for g in 0..ng {
            let g1 = (g + 1) % ng;
            polys.push_cell(&[(b0 + g) as i64, (b1 + g) as i64, (b1 + g1) as i64]);
            polys.push_cell(&[(b0 + g) as i64, (b1 + g1) as i64, (b0 + g1) as i64]);
        }
    }
    // Payload point
    let payload = pts.len();
    pts.push([0.0, 0.0, -line_length]);
    // Suspension lines from canopy skirt to payload
    let skirt_base = apex + 1 + (nr - 1) * ng;
    for g in 0..ng {
        lines.push_cell(&[(skirt_base + g) as i64, payload as i64]);
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
    fn test_parachute() {
        let m = parachute(5.0, 10.0, 12, 4);
        assert!(m.points.len() > 40);
        assert!(m.polys.num_cells() > 30);
        assert_eq!(m.lines.num_cells(), 12);
    }
}
