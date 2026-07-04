use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the angle between each face normal and a reference direction.
///
/// Adds "NormalAngle" cell data in degrees. Useful for selecting faces
/// facing a particular direction (e.g., upward-facing for terrain).
pub fn face_normal_angle(input: &PolyData, reference: [f64; 3]) -> PolyData {
    let rlen =
        (reference[0] * reference[0] + reference[1] * reference[1] + reference[2] * reference[2])
            .sqrt();
    let rn = if rlen > 1e-15 {
        [
            reference[0] / rlen,
            reference[1] / rlen,
            reference[2] / rlen,
        ]
    } else {
        [0.0, 0.0, 1.0]
    };

    let mut angles = Vec::new();
    for cell in input.polys.iter() {
        let fn_ = compute_polygon_normal(input, cell);
        let fl = (fn_[0] * fn_[0] + fn_[1] * fn_[1] + fn_[2] * fn_[2]).sqrt();
        if fl < 1e-15 {
            angles.push(90.0);
            continue;
        }
        let dot = (fn_[0] * rn[0] + fn_[1] * rn[1] + fn_[2] * rn[2]) / fl;
        angles.push(dot.clamp(-1.0, 1.0).acos().to_degrees());
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "NormalAngle",
            angles,
            1,
        )));
    pd
}

/// Compute face normal dot product with a direction. Adds "NormalDot" cell data.
pub fn face_normal_dot(input: &PolyData, direction: [f64; 3]) -> PolyData {
    let dlen =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    let dn = if dlen > 1e-15 {
        [
            direction[0] / dlen,
            direction[1] / dlen,
            direction[2] / dlen,
        ]
    } else {
        [0.0, 0.0, 1.0]
    };

    let mut dots = Vec::new();
    for cell in input.polys.iter() {
        let fn_ = compute_polygon_normal(input, cell);
        let fl = (fn_[0] * fn_[0] + fn_[1] * fn_[1] + fn_[2] * fn_[2]).sqrt();
        dots.push(if fl > 1e-15 {
            (fn_[0] * dn[0] + fn_[1] * dn[1] + fn_[2] * dn[2]) / fl
        } else {
            0.0
        });
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("NormalDot", dots, 1)));
    pd
}

fn compute_polygon_normal(input: &PolyData, cell: &[i64]) -> [f64; 3] {
    if cell.len() < 3 {
        return [0.0; 3];
    }

    let mut common = None;
    let mut point_id = 0;
    let mut v1 = [0.0; 3];
    while point_id < cell.len() - 2 {
        let p0 = input.points.get(cell[point_id] as usize);
        let p1 = input.points.get(cell[point_id + 1] as usize);
        v1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        if squared_norm(v1) > 0.0 {
            common = Some(point_id);
            point_id += 2;
            break;
        }
        point_id += 1;
    }

    let Some(common_id) = common else {
        return [0.0; 3];
    };
    if point_id >= cell.len() {
        return [0.0; 3];
    }

    let p0 = input.points.get(cell[common_id] as usize);
    let mut n = [0.0; 3];
    while point_id < cell.len() {
        let p = input.points.get(cell[point_id] as usize);
        let v2 = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        n[0] += cross[0];
        n[1] += cross[1];
        n[2] += cross[2];
        v1 = v2;
        point_id += 1;
    }

    let len = squared_norm(n).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0; 3]
    }
}

fn squared_norm(v: [f64; 3]) -> f64 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upward_face_zero_angle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = face_normal_angle(&pd, [0.0, 0.0, 1.0]);
        let arr = result.cell_data().get_array("NormalAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 5.0); // nearly 0 degrees from +Z
    }

    #[test]
    fn perpendicular_90() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // normal = +Z

        let result = face_normal_angle(&pd, [1.0, 0.0, 0.0]); // reference = +X
        let arr = result.cell_data().get_array("NormalAngle").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 90.0).abs() < 5.0);
    }

    #[test]
    fn dot_product() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = face_normal_dot(&pd, [0.0, 0.0, 1.0]);
        let arr = result.cell_data().get_array("NormalDot").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.1); // aligned with +Z
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(face_normal_angle(&pd, [0.0, 0.0, 1.0]).polys.num_cells(), 0);
    }

    #[test]
    fn skips_initial_collinear_vertices() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([2.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = face_normal_dot(&pd, [0.0, 0.0, 1.0]);
        let arr = result.cell_data().get_array("NormalDot").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-12);
    }
}
