//! Simplified spine (vertebral column) model.
use crate::data::{CellArray, Points, PolyData};
pub fn spine(
    num_vertebrae: usize,
    vertebra_r: f64,
    vertebra_h: f64,
    disc_h: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(6);
    let nv = num_vertebrae.max(3);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let _total_h = (vertebra_h + disc_h) * nv as f64;
    for vi in 0..nv {
        let z = vi as f64 * (vertebra_h + disc_h);
        // Vertebral body (cylinder with slight bulge)
        for ring in 0..=2 {
            let t = ring as f64 / 2.0;
            let rz = z + t * vertebra_h;
            let bulge = 1.0 + 0.1 * (std::f64::consts::PI * t).sin();
            let r = vertebra_r * bulge;
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([r * a.cos(), r * a.sin(), rz]);
            }
        }
        let base = pts.len() - res * 3;
        for ring in 0..2 {
            for i in 0..res {
                let j = (i + 1) % res;
                polys.push_cell(&[
                    (base + ring * res + i) as i64,
                    (base + ring * res + j) as i64,
                    (base + (ring + 1) * res + j) as i64,
                    (base + (ring + 1) * res + i) as i64,
                ]);
            }
        }
        // Spinous process (posterior projection)
        let sp_z = z + vertebra_h * 0.5;
        let sp_len = vertebra_r * 1.2;
        let sb = pts.len();
        pts.push([0.0, -vertebra_r, sp_z]);
        pts.push([0.0, -vertebra_r - sp_len, sp_z - vertebra_h * 0.1]);
        pts.push([0.0, -vertebra_r - sp_len, sp_z + vertebra_h * 0.1]);
        polys.push_cell(&[sb as i64, (sb + 1) as i64, (sb + 2) as i64]);
        // Transverse processes (lateral projections)
        for &side in &[-1.0f64, 1.0] {
            let tb = pts.len();
            pts.push([side * vertebra_r, 0.0, sp_z]);
            pts.push([side * (vertebra_r + sp_len * 0.7), 0.0, sp_z]);
            pts.push([
                side * (vertebra_r + sp_len * 0.5),
                0.0,
                sp_z + vertebra_h * 0.15,
            ]);
            polys.push_cell(&[tb as i64, (tb + 1) as i64, (tb + 2) as i64]);
        }
    }
    // Intervertebral discs (slightly smaller cylinders between vertebrae)
    for vi in 0..nv - 1 {
        let z = (vi + 1) as f64 * (vertebra_h + disc_h) - disc_h;
        let dr = vertebra_r * 0.95;
        let db = pts.len();
        for ring in 0..=1 {
            let dz = z + ring as f64 * disc_h;
            for i in 0..res {
                let a = 2.0 * std::f64::consts::PI * i as f64 / res as f64;
                pts.push([dr * a.cos(), dr * a.sin(), dz]);
            }
        }
        for i in 0..res {
            let j = (i + 1) % res;
            polys.push_cell(&[
                (db + i) as i64,
                (db + j) as i64,
                (db + res + j) as i64,
                (db + res + i) as i64,
            ]);
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
        let s = spine(5, 0.025, 0.025, 0.008, 8);
        assert!(s.polys.num_cells() > 30);
    }
}
