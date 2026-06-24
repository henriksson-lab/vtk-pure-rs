//! Nautilus shell (logarithmic spiral surface).
use crate::data::{CellArray, Points, PolyData};

pub fn nautilus_shell(turns: f64, growth_rate: f64, n_per_turn: usize, n_cross: usize) -> PolyData {
    let npt = n_per_turn.max(12);
    let nc = n_cross.max(6);
    let total = (turns * npt as f64) as usize;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    for i in 0..=total {
        let t = i as f64 / npt as f64; // in turns
        let angle = 2.0 * std::f64::consts::PI * t;
        let r = growth_rate.powf(t); // logarithmic spiral radius
        let cx = r * angle.cos();
        let cy = r * angle.sin();
        let tube_r = r * 0.3; // tube radius proportional to spiral radius
                              // Cross section perpendicular to spiral
        let tangent = [
            -(angle.sin()) * r + growth_rate.ln() * r * angle.cos(),
            angle.cos() * r + growth_rate.ln() * r * angle.sin(),
            0.0,
        ];
        let tlen = (tangent[0] * tangent[0] + tangent[1] * tangent[1])
            .sqrt()
            .max(1e-10);
        let tx = tangent[0] / tlen;
        let ty = tangent[1] / tlen;
        let nx = -ty;
        let ny = tx; // normal in plane
        for j in 0..nc {
            let ca = 2.0 * std::f64::consts::PI * j as f64 / nc as f64;
            let dx = tube_r * ca.cos() * nx;
            let dy = tube_r * ca.cos() * ny;
            let dz = tube_r * ca.sin();
            pts.push([cx + dx, cy + dy, dz]);
        }
    }
    for i in 0..total {
        let b0 = i * nc;
        let b1 = (i + 1) * nc;
        for j in 0..nc {
            let j1 = (j + 1) % nc;
            polys.push_cell(&[(b0 + j) as i64, (b1 + j) as i64, (b1 + j1) as i64]);
            polys.push_cell(&[(b0 + j) as i64, (b1 + j1) as i64, (b0 + j1) as i64]);
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
    fn test_nautilus() {
        let m = nautilus_shell(3.0, 1.3, 20, 8);
        assert!(m.points.len() > 200);
        assert!(m.polys.num_cells() > 200);
    }
}
