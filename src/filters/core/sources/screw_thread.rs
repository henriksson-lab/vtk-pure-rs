//! Screw thread geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a screw thread (helical groove on a cylinder).
pub fn screw_thread(
    shaft_radius: f64,
    thread_height: f64,
    pitch: f64,
    length: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let turns = length / pitch;
    let total_steps = (res as f64 * turns).ceil() as usize;
    let total_steps = total_steps.max(4);

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..=total_steps {
        let t = i as f64 / total_steps as f64;
        let angle = 2.0 * std::f64::consts::PI * turns * t;
        let z = t * length;
        let c = angle.cos();
        let s = angle.sin();

        // Inner point (shaft surface)
        pts.push([shaft_radius * c, shaft_radius * s, z]);
        // Outer point (thread tip)
        let r = shaft_radius + thread_height;
        pts.push([r * c, r * s, z + pitch * 0.25]);
    }

    for i in 0..total_steps {
        let i0 = i * 2;
        let i1 = i * 2 + 1;
        let j0 = (i + 1) * 2;
        let j1 = (i + 1) * 2 + 1;
        polys.push_cell(&[i0 as i64, j0 as i64, j1 as i64, i1 as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_screw() {
        let s = screw_thread(1.0, 0.2, 0.5, 3.0, 16);
        assert!(s.points.len() > 20);
        assert!(s.polys.num_cells() > 10);
    }
}
