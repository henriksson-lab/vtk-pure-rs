//! Saguaro cactus with trunk and arms.
use crate::data::{CellArray, Points, PolyData};

pub fn cactus(trunk_height: f64, trunk_radius: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Main trunk
    let nf = 6;
    for f in 0..=nf {
        let t = f as f64 / nf as f64;
        let z = trunk_height * t;
        let r = trunk_radius * (1.0 - 0.1 * t);
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            // Add fluting (ribs)
            let rib = 1.0 + 0.15 * (a * 8.0).cos().max(0.0);
            pts.push([r * rib * a.cos(), r * rib * a.sin(), z]);
        }
    }
    for f in 0..nf {
        let b0 = f * na;
        let b1 = (f + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Top dome
    let top_ring = nf * na;
    let apex = pts.len();
    pts.push([0.0, 0.0, trunk_height + trunk_radius * 0.5]);
    for j in 0..na {
        polys.push_cell(&[
            (top_ring + j) as i64,
            apex as i64,
            (top_ring + (j + 1) % na) as i64,
        ]);
    }
    // Right arm
    let arm_z = trunk_height * 0.55;
    let arm_r = trunk_radius * 0.7;
    let arm_len = trunk_height * 0.4;
    let arm_base = pts.len();
    let arm_segs = 4;
    for s in 0..=arm_segs {
        let t = s as f64 / arm_segs as f64;
        let x = trunk_radius + arm_len * t.min(0.5) * 2.0;
        let z = arm_z + arm_len * (t - 0.5).max(0.0) * 2.0;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x + arm_r * a.cos() * 0.3, arm_r * a.sin(), z]);
        }
    }
    for s in 0..arm_segs {
        let b0 = arm_base + s * na;
        let b1 = arm_base + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
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
    fn test_cactus() {
        let m = cactus(5.0, 0.5, 10);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 40);
    }
}
