use crate::data::{AnyDataArray, DataArray, Points, PolyData};

/// Displace vertices along their normals by a scalar field.
///
/// For each vertex, moves it along its normal by the value in `array_name`.
/// Positive = outward, negative = inward.
pub fn displace_by_scalar(input: &PolyData, array_name: &str) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    // Compute vertex normals
    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }
    for nm in &mut vnormals {
        let l = (nm[0] * nm[0] + nm[1] * nm[1] + nm[2] * nm[2]).sqrt();
        if l > 1e-15 {
            nm[0] /= l;
            nm[1] /= l;
            nm[2] /= l;
        }
    }

    let mut buf = [0.0f64];
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        arr.tuple_as_f64(i, &mut buf);
        let d = buf[0];
        points.push([
            p[0] + vnormals[i][0] * d,
            p[1] + vnormals[i][1] * d,
            p[2] + vnormals[i][2] * d,
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Displace vertices by a vector field stored as a 3-component array.
pub fn displace_by_vector(input: &PolyData, array_name: &str, scale: f64) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 => a,
        _ => return input.clone(),
    };

    let mut buf = [0.0f64; 3];
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        arr.tuple_as_f64(i, &mut buf);
        points.push([
            p[0] + buf[0] * scale,
            p[1] + buf[1] * scale,
            p[2] + buf[2] * scale,
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displace_outward() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "d",
                vec![0.5, 0.5, 0.5],
                1,
            )));

        let result = displace_by_scalar(&pd, "d");
        let p = result.points.get(0);
        assert!(p[2].abs() > 0.1); // displaced along normal (Z)
    }

    #[test]
    fn displace_vector() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0],
                3,
            )));

        let result = displace_by_vector(&pd, "v", 2.0);
        let p = result.points.get(0);
        assert_eq!(p, [2.0, 4.0, 6.0]);
    }

    #[test]
    fn zero_displacement() {
        let mut pd = PolyData::new();
        pd.points.push([5.0, 5.0, 5.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("d", vec![0.0], 1)));

        let result = displace_by_scalar(&pd, "d");
        assert_eq!(result.points.get(0), [5.0, 5.0, 5.0]);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = displace_by_scalar(&pd, "nope");
        assert_eq!(result.points.len(), 0);
    }
}
