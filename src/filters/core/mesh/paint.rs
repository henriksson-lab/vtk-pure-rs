use crate::data::{AnyDataArray, DataArray, PolyData};

/// Paint a scalar value onto mesh vertices within a sphere.
///
/// Sets vertices within `radius` of `center` to `value` in the named array.
/// Creates the array if it doesn't exist. Useful for interactive editing.
pub fn paint_sphere(
    input: &PolyData,
    array_name: &str,
    center: [f64; 3],
    radius: f64,
    value: f64,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let r2 = radius * radius;

    // Get or create array
    let mut values: Vec<f64> = if let Some(arr) = input.point_data().get_array(array_name) {
        let mut buf = [0.0f64];
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            })
            .collect()
    } else {
        vec![0.0; n]
    };

    for i in 0..n {
        let p = input.points.get(i);
        let d2 =
            (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2);
        if d2 <= r2 {
            values[i] = value;
        }
    }

    let mut pd = input.clone();
    // Replace or add array
    let mut attrs = crate::data::DataSetAttributes::new();
    let mut found = false;
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                array_name,
                values.clone(),
                1,
            )));
            found = true;
        } else {
            attrs.add_array(a.clone());
        }
    }
    if !found {
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    }
    *pd.point_data_mut() = attrs;
    pd
}

/// Smooth-paint with falloff: value decreases with distance from center.
pub fn paint_sphere_smooth(
    input: &PolyData,
    array_name: &str,
    center: [f64; 3],
    radius: f64,
    value: f64,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }
    let r2 = radius * radius;

    let mut values: Vec<f64> = if let Some(arr) = input.point_data().get_array(array_name) {
        let mut buf = [0.0f64];
        (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            })
            .collect()
    } else {
        vec![0.0; n]
    };

    for i in 0..n {
        let p = input.points.get(i);
        let d2 =
            (p[0] - center[0]).powi(2) + (p[1] - center[1]).powi(2) + (p[2] - center[2]).powi(2);
        if d2 <= r2 {
            if radius <= 0.0 {
                values[i] = value;
            } else {
                let t = 1.0 - (d2 / r2).sqrt(); // falloff
                let t = t * t * (3.0 - 2.0 * t); // smoothstep
                values[i] = values[i] * (1.0 - t) + value * t;
            }
        }
    }

    let mut pd = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    let mut found = false;
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == array_name {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                array_name,
                values.clone(),
                1,
            )));
            found = true;
        } else {
            attrs.add_array(a.clone());
        }
    }
    if !found {
        attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    }
    *pd.point_data_mut() = attrs;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_creates_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);

        let result = paint_sphere(&pd, "paint", [0.0, 0.0, 0.0], 2.0, 1.0);
        let arr = result.point_data().get_array("paint").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // inside
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0); // inside
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0); // outside
    }

    #[test]
    fn smooth_paint_falloff() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.5, 0.0, 0.0]);
        pd.points.push([0.9, 0.0, 0.0]);

        let result = paint_sphere_smooth(&pd, "p", [0.0, 0.0, 0.0], 1.0, 10.0);
        let arr = result.point_data().get_array("p").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        let v0 = buf[0];
        arr.tuple_as_f64(2, &mut buf);
        let v2 = buf[0];
        assert!(v0 > v2); // center gets more than edge
    }

    #[test]
    fn paint_updates_existing() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", vec![5.0], 1)));

        let result = paint_sphere(&pd, "val", [0.0, 0.0, 0.0], 1.0, 99.0);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 99.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = paint_sphere(&pd, "p", [0.0; 3], 1.0, 1.0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn smooth_zero_radius_paints_exact_center() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = paint_sphere_smooth(&pd, "p", [0.0, 0.0, 0.0], 0.0, 7.0);
        let arr = result.point_data().get_array("p").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 7.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
