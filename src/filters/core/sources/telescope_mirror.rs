//! Parabolic telescope mirror surface.
use crate::data::{CellArray, Points, PolyData};

pub fn telescope_mirror(
    diameter: f64,
    focal_length: f64,
    n_radial: usize,
    n_angular: usize,
) -> PolyData {
    let nr = n_radial.max(3);
    let na = n_angular.max(6);
    let r_max = diameter / 2.0;
    let mut pts = Points::<f64>::new();
    // Center point
    pts.push([0.0, 0.0, 0.0]);
    for i in 1..=nr {
        let r = r_max * i as f64 / nr as f64;
        for j in 0..na {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            let x = r * theta.cos();
            let y = r * theta.sin();
            let z = r * r / (4.0 * focal_length); // parabola: z = r^2/(4f)
            pts.push([x, y, z]);
        }
    }
    let mut polys = CellArray::new();
    // Center fan
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[0, (1 + j) as i64, (1 + j1) as i64]);
    }
    // Rings
    for i in 1..nr {
        let base0 = 1 + (i - 1) * na;
        let base1 = 1 + i * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(base0 + j) as i64, (base1 + j) as i64, (base1 + j1) as i64]);
            polys.push_cell(&[(base0 + j) as i64, (base1 + j1) as i64, (base0 + j1) as i64]);
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
    fn test_mirror() {
        let m = telescope_mirror(1.0, 2.0, 5, 12);
        assert!(m.points.len() > 10);
        assert!(m.polys.num_cells() > 0);
        // Center should be at origin
        let c = m.points.get(0);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }
}
