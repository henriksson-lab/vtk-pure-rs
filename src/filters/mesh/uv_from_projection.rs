use crate::data::{AnyDataArray, DataArray, PolyData};

/// Generate UV coordinates by projecting mesh points onto the XY plane.
///
/// The UVs are normalized to [0,1] based on the bounding box of the projected coordinates.
/// Adds a 2-component "UV" array to point data.
pub fn uv_planar_xy(input: &PolyData) -> PolyData {
    uv_planar(input, 0, 1)
}

/// Generate UV coordinates by projecting mesh points onto the XZ plane.
pub fn uv_planar_xz(input: &PolyData) -> PolyData {
    uv_planar(input, 0, 2)
}

/// Generate UV coordinates by projecting mesh points onto the YZ plane.
pub fn uv_planar_yz(input: &PolyData) -> PolyData {
    uv_planar(input, 1, 2)
}

fn uv_planar(input: &PolyData, axis_u: usize, axis_v: usize) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 {
        let mut pd = input.clone();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "UV",
                Vec::<f64>::new(),
                2,
            )));
        return pd;
    }

    let mut min_u: f64 = f64::MAX;
    let mut max_u: f64 = f64::MIN;
    let mut min_v: f64 = f64::MAX;
    let mut max_v: f64 = f64::MIN;

    for i in 0..n {
        let p = input.points.get(i);
        let u: f64 = p[axis_u];
        let v: f64 = p[axis_v];
        min_u = min_u.min(u);
        max_u = max_u.max(u);
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }

    let range_u: f64 = max_u - min_u;
    let range_v: f64 = max_v - min_v;
    let inv_u: f64 = if range_u > 1e-15 { 1.0 / range_u } else { 0.0 };
    let inv_v: f64 = if range_v > 1e-15 { 1.0 / range_v } else { 0.0 };

    let mut uvs: Vec<f64> = Vec::with_capacity(n * 2);
    for i in 0..n {
        let p = input.points.get(i);
        let u: f64 = (p[axis_u] - min_u) * inv_u;
        let v: f64 = (p[axis_v] - min_v) * inv_v;
        uvs.push(u);
        uvs.push(v);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uvs, 2)));
    pd
}

/// Generate UV coordinates by cylindrical projection around a given axis.
///
/// `axis` is the cylinder axis direction (will be normalized).
/// U is the angle (0..1 for 0..2pi), V is the height along the axis (normalized).
pub fn uv_cylindrical(input: &PolyData, axis: [f64; 3]) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 {
        let mut pd = input.clone();
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "UV",
                Vec::<f64>::new(),
                2,
            )));
        return pd;
    }

    // Normalize axis
    let len: f64 = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let ax: [f64; 3] = if len > 1e-15 {
        [axis[0] / len, axis[1] / len, axis[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    };

    // Build an orthonormal frame: ax, e1, e2
    let arbitrary: [f64; 3] = if ax[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    // e1 = normalize(arbitrary - dot(arbitrary, ax)*ax)
    let dot_arb: f64 = arbitrary[0] * ax[0] + arbitrary[1] * ax[1] + arbitrary[2] * ax[2];
    let e1_raw: [f64; 3] = [
        arbitrary[0] - dot_arb * ax[0],
        arbitrary[1] - dot_arb * ax[1],
        arbitrary[2] - dot_arb * ax[2],
    ];
    let e1_len: f64 =
        (e1_raw[0] * e1_raw[0] + e1_raw[1] * e1_raw[1] + e1_raw[2] * e1_raw[2]).sqrt();
    let e1: [f64; 3] = [e1_raw[0] / e1_len, e1_raw[1] / e1_len, e1_raw[2] / e1_len];
    // e2 = ax cross e1
    let e2: [f64; 3] = [
        ax[1] * e1[2] - ax[2] * e1[1],
        ax[2] * e1[0] - ax[0] * e1[2],
        ax[0] * e1[1] - ax[1] * e1[0],
    ];

    // Compute height and angle for each point
    let mut heights: Vec<f64> = Vec::with_capacity(n);
    let mut angles: Vec<f64> = Vec::with_capacity(n);
    let mut min_h: f64 = f64::MAX;
    let mut max_h: f64 = f64::MIN;

    for i in 0..n {
        let p = input.points.get(i);
        let h: f64 = p[0] * ax[0] + p[1] * ax[1] + p[2] * ax[2];
        let proj1: f64 = p[0] * e1[0] + p[1] * e1[1] + p[2] * e1[2];
        let proj2: f64 = p[0] * e2[0] + p[1] * e2[1] + p[2] * e2[2];
        let angle: f64 = proj2.atan2(proj1);
        heights.push(h);
        angles.push(angle);
        min_h = min_h.min(h);
        max_h = max_h.max(h);
    }

    let range_h: f64 = max_h - min_h;
    let inv_h: f64 = if range_h > 1e-15 { 1.0 / range_h } else { 0.0 };
    let pi2: f64 = 2.0 * std::f64::consts::PI;

    let mut uvs: Vec<f64> = Vec::with_capacity(n * 2);
    for i in 0..n {
        let u: f64 = (angles[i] + std::f64::consts::PI) / pi2; // map [-pi, pi] to [0, 1]
        let v: f64 = (heights[i] - min_h) * inv_h;
        uvs.push(u);
        uvs.push(v);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("UV", uvs, 2)));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad() -> PolyData {
        PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        )
    }

    #[test]
    fn planar_xy_bounds() {
        let pd = make_quad();
        let result = uv_planar_xy(&pd);
        let arr = result.point_data().get_array("UV").unwrap();
        assert_eq!(arr.num_tuples(), 4);
        assert_eq!(arr.num_components(), 2);

        // Point (0,0,0) should map to UV (0,0)
        let mut uv = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut uv);
        assert!(uv[0].abs() < 1e-10);
        assert!(uv[1].abs() < 1e-10);

        // Point (1,1,0) should map to UV (1,1)
        arr.tuple_as_f64(2, &mut uv);
        assert!((uv[0] - 1.0).abs() < 1e-10);
        assert!((uv[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cylindrical_produces_uv() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 1.0, 1.0], [-1.0, 0.0, 2.0]],
            vec![[0, 1, 2]],
        );
        let result = uv_cylindrical(&pd, [0.0, 0.0, 1.0]);
        let arr = result.point_data().get_array("UV").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        assert_eq!(arr.num_components(), 2);

        // V should be normalized: first point at z=0 -> v=0, last at z=2 -> v=1
        let mut uv = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut uv);
        assert!(uv[1].abs() < 1e-10);
        arr.tuple_as_f64(2, &mut uv);
        assert!((uv[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::from_triangles(vec![], vec![]);
        let result = uv_planar_xy(&pd);
        let arr = result.point_data().get_array("UV").unwrap();
        assert_eq!(arr.num_tuples(), 0);
    }
}
