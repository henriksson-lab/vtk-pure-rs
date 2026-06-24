//! Combine multiple scalar arrays into one via arithmetic operations.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn add_arrays(mesh: &PolyData, a: &str, b: &str, output: &str) -> PolyData {
    combine_arrays(mesh, a, b, output, |x, y| x + y)
}
pub fn subtract_arrays(mesh: &PolyData, a: &str, b: &str, output: &str) -> PolyData {
    combine_arrays(mesh, a, b, output, |x, y| x - y)
}
pub fn multiply_arrays(mesh: &PolyData, a: &str, b: &str, output: &str) -> PolyData {
    combine_arrays(mesh, a, b, output, |x, y| x * y)
}
pub fn max_arrays(mesh: &PolyData, a: &str, b: &str, output: &str) -> PolyData {
    combine_arrays(mesh, a, b, output, |x, y| x.max(y))
}
pub fn min_arrays(mesh: &PolyData, a: &str, b: &str, output: &str) -> PolyData {
    combine_arrays(mesh, a, b, output, |x, y| x.min(y))
}
fn combine_arrays(
    mesh: &PolyData,
    a: &str,
    b: &str,
    output: &str,
    op: impl Fn(f64, f64) -> f64,
) -> PolyData {
    let aa = match mesh.point_data().get_array(a) {
        Some(x) if x.num_components() == 1 => x,
        _ => return mesh.clone(),
    };
    let bb = match mesh.point_data().get_array(b) {
        Some(x) if x.num_components() == 1 => x,
        _ => return mesh.clone(),
    };
    let n = aa.num_tuples().min(bb.num_tuples());
    let mut ba = [0.0f64];
    let mut bb2 = [0.0f64];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            aa.tuple_as_f64(i, &mut ba);
            bb.tuple_as_f64(i, &mut bb2);
            op(ba[0], bb2[0])
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, data, 1)));
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0],
                1,
            )));
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![10.0, 20.0, 30.0],
                1,
            )));
        let r = add_arrays(&m, "a", "b", "c");
        let mut buf = [0.0];
        r.point_data()
            .get_array("c")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 11.0).abs() < 1e-10);
    }
    #[test]
    fn test_mul() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![2.0, 3.0, 4.0],
                1,
            )));
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![5.0, 6.0, 7.0],
                1,
            )));
        let r = multiply_arrays(&m, "a", "b", "c");
        let mut buf = [0.0];
        r.point_data()
            .get_array("c")
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 10.0).abs() < 1e-10);
    }
}
