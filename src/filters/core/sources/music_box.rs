//! Music box mechanism (cylinder with pins and comb).
use crate::data::{CellArray, Points, PolyData};

pub fn music_box(
    cylinder_radius: f64,
    cylinder_length: f64,
    n_pins: usize,
    n_teeth: usize,
    na: usize,
) -> PolyData {
    let na = na.max(12);
    let np = n_pins.max(10);
    let nt = n_teeth.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Cylinder
    let cyl_segs = 4;
    for s in 0..=cyl_segs {
        let x = cylinder_length * s as f64 / cyl_segs as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x, cylinder_radius * a.cos(), cylinder_radius * a.sin()]);
        }
    }
    for s in 0..cyl_segs {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Pins on cylinder surface
    let pin_len = cylinder_radius * 0.15;
    let mut rng = 42u64;
    for _ in 0..np {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x = cylinder_length * ((rng >> 33) as f64 / u32::MAX as f64);
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = 2.0 * std::f64::consts::PI * ((rng >> 33) as f64 / u32::MAX as f64);
        let b = pts.len();
        pts.push([x, cylinder_radius * a.cos(), cylinder_radius * a.sin()]);
        let t = pts.len();
        pts.push([
            x,
            (cylinder_radius + pin_len) * a.cos(),
            (cylinder_radius + pin_len) * a.sin(),
        ]);
        lines.push_cell(&[b as i64, t as i64]);
    }
    // Comb (row of teeth)
    let comb_y = cylinder_radius + pin_len * 2.0;
    let tooth_len = cylinder_radius * 0.5;
    for i in 0..nt {
        let x = cylinder_length * (i as f64 + 0.5) / nt as f64;
        let tb = pts.len();
        pts.push([x, comb_y, 0.0]);
        let tt = pts.len();
        pts.push([x, comb_y, -tooth_len]);
        lines.push_cell(&[tb as i64, tt as i64]);
    }
    // Comb base bar
    let cb0 = pts.len();
    pts.push([0.0, comb_y, 0.0]);
    let cb1 = pts.len();
    pts.push([cylinder_length, comb_y, 0.0]);
    lines.push_cell(&[cb0 as i64, cb1 as i64]);
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
    fn test_music_box() {
        let m = music_box(0.5, 3.0, 20, 12, 12);
        assert!(m.points.len() > 80);
        assert!(m.polys.num_cells() > 30);
        assert!(m.lines.num_cells() > 20);
    }
}
