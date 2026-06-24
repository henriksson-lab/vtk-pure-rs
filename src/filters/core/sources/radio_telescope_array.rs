//! Array of radio telescope dishes in a Y-shaped configuration.
use crate::data::{CellArray, Points, PolyData};

pub fn radio_telescope_array(
    dish_radius: f64,
    arm_length: f64,
    n_per_arm: usize,
    na: usize,
) -> PolyData {
    let npa = n_per_arm.max(2);
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Y-shaped track lines
    let center = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let arms: Vec<[f64; 2]> = (0..3)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / 3.0 - std::f64::consts::PI / 2.0;
            [a.cos(), a.sin()]
        })
        .collect();
    for arm in &arms {
        let tip = pts.len();
        pts.push([arm[0] * arm_length, arm[1] * arm_length, 0.0]);
        lines.push_cell(&[center as i64, tip as i64]);
    }
    // Place dishes along each arm
    for arm in &arms {
        for d in 0..npa {
            let t = (d + 1) as f64 / (npa + 1) as f64;
            let cx = arm[0] * arm_length * t;
            let cy = arm[1] * arm_length * t;
            // Dish (parabolic bowl)
            let dc = pts.len();
            pts.push([cx, cy, dish_radius * 0.5]); // top center
            let np = 3;
            for p in 1..=np {
                let phi = std::f64::consts::PI / 3.0 * p as f64 / np as f64;
                let r = dish_radius * phi.sin();
                let z = dish_radius * 0.5 - dish_radius * 0.3 * phi.sin().powi(2);
                for j in 0..na {
                    let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                    pts.push([cx + r * a.cos(), cy + r * a.sin(), z]);
                }
            }
            // Top cap
            for j in 0..na {
                polys.push_cell(&[
                    dc as i64,
                    (dc + 1 + j) as i64,
                    (dc + 1 + (j + 1) % na) as i64,
                ]);
            }
            for p in 0..(np - 1) {
                let b0 = dc + 1 + p * na;
                let b1 = dc + 1 + (p + 1) * na;
                for j in 0..na {
                    let j1 = (j + 1) % na;
                    polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                    polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
                }
            }
            // Pedestal
            let ped0 = pts.len();
            pts.push([cx, cy, 0.0]);
            let ped1 = pts.len();
            pts.push([cx, cy, dish_radius * 0.2]);
            lines.push_cell(&[ped0 as i64, ped1 as i64]);
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
    fn test_array() {
        let m = radio_telescope_array(1.0, 20.0, 3, 8);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 50);
    }
}
