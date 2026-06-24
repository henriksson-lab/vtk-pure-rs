//! 3D structured grid source with cell/point data.

use crate::data::{CellArray, Points, PolyData};

/// Generate a 3D grid of hexahedral cells as surface quads.
///
/// Returns the surface faces of an nx × ny × nz grid.
pub fn grid_3d_surface(nx: usize, ny: usize, nz: usize, spacing: [f64; 3]) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Generate all corner points
    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                points.push([
                    ix as f64 * spacing[0],
                    iy as f64 * spacing[1],
                    iz as f64 * spacing[2],
                ]);
            }
        }
    }

    let pt = |ix: usize, iy: usize, iz: usize| -> i64 {
        (iz * (ny + 1) * (nx + 1) + iy * (nx + 1) + ix) as i64
    };

    // X-faces (at ix=0 and ix=nx)
    for iz in 0..nz {
        for iy in 0..ny {
            polys.push_cell(&[
                pt(0, iy, iz),
                pt(0, iy + 1, iz),
                pt(0, iy + 1, iz + 1),
                pt(0, iy, iz + 1),
            ]);
            polys.push_cell(&[
                pt(nx, iy, iz),
                pt(nx, iy, iz + 1),
                pt(nx, iy + 1, iz + 1),
                pt(nx, iy + 1, iz),
            ]);
        }
    }
    // Y-faces
    for iz in 0..nz {
        for ix in 0..nx {
            polys.push_cell(&[
                pt(ix, 0, iz),
                pt(ix, 0, iz + 1),
                pt(ix + 1, 0, iz + 1),
                pt(ix + 1, 0, iz),
            ]);
            polys.push_cell(&[
                pt(ix, ny, iz),
                pt(ix + 1, ny, iz),
                pt(ix + 1, ny, iz + 1),
                pt(ix, ny, iz + 1),
            ]);
        }
    }
    // Z-faces
    for iy in 0..ny {
        for ix in 0..nx {
            polys.push_cell(&[
                pt(ix, iy, 0),
                pt(ix + 1, iy, 0),
                pt(ix + 1, iy + 1, 0),
                pt(ix, iy + 1, 0),
            ]);
            polys.push_cell(&[
                pt(ix, iy, nz),
                pt(ix, iy + 1, nz),
                pt(ix + 1, iy + 1, nz),
                pt(ix + 1, iy, nz),
            ]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Generate a wireframe grid (edges only).
pub fn grid_3d_wireframe(nx: usize, ny: usize, nz: usize, spacing: [f64; 3]) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();

    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                points.push([
                    ix as f64 * spacing[0],
                    iy as f64 * spacing[1],
                    iz as f64 * spacing[2],
                ]);
            }
        }
    }

    let pt = |ix: usize, iy: usize, iz: usize| -> i64 {
        (iz * (ny + 1) * (nx + 1) + iy * (nx + 1) + ix) as i64
    };

    // X edges
    for iz in 0..=nz {
        for iy in 0..=ny {
            for ix in 0..nx {
                lines.push_cell(&[pt(ix, iy, iz), pt(ix + 1, iy, iz)]);
            }
        }
    }
    // Y edges
    for iz in 0..=nz {
        for iy in 0..ny {
            for ix in 0..=nx {
                lines.push_cell(&[pt(ix, iy, iz), pt(ix, iy + 1, iz)]);
            }
        }
    }
    // Z edges
    for iz in 0..nz {
        for iy in 0..=ny {
            for ix in 0..=nx {
                lines.push_cell(&[pt(ix, iy, iz), pt(ix, iy, iz + 1)]);
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surface_cube() {
        let g = grid_3d_surface(1, 1, 1, [1.0, 1.0, 1.0]);
        assert_eq!(g.points.len(), 8);
        assert_eq!(g.polys.num_cells(), 6); // 6 faces
    }
    #[test]
    fn surface_grid() {
        let g = grid_3d_surface(2, 2, 2, [1.0, 1.0, 1.0]);
        assert!(g.polys.num_cells() > 6);
    }
    #[test]
    fn wireframe() {
        let g = grid_3d_wireframe(2, 2, 2, [1.0, 1.0, 1.0]);
        assert!(g.lines.num_cells() > 10);
    }
}
