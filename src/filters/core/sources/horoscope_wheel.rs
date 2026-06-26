//! Horoscope / zodiac wheel with 12 segments.
use crate::data::{CellArray, Points, PolyData};

pub fn horoscope_wheel(outer_radius: f64, inner_radius: f64) -> PolyData {
    let outer_radius = outer_radius.abs();
    let inner_radius = inner_radius.abs().min(outer_radius);
    let n_segments = 12;
    let na_per_seg = 4;
    let na = n_segments * na_per_seg;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Inner and outer rings
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([outer_radius * a.cos(), outer_radius * a.sin(), 0.0]);
        pts.push([inner_radius * a.cos(), inner_radius * a.sin(), 0.0]);
    }
    // Ring quads
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (j * 2) as i64,
            (j1 * 2) as i64,
            (j1 * 2 + 1) as i64,
            (j * 2 + 1) as i64,
        ]);
    }
    // Segment dividers
    for s in 0..n_segments {
        let j = s * na_per_seg;
        lines.push_cell(&[(j * 2) as i64, (j * 2 + 1) as i64]);
    }
    // Center point
    let center = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    // Inner ring fill
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[center as i64, (j * 2 + 1) as i64, (j1 * 2 + 1) as i64]);
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
    fn test_wheel() {
        let m = horoscope_wheel(5.0, 3.0);
        assert!(m.points.len() > 90);
        assert!(m.polys.num_cells() > 48);
    }
}
