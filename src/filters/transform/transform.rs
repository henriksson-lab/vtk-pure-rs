use crate::data::PolyData;

/// A 4x4 transformation matrix in column-major order.
pub type Matrix4 = [[f64; 4]; 4];

/// Identity matrix.
pub const IDENTITY: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Create a translation matrix.
pub fn translation(tx: f64, ty: f64, tz: f64) -> Matrix4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx, ty, tz, 1.0],
    ]
}

/// Create a uniform scale matrix.
pub fn scale(sx: f64, sy: f64, sz: f64) -> Matrix4 {
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Create a rotation matrix around the X axis (angle in radians).
pub fn rotate_x(angle: f64) -> Matrix4 {
    let c = angle.cos();
    let s = angle.sin();
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, s, 0.0],
        [0.0, -s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Create a rotation matrix around the Y axis (angle in radians).
pub fn rotate_y(angle: f64) -> Matrix4 {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, 0.0, -s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Create a rotation matrix around the Z axis (angle in radians).
pub fn rotate_z(angle: f64) -> Matrix4 {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, s, 0.0, 0.0],
        [-s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Multiply two 4x4 matrices (column-major): result = a * b.
pub fn mul(a: &Matrix4, b: &Matrix4) -> Matrix4 {
    let mut result = [[0.0; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k][row] * b[col][k];
            }
            result[col][row] = sum;
        }
    }
    result
}

/// Apply a 4x4 transformation matrix to all points (and normals) of a PolyData.
pub fn transform(input: &PolyData, matrix: &Matrix4) -> PolyData {
    let mut output = input.clone();

    // Transform points directly on flat slice — avoids per-point get/set overhead.
    // Matrix multiply is inlined; remaining cost is dominated by clone().
    let m = matrix;
    let pts = output.points.as_flat_slice_mut();
    let n = pts.len() / 3;
    for i in 0..n {
        let base = i * 3;
        let (x, y, z) = (pts[base], pts[base + 1], pts[base + 2]);
        pts[base] = m[0][0] * x + m[1][0] * y + m[2][0] * z + m[3][0];
        pts[base + 1] = m[0][1] * x + m[1][1] * y + m[2][1] * z + m[3][1];
        pts[base + 2] = m[0][2] * x + m[1][2] * y + m[2][2] * z + m[3][2];
    }

    // Transform normals if present
    if let Some(normals_arr) = output.point_data().normals() {
        let nc = normals_arr.num_components();
        let nt = normals_arr.num_tuples();
        if nc == 3 {
            let name = normals_arr.name().to_string();
            let mut flat = vec![0.0f64; nt * 3];
            let mut buf = [0.0f64; 3];
            for i in 0..nt {
                normals_arr.tuple_as_f64(i, &mut buf);
                let tx = m[0][0] * buf[0] + m[1][0] * buf[1] + m[2][0] * buf[2];
                let ty = m[0][1] * buf[0] + m[1][1] * buf[1] + m[2][1] * buf[2];
                let tz = m[0][2] * buf[0] + m[1][2] * buf[1] + m[2][2] * buf[2];
                let len = (tx * tx + ty * ty + tz * tz).sqrt();
                let base = i * 3;
                if len > 1e-10 {
                    let inv = 1.0 / len;
                    flat[base] = tx * inv;
                    flat[base + 1] = ty * inv;
                    flat[base + 2] = tz * inv;
                } else {
                    flat[base + 2] = 1.0;
                }
            }
            let new_normals = crate::data::DataArray::<f64>::from_vec(&name, flat, 3);
            output.point_data_mut().add_array(new_normals.into());
            output.point_data_mut().set_active_normals(&name);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let m = translation(10.0, 20.0, 30.0);
        let result = transform(&pd, &m);

        assert_eq!(result.points.get(0), [10.0, 20.0, 30.0]);
        assert_eq!(result.points.get(1), [11.0, 20.0, 30.0]);
    }

    #[test]
    fn scale_triangle() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let m = scale(2.0, 3.0, 4.0);
        let result = transform(&pd, &m);

        assert_eq!(result.points.get(0), [2.0, 6.0, 12.0]);
    }

    #[test]
    fn compose_transforms() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        // Scale then translate
        let m = mul(&translation(5.0, 0.0, 0.0), &scale(2.0, 2.0, 2.0));
        let result = transform(&pd, &m);

        assert_eq!(result.points.get(0), [5.0, 0.0, 0.0]);
        assert_eq!(result.points.get(1), [7.0, 0.0, 0.0]);
    }

    #[test]
    fn identity_noop() {
        let pd = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let result = transform(&pd, &IDENTITY);

        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
        assert_eq!(result.points.get(2), [7.0, 8.0, 9.0]);
    }
}
