use crate::data::{AnyDataArray, CellArray, ImageData, Points, PolyData};

/// Smooth 2D contour extraction from ImageData using Surface Nets.
///
/// Similar to marching squares but produces a surface-net dual contour by
/// placing vertices at active cell centers.
/// Input: 2D ImageData (nz == 1) with a named scalar array and an isovalue.
/// Output: PolyData with line segments forming the contour.
pub fn surface_nets_2d(input: &ImageData, scalars: &str, isovalue: f64) -> PolyData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        None => return PolyData::new(),
        _ => return PolyData::new(),
    };

    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    if nx < 2 || ny < 2 {
        return PolyData::new();
    }

    let spacing = input.spacing();
    let origin = input.origin();

    // Read all scalar values
    let n = nx * ny;
    if arr.num_tuples() < n {
        return PolyData::new();
    }
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let cell_nx = nx - 1;
    let cell_ny = ny - 1;

    // For each cell, determine if the contour crosses it and compute a vertex position.
    // A cell is crossed if its 4 corner values are not all on the same side of the isovalue.
    let mut cell_vertex: Vec<Option<usize>> = vec![None; cell_nx * cell_ny];
    let mut points = Points::<f64>::new();

    for cj in 0..cell_ny {
        for ci in 0..cell_nx {
            let v00 = values[cj * nx + ci];
            let v10 = values[cj * nx + ci + 1];
            let v01 = values[(cj + 1) * nx + ci];
            let v11 = values[(cj + 1) * nx + ci + 1];

            let s00 = v00 >= isovalue;
            let s10 = v10 >= isovalue;
            let s01 = v01 >= isovalue;
            let s11 = v11 >= isovalue;

            if s00 == s10 && s10 == s01 && s01 == s11 {
                continue; // No crossing
            }

            // VTK's unsmoothed SurfaceNets2D generates one point at the
            // center of each active square, then connects active neighboring
            // square centers along split shared edges.
            let cx = ci as f64 + 0.5;
            let cy = cj as f64 + 0.5;

            // Transform to world coordinates
            let wx = origin[0] + cx * spacing[0];
            let wy = origin[1] + cy * spacing[1];
            let wz = origin[2];

            let idx = points.len();
            points.push([wx, wy, wz]);
            cell_vertex[cj * cell_nx + ci] = Some(idx);
        }
    }

    // Connect adjacent cell vertices with line segments
    let mut lines = CellArray::new();

    for cj in 0..cell_ny {
        for ci in 0..cell_nx {
            if let Some(v0) = cell_vertex[cj * cell_nx + ci] {
                // Connect to right neighbor only when the shared vertical edge is crossed.
                if ci + 1 < cell_nx {
                    let v10 = values[cj * nx + ci + 1];
                    let v11 = values[(cj + 1) * nx + ci + 1];
                    if (v10 >= isovalue) != (v11 >= isovalue) {
                        if let Some(v1) = cell_vertex[cj * cell_nx + ci + 1] {
                            lines.push_cell(&[v0 as i64, v1 as i64]);
                        }
                    }
                }
                // Connect to top neighbor only when the shared horizontal edge is crossed.
                if cj + 1 < cell_ny {
                    let v01 = values[(cj + 1) * nx + ci];
                    let v11 = values[(cj + 1) * nx + ci + 1];
                    if (v01 >= isovalue) != (v11 >= isovalue) {
                        if let Some(v1) = cell_vertex[(cj + 1) * cell_nx + ci] {
                            lines.push_cell(&[v0 as i64, v1 as i64]);
                        }
                    }
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn circle_contour() {
        // Create a 2D image with a circular scalar field
        let nx = 20;
        let ny = 20;
        let mut img = ImageData::with_dimensions(nx, ny, 1);
        img.set_spacing([0.1, 0.1, 1.0]);
        img.set_origin([-1.0, -1.0, 0.0]);

        let mut vals = Vec::with_capacity(nx * ny);
        for j in 0..ny {
            for i in 0..nx {
                let x = -1.0 + i as f64 * 0.1;
                let y = -1.0 + j as f64 * 0.1;
                vals.push(x * x + y * y);
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("dist", vals, 1)));

        let result = surface_nets_2d(&img, "dist", 0.5);
        // Should produce contour lines
        assert!(result.points.len() > 0);
        assert!(result.lines.num_cells() > 0);
    }

    #[test]
    fn no_crossing() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let vals = vec![1.0; 9]; // All above isovalue
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));

        let result = surface_nets_2d(&img, "v", 0.5);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let result = surface_nets_2d(&img, "nope", 0.5);
        assert_eq!(result.points.len(), 0);
    }
}
