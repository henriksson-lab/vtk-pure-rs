//! Gatling gun with rotating barrel cluster.
use crate::data::{CellArray, Points, PolyData};

pub fn gatling_gun(
    barrel_length: f64,
    barrel_radius: f64,
    n_barrels: usize,
    na: usize,
) -> PolyData {
    let nb = n_barrels.max(4);
    let na = na.max(6);
    let cluster_r = barrel_radius * 3.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Individual barrels arranged in circle
    for b in 0..nb {
        let angle = 2.0 * std::f64::consts::PI * b as f64 / nb as f64;
        let cx = cluster_r * angle.cos();
        let cy = cluster_r * angle.sin();
        let base = pts.len();
        for s in 0..=3 {
            let z = barrel_length * s as f64 / 3.0;
            for j in 0..na {
                let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([
                    cx + barrel_radius * a.cos(),
                    cy + barrel_radius * a.sin(),
                    z,
                ]);
            }
        }
        for s in 0..3 {
            let b0 = base + s * na;
            let b1 = base + (s + 1) * na;
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
                polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
            }
        }
    }
    // Housing cylinder around breech end
    let housing_r = cluster_r + barrel_radius * 2.0;
    let hb = pts.len();
    for s in 0..=1 {
        let z = -barrel_length * 0.2 + barrel_length * 0.3 * s as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([housing_r * a.cos(), housing_r * a.sin(), z]);
        }
    }
    for j in 0..na {
        let j1 = (j + 1) % na;
        polys.push_cell(&[
            (hb + j) as i64,
            (hb + na + j) as i64,
            (hb + na + j1) as i64,
            (hb + j1) as i64,
        ]);
    }
    // Crank handle
    let mut lines = CellArray::new();
    let ch0 = pts.len();
    pts.push([0.0, 0.0, -barrel_length * 0.2]);
    let ch1 = pts.len();
    pts.push([housing_r * 1.3, 0.0, -barrel_length * 0.2]);
    let ch2 = pts.len();
    pts.push([housing_r * 1.3, 0.0, -barrel_length * 0.35]);
    lines.push_cell(&[ch0 as i64, ch1 as i64]);
    lines.push_cell(&[ch1 as i64, ch2 as i64]);
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
    fn test_gatling() {
        let m = gatling_gun(3.0, 0.1, 6, 8);
        assert!(m.points.len() > 150);
        assert!(m.polys.num_cells() > 100);
    }
}
