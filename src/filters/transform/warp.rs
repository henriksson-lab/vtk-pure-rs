use crate::data::{AnyDataArray, DataSet, Points, PolyData};

/// Warp mesh vertices along the surface normal by the active scalar value.
///
/// Each vertex is displaced as: `p_new = p + normal * scalar * scale_factor`.
/// If active point normals are absent, uses VTK's default normal `(0, 0, 1)`.
pub fn warp_by_scalar(input: &PolyData, scale_factor: f64) -> PolyData {
    let normals = input
        .point_data()
        .normals()
        .filter(|n| n.num_components() == 3);
    let scalars = match input.point_data().scalars() {
        Some(s) => s,
        None => return input.clone(),
    };

    let n = input.num_points();
    let in_pts = input.points.as_flat_slice();
    let mut out_pts = Vec::with_capacity(in_pts.len());
    unsafe {
        out_pts.set_len(in_pts.len());
    }

    match (normals, scalars) {
        (Some(AnyDataArray::F64(normals)), AnyDataArray::F64(scalars))
            if normals.num_components() == 3 && scalars.num_components() == 1 =>
        {
            let normals = normals.as_slice();
            let scalars = scalars.as_slice();
            for i in 0..n {
                let p = i * 3;
                let d = scalars[i] * scale_factor;
                out_pts[p] = in_pts[p] + normals[p] * d;
                out_pts[p + 1] = in_pts[p + 1] + normals[p + 1] * d;
                out_pts[p + 2] = in_pts[p + 2] + normals[p + 2] * d;
            }
            return warped_output(input, out_pts);
        }
        (None, AnyDataArray::F64(scalars)) if scalars.num_components() == 1 => {
            let scalars = scalars.as_slice();
            for i in 0..n {
                let p = i * 3;
                out_pts[p] = in_pts[p];
                out_pts[p + 1] = in_pts[p + 1];
                out_pts[p + 2] = in_pts[p + 2] + scalars[i] * scale_factor;
            }
            return warped_output(input, out_pts);
        }
        _ => {}
    }

    let mut nbuf = [0.0f64; 3];
    let mut sbuf = [0.0f64];

    for i in 0..n {
        if let Some(normals) = normals {
            normals.tuple_as_f64(i, &mut nbuf);
        } else {
            nbuf = [0.0, 0.0, 1.0];
        }
        scalars.tuple_as_f64(i, &mut sbuf);
        let d = sbuf[0] * scale_factor;
        let b = i * 3;
        out_pts[b] = in_pts[b] + nbuf[0] * d;
        out_pts[b + 1] = in_pts[b + 1] + nbuf[1] * d;
        out_pts[b + 2] = in_pts[b + 2] + nbuf[2] * d;
    }

    warped_output(input, out_pts)
}

fn warped_output(input: &PolyData, points: Vec<f64>) -> PolyData {
    let mut output = input.clone();
    output.points = Points::from_flat_vec(points);
    if let Some(name) = input.point_data().normals().map(|a| a.name().to_string()) {
        output.point_data_mut().remove_array(&name);
    }
    if let Some(name) = input.cell_data().normals().map(|a| a.name().to_string()) {
        output.cell_data_mut().remove_array(&name);
    }
    output
}

/// Warp mesh vertices by a vector field from point data.
///
/// The named array must have 3 components. Each vertex is displaced as:
/// `p_new = p + vector * scale_factor`.
pub fn warp_by_vector(input: &PolyData, array_name: &str, scale_factor: f64) -> PolyData {
    let mut output = input.clone();

    let vectors = match input.point_data().get_array(array_name) {
        Some(v) if v.num_components() == 3 => v,
        _ => return output,
    };

    let n = input.num_points();
    let mut vbuf = [0.0f64; 3];

    for i in 0..n {
        let p = output.points.get(i);
        vectors.tuple_as_f64(i, &mut vbuf);
        output.points.set(
            i,
            [
                p[0] + vbuf[0] * scale_factor,
                p[1] + vbuf[1] * scale_factor,
                p[2] + vbuf[2] * scale_factor,
            ],
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray;

    #[test]
    fn warp_by_scalar_displaces() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        // Add normals pointing in +Z
        let normals = DataArray::from_vec(
            "Normals",
            vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            3,
        );
        pd.point_data_mut().add_array(normals.into());
        pd.point_data_mut().set_active_normals("Normals");

        // Add scalars
        let scalars = DataArray::from_vec("Height", vec![1.0f64, 2.0, 3.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("Height");

        let result = warp_by_scalar(&pd, 0.5);

        // Point 0: z should be 0 + 1.0 * 0.5 = 0.5
        assert!((result.points.get(0)[2] - 0.5).abs() < 1e-10);
        // Point 2: z should be 0 + 3.0 * 0.5 = 1.5
        assert!((result.points.get(2)[2] - 1.5).abs() < 1e-10);
    }

    #[test]
    fn warp_without_data_is_noop() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = warp_by_scalar(&pd, 1.0);
        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn warp_by_scalar_uses_default_z_normal_without_normals() {
        let mut pd = PolyData::from_points(vec![[1.0, 2.0, 3.0]]);
        let scalars = DataArray::from_vec("Height", vec![2.0f64], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("Height");

        let result = warp_by_scalar(&pd, 0.5);

        assert_eq!(result.points.get(0), [1.0, 2.0, 4.0]);
    }
}
