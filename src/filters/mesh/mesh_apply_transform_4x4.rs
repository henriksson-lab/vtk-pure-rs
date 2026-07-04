//! Apply 4x4 transformation matrices.
use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};

pub fn apply_transform(mesh: &PolyData, m: &[[f64; 4]; 4]) -> PolyData {
    let n = mesh.points.len();
    let mut r = mesh.clone();
    for i in 0..n {
        let p = mesh.points.get(i);
        r.points.set(i, transform_point(p, m));
    }

    let normal_matrix = inverse_transpose_3x3(m).unwrap_or([
        [m[0][0], m[0][1], m[0][2]],
        [m[1][0], m[1][1], m[1][2]],
        [m[2][0], m[2][1], m[2][2]],
    ]);
    transform_active_vectors(r.point_data_mut(), m);
    transform_active_normals(r.point_data_mut(), &normal_matrix);
    transform_active_vectors(r.cell_data_mut(), m);
    transform_active_normals(r.cell_data_mut(), &normal_matrix);

    r
}
pub fn rotation_matrix_z(angle: f64) -> [[f64; 4]; 4] {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, -s, 0.0, 0.0],
        [s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn rotation_matrix_y(angle: f64) -> [[f64; 4]; 4] {
    let c = angle.cos();
    let s = angle.sin();
    [
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn rotation_matrix_x(angle: f64) -> [[f64; 4]; 4] {
    let c = angle.cos();
    let s = angle.sin();
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn translation_matrix(tx: f64, ty: f64, tz: f64) -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, tx],
        [0.0, 1.0, 0.0, ty],
        [0.0, 0.0, 1.0, tz],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn scale_matrix(sx: f64, sy: f64, sz: f64) -> [[f64; 4]; 4] {
    [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
pub fn multiply_matrices(a: &[[f64; 4]; 4], b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut r = [[0.0f64; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

fn transform_point(p: [f64; 3], m: &[[f64; 4]; 4]) -> [f64; 3] {
    let x = m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3];
    let y = m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3];
    let z = m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3];
    let w = m[3][0] * p[0] + m[3][1] * p[1] + m[3][2] * p[2] + m[3][3];
    if w != 0.0 && (w - 1.0).abs() > 1e-15 {
        [x / w, y / w, z / w]
    } else {
        [x, y, z]
    }
}

fn transform_active_vectors(attrs: &mut DataSetAttributes, m: &[[f64; 4]; 4]) {
    let Some(vectors) = attrs.vectors() else {
        return;
    };
    let Some(array) = transform_vector_array(vectors, m, false) else {
        return;
    };
    let name = array.name().to_string();
    attrs.add_array(AnyDataArray::F64(array));
    attrs.set_active_vectors(&name);
}

fn transform_active_normals(attrs: &mut DataSetAttributes, normal_matrix: &[[f64; 3]; 3]) {
    let Some(normals) = attrs.normals() else {
        return;
    };
    let Some(array) = transform_normal_array(normals, normal_matrix) else {
        return;
    };
    let name = array.name().to_string();
    attrs.add_array(AnyDataArray::F64(array));
    attrs.set_active_normals(&name);
}

fn transform_vector_array(
    array: &AnyDataArray,
    m: &[[f64; 4]; 4],
    normalize: bool,
) -> Option<DataArray<f64>> {
    if array.num_components() != 3 {
        return None;
    }
    let mut out = Vec::with_capacity(array.num_tuples() * 3);
    let mut buf = [0.0f64; 3];
    for i in 0..array.num_tuples() {
        array.tuple_as_f64(i, &mut buf);
        let mut v = [
            m[0][0] * buf[0] + m[0][1] * buf[1] + m[0][2] * buf[2],
            m[1][0] * buf[0] + m[1][1] * buf[1] + m[1][2] * buf[2],
            m[2][0] * buf[0] + m[2][1] * buf[1] + m[2][2] * buf[2],
        ];
        if normalize {
            normalize_vector(&mut v);
        }
        out.extend_from_slice(&v);
    }
    Some(DataArray::from_vec(array.name(), out, 3))
}

fn transform_normal_array(
    array: &AnyDataArray,
    normal_matrix: &[[f64; 3]; 3],
) -> Option<DataArray<f64>> {
    if array.num_components() != 3 {
        return None;
    }
    let mut out = Vec::with_capacity(array.num_tuples() * 3);
    let mut buf = [0.0f64; 3];
    for i in 0..array.num_tuples() {
        array.tuple_as_f64(i, &mut buf);
        let mut v = [
            normal_matrix[0][0] * buf[0]
                + normal_matrix[0][1] * buf[1]
                + normal_matrix[0][2] * buf[2],
            normal_matrix[1][0] * buf[0]
                + normal_matrix[1][1] * buf[1]
                + normal_matrix[1][2] * buf[2],
            normal_matrix[2][0] * buf[0]
                + normal_matrix[2][1] * buf[1]
                + normal_matrix[2][2] * buf[2],
        ];
        normalize_vector(&mut v);
        out.extend_from_slice(&v);
    }
    Some(DataArray::from_vec(array.name(), out, 3))
}

fn normalize_vector(v: &mut [f64; 3]) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

fn inverse_transpose_3x3(m: &[[f64; 4]; 4]) -> Option<[[f64; 3]; 3]> {
    let a00 = m[0][0];
    let a01 = m[0][1];
    let a02 = m[0][2];
    let a10 = m[1][0];
    let a11 = m[1][1];
    let a12 = m[1][2];
    let a20 = m[2][0];
    let a21 = m[2][1];
    let a22 = m[2][2];

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
        return None;
    }
    let inv_det = 1.0 / det;

    Some([
        [c00 * inv_det, c01 * inv_det, c02 * inv_det],
        [c10 * inv_det, c11 * inv_det, c12 * inv_det],
        [c20 * inv_det, c21 * inv_det, c22 * inv_det],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test_translate() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let t = translation_matrix(10.0, 20.0, 30.0);
        let r = apply_transform(&m, &t);
        let p = r.points.get(0);
        assert!((p[0] - 10.0).abs() < 1e-10);
    }
    #[test]
    fn test_rotate() {
        let m = PolyData::from_triangles(
            vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let rz = rotation_matrix_z(std::f64::consts::FRAC_PI_2);
        let r = apply_transform(&m, &rz);
        let p = r.points.get(0);
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }
    #[test]
    fn test_compose() {
        let t = translation_matrix(5.0, 0.0, 0.0);
        let s = scale_matrix(2.0, 2.0, 2.0);
        let c = multiply_matrices(&t, &s);
        assert!((c[0][3] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn transforms_active_vectors_and_normals() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Velocity",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                3,
            )));
        m.point_data_mut().set_active_vectors("Velocity");
        m.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        m.cell_data_mut().set_active_normals("Normals");

        let r = apply_transform(&m, &scale_matrix(2.0, 3.0, 4.0));
        let mut buf = [0.0; 3];
        r.point_data().vectors().unwrap().tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [2.0, 0.0, 0.0]);
        r.cell_data().normals().unwrap().tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn transforms_normals_with_inverse_transpose() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![1.0, 0.0, 0.0],
                3,
            )));
        mesh.point_data_mut().set_active_normals("Normals");

        let shear = [
            [1.0, 1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let result = apply_transform(&mesh, &shear);

        let mut normal = [0.0; 3];
        result
            .point_data()
            .normals()
            .unwrap()
            .tuple_as_f64(0, &mut normal);
        let inv_sqrt2 = 1.0 / 2.0f64.sqrt();
        assert!((normal[0] - inv_sqrt2).abs() < 1e-12);
        assert!((normal[1] + inv_sqrt2).abs() < 1e-12);
        assert!(normal[2].abs() < 1e-12);
    }
}
