use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};

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
    output.points = transform_points(&input.points, matrix);

    let normal_matrix = inverse_transpose_3x3(matrix);
    transform_active_vectors(output.point_data_mut(), matrix);
    transform_active_normals(output.point_data_mut(), &normal_matrix);
    transform_active_vectors(output.cell_data_mut(), matrix);
    transform_active_normals(output.cell_data_mut(), &normal_matrix);

    output
}

fn transform_points(
    points: &crate::data::Points<f64>,
    matrix: &Matrix4,
) -> crate::data::Points<f64> {
    let src = points.as_flat_slice();
    let mut dst = Vec::with_capacity(src.len());
    unsafe {
        dst.set_len(src.len());
    }
    let m00 = matrix[0][0];
    let m10 = matrix[1][0];
    let m20 = matrix[2][0];
    let m30 = matrix[3][0];
    let m01 = matrix[0][1];
    let m11 = matrix[1][1];
    let m21 = matrix[2][1];
    let m31 = matrix[3][1];
    let m02 = matrix[0][2];
    let m12 = matrix[1][2];
    let m22 = matrix[2][2];
    let m32 = matrix[3][2];

    let mut i = 0;
    while i < src.len() {
        let x = src[i];
        let y = src[i + 1];
        let z = src[i + 2];
        dst[i] = m00 * x + m10 * y + m20 * z + m30;
        dst[i + 1] = m01 * x + m11 * y + m21 * z + m31;
        dst[i + 2] = m02 * x + m12 * y + m22 * z + m32;
        i += 3;
    }

    crate::data::Points::from_flat_vec(dst)
}

fn transform_active_vectors(attrs: &mut DataSetAttributes, matrix: &Matrix4) {
    let Some(vectors) = attrs.vectors() else {
        return;
    };
    let Some(new_vectors) = transform_vector_array(vectors, matrix, false) else {
        return;
    };
    let name = new_vectors.name().to_string();
    attrs.add_array(new_vectors.into());
    attrs.set_active_vectors(&name);
}

fn transform_active_normals(attrs: &mut DataSetAttributes, normal_matrix: &[[f64; 3]; 3]) {
    let Some(normals) = attrs.normals() else {
        return;
    };
    let Some(new_normals) = transform_normal_array(normals, normal_matrix) else {
        return;
    };
    let name = new_normals.name().to_string();
    attrs.add_array(new_normals.into());
    attrs.set_active_normals(&name);
}

fn transform_vector_array(
    array: &AnyDataArray,
    matrix: &Matrix4,
    normalize: bool,
) -> Option<DataArray<f32>> {
    if array.num_components() != 3 {
        return None;
    }

    if let Some(array) = array.as_f64() {
        return Some(transform_vector_slice(
            array.name(),
            array.as_slice(),
            matrix,
            normalize,
        ));
    }
    if let Some(array) = array.as_f32() {
        return Some(transform_vector_slice(
            array.name(),
            array.as_slice(),
            matrix,
            normalize,
        ));
    }

    let nt = array.num_tuples();
    let mut flat = vec![0.0f32; nt * 3];
    let mut buf = [0.0f64; 3];
    for i in 0..nt {
        array.tuple_as_f64(i, &mut buf);
        let tx = matrix[0][0] * buf[0] + matrix[1][0] * buf[1] + matrix[2][0] * buf[2];
        let ty = matrix[0][1] * buf[0] + matrix[1][1] * buf[1] + matrix[2][1] * buf[2];
        let tz = matrix[0][2] * buf[0] + matrix[1][2] * buf[1] + matrix[2][2] * buf[2];
        write_vector(&mut flat, i, [tx, ty, tz], normalize);
    }

    Some(DataArray::<f32>::from_vec(array.name(), flat, 3))
}

fn transform_vector_slice<T: crate::types::Scalar>(
    name: &str,
    src: &[T],
    matrix: &Matrix4,
    normalize: bool,
) -> DataArray<f32> {
    let mut flat = Vec::with_capacity(src.len());
    unsafe {
        flat.set_len(src.len());
    }
    let m00 = matrix[0][0];
    let m10 = matrix[1][0];
    let m20 = matrix[2][0];
    let m01 = matrix[0][1];
    let m11 = matrix[1][1];
    let m21 = matrix[2][1];
    let m02 = matrix[0][2];
    let m12 = matrix[1][2];
    let m22 = matrix[2][2];

    let mut i = 0;
    while i < src.len() {
        let x = src[i].to_f64();
        let y = src[i + 1].to_f64();
        let z = src[i + 2].to_f64();
        write_vector(
            &mut flat,
            i / 3,
            [
                m00 * x + m10 * y + m20 * z,
                m01 * x + m11 * y + m21 * z,
                m02 * x + m12 * y + m22 * z,
            ],
            normalize,
        );
        i += 3;
    }

    DataArray::<f32>::from_vec(name, flat, 3)
}

fn transform_normal_array(
    array: &AnyDataArray,
    normal_matrix: &[[f64; 3]; 3],
) -> Option<DataArray<f32>> {
    if array.num_components() != 3 {
        return None;
    }

    if let Some(array) = array.as_f64() {
        return Some(transform_normal_slice(
            array.name(),
            array.as_slice(),
            normal_matrix,
        ));
    }
    if let Some(array) = array.as_f32() {
        return Some(transform_normal_slice(
            array.name(),
            array.as_slice(),
            normal_matrix,
        ));
    }

    let nt = array.num_tuples();
    let mut flat = vec![0.0f32; nt * 3];
    let mut buf = [0.0f64; 3];
    for i in 0..nt {
        array.tuple_as_f64(i, &mut buf);
        let tx = normal_matrix[0][0] * buf[0]
            + normal_matrix[0][1] * buf[1]
            + normal_matrix[0][2] * buf[2];
        let ty = normal_matrix[1][0] * buf[0]
            + normal_matrix[1][1] * buf[1]
            + normal_matrix[1][2] * buf[2];
        let tz = normal_matrix[2][0] * buf[0]
            + normal_matrix[2][1] * buf[1]
            + normal_matrix[2][2] * buf[2];
        write_vector(&mut flat, i, [tx, ty, tz], true);
    }

    Some(DataArray::<f32>::from_vec(array.name(), flat, 3))
}

fn transform_normal_slice<T: crate::types::Scalar>(
    name: &str,
    src: &[T],
    normal_matrix: &[[f64; 3]; 3],
) -> DataArray<f32> {
    let mut flat = Vec::with_capacity(src.len());
    unsafe {
        flat.set_len(src.len());
    }
    let m00 = normal_matrix[0][0];
    let m01 = normal_matrix[0][1];
    let m02 = normal_matrix[0][2];
    let m10 = normal_matrix[1][0];
    let m11 = normal_matrix[1][1];
    let m12 = normal_matrix[1][2];
    let m20 = normal_matrix[2][0];
    let m21 = normal_matrix[2][1];
    let m22 = normal_matrix[2][2];

    let mut i = 0;
    while i < src.len() {
        let x = src[i].to_f64();
        let y = src[i + 1].to_f64();
        let z = src[i + 2].to_f64();
        write_vector(
            &mut flat,
            i / 3,
            [
                m00 * x + m01 * y + m02 * z,
                m10 * x + m11 * y + m12 * z,
                m20 * x + m21 * y + m22 * z,
            ],
            true,
        );
        i += 3;
    }

    DataArray::<f32>::from_vec(name, flat, 3)
}

fn write_vector(flat: &mut [f32], idx: usize, v: [f64; 3], normalize: bool) {
    let base = idx * 3;
    write_vector_tuple(&mut flat[base..base + 3], v, normalize);
}

fn write_vector_tuple(out: &mut [f32], v: [f64; 3], normalize: bool) {
    if normalize {
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len > 1e-10 {
            let inv = 1.0 / len;
            out[0] = (v[0] * inv) as f32;
            out[1] = (v[1] * inv) as f32;
            out[2] = (v[2] * inv) as f32;
        } else {
            out[2] = 1.0;
        }
    } else {
        out[0] = v[0] as f32;
        out[1] = v[1] as f32;
        out[2] = v[2] as f32;
    }
}

fn inverse_transpose_3x3(matrix: &Matrix4) -> [[f64; 3]; 3] {
    let a00 = matrix[0][0];
    let a01 = matrix[1][0];
    let a02 = matrix[2][0];
    let a10 = matrix[0][1];
    let a11 = matrix[1][1];
    let a12 = matrix[2][1];
    let a20 = matrix[0][2];
    let a21 = matrix[1][2];
    let a22 = matrix[2][2];

    let c00 = a11 * a22 - a12 * a21;
    let c01 = -(a10 * a22 - a12 * a20);
    let c02 = a10 * a21 - a11 * a20;
    let c10 = -(a01 * a22 - a02 * a21);
    let c11 = a00 * a22 - a02 * a20;
    let c12 = -(a00 * a21 - a01 * a20);
    let c20 = a01 * a12 - a02 * a11;
    let c21 = -(a00 * a12 - a02 * a10);
    let c22 = a00 * a11 - a01 * a10;

    let det = a00 * c00 + a01 * c01 + a02 * c02;
    if det.abs() <= 1e-15 {
        return [
            [matrix[0][0], matrix[0][1], matrix[0][2]],
            [matrix[1][0], matrix[1][1], matrix[1][2]],
            [matrix[2][0], matrix[2][1], matrix[2][2]],
        ];
    }

    let inv_det = 1.0 / det;
    [
        [c00 * inv_det, c01 * inv_det, c02 * inv_det],
        [c10 * inv_det, c11 * inv_det, c12 * inv_det],
        [c20 * inv_det, c21 * inv_det, c22 * inv_det],
    ]
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

    #[test]
    fn transforms_active_vectors_and_cell_normals() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut().add_array(
            crate::data::DataArray::<f64>::from_vec("Vectors", vec![1.0, 0.0, 0.0], 3).into(),
        );
        pd.point_data_mut().set_active_vectors("Vectors");
        pd.cell_data_mut().add_array(
            crate::data::DataArray::<f64>::from_vec("Normals", vec![1.0, 1.0, 0.0], 3).into(),
        );
        pd.cell_data_mut().set_active_normals("Normals");

        let result = transform(&pd, &scale(2.0, 4.0, 1.0));
        let vectors = result.point_data().vectors().unwrap();
        assert!(vectors.as_f32().is_some());
        let mut buf = [0.0; 3];
        vectors.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [2.0, 0.0, 0.0]);

        let normals = result.cell_data().normals().unwrap();
        assert!(normals.as_f32().is_some());
        normals.tuple_as_f64(0, &mut buf);
        let expected = [2.0 / 5.0_f64.sqrt(), 1.0 / 5.0_f64.sqrt(), 0.0];
        assert!((buf[0] - expected[0]).abs() < 1e-6);
        assert!((buf[1] - expected[1]).abs() < 1e-6);
        assert!((buf[2] - expected[2]).abs() < 1e-6);
    }
}
