//! Dumbbell (two spheres connected by a bar).
use crate::data::{CellArray, Points, PolyData};

pub fn dumbbell(bar_length: f64, weight_radius: f64, bar_radius: f64, na: usize) -> PolyData {
    let na = na.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Bar (cylinder along X axis)
    let bar_segs = 4;
    for s in 0..=bar_segs {
        let x = -bar_length / 2.0 + bar_length * s as f64 / bar_segs as f64;
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([x, bar_radius * a.cos(), bar_radius * a.sin()]);
        }
    }
    for s in 0..bar_segs {
        let b0 = s * na;
        let b1 = (s + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
        }
    }
    // Two weight spheres
    for &cx in &[-bar_length / 2.0, bar_length / 2.0] {
        let np = 4;
        let sphere_base = pts.len();
        pts.push([cx, 0.0, weight_radius]); // top
        for p in 1..np {
            let phi = std::f64::consts::PI * p as f64 / np as f64;
            for j in 0..na {
                let theta = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
                pts.push([
                    cx + weight_radius * phi.sin() * theta.cos(),
                    weight_radius * phi.sin() * theta.sin(),
                    weight_radius * phi.cos(),
                ]);
            }
        }
        let south = pts.len();
        pts.push([cx, 0.0, -weight_radius]);
        // Top cap
        for j in 0..na {
            polys.push_cell(&[
                sphere_base as i64,
                (sphere_base + 1 + j) as i64,
                (sphere_base + 1 + (j + 1) % na) as i64,
            ]);
        }
        for p in 0..(np - 2) {
            let r0 = sphere_base + 1 + p * na;
            let r1 = sphere_base + 1 + (p + 1) * na;
            for j in 0..na {
                let j1 = (j + 1) % na;
                polys.push_cell(&[(r0 + j) as i64, (r1 + j) as i64, (r1 + j1) as i64]);
                polys.push_cell(&[(r0 + j) as i64, (r1 + j1) as i64, (r0 + j1) as i64]);
            }
        }
        let last_ring = sphere_base + 1 + (np - 2) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (last_ring + j) as i64,
                south as i64,
                (last_ring + j1) as i64,
            ]);
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
    fn test_dumbbell() {
        let m = dumbbell(4.0, 1.0, 0.2, 8);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 30);
    }
}
