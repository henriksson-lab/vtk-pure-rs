//! Viking longship (hull with dragon prow and oars).
use crate::data::{CellArray, Points, PolyData};

pub fn viking_ship(length: f64, beam: f64, n_sections: usize) -> PolyData {
    let ns = n_sections.max(8);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    // Hull (symmetric, sharp bow and stern)
    let nw = 4;
    for s in 0..=ns {
        let t = s as f64 / ns as f64;
        let x = length * (t - 0.5);
        let taper = (std::f64::consts::PI * t).sin().max(0.05);
        let rise = 0.5 * (1.0 - taper) * length * 0.1; // bow/stern rise
        for w in 0..=2 * nw {
            let u = w as f64 / nw as f64 - 1.0;
            let y = beam / 2.0 * taper * u;
            let z = -beam * 0.3 * taper * (1.0 - u.abs()) + rise;
            pts.push([x, y, z]);
        }
    }
    let stride = 2 * nw + 1;
    for s in 0..ns {
        for w in 0..2 * nw {
            let i0 = s * stride + w;
            let i1 = (s + 1) * stride + w;
            polys.push_cell(&[i0 as i64, i1 as i64, (i1 + 1) as i64, (i0 + 1) as i64]);
        }
    }
    // Mast
    let mast_b = pts.len();
    pts.push([0.0, 0.0, 0.0]);
    let mast_t = pts.len();
    pts.push([0.0, 0.0, length * 0.4]);
    lines.push_cell(&[mast_b as i64, mast_t as i64]);
    // Yard (crossbeam)
    let yard_l = pts.len();
    pts.push([0.0, -beam, length * 0.35]);
    let yard_r = pts.len();
    pts.push([0.0, beam, length * 0.35]);
    lines.push_cell(&[yard_l as i64, yard_r as i64]);
    // Oars (simplified as lines)
    let n_oars = 6;
    for oi in 0..n_oars {
        let x = -length * 0.3 + length * 0.6 * oi as f64 / (n_oars - 1) as f64;
        for &side in &[-1.0f64, 1.0] {
            let hull_y = side * beam * 0.45;
            let oar_tip_y = side * beam * 1.2;
            let o0 = pts.len();
            pts.push([x, hull_y, 0.0]);
            let o1 = pts.len();
            pts.push([x, oar_tip_y, -beam * 0.2]);
            lines.push_cell(&[o0 as i64, o1 as i64]);
        }
    }
    // Dragon prow
    let prow_b = pts.len();
    pts.push([length * 0.5, 0.0, length * 0.05]);
    let prow_t = pts.len();
    pts.push([length * 0.55, 0.0, length * 0.15]);
    let prow_head = pts.len();
    pts.push([length * 0.52, 0.0, length * 0.2]);
    lines.push_cell(&[prow_b as i64, prow_t as i64]);
    lines.push_cell(&[prow_t as i64, prow_head as i64]);
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
    fn test_viking() {
        let m = viking_ship(10.0, 2.0, 12);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 30);
        assert!(m.lines.num_cells() > 10);
    }
}
