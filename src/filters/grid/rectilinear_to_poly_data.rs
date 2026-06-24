use crate::data::{CellArray, DataSet, Points, PolyData, RectilinearGrid};

/// Convert the outer surface of a RectilinearGrid to PolyData quads.
///
/// Extracts the 6 boundary faces of the grid as quad cells, similar
/// to `geometry_filter_image` but for non-uniform grids.
pub fn rectilinear_to_poly_data(input: &RectilinearGrid) -> PolyData {
    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];

    if nx < 2 || ny < 2 || nz < 2 {
        return PolyData::new();
    }

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Generate all grid points
    let point_idx = |i: usize, j: usize, k: usize| -> i64 { (k * ny * nx + j * nx + i) as i64 };

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                points.push(input.point(k * ny * nx + j * nx + i));
            }
        }
    }

    // -X face (i=0)
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            polys.push_cell(&[
                point_idx(0, j, k),
                point_idx(0, j, k + 1),
                point_idx(0, j + 1, k + 1),
                point_idx(0, j + 1, k),
            ]);
        }
    }
    // +X face (i=nx-1)
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            polys.push_cell(&[
                point_idx(nx - 1, j, k),
                point_idx(nx - 1, j + 1, k),
                point_idx(nx - 1, j + 1, k + 1),
                point_idx(nx - 1, j, k + 1),
            ]);
        }
    }
    // -Y face
    for k in 0..nz - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, 0, k),
                point_idx(i + 1, 0, k),
                point_idx(i + 1, 0, k + 1),
                point_idx(i, 0, k + 1),
            ]);
        }
    }
    // +Y face
    for k in 0..nz - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, ny - 1, k),
                point_idx(i, ny - 1, k + 1),
                point_idx(i + 1, ny - 1, k + 1),
                point_idx(i + 1, ny - 1, k),
            ]);
        }
    }
    // -Z face
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, j, 0),
                point_idx(i, j + 1, 0),
                point_idx(i + 1, j + 1, 0),
                point_idx(i + 1, j, 0),
            ]);
        }
    }
    // +Z face
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            polys.push_cell(&[
                point_idx(i, j, nz - 1),
                point_idx(i + 1, j, nz - 1),
                point_idx(i + 1, j + 1, nz - 1),
                point_idx(i, j + 1, nz - 1),
            ]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_grid() {
        let mut grid = RectilinearGrid::new();
        grid.set_x_coords(vec![0.0, 1.0, 2.0]);
        grid.set_y_coords(vec![0.0, 1.0]);
        grid.set_z_coords(vec![0.0, 1.0]);

        let result = rectilinear_to_poly_data(&grid);
        assert_eq!(result.points.len(), 12); // 3*2*2
                                             // 6 faces: -X(1), +X(1), -Y(2), +Y(2), -Z(2), +Z(2) = 10
        assert_eq!(result.polys.num_cells(), 10);
    }

    #[test]
    fn uniform_2x2x2() {
        let mut grid = RectilinearGrid::new();
        grid.set_x_coords(vec![0.0, 1.0]);
        grid.set_y_coords(vec![0.0, 1.0]);
        grid.set_z_coords(vec![0.0, 1.0]);

        let result = rectilinear_to_poly_data(&grid);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.polys.num_cells(), 6);
    }
}
