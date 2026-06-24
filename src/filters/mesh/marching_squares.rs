use crate::data::{CellArray, ImageData, Points, PolyData};

/// 2D marching squares: extract contour lines from a 2D scalar field on a
/// regular grid (ImageData with nz=1).
///
/// For each cell in the 2D grid, classifies the four corner values relative
/// to `iso_value` and emits zero, one, or two line segments using the
/// standard 16-case lookup table.
pub fn marching_squares(image: &ImageData, scalars: &str, iso_value: f64) -> PolyData {
    let arr = match image.point_data().get_array(scalars) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let dims = image.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;

    if nx < 2 || ny < 2 {
        return PolyData::new();
    }

    let n: usize = nx * ny;
    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let spacing = image.spacing();
    let origin = image.origin();

    let point_pos = |i: usize, j: usize| -> [f64; 2] {
        [
            origin[0] + i as f64 * spacing[0],
            origin[1] + j as f64 * spacing[1],
        ]
    };

    let idx = |i: usize, j: usize| -> usize { j * nx + i };

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    // Edge indices for a square cell:
    //   3---2---2
    //   |       |
    //   3       1
    //   |       |
    //   0---0---1
    //
    // Edges: 0=bottom, 1=right, 2=top, 3=left

    // Lookup table: for each of the 16 cases, list of edge pairs forming line segments.
    // Each sub-array contains pairs (edge_a, edge_b).
    let edge_table: [&[(usize, usize)]; 16] = [
        &[],               // 0000
        &[(0, 3)],         // 0001
        &[(0, 1)],         // 0010
        &[(1, 3)],         // 0011
        &[(1, 2)],         // 0100
        &[(0, 1), (2, 3)], // 0101 - saddle
        &[(0, 2)],         // 0110
        &[(2, 3)],         // 0111
        &[(2, 3)],         // 1000
        &[(0, 2)],         // 1001
        &[(0, 3), (1, 2)], // 1010 - saddle
        &[(1, 2)],         // 1011
        &[(1, 3)],         // 1100
        &[(0, 1)],         // 1101
        &[(0, 3)],         // 1110
        &[],               // 1111
    ];

    for j in 0..ny - 1 {
        for i in 0..nx - 1 {
            let v0: f64 = values[idx(i, j)];
            let v1: f64 = values[idx(i + 1, j)];
            let v2: f64 = values[idx(i + 1, j + 1)];
            let v3: f64 = values[idx(i, j + 1)];

            let mut case_idx: u8 = 0;
            if v0 >= iso_value {
                case_idx |= 1;
            }
            if v1 >= iso_value {
                case_idx |= 2;
            }
            if v2 >= iso_value {
                case_idx |= 4;
            }
            if v3 >= iso_value {
                case_idx |= 8;
            }

            let segments = edge_table[case_idx as usize];
            if segments.is_empty() {
                continue;
            }

            let p0 = point_pos(i, j);
            let p1 = point_pos(i + 1, j);
            let p2 = point_pos(i + 1, j + 1);
            let p3 = point_pos(i, j + 1);

            let interp_edge = |edge: usize| -> [f64; 3] {
                let (va, vb, pa, pb) = match edge {
                    0 => (v0, v1, p0, p1),             // bottom
                    1 => (v1, v2, p1, p2),             // right
                    2 => (v3, v3 + (v2 - v3), p3, p2), // top (v3->v2)
                    3 => (v0, v3, p0, p3),             // left
                    _ => unreachable!(),
                };
                let t: f64 = if (vb - va).abs() > 1e-20 {
                    (iso_value - va) / (vb - va)
                } else {
                    0.5
                };
                [
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    0.0,
                ]
            };

            for &(ea, eb) in segments {
                let pt_a = interp_edge(ea);
                let pt_b = interp_edge(eb);
                let id_a: i64 = out_points.len() as i64;
                out_points.push(pt_a);
                out_points.push(pt_b);
                out_lines.push_cell(&[id_a, id_a + 1]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn simple_contour() {
        // 3x3 grid with a step function: left column = 0, rest = 1
        let mut img = ImageData::with_dimensions(3, 3, 1);
        let vals: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalar", vals, 1)));

        let result = marching_squares(&img, "scalar", 0.5);
        // Should produce contour lines along x=0.5
        assert!(result.lines.num_cells() > 0);
        // All points should have x near 0.5
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            assert!((p[0] - 0.5).abs() < 1e-10);
            assert!((p[2]).abs() < 1e-10);
        }
    }

    #[test]
    fn no_contour_uniform() {
        // Uniform field: no contour lines
        let mut img = ImageData::with_dimensions(4, 4, 1);
        let vals: Vec<f64> = vec![1.0; 16];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("scalar", vals, 1)));

        let result = marching_squares(&img, "scalar", 0.5);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn missing_array_returns_empty() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let result = marching_squares(&img, "nope", 0.5);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
