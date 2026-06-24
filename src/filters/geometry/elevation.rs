use crate::data::{DataArray, DataSet, PolyData};
use rayon::prelude::*;

/// Compute an elevation scalar for each point, measuring distance along an axis.
///
/// The scalar is the projection of each point onto the line from `low_point` to
/// `high_point`. Values are in `[0, 1]` if points lie between the two endpoints.
pub fn elevation(input: &PolyData, low_point: [f64; 3], high_point: [f64; 3]) -> PolyData {
    let mut output = input.clone();

    let dir = [
        high_point[0] - low_point[0],
        high_point[1] - low_point[1],
        high_point[2] - low_point[2],
    ];
    let len2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
    let inv_len2 = if len2 > 1e-20 { 1.0 / len2 } else { 0.0 };

    // Direct flat slice access + precomputed inverse avoids per-point overhead.
    let pts = input.points.as_flat_slice();
    let n = input.points.len();
    let mut values = Vec::with_capacity(n);

    for i in 0..n {
        let base = i * 3;
        let dx = pts[base] - low_point[0];
        let dy = pts[base + 1] - low_point[1];
        let dz = pts[base + 2] - low_point[2];
        values.push((dx * dir[0] + dy * dir[1] + dz * dir[2]) * inv_len2);
    }

    let scalars = DataArray::<f64>::from_vec("Elevation", values, 1);
    output.point_data_mut().add_array(scalars.into());
    output.point_data_mut().set_active_scalars("Elevation");
    output
}

/// Parallel version of `elevation` using rayon.
pub fn elevation_par(input: &PolyData, low_point: [f64; 3], high_point: [f64; 3]) -> PolyData {
    let mut output = input.clone();

    let dir = [
        high_point[0] - low_point[0],
        high_point[1] - low_point[1],
        high_point[2] - low_point[2],
    ];
    let len2 = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];

    let values: Vec<f64> = (0..input.num_points())
        .into_par_iter()
        .map(|i| {
            let p = input.point(i);
            let dp = [
                p[0] - low_point[0],
                p[1] - low_point[1],
                p[2] - low_point[2],
            ];
            if len2 > 1e-20 {
                (dp[0] * dir[0] + dp[1] * dir[1] + dp[2] * dir[2]) / len2
            } else {
                0.0
            }
        })
        .collect();

    let scalars = DataArray::<f64>::from_vec("Elevation", values, 1);
    output.point_data_mut().add_array(scalars.into());
    output.point_data_mut().set_active_scalars("Elevation");
    output
}

/// Convenience: compute elevation along the Z axis using the data's bounding box.
pub fn elevation_z(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    elevation(input, [0.0, 0.0, bb.z_min], [0.0, 0.0, bb.z_max])
}

/// Convenience: compute elevation along the Y axis using the data's bounding box.
pub fn elevation_y(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    elevation(input, [0.0, bb.y_min, 0.0], [0.0, bb.y_max, 0.0])
}

/// Convenience: compute elevation along the X axis using the data's bounding box.
pub fn elevation_x(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    elevation(input, [bb.x_min, 0.0, 0.0], [bb.x_max, 0.0, 0.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevation_along_z() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 5.0], [0.0, 1.0, 10.0]],
            vec![[0, 1, 2]],
        );

        let result = elevation(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 10.0]);
        let scalars = result.point_data().scalars().unwrap();

        let mut buf = [0.0f64];
        scalars.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);

        scalars.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-10);

        scalars.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn elevation_z_auto() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 5.0], [0.0, 1.0, 10.0]],
            vec![[0, 1, 2]],
        );

        let result = elevation_z(&pd);
        let scalars = result.point_data().scalars().unwrap();

        let mut buf = [0.0f64];
        scalars.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);

        scalars.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);
    }
}
