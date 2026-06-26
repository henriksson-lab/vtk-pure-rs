use crate::data::{AnyDataArray, DataArray, PolyData};

/// Recompute point normals, replacing any existing "Normals" array.
///
/// Uses polygon cell normals averaged to vertices.
pub fn recompute_normals(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut vnormals = vec![[0.0f64; 3]; n];
    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let fn_ = polygon_normal(input, cell);
        for &id in cell.iter() {
            let i = id as usize;
            vnormals[i][0] += fn_[0];
            vnormals[i][1] += fn_[1];
            vnormals[i][2] += fn_[2];
        }
    }

    let flat: Vec<f64> = vnormals
        .iter()
        .flat_map(|n| {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l > 1e-15 {
                vec![n[0] / l, n[1] / l, n[2] / l]
            } else {
                vec![0.0; 3]
            }
        })
        .collect();

    let mut pd = input.clone();
    // Remove old Normals if present, add new
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() != "Normals" {
            attrs.add_array(a.clone());
        }
    }
    attrs.add_array(AnyDataArray::F64(DataArray::from_vec("Normals", flat, 3)));
    attrs.set_active_normals("Normals");
    *pd.point_data_mut() = attrs;
    pd
}

/// Flip all normals (negate).
pub fn flip_normals(input: &PolyData) -> PolyData {
    let arr = match input.point_data().get_array("Normals") {
        Some(a) if a.num_components() == 3 => a,
        _ => return recompute_and_flip(input),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    let mut flipped = Vec::with_capacity(n * 3);
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        flipped.push(-buf[0]);
        flipped.push(-buf[1]);
        flipped.push(-buf[2]);
    }

    let mut pd = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == "Normals" {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                flipped.clone(),
                3,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    attrs.set_active_normals("Normals");
    *pd.point_data_mut() = attrs;
    pd
}

fn polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    let mut normal = [0.0f64; 3];
    for i in 0..cell.len() {
        let p = input.points.get(cell[i] as usize);
        let q = input.points.get(cell[(i + 1) % cell.len()] as usize);
        normal[0] += (p[1] - q[1]) * (p[2] + q[2]);
        normal[1] += (p[2] - q[2]) * (p[0] + q[0]);
        normal[2] += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len > 1e-15 {
        [normal[0] / len, normal[1] / len, normal[2] / len]
    } else {
        normal
    }
}

fn recompute_and_flip(input: &PolyData) -> PolyData {
    let with_normals = recompute_normals(input);
    flip_normals(&with_normals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recompute_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = recompute_normals(&pd);
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[2] - 1.0).abs() < 0.1); // +Z normal
    }

    #[test]
    fn flip_reverses() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let with_n = recompute_normals(&pd);
        let flipped = flip_normals(&with_n);
        let arr = flipped.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] < -0.5); // flipped to -Z
    }

    #[test]
    fn replaces_existing() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));

        let result = recompute_normals(&pd);
        let arr = result.point_data().get_array("Normals").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.5); // should be +Z, not +X
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = recompute_normals(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
