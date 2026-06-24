use crate::data::{CellArray, DataSet, Points, PolyData, StructuredGrid};

/// Convert the outer surface of a StructuredGrid to PolyData quads.
pub fn structured_to_poly_data(input: &StructuredGrid) -> PolyData {
    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];

    if nx < 2 || ny < 2 || nz < 2 {
        return PolyData::new();
    }

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Copy all points
    let n_pts = nx * ny * nz;
    for i in 0..n_pts {
        points.push(input.point(i));
    }

    let point_idx = |i: usize, j: usize, k: usize| -> i64 { (k * ny * nx + j * nx + i) as i64 };

    // -X face
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
    // +X face
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
    fn simple_structured() {
        let mut grid = StructuredGrid::new();
        grid.set_dimensions([2, 2, 2]);
        for k in 0..2 {
            for j in 0..2 {
                for i in 0..2 {
                    grid.points.push([i as f64, j as f64, k as f64]);
                }
            }
        }

        let result = structured_to_poly_data(&grid);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.polys.num_cells(), 6); // 6 faces of a cube
    }
}
