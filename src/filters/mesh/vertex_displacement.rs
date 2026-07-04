use crate::data::{Points, PolyData};

/// Displace vertices along their normals by a scalar field.
///
/// For each vertex, moves it along its normal by the value in `array_name`.
/// Positive = outward, negative = inward.
pub fn displace_by_scalar(input: &PolyData, array_name: &str) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() >= n => a,
        _ => return input.clone(),
    };

    let vnormals = vertex_normals(input);

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

fn vertex_normals(input: &PolyData) -> Vec<[f64; 3]> {
    let n = input.points.len();
    if let Some(normals) = input.point_data().normals() {
        if normals.num_components() == 3 && normals.num_tuples() >= n {
            let mut vnormals = Vec::with_capacity(n);
            let mut normal = [0.0f64; 3];
            for i in 0..n {
                normals.tuple_as_f64(i, &mut normal);
                vnormals.push(normal);
            }
            return vnormals;
        }
    }

    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let Some(indices) = valid_cell_indices(cell, n) else {
            continue;
        };
        let v0 = input.points.get(indices[0]);
        let v1 = input.points.get(indices[1]);
        let v2 = input.points.get(indices[2]);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let fn_ = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for &i in &indices {
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }
    for strip in input.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            let Some(indices) = valid_cell_indices(&tri, n) else {
                continue;
            };
            accumulate_triangle_normal(input, &indices, &mut vnormals);
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
    vnormals
}

fn accumulate_triangle_normal(input: &PolyData, indices: &[usize], vnormals: &mut [[f64; 3]]) {
    let v0 = input.points.get(indices[0]);
    let v1 = input.points.get(indices[1]);
    let v2 = input.points.get(indices[2]);
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let fn_ = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    for &i in indices {
        vnormals[i][0] += fn_[0];
        vnormals[i][1] += fn_[1];
        vnormals[i][2] += fn_[2];
    }
}

/// Displace vertices by a vector field stored as a 3-component array.
pub fn displace_by_vector(input: &PolyData, array_name: &str, scale: f64) -> PolyData {
    let n = input.points.len();
    let arr = match input.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 3 && a.num_tuples() >= n => a,
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

fn valid_cell_indices(cell: &[i64], npoints: usize) -> Option<Vec<usize>> {
    let mut indices = Vec::with_capacity(cell.len());
    for &id in cell {
        if id < 0 || id as usize >= npoints {
            return None;
        }
        indices.push(id as usize);
    }
    Some(indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

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

    #[test]
    fn scalar_displacement_uses_active_normals() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("d", vec![2.0], 1)));
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "n",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        pd.point_data_mut().set_active_normals("n");

        let result = displace_by_scalar(&pd, "d");
        assert_eq!(result.points.get(0), [2.0, 0.0, 0.0]);
    }

    #[test]
    fn scalar_displacement_handles_triangle_strips() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.strips.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "d",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let result = displace_by_scalar(&pd, "d");
        assert!(result.points.get(0)[2] > 0.9);
    }
}
