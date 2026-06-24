//! Simplified ship hull shape.
use crate::data::{CellArray, Points, PolyData};

pub fn ship_hull(
    length: f64,
    beam: f64,
    draft: f64,
    n_sections: usize,
    n_waterline: usize,
) -> PolyData {
    let ns = n_sections.max(6);
    let nw = n_waterline.max(4);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for s in 0..=ns {
        let t = s as f64 / ns as f64;
        let x = length * (t - 0.5);
        // Hull cross-section: width tapers at bow and stern
        let taper = (std::f64::consts::PI * t).sin(); // 0 at ends, 1 at middle
        let half_beam = beam / 2.0 * taper.max(0.05);
        for w in 0..=nw {
            let u = w as f64 / nw as f64;
            let angle = std::f64::consts::PI / 2.0 * u; // 0=waterline, pi/2=keel
            let y = half_beam * angle.cos();
            let z = -draft * angle.sin() * taper.max(0.1);
            pts.push([x, y, z]);
        }
    }
    let stride = nw + 1;
    for s in 0..ns {
        for w in 0..nw {
            let i0 = s * stride + w;
            let i1 = (s + 1) * stride + w;
            polys.push_cell(&[i0 as i64, i1 as i64, (i1 + 1) as i64]);
            polys.push_cell(&[i0 as i64, (i1 + 1) as i64, (i0 + 1) as i64]);
        }
    }
    // Mirror for port side
    let n_half = pts.len();
    for i in 0..n_half {
        let p = pts.get(i);
        if p[1].abs() > 1e-10 {
            pts.push([p[0], -p[1], p[2]]);
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
    fn test_hull() {
        let m = ship_hull(20.0, 5.0, 2.0, 10, 5);
        assert!(m.points.len() > 50);
        assert!(m.polys.num_cells() > 30);
    }
}
