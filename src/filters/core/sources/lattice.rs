//! 3D lattice structure generators (BCC, FCC, diamond).

use crate::data::{CellArray, Points, PolyData};

/// Create a simple cubic lattice of points connected by edges.
pub fn cubic_lattice(nx: usize, ny: usize, nz: usize, spacing: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let idx = |x: usize, y: usize, z: usize| x + y * nx + z * nx * ny;

    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                pts.push([
                    ix as f64 * spacing,
                    iy as f64 * spacing,
                    iz as f64 * spacing,
                ]);
            }
        }
    }
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let i = idx(ix, iy, iz);
                if ix + 1 < nx {
                    lines.push_cell(&[i as i64, idx(ix + 1, iy, iz) as i64]);
                }
                if iy + 1 < ny {
                    lines.push_cell(&[i as i64, idx(ix, iy + 1, iz) as i64]);
                }
                if iz + 1 < nz {
                    lines.push_cell(&[i as i64, idx(ix, iy, iz + 1) as i64]);
                }
            }
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Create a BCC (body-centered cubic) lattice.
pub fn bcc_lattice(nx: usize, ny: usize, nz: usize, spacing: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let half = spacing / 2.0;

    // Corner points
    let corner_start = 0;
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                pts.push([
                    ix as f64 * spacing,
                    iy as f64 * spacing,
                    iz as f64 * spacing,
                ]);
            }
        }
    }
    // Body center points
    let body_start = pts.len();
    for iz in 0..nz - 1 {
        for iy in 0..ny - 1 {
            for ix in 0..nx - 1 {
                pts.push([
                    ix as f64 * spacing + half,
                    iy as f64 * spacing + half,
                    iz as f64 * spacing + half,
                ]);
            }
        }
    }

    let corner_idx = |x: usize, y: usize, z: usize| corner_start + x + y * nx + z * nx * ny;
    let body_idx =
        |x: usize, y: usize, z: usize| body_start + x + y * (nx - 1) + z * (nx - 1) * (ny - 1);

    // Connect body centers to 8 surrounding corners
    for iz in 0..nz - 1 {
        for iy in 0..ny - 1 {
            for ix in 0..nx - 1 {
                let bi = body_idx(ix, iy, iz) as i64;
                for dz in 0..=1 {
                    for dy in 0..=1 {
                        for dx in 0..=1 {
                            lines.push_cell(&[bi, corner_idx(ix + dx, iy + dy, iz + dz) as i64]);
                        }
                    }
                }
            }
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Create a 2D honeycomb lattice.
pub fn honeycomb_lattice(nx: usize, ny: usize, cell_size: f64) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();
    let dx = cell_size * 1.5;
    let dy = cell_size * 3.0f64.sqrt();

    for iy in 0..ny {
        for ix in 0..nx {
            let cx = ix as f64 * dx;
            let cy = iy as f64 * dy + if ix % 2 == 1 { dy / 2.0 } else { 0.0 };
            let base = pts.len();
            for k in 0..6 {
                let angle = std::f64::consts::PI / 3.0 * k as f64;
                pts.push([
                    cx + cell_size * angle.cos(),
                    cy + cell_size * angle.sin(),
                    0.0,
                ]);
            }
            polys.push_cell(&(0..6).map(|k| (base + k) as i64).collect::<Vec<_>>());
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cubic() {
        let l = cubic_lattice(3, 3, 3, 1.0);
        assert_eq!(l.points.len(), 27);
        assert_eq!(l.lines.num_cells(), 54); // 3*2*3 + 3*3*2 + 2*3*3
    }
    #[test]
    fn test_bcc() {
        let l = bcc_lattice(3, 3, 3, 1.0);
        assert_eq!(l.points.len(), 27 + 8); // 27 corners + 8 body centers
    }
    #[test]
    fn test_honeycomb() {
        let h = honeycomb_lattice(4, 3, 1.0);
        assert_eq!(h.polys.num_cells(), 12);
    }
}
