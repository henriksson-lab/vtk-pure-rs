//! Calabi-Yau manifold cross-section (quintic in CP4, projected).
use crate::data::{CellArray, Points, PolyData};
pub fn calabi_yau_section(
    n_param: usize,
    k_param: usize,
    scale: f64,
    resolution: usize,
) -> PolyData {
    let res = resolution.max(8);
    let n = n_param.max(1);
    let k = k_param.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    // Parametric CY surface: z1^n + z2^n = 1 cross-section
    for iv in 0..res {
        let v = 2.0 * std::f64::consts::PI * iv as f64 / res as f64;
        for iu in 0..res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let r1 = (u.cos().abs().powf(2.0 / n as f64)).max(0.01);
            let r2 = (u.sin().abs().powf(2.0 / n as f64)).max(0.01);
            let x = scale * r1 * (v * k as f64).cos();
            let y = scale * r1 * (v * k as f64).sin();
            let z = scale * r2 * (v * k as f64 + u).cos();
            pts.push([x, y, z]);
        }
    }
    for iv in 0..res {
        let iv1 = (iv + 1) % res;
        for iu in 0..res {
            let iu1 = (iu + 1) % res;
            polys.push_cell(&[
                (iv * res + iu) as i64,
                (iv * res + iu1) as i64,
                (iv1 * res + iu1) as i64,
                (iv1 * res + iu) as i64,
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
        let cy = calabi_yau_section(5, 2, 1.0, 12);
        assert!(cy.polys.num_cells() > 100);
    }
}
