//! Spiral surface geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Generate a spiral surface (like a nautilus shell cross-section extruded).
pub fn spiral_surface(
    turns: f64,
    inner_radius: f64,
    growth_rate: f64,
    height: f64,
    resolution: usize,
    height_segments: usize,
) -> PolyData {
    let n_angle = resolution.max(8);
    let n_h = height_segments.max(1);

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for ih in 0..=n_h {
        let z = height * ih as f64 / n_h as f64;
        for ia in 0..=n_angle {
            let t = turns * 2.0 * std::f64::consts::PI * ia as f64 / n_angle as f64;
            let r = inner_radius * (growth_rate * t).exp();
            points.push([r * t.cos(), r * t.sin(), z]);
        }
    }

    let row = n_angle + 1;
    for ih in 0..n_h {
        for ia in 0..n_angle {
            let p0 = (ih * row + ia) as i64;
            polys.push_cell(&[p0, p0 + 1, p0 + row as i64 + 1]);
            polys.push_cell(&[p0, p0 + row as i64 + 1, p0 + row as i64]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Generate a logarithmic spiral curve.
pub fn log_spiral_curve(
    turns: f64,
    inner_radius: f64,
    growth_rate: f64,
    resolution: usize,
) -> PolyData {
    let n = resolution.max(16);
    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();

    let ids: Vec<i64> = (0..=n)
        .map(|i| {
            let t = turns * 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            let r = inner_radius * (growth_rate * t).exp();
            points.push([r * t.cos(), r * t.sin(), 0.0]);
            i as i64
        })
        .collect();

    lines.push_cell(&ids);
    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spiral_surf() {
        let s = spiral_surface(2.0, 0.1, 0.15, 1.0, 32, 4);
        assert!(s.points.len() > 100);
        assert!(s.polys.num_cells() > 100);
    }
    #[test]
    fn log_spiral() {
        let s = log_spiral_curve(3.0, 0.1, 0.1, 64);
        assert_eq!(s.lines.num_cells(), 1);
        assert_eq!(s.points.len(), 65);
    }
}
