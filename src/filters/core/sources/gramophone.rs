//! Gramophone with horn and turntable.
use crate::data::{CellArray, Points, PolyData};

pub fn gramophone(horn_length: f64, horn_radius: f64, na: usize) -> PolyData {
    let na = na.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Turntable disk
    let tc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let disk_r = horn_length * 0.3;
    let tb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([disk_r * a.cos(), disk_r * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[tc as i64, (tb + j) as i64, (tb + (j + 1) % na) as i64]);
    }
    // Tone arm
    let arm0 = pts.len();
    pts.push([0.0, 0.0, 0.02]);
    let arm1 = pts.len();
    pts.push([disk_r * 0.7, 0.0, horn_length * 0.2]);
    lines.push_cell(&[arm0 as i64, arm1 as i64]);
    // Horn (expanding cone)
    let horn_segs = 8;
    for s in 0..=horn_segs {
        let t = s as f64 / horn_segs as f64;
        let r = 0.05 + horn_radius * t.powi(2); // exponential expansion
        let z = horn_length * 0.2 + horn_length * 0.8 * t;
        let tilt = 0.3f64; // tilt angle
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                disk_r * 0.7 + z * tilt.sin() + r * a.cos() * (1.0 - tilt.sin()),
                r * a.sin(),
                z * tilt.cos(),
            ]);
        }
    }
    let horn_base = arm1 + 1;
    for s in 0..horn_segs {
        let b0 = horn_base + s * na;
        let b1 = horn_base + (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Cabinet box (simplified)
    let hw = disk_r * 1.2;
    let hd = disk_r * 0.8;
    let ch = disk_r * 0.5;
    let cb = pts.len();
    pts.push([-hw, -hd, -ch]);
    pts.push([hw, -hd, -ch]);
    pts.push([hw, hd, -ch]);
    pts.push([-hw, hd, -ch]);
    pts.push([-hw, -hd, 0.0]);
    pts.push([hw, -hd, 0.0]);
    pts.push([hw, hd, 0.0]);
    pts.push([-hw, hd, 0.0]);
    polys.push_cell(&[cb as i64, (cb + 1) as i64, (cb + 5) as i64, (cb + 4) as i64]);
    polys.push_cell(&[
        (cb + 1) as i64,
        (cb + 2) as i64,
        (cb + 6) as i64,
        (cb + 5) as i64,
    ]);
    polys.push_cell(&[
        (cb + 2) as i64,
        (cb + 3) as i64,
        (cb + 7) as i64,
        (cb + 6) as i64,
    ]);
    polys.push_cell(&[(cb + 3) as i64, cb as i64, (cb + 4) as i64, (cb + 7) as i64]);
    polys.push_cell(&[cb as i64, (cb + 3) as i64, (cb + 2) as i64, (cb + 1) as i64]); // bottom
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
    fn test_gramophone() {
        let m = gramophone(3.0, 1.5, 12);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 50);
    }
}
