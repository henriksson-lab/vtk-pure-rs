//! Convert point data to cell data and vice versa.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Convert a point data array to cell data by averaging over cell vertices.
pub fn point_data_to_cell_data(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) => a,
        None => return mesh.clone(),
    };
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let mut cell_data = Vec::new();

    for cell in mesh.polys.iter() {
        let nv = cell.len();
        if nv == 0 {
            for _ in 0..nc {
                cell_data.push(0.0);
            }
            continue;
        }
        let mut avg = vec![0.0f64; nc];
        for &v in cell {
            arr.tuple_as_f64(v as usize, &mut buf);
            for c in 0..nc {
                avg[c] += buf[c];
            }
        }
        for c in 0..nc {
            cell_data.push(avg[c] / nv as f64);
        }
    }

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

    for (ci, cell) in mesh.polys.iter().enumerate() {
        arr.tuple_as_f64(ci, &mut buf);
        for &v in cell {
            let vi = v as usize;
            counts[vi] += 1;
            for c in 0..nc {
                sums[vi * nc + c] += buf[c];
            }
        }
    }

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
}
