use crate::data::{AnyDataArray, CellArray, DataArray, DataSet, Points, PolyData, StructuredGrid};

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
    let mut source_cell_ids = Vec::new();

    // Copy all points
    let n_pts = nx * ny * nz;
    for i in 0..n_pts {
        points.push(input.point(i));
    }

    let point_idx = |i: usize, j: usize, k: usize| -> i64 { (k * ny * nx + j * nx + i) as i64 };
    let cell_idx =
        |i: usize, j: usize, k: usize| -> usize { k * (ny - 1) * (nx - 1) + j * (nx - 1) + i };

    // -X face
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            polys.push_cell(&[
                point_idx(0, j, k),
                point_idx(0, j, k + 1),
                point_idx(0, j + 1, k + 1),
                point_idx(0, j + 1, k),
            ]);
            source_cell_ids.push(cell_idx(0, j, k));
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
            source_cell_ids.push(cell_idx(nx - 2, j, k));
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
            source_cell_ids.push(cell_idx(i, 0, k));
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
            source_cell_ids.push(cell_idx(i, ny - 2, k));
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
            source_cell_ids.push(cell_idx(i, j, 0));
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
            source_cell_ids.push(cell_idx(i, j, nz - 2));
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    for i in 0..input.point_data().num_arrays() {
        if let Some(array) = input.point_data().get_array_by_index(i) {
            pd.point_data_mut().add_array(array.clone());
        }
    }
    copy_cell_data(input, &source_cell_ids, &mut pd);
    pd
}

fn copy_cell_data(input: &StructuredGrid, source_cell_ids: &[usize], output: &mut PolyData) {
    for i in 0..input.cell_data().num_arrays() {
        let Some(array) = input.cell_data().get_array_by_index(i) else {
            continue;
        };
        if array.num_tuples() >= input.num_cells() {
            output
                .cell_data_mut()
                .add_array(select_tuples(array, source_cell_ids));
        }
    }
}

fn select_tuples(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! select {
        ($array:expr, $variant:ident) => {{
            let mut out = DataArray::new($array.name(), $array.num_components());
            for &tuple_id in tuple_ids {
                out.push_tuple($array.tuple(tuple_id));
            }
            AnyDataArray::$variant(out)
        }};
    }

    match array {
        AnyDataArray::F32(a) => select!(a, F32),
        AnyDataArray::F64(a) => select!(a, F64),
        AnyDataArray::I8(a) => select!(a, I8),
        AnyDataArray::I16(a) => select!(a, I16),
        AnyDataArray::I32(a) => select!(a, I32),
        AnyDataArray::I64(a) => select!(a, I64),
        AnyDataArray::U8(a) => select!(a, U8),
        AnyDataArray::U16(a) => select!(a, U16),
        AnyDataArray::U32(a) => select!(a, U32),
        AnyDataArray::U64(a) => select!(a, U64),
    }
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
