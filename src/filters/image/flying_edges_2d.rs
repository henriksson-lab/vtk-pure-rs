//! Flying Edges 2D — fast contour extraction from 2D ImageData.
//!
//! Extracts isocontour line segments from a 2D scalar field on a regular grid.
//! This is the 2D analog of Flying Edges 3D / Marching Cubes.

use crate::data::{AnyDataArray, CellArray, ImageData, Points, PolyData};

/// Extract 2D contour lines at a given isovalue from ImageData.
///
/// The input must be a 2D ImageData (dims[2] == 1) with a scalar array.
/// Returns line segments as PolyData.
pub fn flying_edges_2d(input: &ImageData, array_name: &str, isovalue: f64) -> PolyData {
    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    if nx < 2 || ny < 2 {
        return PolyData::new();
    }

    let scalars = match input.point_data().get_array(array_name) {
        Some(arr) => arr,
        None => return PolyData::new(),
    };

    let spacing = input.spacing();
    let origin = input.origin();

    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();

    // Marching squares: iterate over each cell (i, j)
    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            // Cell corner indices
            let idx00 = j * nx + i;
            let idx10 = j * nx + i + 1;
            let idx01 = (j + 1) * nx + i;
            let idx11 = (j + 1) * nx + i + 1;

            let mut v = [0.0f64; 1];
            scalars.tuple_as_f64(idx00, &mut v);
            let v00 = v[0];
            scalars.tuple_as_f64(idx10, &mut v);
            let v10 = v[0];
            scalars.tuple_as_f64(idx01, &mut v);
            let v01 = v[0];
            scalars.tuple_as_f64(idx11, &mut v);
            let v11 = v[0];

            // Classify corners
            let case = ((v00 >= isovalue) as u8)
                | (((v10 >= isovalue) as u8) << 1)
                | (((v01 >= isovalue) as u8) << 2)
                | (((v11 >= isovalue) as u8) << 3);

            if case == 0 || case == 15 {
                continue; // no intersection
            }

            let x0 = origin[0] + i as f64 * spacing[0];
            let y0 = origin[1] + j as f64 * spacing[1];
            let z = origin[2];

            // Edge intersection helper
            let interp = |va: f64, vb: f64, pa: f64, pb: f64| -> f64 {
                let t = if (vb - va).abs() > 1e-30 {
                    (isovalue - va) / (vb - va)
                } else {
                    0.5
                };
                pa + t * (pb - pa)
            };

            // Edges: bottom (0-1), right (1-3), top (2-3), left (0-2)
            let edge_points: [Option<[f64; 3]>; 4] = [
                if (case & 1) != (case >> 1 & 1) {
                    Some([interp(v00, v10, x0, x0 + spacing[0]), y0, z])
                } else {
                    None
                },
                if (case >> 1 & 1) != (case >> 3 & 1) {
                    Some([x0 + spacing[0], interp(v10, v11, y0, y0 + spacing[1]), z])
                } else {
                    None
                },
                if (case >> 2 & 1) != (case >> 3 & 1) {
                    Some([interp(v01, v11, x0, x0 + spacing[0]), y0 + spacing[1], z])
                } else {
                    None
                },
                if (case & 1) != (case >> 2 & 1) {
                    Some([x0, interp(v00, v01, y0, y0 + spacing[1]), z])
                } else {
                    None
                },
            ];

            // Marching squares line segments lookup
            let segments: &[(usize, usize)] = match case {
                1 | 14 => &[(0, 3)],
                2 | 13 => &[(0, 1)],
                3 | 12 => &[(1, 3)],
                4 | 11 => &[(2, 3)],
                5 => &[(0, 3), (1, 2)], // saddle
                6 | 9 => &[(0, 2)],
                7 | 8 => &[(1, 2)],
                10 => &[(0, 1), (2, 3)], // saddle
                _ => &[],
            };

            for &(e0, e1) in segments {
                if let (Some(p0), Some(p1)) = (edge_points[e0], edge_points[e1]) {
                    let base = points.len();
                    points.push(p0);
                    points.push(p1);
                    lines.push_cell(&[base as i64, (base + 1) as i64]);
                }
            }
        }
    }

    let mut result = PolyData::new();
    result.points = points;
    result.lines = lines;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray, ImageData};

    #[test]
    fn circle_contour() {
        // Create a 2D distance field: f(x,y) = x^2 + y^2
        let mut grid = ImageData::new([21, 21, 1], [0.1, 0.1, 1.0], [-1.0, -1.0, 0.0]);
        let dims = grid.dimensions();
        let n = dims[0] * dims[1] * dims[2];
        let spacing = grid.spacing();
        let origin = grid.origin();
        let mut vals = Vec::with_capacity(n);
        for j in 0..dims[1] {
            for i in 0..dims[0] {
                let x = origin[0] + i as f64 * spacing[0];
                let y = origin[1] + j as f64 * spacing[1];
                vals.push(x * x + y * y);
            }
        }
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("dist", vals, 1)));

        let contour = flying_edges_2d(&grid, "dist", 0.5);
        assert!(
            contour.points.len() > 10,
            "should produce contour points for circle r=sqrt(0.5)"
        );
        assert!(
            contour.lines.num_cells() > 5,
            "should produce line segments"
        );
    }

    #[test]
    fn no_intersection() {
        let mut grid = ImageData::new([5, 5, 1], [1.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let vals = vec![1.0; 25];
        grid.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("s", vals, 1)));
        let contour = flying_edges_2d(&grid, "s", 0.5);
        assert_eq!(contour.points.len(), 0);
    }
}
