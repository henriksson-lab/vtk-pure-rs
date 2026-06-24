//! Egg timer (small hourglass with frame).
use crate::data::{CellArray, Points, PolyData};

pub fn egg_timer(height: f64, radius: f64, na: usize) -> PolyData {
    let na = na.max(10);
    let waist = radius * 0.1;
    let nf = 8;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Glass bulbs (two cones meeting at waist)
    for f in 0..=nf {
        let t = f as f64 / nf as f64;
        let z = height * t;
        let r = if t < 0.5 {
            radius - (radius - waist) * (t * 2.0)
        } else {
            waist + (radius - waist) * ((t - 0.5) * 2.0)
        };
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([r * a.cos(), r * a.sin(), z]);
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
    // Frame posts (4 vertical lines)
    let frame_r = radius * 1.1;
    for k in 0..4 {
        let a = std::f64::consts::PI / 2.0 * k as f64;
        let fb = pts.len();
        pts.push([frame_r * a.cos(), frame_r * a.sin(), 0.0]);
        let ft = pts.len();
        pts.push([frame_r * a.cos(), frame_r * a.sin(), height]);
        lines.push_cell(&[fb as i64, ft as i64]);
    }
    // Top and bottom rings
    for &z in &[0.0, height] {
        let rb = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([frame_r * a.cos(), frame_r * a.sin(), z]);
        }
        for j in 0..na {
            lines.push_cell(&[(rb + j) as i64, (rb + (j + 1) % na) as i64]);
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
    fn test_egg_timer() {
        let m = egg_timer(4.0, 1.0, 12);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 60);
        assert!(m.lines.num_cells() > 20);
    }
}
