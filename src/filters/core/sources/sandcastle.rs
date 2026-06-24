//! Sandcastle with towers and walls.
use crate::data::{CellArray, Points, PolyData};

pub fn sandcastle(base_size: f64, tower_height: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let _hs = base_size / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Base mound (truncated cone)
    let base_r = base_size * 0.7;
    let top_r = base_size * 0.5;
    let mound_h = tower_height * 0.3;
    for f in 0..=2 {
        let t = f as f64 / 2.0;
        let r = base_r * (1.0 - t) + top_r * t;
        let z = mound_h * t;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for f in 0..2 {
        let b0 = f * na;
        let b1 = (f + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (b0 + j) as i64,
                (b1 + j) as i64,
                (b1 + j1) as i64,
                (b0 + j1) as i64,
            ]);
        }
    }
    // Four corner towers
    let tower_r = base_size * 0.1;
    for &(tx, ty) in &[(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let cx = tx * top_r * 0.7;
        let cy = ty * top_r * 0.7;
        let tb = pts.len();
        for s in 0..=3 {
            let z = mound_h + tower_height * 0.7 * s as f64 / 3.0;
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([cx + tower_r * a.cos(), cy + tower_r * a.sin(), z]);
            }
        }
        for s in 0..3 {
            let b0 = tb + s * na;
            let b1 = tb + (s + 1) * na;
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
            }
        }
        // Cone top
        let cone_apex = pts.len();
        pts.push([cx, cy, mound_h + tower_height]);
        let top_ring = tb + 3 * na;
        for j in 0..na {
            polys.push_cell(&[
                (top_ring + j) as i64,
                cone_apex as i64,
                (top_ring + (j + 1) % na) as i64,
            ]);
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
    fn test_sandcastle() {
        let m = sandcastle(4.0, 3.0, 10);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 80);
    }
}
