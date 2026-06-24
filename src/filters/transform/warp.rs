use crate::data::{DataSet, PolyData};

/// Warp mesh vertices along the surface normal by the active scalar value.
///
/// Each vertex is displaced as: `p_new = p + normal * scalar * scale_factor`.
/// Requires both normals and scalars in point data.
pub fn warp_by_scalar(input: &PolyData, scale_factor: f64) -> PolyData {
    let mut output = input.clone();

    let normals = match input.point_data().normals() {
        Some(n) if n.num_components() == 3 => n,
        _ => return output,
    };
    let scalars = match input.point_data().scalars() {
        Some(s) => s,
        None => return output,
    };

    let n = input.num_points();
    let mut nbuf = [0.0f64; 3];
    let mut sbuf = [0.0f64];
    let pts = output.points.as_flat_slice_mut();

    for i in 0..n {
        normals.tuple_as_f64(i, &mut nbuf);
        scalars.tuple_as_f64(i, &mut sbuf);
        let d = sbuf[0] * scale_factor;
        let b = i * 3;
        pts[b] += nbuf[0] * d;
        pts[b + 1] += nbuf[1] * d;
        pts[b + 2] += nbuf[2] * d;
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
}
