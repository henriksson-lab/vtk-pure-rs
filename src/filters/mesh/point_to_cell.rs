//! Convert point data to cell data and vice versa.

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};

fn for_each_cell(mesh: &PolyData, mut f: impl FnMut(usize, &[i64])) {
    let cell_arrays: [&CellArray; 4] = [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips];
    let mut cell_id = 0;
    for cells in cell_arrays {
        for cell in cells.iter() {
            f(cell_id, cell);
            cell_id += 1;
        }
    }
}

/// Convert a point data array to cell data by averaging over cell vertices.
pub fn point_data_to_cell_data(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let mut cell_data = Vec::new();

    for_each_cell(mesh, |_, cell| {
        let mut avg = vec![0.0f64; nc];
        let mut num_valid = 0usize;
        for &v in cell {
            let Ok(vi) = usize::try_from(v) else {
                continue;
            };
            if vi >= arr.num_tuples() {
                continue;
            }
            arr.tuple_as_f64(vi, &mut buf);
            num_valid += 1;
            for c in 0..nc {
                avg[c] += buf[c];
            }
        }
        for c in 0..nc {
            cell_data.push(if num_valid > 0 {
                avg[c] / num_valid as f64
            } else {
                0.0
            });
        }
    });

    let mut result = mesh.clone();
    result
        .cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, cell_data, nc,
        )));
    result
}

/// Convert a cell data array to point data by averaging over incident cells.
pub fn cell_data_to_point_data(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.cell_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let npts = mesh.points.len();
    let mut sums = vec![0.0f64; npts * nc];
    let mut counts = vec![0usize; npts];
    let mut buf = vec![0.0f64; nc];

    for_each_cell(mesh, |ci, cell| {
        if ci >= arr.num_tuples() {
            return;
        }
        arr.tuple_as_f64(ci, &mut buf);
        for &v in cell {
            let Ok(vi) = usize::try_from(v) else {
                continue;
            };
            if vi >= npts {
                continue;
            }
            counts[vi] += 1;
            for c in 0..nc {
                sums[vi * nc + c] += buf[c];
            }
        }
    });

    let mut data = Vec::with_capacity(npts * nc);
    for i in 0..npts {
        for c in 0..nc {
            data.push(if counts[i] > 0 {
                sums[i * nc + c] / counts[i] as f64
            } else {
                0.0
            });
        }
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, nc)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pt_to_cell() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        let r = point_data_to_cell_data(&mesh, "s");
        let arr = r.cell_data().get_array("s").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 1e-10); // (1+2+3)/3
    }
    #[test]
    fn test_cell_to_pt() {
        let mut mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "c",
                vec![10.0, 20.0],
                1,
            )));
        let r = cell_data_to_point_data(&mesh, "c");
        let arr = r.point_data().get_array("c").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(1, &mut buf); // vertex 1 is shared
        assert!((buf[0] - 15.0).abs() < 1e-10); // (10+20)/2
    }

    #[test]
    fn test_pt_to_cell_uses_all_polydata_cell_arrays_in_vtk_order() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[1, 2]);
        mesh.polys.push_cell(&[0, 1, 3]);
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 3.0, 5.0, 7.0],
                1,
            )));

        let r = point_data_to_cell_data(&mesh, "s");
        let arr = r.cell_data().get_array("s").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 4.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 11.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_cell_to_pt_uses_all_polydata_cell_arrays_in_vtk_order() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.verts.push_cell(&[0]);
        mesh.lines.push_cell(&[0, 1]);
        mesh.polys.push_cell(&[1, 2, 0]);
        mesh.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "c",
                vec![6.0, 12.0, 30.0],
                1,
            )));

        let r = cell_data_to_point_data(&mesh, "c");
        let arr = r.point_data().get_array("c").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 16.0).abs() < 1e-10);
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 21.0).abs() < 1e-10);
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 30.0).abs() < 1e-10);
    }
}
