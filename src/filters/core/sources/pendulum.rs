//! Simple pendulum (bob on string from pivot).
use crate::data::{CellArray, Points, PolyData};

pub fn pendulum(length: f64, bob_radius: f64, angle_deg: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let angle = angle_deg * std::f64::consts::PI / 180.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Pivot point
    let pivot = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    // Bob position (swung to angle)
    let bob_x = length * angle.sin();
    let bob_z = -length * angle.cos();
    // String
    let bob_center = pts.len();
    pts.push([bob_x, 0.0, bob_z]);
    lines.push_cell(&[pivot as i64, bob_center as i64]);
    // Bob sphere
    let np = 3;
    let top = pts.len();
    pts.push([bob_x, 0.0, bob_z + bob_radius]);
    for p in 1..=np {
        let phi = std::f64::consts::PI * p as f64 / np as f64;
        for j in 0..na {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                bob_x + bob_radius * phi.sin() * theta.cos(),
                bob_radius * phi.sin() * theta.sin(),
                bob_z + bob_radius * phi.cos(),
            ]);
        }
    }
    for j in 0..na {
        polys.push_cell(&[
            top as i64,
            (top + 1 + j) as i64,
            (top + 1 + (j + 1) % na) as i64,
        ]);
    }
    for p in 0..(np - 1) {
        let b0 = top + 1 + p * na;
        let b1 = top + 1 + (p + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Pivot bracket (small triangle)
    let pb = pts.len();
    pts.push([-bob_radius, 0.0, 0.0]);
    pts.push([bob_radius, 0.0, 0.0]);
    pts.push([0.0, 0.0, bob_radius]);
    polys.push_cell(&[pb as i64, (pb + 1) as i64, (pb + 2) as i64]);
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
    fn test_pendulum() {
        let m = pendulum(5.0, 0.3, 30.0, 8);
        assert!(m.points.len() > 20);
        assert!(m.polys.num_cells() > 10);
        assert_eq!(m.lines.num_cells(), 1);
    }
}
