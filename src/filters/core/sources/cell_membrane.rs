//! Biological cell membrane (lipid bilayer approximation).
use crate::data::{CellArray, Points, PolyData};
pub fn cell_membrane(radius: f64, undulation: f64, resolution: usize) -> PolyData {
    let res = resolution.max(8);
    let vres = res;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Outer leaflet (slightly perturbed sphere)
    let mut rng = 42u64;
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * undulation;
            let r = radius + noise;
            pts.push([r * sv * u.cos(), r * sv * u.sin(), r * cv]);
        }
    }
    let w = res + 1;
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (iv * w + iu) as i64,
                (iv * w + iu + 1) as i64,
                ((iv + 1) * w + iu + 1) as i64,
                ((iv + 1) * w + iu) as i64,
            ]);
        }
    }
    // Inner leaflet (slightly smaller)
    let inner_r = radius * 0.95;
    let ib = pts.len();
    for iv in 0..=vres {
        let v = std::f64::consts::PI * iv as f64 / vres as f64;
        let sv = v.sin();
        let cv = v.cos();
        for iu in 0..=res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let noise = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * undulation * 0.8;
            let r = inner_r + noise;
            pts.push([r * sv * u.cos(), r * sv * u.sin(), r * cv]);
        }
    }
    for iv in 0..vres {
        for iu in 0..res {
            polys.push_cell(&[
                (ib + (iv + 1) * w + iu) as i64,
                (ib + (iv + 1) * w + iu + 1) as i64,
                (ib + iv * w + iu + 1) as i64,
                (ib + iv * w + iu) as i64,
            ]);
        }
    }
    // Nucleus (smaller sphere inside)
    let nuc_r = radius * 0.3;
    let nb = pts.len();
    for iv in 0..=vres / 2 {
        let v = std::f64::consts::PI * iv as f64 / (vres / 2) as f64;
        for iu in 0..=res / 2 {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / (res / 2) as f64;
            pts.push([
                nuc_r * v.sin() * u.cos(),
                nuc_r * v.sin() * u.sin(),
                nuc_r * v.cos(),
            ]);
        }
    }
    let nw = res / 2 + 1;
    for iv in 0..vres / 2 {
        for iu in 0..res / 2 {
            polys.push_cell(&[
                (nb + iv * nw + iu) as i64,
                (nb + iv * nw + iu + 1) as i64,
                (nb + (iv + 1) * nw + iu + 1) as i64,
                (nb + (iv + 1) * nw + iu) as i64,
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
        let c = cell_membrane(5.0, 0.2, 10);
        assert!(c.polys.num_cells() > 100);
    }
}
