//! Fresnel lens (concentric stepped rings).
use crate::data::{CellArray, Points, PolyData};

pub fn fresnel_lens(radius: f64, n_rings: usize, na: usize) -> PolyData {
    let nr = n_rings.max(3);
    let na = na.max(12);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Center disk
    let cc = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let first_r = radius / nr as f64;
    let cb = pts.len();
    for j in 0..na {
        let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
        pts.push([first_r * a.cos(), first_r * a.sin(), 0.0]);
    }
    for j in 0..na {
        polys.push_cell(&[cc as i64, (cb + j) as i64, (cb + (j + 1) % na) as i64]);
    }
    // Concentric stepped rings
    for r in 1..nr {
        let ri = radius * r as f64 / nr as f64;
        let ro = radius * (r + 1) as f64 / nr as f64;
        let step_z = 0.01 * (nr - r) as f64; // steps get flatter outward
        let ib = pts.len();
        for j in 0..na {
            let a = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([ri * a.cos(), ri * a.sin(), step_z]);
            pts.push([ro * a.cos(), ro * a.sin(), step_z]);
        }
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (ib + j * 2) as i64,
                (ib + j * 2 + 1) as i64,
                (ib + j1 * 2 + 1) as i64,
                (ib + j1 * 2) as i64,
            ]);
        }
        // Vertical riser connecting to previous ring
        let prev_ring_outer = if r == 1 { cb } else { ib - na * 2 };
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[
                (ib + j * 2) as i64,
                (ib + j1 * 2) as i64,
                if r == 1 {
                    (cb + j1) as i64
                } else {
                    (prev_ring_outer + j1 * 2 + 1) as i64
                },
                if r == 1 {
                    (cb + j) as i64
                } else {
                    (prev_ring_outer + j * 2 + 1) as i64
                },
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
    fn test_fresnel() {
        let m = fresnel_lens(3.0, 5, 16);
        assert!(m.points.len() > 100);
        assert!(m.polys.num_cells() > 50);
    }
}
