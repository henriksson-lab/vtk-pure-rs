//! Store vertex Z coordinate as a scalar field (height map).
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn height_map(mesh: &PolyData, axis: usize) -> PolyData {
    let n = mesh.points.len();
    let axis = axis.min(2);
    let heights: Vec<f64> = (0..n).map(|i| mesh.points.get(i)[axis]).collect();
    let mut result = mesh.clone();
    let name = match axis {
        0 => "HeightX",
        1 => "HeightY",
        _ => "HeightZ",
    };
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, heights, 1)));
    result.point_data_mut().set_active_scalars(name);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_height() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 2.0], [0.5, 1.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let r = height_map(&mesh, 2);
        let arr = r.point_data().get_array("HeightZ").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 1.0);
        arr.tuple_as_f64(2, &mut b);
        assert_eq!(b[0], 3.0);
    }
}
