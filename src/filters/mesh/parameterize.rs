use crate::data::{AnyDataArray, DataArray, DataSet, PolyData};

/// Simple planar UV parameterization.
///
/// Projects mesh vertices onto a plane to generate texture coordinates.
/// The plane is determined by the bounding box: maps X to U and Y to V
/// in [0, 1] range. Adds "UV" 2-component point data.
pub fn planar_parameterize(input: &PolyData, axis_u: usize, axis_v: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let bb = input.bounds();
    let bounds = [bb.x_min, bb.x_max, bb.y_min, bb.y_max, bb.z_min, bb.z_max];
    let u_min = bounds[axis_u.min(2) * 2];
    let u_max = bounds[axis_u.min(2) * 2 + 1];
    let v_min = bounds[axis_v.min(2) * 2];
    let v_max = bounds[axis_v.min(2) * 2 + 1];
    let u_range = (u_max - u_min).max(1e-15);
    let v_range = (v_max - v_min).max(1e-15);

    let mut uv = Vec::with_capacity(n * 2);
    for i in 0..n {
        let p = input.points.get(i);
        let u = (p[axis_u.min(2)] - u_min) / u_range;
        let v = (p[axis_v.min(2)] - v_min) / v_range;
        uv.push(u);
        uv.push(v);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uv, 2)));
    pd.point_data_mut().set_active_tcoords("UV");
    pd
}

/// Cylindrical UV parameterization around the Y axis.
///
/// U = atan2(z, x) / (2π) + 0.5, V = (y - y_min) / (y_max - y_min).
pub fn cylindrical_parameterize(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let bb = input.bounds();
    let y_range = (bb.y_max - bb.y_min).max(1e-15);

    let mut uv = Vec::with_capacity(n * 2);
    for i in 0..n {
        let p = input.points.get(i);
        let u = p[2].atan2(p[0]) / (2.0 * std::f64::consts::PI) + 0.5;
        let v = (p[1] - bb.y_min) / y_range;
        uv.push(u);
        uv.push(v);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uv, 2)));
    pd.point_data_mut().set_active_tcoords("UV");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planar_uv() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);

        let result = planar_parameterize(&pd, 0, 1); // X->U, Y->V
        let arr = result.point_data().get_array("UV").unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [0.0, 0.0]);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf, [1.0, 1.0]);
    }

    #[test]
    fn cylindrical_uv() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]); // theta=0
        pd.points.push([0.0, 1.0, 1.0]); // theta=pi/2

        let result = cylindrical_parameterize(&pd);
        let arr = result.point_data().get_array("UV").unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10); // theta=0 -> u=0.5
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = planar_parameterize(&pd, 0, 1);
        assert_eq!(result.points.len(), 0);
    }
}
