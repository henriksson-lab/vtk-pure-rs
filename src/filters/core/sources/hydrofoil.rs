//! NACA 4-digit airfoil/hydrofoil shape.
use crate::data::{CellArray, Points, PolyData};

pub fn hydrofoil(
    chord: f64,
    max_camber: f64,
    camber_pos: f64,
    thickness: f64,
    n_pts: usize,
) -> PolyData {
    if chord <= 0.0 {
        return PolyData::new();
    }

    let n = n_pts.max(10);
    let p = camber_pos.clamp(f64::EPSILON, 1.0 - f64::EPSILON);
    let mut pts = Points::<f64>::new();
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let x = chord * (1.0 - (std::f64::consts::PI * t).cos()) / 2.0;
        let xc = x / chord;
        let yt = thickness / 0.2
            * chord
            * (0.2969 * xc.sqrt() - 0.126 * xc - 0.3516 * xc.powi(2) + 0.2843 * xc.powi(3)
                - 0.1015 * xc.powi(4));
        let (yc, dyc) = if camber_pos > 0.0 && camber_pos < 1.0 && max_camber > 0.0 {
            if xc < p {
                let yc = max_camber * (2.0 * p * xc - xc.powi(2)) / p.powi(2);
                let dy = 2.0 * max_camber * (p - xc) / p.powi(2);
                (yc * chord, dy)
            } else {
                let yc =
                    max_camber * ((1.0 - 2.0 * p) + 2.0 * p * xc - xc.powi(2)) / (1.0 - p).powi(2);
                let dy = 2.0 * max_camber * (p - xc) / (1.0 - p).powi(2);
                (yc * chord, dy)
            }
        } else {
            (0.0, 0.0)
        };
        let theta = dyc.atan();
        let xu = x - yt * theta.sin();
        let yu = yc + yt * theta.cos();
        let xl = x + yt * theta.sin();
        let yl = yc - yt * theta.cos();
        upper.push(pts.len());
        pts.push([xu, yu, 0.0]);
        lower.push(pts.len());
        pts.push([xl, yl, 0.0]);
    }
    let mut polys = CellArray::new();
    for i in 0..n {
        polys.push_cell(&[upper[i] as i64, upper[i + 1] as i64, lower[i + 1] as i64]);
        polys.push_cell(&[upper[i] as i64, lower[i + 1] as i64, lower[i] as i64]);
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
    fn test_hydrofoil() {
        let m = hydrofoil(1.0, 0.04, 0.4, 0.12, 20);
        assert!(m.points.len() > 20);
        assert!(m.polys.num_cells() > 0);
    }

    #[test]
    fn invalid_chord_is_empty() {
        let m = hydrofoil(0.0, 0.04, 0.4, 0.12, 20);
        assert_eq!(m.points.len(), 0);
        assert_eq!(m.polys.num_cells(), 0);
    }
}
