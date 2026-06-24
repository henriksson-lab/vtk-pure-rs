//! Japanese katana sword.
use crate::data::{CellArray, Points, PolyData};

pub fn katana(blade_length: f64, handle_length: f64, n_profile: usize) -> PolyData {
    let np = n_profile.max(20);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let total = blade_length + handle_length;
    // Blade profile (curved, tapered)
    for i in 0..=np {
        let t = i as f64 / np as f64;
        let x = total * t - handle_length;
        let in_blade = x > 0.0;
        let width = if in_blade {
            let bt = x / blade_length;
            0.03 * blade_length * (1.0 - bt * 0.7) // taper toward tip
        } else {
            0.025 * blade_length // handle width
        };
        let curve = if in_blade {
            0.05 * blade_length * (std::f64::consts::PI * x / blade_length).sin()
        } else {
            0.0
        };
        pts.push([x, -width / 2.0, curve]);
        pts.push([x, width / 2.0, curve]);
    }
    for i in 0..np {
        let b = i * 2;
        polys.push_cell(&[b as i64, (b + 2) as i64, (b + 3) as i64, (b + 1) as i64]);
    }
    // Tsuba (guard) - small disk at blade/handle junction
    let mut lines = CellArray::new();
    let guard_r = 0.06 * blade_length;
    let na = 12;
    let gb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([0.0, guard_r * a.cos(), guard_r * a.sin()]);
    }
    for j in 0..na {
        lines.push_cell(&[(gb + j) as i64, (gb + (j + 1) % na) as i64]);
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
    fn test_katana() {
        let m = katana(0.7, 0.25, 30);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() >= 30);
    }
}
