//! Wine barrel (bulging cylinder with hoops).
use crate::data::{CellArray, Points, PolyData};

pub fn wine_barrel(
    height: f64,
    max_radius: f64,
    end_radius: f64,
    na: usize,
    nf: usize,
) -> PolyData {
    let na = na.max(12);
    let nf = nf.max(6);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    for f in 0..=nf {
        let t = f as f64 / nf as f64;
        let z = height * t;
        let r = end_radius + (max_radius - end_radius) * (std::f64::consts::PI * t).sin();
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
    // Hoops (rings at 1/4 and 3/4 height)
    for &hz in &[height * 0.2, height * 0.5, height * 0.8] {
        let f_idx = (hz / height * nf as f64).round() as usize;
        let ring_base = f_idx * na;
        for j in 0..na {
            lines.push_cell(&[(ring_base + j) as i64, (ring_base + (j + 1) % na) as i64]);
        }
    }
    // End caps
    let top_c = pts.len();
    pts.push([0.0, 0.0, height]);
    for j in 0..na {
        polys.push_cell(&[
            (nf * na + j) as i64,
            top_c as i64,
            (nf * na + (j + 1) % na) as i64,
        ]);
    }
    let bot_c = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    for j in 0..na {
        polys.push_cell(&[j as i64, ((j + 1) % na) as i64, bot_c as i64]);
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
    fn test_barrel() {
        let m = wine_barrel(3.0, 1.0, 0.8, 16, 8);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 100);
        assert!(m.lines.num_cells() > 30);
    }
}
