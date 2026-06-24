//! Fluted column (classical architecture).
use crate::data::{CellArray, Points, PolyData};
pub fn fluted_column(
    radius: f64,
    height: f64,
    flutes: usize,
    flute_depth: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(flutes * 2);
    let half_h = height / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let nz = 10;
    for iz in 0..=nz {
        let z = -half_h + height * iz as f64 / nz as f64;
        for iu in 0..res {
            let a = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let flute_mod = (a * flutes as f64).cos();
            let r = radius - flute_depth * 0.5 * (1.0 - flute_mod);
            pts.push([r * a.cos(), r * a.sin(), z]);
        }
    }
    for iz in 0..nz {
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (iz * res + iu) as i64,
                (iz * res + iu1) as i64,
                ((iz + 1) * res + iu1) as i64,
                ((iz + 1) * res + iu) as i64,
            ]);
        }
    }
    // Caps
    let bc = pts.len();
    pts.push([0.0, 0.0, -half_h]);
    for iu in 0..res {
        polys.push_cell(&[bc as i64, ((iu + 1) % res) as i64, iu as i64]);
    }
    let tc = pts.len();
    pts.push([0.0, 0.0, half_h]);
    let top = nz * res;
    for iu in 0..res {
        polys.push_cell(&[tc as i64, (top + iu) as i64, (top + (iu + 1) % res) as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let c = fluted_column(1.0, 5.0, 8, 0.1, 24);
        assert!(c.points.len() > 100);
        assert!(c.polys.num_cells() > 100);
    }
}
