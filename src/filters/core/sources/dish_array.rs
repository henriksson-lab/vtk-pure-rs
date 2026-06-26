//! Array of satellite dishes on a grid.
use crate::data::{CellArray, Points, PolyData};

pub fn dish_array(dish_radius: f64, spacing: f64, nx: usize, ny: usize, na: usize) -> PolyData {
    let dish_radius = dish_radius.abs().max(1e-12);
    let na = na.max(8);
    let nr = 3;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    for ix in 0..nx {
        for iy in 0..ny {
            let cx = spacing * ix as f64;
            let cy = spacing * iy as f64;
            let a_coeff = 0.3 / (dish_radius * dish_radius);
            // Dish surface
            let dc = pts.len();
            pts.push([cx, cy, 0.0]);
            for r in 1..=nr {
                let rr = dish_radius * r as f64 / nr as f64;
                for j in 0..na {
                    let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                    pts.push([cx + rr * a.cos(), cy + rr * a.sin(), a_coeff * rr * rr]);
                }
            }
            for j in 0..na {
                polys.push_cell(&[
                    dc as i64,
                    (dc + 1 + j) as i64,
                    (dc + 1 + (j + 1) % na) as i64,
                ]);
            }
            for r in 0..(nr - 1) {
                let b0 = dc + 1 + r * na;
                let b1 = dc + 1 + (r + 1) * na;
                for j in 0..na {
                    let j1 = (j + 1) % na;
                    polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                    polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
                }
            }
            // Feed support (3 struts to focal point)
            let focal_z = 1.0 / (4.0 * a_coeff);
            let feed = pts.len();
            pts.push([cx, cy, focal_z]);
            let rim = dc + 1 + (nr - 1) * na;
            for k in 0..3 {
                let j = k * na / 3;
                lines.push_cell(&[(rim + j) as i64, feed as i64]);
            }
            // Pedestal
            let ped = pts.len();
            pts.push([cx, cy, -dish_radius * 0.5]);
            lines.push_cell(&[dc as i64, ped as i64]);
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
    fn test_dish_array() {
        let m = dish_array(1.0, 4.0, 2, 2, 10);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 50);
    }
}
