//! Water molecule (H2O) ball-and-stick model.
use crate::data::{CellArray, Points, PolyData};

fn add_sphere(
    pts: &mut Points<f64>,
    polys: &mut CellArray,
    cx: f64,
    cy: f64,
    cz: f64,
    r: f64,
    na: usize,
) -> usize {
    let base = pts.len();
    let nr = 4;
    pts.push([cx, cy, cz + r]);
    for ring in 1..nr {
        let phi = std::f64::consts::PI * ring as f64 / nr as f64;
        for j in 0..na {
            let theta = 2.0 * std::f64::consts::PI * j as f64 / na as f64;
            pts.push([
                cx + r * phi.sin() * theta.cos(),
                cy + r * phi.sin() * theta.sin(),
                cz + r * phi.cos(),
            ]);
        }
    }
    pts.push([cx, cy, cz - r]);
    let south = pts.len() - 1;
    for j in 0..na {
        polys.push_cell(&[
            base as i64,
            (base + 1 + j) as i64,
            (base + 1 + (j + 1) % na) as i64,
        ]);
    }
    for ring in 0..(nr - 2) {
        let r0 = base + 1 + ring * na;
        let r1 = base + 1 + (ring + 1) * na;
        for j in 0..na {
            let j1 = (j + 1) % na;
            polys.push_cell(&[(r0 + j) as i64, (r1 + j) as i64, (r1 + j1) as i64]);
            polys.push_cell(&[(r0 + j) as i64, (r1 + j1) as i64, (r0 + j1) as i64]);
        }
    }
    let last_ring = base + 1 + (nr - 2) * na;
    for j in 0..na {
        polys.push_cell(&[
            (last_ring + j) as i64,
            south as i64,
            (last_ring + (j + 1) % na) as i64,
        ]);
    }
    base
}

pub fn water_molecule(bond_length: f64, ball_radius: f64, n_angular: usize) -> PolyData {
    let na = n_angular.max(6);
    let angle = 104.5f64 * std::f64::consts::PI / 180.0;
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();
    let o_center = add_sphere(&mut pts, &mut polys, 0.0, 0.0, 0.0, ball_radius * 1.2, na);
    let h1x = bond_length * (angle / 2.0).sin();
    let h1z = bond_length * (angle / 2.0).cos();
    let h1_center = add_sphere(&mut pts, &mut polys, h1x, 0.0, h1z, ball_radius, na);
    let h2_center = add_sphere(&mut pts, &mut polys, -h1x, 0.0, h1z, ball_radius, na);
    lines.push_cell(&[o_center as i64, h1_center as i64]);
    lines.push_cell(&[o_center as i64, h2_center as i64]);
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
    fn test_water() {
        let m = water_molecule(0.96, 0.3, 8);
        assert!(m.points.len() > 20);
        assert!(m.polys.num_cells() > 10);
        assert_eq!(m.lines.num_cells(), 2);
    }
}
