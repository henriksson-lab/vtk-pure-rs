//! Bone cross-section (cortical + trabecular structure).
use crate::data::{CellArray, Points, PolyData};
pub fn long_bone(length: f64, outer_r: f64, inner_r: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let hl = length / 2.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer cortical shell
    for ring in 0..=1 {
        let z = if ring == 0 { -hl } else { hl };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([outer_r * a.cos(), outer_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[i as i64, j as i64, (res + j) as i64, (res + i) as i64]);
    }
    // Medullary cavity (inner surface)
    let ib = pts.len();
    for ring in 0..=1 {
        let z = if ring == 0 { -hl } else { hl };
        for i in 0..res {
            let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
            pts.push([inner_r * a.cos(), inner_r * a.sin(), z]);
        }
    }
    for i in 0..res {
        let j = (i + 1) % res;
        polys.push_cell(&[
            (ib + j) as i64,
            (ib + i) as i64,
            (ib + res + i) as i64,
            (ib + res + j) as i64,
        ]);
    }
    // End caps (annular)
    for &z_ring in &[0, res] {
        let ir = if z_ring == 0 { 0 } else { res };
        let iir = if z_ring == 0 { ib } else { ib + res };
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (ir + i) as i64,
                (ir + j) as i64,
                (iir + j) as i64,
                (iir + i) as i64,
            ]);
        }
    }
    // Epiphysis (rounded ends)
    let dome_res = 4;
    for &side in &[-1.0f64, 1.0] {
        let z = side * hl;
        let db = pts.len();
        for dr in 1..=dome_res {
            let t = dr as f64 / dome_res as f64;
            let a = t * std::f64::consts::FRAC_PI_2;
            let r = outer_r * a.cos();
            let dz = side * outer_r * 0.3 * a.sin();
            for i in 0..res {
                let ang = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([r * ang.cos(), r * ang.sin(), z + dz]);
            }
        }
        let base = if side < 0.0 { 0 } else { res };
        for dr in 0..dome_res {
            let r0 = if dr == 0 { base } else { db + (dr - 1) * res };
            let r1 = db + dr * res;
            if dr == 0 {
                for i in 0..res {
                    let j = (i + 1) % res;
                    polys.push_cell(&[
                        (base + i) as i64,
                        (base + j) as i64,
                        (r1 + j) as i64,
                        (r1 + i) as i64,
                    ]);
                }
            } else {
                for i in 0..res {
                    let j = (i + 1) % res;
                    polys.push_cell(&[
                        (r0 + i) as i64,
                        (r0 + j) as i64,
                        (r1 + j) as i64,
                        (r1 + i) as i64,
                    ]);
                }
            }
        }
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
        let b = long_bone(10.0, 1.5, 0.8, 12);
        assert!(b.polys.num_cells() > 50);
    }
}
