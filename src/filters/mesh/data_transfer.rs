//! Transfer data between meshes: cell↔point, mesh↔mesh, resample.

use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::types::Scalar;

/// Transfer all cell data to point data by averaging adjacent cells.
pub fn cell_data_to_point_data_all(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let cd = mesh.cell_data();
    let mut result = mesh.clone();

    for ai in 0..cd.num_arrays() {
        let arr = match cd.get_array_by_index(ai) {
            Some(a) => a,
            None => continue,
        };
        let nc = arr.num_components();
        let name = arr.name().to_string();

        let mut sums = vec![0.0f64; n * nc];
        let mut counts = vec![0usize; n];
        let mut buf = vec![0.0f64; nc];

        for (ci, cell) in poly_data_cells(mesh).into_iter().enumerate() {
            if ci >= arr.num_tuples() {
                break;
            }
            arr.tuple_as_f64(ci, &mut buf);
            for &pid in cell {
                let idx = pid as usize;
                if idx >= n {
                    continue;
                }
                counts[idx] += 1;
                for c in 0..nc {
                    sums[idx * nc + c] += buf[c];
                }
            }
        }

        let mut data = Vec::with_capacity(n * nc);
        for i in 0..n {
            for c in 0..nc {
                data.push(if counts[i] > 0 {
                    sums[i * nc + c] / counts[i] as f64
                } else {
                    0.0
                });
            }
        }
        result
            .point_data_mut()
            .add_array(array_from_f64_like(arr, &name, data, nc));
    }
    result
}

/// Transfer all point data to cell data by averaging cell vertices.
pub fn point_data_to_cell_data_all(mesh: &PolyData) -> PolyData {
    let pd = mesh.point_data();
    let mut result = mesh.clone();
    let n_cells = mesh.total_cells();

    for ai in 0..pd.num_arrays() {
        let arr = match pd.get_array_by_index(ai) {
            Some(a) => a,
            None => continue,
        };
        let nc = arr.num_components();
        let name = arr.name().to_string();
        let mut data = Vec::with_capacity(n_cells * nc);
        let mut buf = vec![0.0f64; nc];

        for cell in poly_data_cells(mesh) {
            let mut avg = vec![0.0f64; nc];
            let mut count = 0usize;
            for &pid in cell {
                let idx = pid as usize;
                if idx >= arr.num_tuples() {
                    continue;
                }
                arr.tuple_as_f64(idx, &mut buf);
                for c in 0..nc {
                    avg[c] += buf[c];
                }
                count += 1;
            }
            let k = count.max(1) as f64;
            for c in 0..nc {
                data.push(avg[c] / k.max(1.0));
            }
        }
        result
            .cell_data_mut()
            .add_array(array_from_f64_like(arr, &name, data, nc));
    }
    result
}

fn poly_data_cells(mesh: &PolyData) -> Vec<&[i64]> {
    let mut cells = Vec::with_capacity(mesh.total_cells());
    cells.extend(mesh.verts.iter());
    cells.extend(mesh.lines.iter());
    cells.extend(mesh.polys.iter());
    cells.extend(mesh.strips.iter());
    cells
}

/// Resample point data from source onto target mesh via closest point.
pub fn resample_point_data(source: &PolyData, target: &PolyData) -> PolyData {
    let ns = source.points.len();
    let nt = target.points.len();
    if ns == 0 || nt == 0 {
        return target.clone();
    }

    let src_pts: Vec<[f64; 3]> = (0..ns).map(|i| source.points.get(i)).collect();
    let closest: Vec<usize> = (0..nt)
        .map(|ti| {
            let tp = target.points.get(ti);
            let mut best = 0;
            let mut best_d = f64::MAX;
            for (si, sp) in src_pts.iter().enumerate() {
                let d = (tp[0] - sp[0]).powi(2) + (tp[1] - sp[1]).powi(2) + (tp[2] - sp[2]).powi(2);
                if d < best_d {
                    best_d = d;
                    best = si;
                }
            }
            best
        })
        .collect();

    let mut result = target.clone();
    let pd = source.point_data();
    for ai in 0..pd.num_arrays() {
        let arr = match pd.get_array_by_index(ai) {
            Some(a) => a,
            None => continue,
        };
        let nc = arr.num_components();
        let name = arr.name().to_string();
        let mut data = Vec::with_capacity(nt * nc);
        let mut buf = vec![0.0f64; nc];
        for &ci in &closest {
            arr.tuple_as_f64(ci, &mut buf);
            data.extend_from_slice(&buf);
        }
        result
            .point_data_mut()
            .add_array(array_from_f64_like(arr, &name, data, nc));
    }
    result
}

fn array_from_f64_like(
    template: &AnyDataArray,
    name: &str,
    values: Vec<f64>,
    num_components: usize,
) -> AnyDataArray {
    fn convert<T: Scalar>(name: &str, values: &[f64], num_components: usize) -> DataArray<T> {
        DataArray::from_vec(
            name,
            values.iter().copied().map(T::from_f64).collect(),
            num_components,
        )
    }

    match template {
        AnyDataArray::F32(_) => AnyDataArray::F32(convert(name, &values, num_components)),
        AnyDataArray::F64(_) => {
            AnyDataArray::F64(DataArray::from_vec(name, values, num_components))
        }
        AnyDataArray::I8(_) => AnyDataArray::I8(convert(name, &values, num_components)),
        AnyDataArray::I16(_) => AnyDataArray::I16(convert(name, &values, num_components)),
        AnyDataArray::I32(_) => AnyDataArray::I32(convert(name, &values, num_components)),
        AnyDataArray::I64(_) => AnyDataArray::I64(convert(name, &values, num_components)),
        AnyDataArray::U8(_) => AnyDataArray::U8(convert(name, &values, num_components)),
        AnyDataArray::U16(_) => AnyDataArray::U16(convert(name, &values, num_components)),
        AnyDataArray::U32(_) => AnyDataArray::U32(convert(name, &values, num_components)),
        AnyDataArray::U64(_) => AnyDataArray::U64(convert(name, &values, num_components)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cell_to_point() {
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
                "val",
                vec![10.0, 20.0],
                1,
            )));
        let result = cell_data_to_point_data_all(&mesh);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        // Vertex 1 is shared by both cells → average of 10 and 20 = 15
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 15.0).abs() < 0.01);
    }
    #[test]
    fn point_to_cell() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![0.0, 3.0, 6.0],
                1,
            )));
        let result = point_data_to_cell_data_all(&mesh);
        let arr = result.cell_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 3.0).abs() < 0.01); // average of 0,3,6
    }
    #[test]
    fn resample() {
        let mut src = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        src.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "t",
                vec![100.0, 200.0],
                1,
            )));
        let tgt = PolyData::from_points(vec![[0.1, 0.0, 0.0], [0.9, 0.0, 0.0]]);
        let result = resample_point_data(&src, &tgt);
        let arr = result.point_data().get_array("t").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 100.0); // closest to src[0]
    }

    #[test]
    fn preserves_array_type() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "ids",
                vec![1, 2, 3],
                1,
            )));

        let result = point_data_to_cell_data_all(&mesh);
        assert!(matches!(
            result.cell_data().get_array("ids"),
            Some(AnyDataArray::I32(_))
        ));
    }
}
