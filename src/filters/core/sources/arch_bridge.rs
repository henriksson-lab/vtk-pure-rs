//! Stone arch bridge.
use crate::data::{CellArray, Points, PolyData};

pub fn arch_bridge(span: f64, rise: f64, width: f64, n_arch: usize, n_width: usize) -> PolyData {
    let na = n_arch.max(6);
    let nw = n_width.max(2);
    let hw = width / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Arch surface (intrados)
    for w in 0..=nw {
        let y = -hw + width * w as f64 / nw as f64;
        for i in 0..=na {
            let t = i as f64 / na as f64;
            let angle = std::f64::consts::PI * t;
            let x = -span / 2.0 + span * t;
            let z = rise * angle.sin();
            pts.push([x, y, z]);
        }
    }
    for w in 0..nw {
        for i in 0..na {
            let b0 = w * (na + 1) + i;
            let b1 = (w + 1) * (na + 1) + i;
            polys.push_cell(&[b0 as i64, b1 as i64, (b1 + 1) as i64]);
            polys.push_cell(&[b0 as i64, (b1 + 1) as i64, (b0 + 1) as i64]);
        }
    }
    // Deck on top
    let deck_z = rise + 0.5;
    let db = pts.len();
    for w in 0..=nw {
        let y = -hw + width * w as f64 / nw as f64;
        pts.push([-span / 2.0, y, deck_z]);
        pts.push([span / 2.0, y, deck_z]);
    }
    for w in 0..nw {
        let i0 = db + w * 2;
        polys.push_cell(&[i0 as i64, (i0 + 2) as i64, (i0 + 3) as i64, (i0 + 1) as i64]);
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
    fn test_arch_bridge() {
        let m = arch_bridge(20.0, 5.0, 4.0, 12, 3);
        assert!(m.points.len() > 30);
        assert!(m.polys.num_cells() > 20);
    }
}
