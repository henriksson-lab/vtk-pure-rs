//! Honeycomb hexagonal lattice.
use crate::data::{CellArray, Points, PolyData};

pub fn honeycomb(cell_size: f64, nx: usize, ny: usize) -> PolyData {
    let nx = nx.max(1);
    let ny = ny.max(1);
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let dx = cell_size * 3.0f64.sqrt();
    let dy = cell_size * 1.5;
    for iy in 0..ny {
        for ix in 0..nx {
            let cx = dx * ix as f64 + if iy % 2 == 1 { dx / 2.0 } else { 0.0 };
            let cy = dy * iy as f64;
            let base = pts.len();
            for j in 0..6 {
                let a = std::f64::consts::PI / 3.0 * j as f64 + std::f64::consts::PI / 6.0;
                pts.push([cx + cell_size * a.cos(), cy + cell_size * a.sin(), 0.0]);
            }
            polys.push_cell(&[
                base as i64,
                (base + 1) as i64,
                (base + 2) as i64,
                (base + 3) as i64,
                (base + 4) as i64,
                (base + 5) as i64,
            ]);
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
    fn test_honeycomb() {
        let m = honeycomb(1.0, 3, 3);
        assert_eq!(m.polys.num_cells(), 9);
        assert_eq!(m.points.len(), 54); // 9 hexagons * 6 vertices
    }
}
