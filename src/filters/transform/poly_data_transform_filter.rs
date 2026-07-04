use crate::data::{AnyDataArray, DataArray, DataSetAttributes, Points, PolyData};

type Matrix3 = [[f64; 3]; 3];

/// Translate all points by a vector.
pub fn translate(input: &PolyData, delta: [f64; 3]) -> PolyData {
    transform_linear(
        input,
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        delta,
    )
}

/// Scale all points relative to a center.
pub fn scale(input: &PolyData, factors: [f64; 3], center: [f64; 3]) -> PolyData {
    let matrix = [
        [factors[0], 0.0, 0.0],
        [0.0, factors[1], 0.0],
        [0.0, 0.0, factors[2]],
    ];
    let offset = [
        center[0] - factors[0] * center[0],
        center[1] - factors[1] * center[1],
        center[2] - factors[2] * center[2],
    ];
    transform_linear(input, matrix, offset)
}

/// Rotate all points around an axis by angle (degrees).
pub fn rotate(input: &PolyData, axis: [f64; 3], angle_deg: f64, center: [f64; 3]) -> PolyData {
    let angle = angle_deg.to_radians();
    let c = angle.cos();
    let s = angle.sin();
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-15 {
        return input.clone();
    }
    let ux = axis[0] / len;
    let uy = axis[1] / len;
    let uz = axis[2] / len;

    // Rodrigues' rotation formula as matrix
    let r = [
        [
            c + ux * ux * (1.0 - c),
            ux * uy * (1.0 - c) - uz * s,
            ux * uz * (1.0 - c) + uy * s,
        ],
        [
            uy * ux * (1.0 - c) + uz * s,
            c + uy * uy * (1.0 - c),
            uy * uz * (1.0 - c) - ux * s,
        ],
        [
            uz * ux * (1.0 - c) - uy * s,
            uz * uy * (1.0 - c) + ux * s,
            c + uz * uz * (1.0 - c),
        ],
    ];

    let offset = [
        center[0] - (r[0][0] * center[0] + r[0][1] * center[1] + r[0][2] * center[2]),
        center[1] - (r[1][0] * center[0] + r[1][1] * center[1] + r[1][2] * center[2]),
        center[2] - (r[2][0] * center[0] + r[2][1] * center[1] + r[2][2] * center[2]),
    ];
    transform_linear(input, r, offset)
}

fn transform_linear(input: &PolyData, matrix: Matrix3, offset: [f64; 3]) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([
            matrix[0][0] * p[0] + matrix[0][1] * p[1] + matrix[0][2] * p[2] + offset[0],
            matrix[1][0] * p[0] + matrix[1][1] * p[1] + matrix[1][2] * p[2] + offset[1],
            matrix[2][0] * p[0] + matrix[2][1] * p[1] + matrix[2][2] * p[2] + offset[2],
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;

    if let Some((name, data)) = transform_active_vectors(input.point_data(), &matrix) {
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, 3)));
        pd.point_data_mut().set_active_vectors(&name);
    }
    if let Some((name, data)) = transform_active_normals(input.point_data(), &matrix) {
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, 3)));
        pd.point_data_mut().set_active_normals(&name);
    }
    if let Some((name, data)) = transform_active_vectors(input.cell_data(), &matrix) {
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, 3)));
        pd.cell_data_mut().set_active_vectors(&name);
    }
    if let Some((name, data)) = transform_active_normals(input.cell_data(), &matrix) {
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, 3)));
        pd.cell_data_mut().set_active_normals(&name);
    }

    pd
}

fn transform_active_vectors(
    attrs: &DataSetAttributes,
    matrix: &Matrix3,
) -> Option<(String, Vec<f64>)> {
    let vectors = attrs.vectors()?;
    let name = vectors.name().to_string();
    Some((name, transform_array(vectors, matrix, false)))
}

fn transform_active_normals(
    attrs: &DataSetAttributes,
    matrix: &Matrix3,
) -> Option<(String, Vec<f64>)> {
    let normals = attrs.normals()?;
    let name = normals.name().to_string();
    let normal_matrix = inverse(matrix).map(transpose).unwrap_or(*matrix);
    Some((name, transform_array(normals, &normal_matrix, true)))
}

fn transform_array(array: &AnyDataArray, matrix: &Matrix3, normalize: bool) -> Vec<f64> {
    let mut out = Vec::with_capacity(array.num_tuples() * 3);
    let mut buf = [0.0f64; 3];
    for i in 0..array.num_tuples() {
        array.tuple_as_f64(i, &mut buf);
        let mut v = [
            matrix[0][0] * buf[0] + matrix[0][1] * buf[1] + matrix[0][2] * buf[2],
            matrix[1][0] * buf[0] + matrix[1][1] * buf[1] + matrix[1][2] * buf[2],
            matrix[2][0] * buf[0] + matrix[2][1] * buf[1] + matrix[2][2] * buf[2],
        ];
        if normalize {
            normalize_vector(&mut v);
        }
        out.extend_from_slice(&v);
    }
    out
}

fn normalize_vector(v: &mut [f64; 3]) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

fn transpose(matrix: Matrix3) -> Matrix3 {
    [
        [matrix[0][0], matrix[1][0], matrix[2][0]],
        [matrix[0][1], matrix[1][1], matrix[2][1]],
        [matrix[0][2], matrix[1][2], matrix[2][2]],
    ]
}

fn inverse(matrix: &Matrix3) -> Option<Matrix3> {
    let det = matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]);
    if det.abs() <= 1e-15 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        [
            (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1]) * inv_det,
            (matrix[0][2] * matrix[2][1] - matrix[0][1] * matrix[2][2]) * inv_det,
            (matrix[0][1] * matrix[1][2] - matrix[0][2] * matrix[1][1]) * inv_det,
        ],
        [
            (matrix[1][2] * matrix[2][0] - matrix[1][0] * matrix[2][2]) * inv_det,
            (matrix[0][0] * matrix[2][2] - matrix[0][2] * matrix[2][0]) * inv_det,
            (matrix[0][2] * matrix[1][0] - matrix[0][0] * matrix[1][2]) * inv_det,
        ],
        [
            (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]) * inv_det,
            (matrix[0][1] * matrix[2][0] - matrix[0][0] * matrix[2][1]) * inv_det,
            (matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]) * inv_det,
        ],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_points() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = translate(&pd, [10.0, 20.0, 30.0]);
        assert_eq!(result.points.get(0), [11.0, 22.0, 33.0]);
    }

    #[test]
    fn scale_from_origin() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = scale(&pd, [2.0, 2.0, 2.0], [0.0, 0.0, 0.0]);
        assert_eq!(result.points.get(0), [2.0, 4.0, 6.0]);
    }

    #[test]
    fn rotate_90_z() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        let result = rotate(&pd, [0.0, 0.0, 1.0], 90.0, [0.0, 0.0, 0.0]);
        let p = result.points.get(0);
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn identity_rotation() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = rotate(&pd, [0.0, 0.0, 1.0], 0.0, [0.0, 0.0, 0.0]);
        let p = result.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 2.0).abs() < 1e-10);
        assert!((p[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn scale_from_center() {
        let mut pd = PolyData::new();
        pd.points.push([2.0, 0.0, 0.0]);
        let result = scale(&pd, [3.0, 1.0, 1.0], [1.0, 0.0, 0.0]);
        // (2-1)*3 + 1 = 4
        assert!((result.points.get(0)[0] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn transform_vectors_and_normals() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Vectors",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        pd.point_data_mut().set_active_vectors("Vectors");
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        pd.point_data_mut().set_active_normals("Normals");

        let result = scale(&pd, [2.0, 1.0, 1.0], [0.0, 0.0, 0.0]);
        let mut buf = [0.0f64; 3];
        result
            .point_data()
            .vectors()
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [2.0, 0.0, 0.0]);
        result
            .point_data()
            .normals()
            .unwrap()
            .tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0, 0.0]);
    }
}
