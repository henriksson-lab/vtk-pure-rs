use crate::common::core::{
    math::determinant3x3_from_values, object::Object, vtk_type::VtkMTimeType,
};

/// VTK: `vtkMatrix4x4`.
#[derive(Debug, Clone)]
pub struct Matrix4x4 {
    object: Object,
    /// VTK: `vtkMatrix4x4::Element`.
    pub element: [f64; 16],
    float_point: [f32; 4],
    double_point: [f64; 4],
}

impl Matrix4x4 {
    /// VTK: `vtkMatrix4x4::New`.
    pub fn new() -> Self {
        let mut element = [0.0; 16];
        Self::identity_elements(&mut element);
        Self {
            object: Object::with_class_name("vtkMatrix4x4"),
            element,
            float_point: [0.0; 4],
            double_point: [0.0; 4],
        }
    }

    /// VTK: `vtkMatrix4x4::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = format!("{}\nElements:\n", self.object.get_class_name());
        for i in 0..4 {
            out.push_str("  ");
            for j in 0..4 {
                out.push_str(&self.element[4 * i + j].to_string());
                out.push(' ');
            }
            if i < 3 {
                out.push('\n');
            }
        }
        out
    }

    /// VTK: `vtkMatrix4x4::DeepCopy(const vtkMatrix4x4*)`.
    pub fn deep_copy(&mut self, source: &Self) {
        Self::deep_copy_elements(&mut self.element, source.get_data());
        self.object.modified();
    }

    /// VTK: `vtkMatrix4x4::DeepCopy(const double[16])`.
    pub fn deep_copy_from_elements(&mut self, elements: &[f64; 16]) {
        Self::deep_copy_elements(&mut self.element, elements);
        self.object.modified();
    }

    /// VTK: `vtkMatrix4x4::DeepCopy(double[16], const vtkMatrix4x4*)`.
    pub fn deep_copy_matrix(destination: &mut [f64; 16], source: &Self) {
        Self::deep_copy_elements(destination, source.get_data());
    }

    /// VTK: `vtkMatrix4x4::DeepCopy(double[16], const double[16])`.
    pub fn deep_copy_elements(destination: &mut [f64; 16], source: &[f64; 16]) {
        destination.copy_from_slice(source);
    }

    /// VTK: `vtkMatrix4x4::Zero`.
    pub fn zero(&mut self) {
        Self::zero_elements(&mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix4x4::Zero(double[16])`.
    pub fn zero_elements(elements: &mut [f64; 16]) {
        for element in elements {
            *element = 0.0;
        }
    }

    /// VTK: `vtkMatrix4x4::Identity`.
    pub fn identity(&mut self) {
        Self::identity_elements(&mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix4x4::Identity(double[16])`.
    pub fn identity_elements(elements: &mut [f64; 16]) {
        elements[0] = 1.0;
        elements[5] = 1.0;
        elements[10] = 1.0;
        elements[15] = 1.0;
        elements[1] = 0.0;
        elements[2] = 0.0;
        elements[3] = 0.0;
        elements[4] = 0.0;
        elements[6] = 0.0;
        elements[7] = 0.0;
        elements[8] = 0.0;
        elements[9] = 0.0;
        elements[11] = 0.0;
        elements[12] = 0.0;
        elements[13] = 0.0;
        elements[14] = 0.0;
    }

    /// VTK: `vtkMatrix4x4::IsIdentity`.
    pub fn is_identity(&self) -> bool {
        let m = self.element;
        m[0] == 1.0
            && m[1] == 0.0
            && m[2] == 0.0
            && m[3] == 0.0
            && m[4] == 0.0
            && m[5] == 1.0
            && m[6] == 0.0
            && m[7] == 0.0
            && m[8] == 0.0
            && m[9] == 0.0
            && m[10] == 1.0
            && m[11] == 0.0
            && m[12] == 0.0
            && m[13] == 0.0
            && m[14] == 0.0
            && m[15] == 1.0
    }

    /// VTK: `vtkMatrix4x4::Invert`.
    pub fn invert(&mut self) {
        let input = self.element;
        Self::invert_elements(&input, &mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix4x4::Invert(const vtkMatrix4x4*, vtkMatrix4x4*)`.
    pub fn invert_matrix(in_matrix: &Self, out_matrix: &mut Self) {
        Self::invert_elements(in_matrix.get_data(), &mut out_matrix.element);
        out_matrix.object.modified();
    }

    /// VTK: `vtkMatrix4x4::Invert(const double[16], double[16])`.
    pub fn invert_elements(in_elements: &[f64; 16], out_elements: &mut [f64; 16]) {
        let det = Self::determinant_elements(in_elements);
        if det == 0.0 {
            return;
        }

        Self::adjoint_elements(in_elements, out_elements);
        for element in out_elements {
            *element /= det;
        }
    }

    /// VTK: `vtkMatrix4x4::Transpose`.
    pub fn transpose(&mut self) {
        let input = self.element;
        Self::transpose_elements(&input, &mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix4x4::Transpose(const vtkMatrix4x4*, vtkMatrix4x4*)`.
    pub fn transpose_matrix(in_matrix: &Self, out_matrix: &mut Self) {
        Self::transpose_elements(in_matrix.get_data(), &mut out_matrix.element);
        out_matrix.object.modified();
    }

    /// VTK: `vtkMatrix4x4::Transpose(const double[16], double[16])`.
    pub fn transpose_elements(in_elements: &[f64; 16], out_elements: &mut [f64; 16]) {
        for i in 0..4 {
            for j in i..4 {
                let temp = in_elements[4 * i + j];
                out_elements[4 * i + j] = in_elements[4 * j + i];
                out_elements[4 * j + i] = temp;
            }
        }
    }

    /// VTK: `vtkMatrix4x4::MatrixFromRotation(..., vtkMatrix4x4*)`.
    pub fn matrix_from_rotation(angle: f64, x: f64, y: f64, z: f64, result: &mut Self) {
        Self::matrix_from_rotation_elements(angle, x, y, z, &mut result.element);
    }

    /// VTK: `vtkMatrix4x4::MatrixFromRotation(..., double[16])`.
    pub fn matrix_from_rotation_elements(
        mut angle: f64,
        mut x: f64,
        mut y: f64,
        mut z: f64,
        matrix: &mut [f64; 16],
    ) {
        Self::identity_elements(matrix);

        if angle == 0.0 || (x == 0.0 && y == 0.0 && z == 0.0) {
            return;
        }

        angle = angle.to_radians();
        let w = (0.5 * angle).cos();
        let f = (0.5 * angle).sin() / (x * x + y * y + z * z).sqrt();
        x *= f;
        y *= f;
        z *= f;

        let ww = w * w;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;

        let xy = x * y;
        let xz = x * z;
        let yz = y * z;

        let s = ww - xx - yy - zz;

        matrix[0] = xx * 2.0 + s;
        matrix[4] = (xy + wz) * 2.0;
        matrix[8] = (xz - wy) * 2.0;

        matrix[1] = (xy - wz) * 2.0;
        matrix[5] = yy * 2.0 + s;
        matrix[9] = (yz + wx) * 2.0;

        matrix[2] = (xz + wy) * 2.0;
        matrix[6] = (yz - wx) * 2.0;
        matrix[10] = zz * 2.0 + s;
    }

    /// VTK: `vtkMatrix4x4::PoseToMatrix`.
    pub fn pose_to_matrix(pos: &[f64; 3], ori: &[f64; 4], mat: &mut Self) {
        Self::matrix_from_rotation(ori[0], ori[1], ori[2], ori[3], mat);
        let data = mat.get_data_mut();
        data[3] = pos[0];
        data[7] = pos[1];
        data[11] = pos[2];
    }

    /// VTK: `vtkMatrix4x4::MultiplyPoint(const double[4], double[4])`.
    pub fn multiply_point(&self, in_point: &[f64; 4], out_point: &mut [f64; 4]) {
        Self::multiply_point_elements(self.get_data(), in_point, out_point);
    }

    /// VTK: `vtkMatrix4x4::MultiplyPoint(const float[4], float[4])`.
    pub fn multiply_point_f32(&self, in_point: &[f32; 4], out_point: &mut [f32; 4]) {
        Self::multiply_point_elements_f32(self.get_data(), in_point, out_point);
    }

    /// VTK: `vtkMatrix4x4::MultiplyPoint(const double[9], const double[4], double[4])`.
    pub fn multiply_point_elements(
        elements: &[f64; 16],
        in_point: &[f64; 4],
        out_point: &mut [f64; 4],
    ) {
        let v1 = in_point[0];
        let v2 = in_point[1];
        let v3 = in_point[2];
        let v4 = in_point[3];

        out_point[0] = v1 * elements[0] + v2 * elements[1] + v3 * elements[2] + v4 * elements[3];
        out_point[1] = v1 * elements[4] + v2 * elements[5] + v3 * elements[6] + v4 * elements[7];
        out_point[2] = v1 * elements[8] + v2 * elements[9] + v3 * elements[10] + v4 * elements[11];
        out_point[3] =
            v1 * elements[12] + v2 * elements[13] + v3 * elements[14] + v4 * elements[15];
    }

    /// VTK: `vtkMatrix4x4::MultiplyPoint(const double[16], const float[4], float[4])`.
    pub fn multiply_point_elements_f32(
        elements: &[f64; 16],
        in_point: &[f32; 4],
        out_point: &mut [f32; 4],
    ) {
        let v1 = in_point[0] as f64;
        let v2 = in_point[1] as f64;
        let v3 = in_point[2] as f64;
        let v4 = in_point[3] as f64;

        out_point[0] =
            (v1 * elements[0] + v2 * elements[1] + v3 * elements[2] + v4 * elements[3]) as f32;
        out_point[1] =
            (v1 * elements[4] + v2 * elements[5] + v3 * elements[6] + v4 * elements[7]) as f32;
        out_point[2] =
            (v1 * elements[8] + v2 * elements[9] + v3 * elements[10] + v4 * elements[11]) as f32;
        out_point[3] =
            (v1 * elements[12] + v2 * elements[13] + v3 * elements[14] + v4 * elements[15]) as f32;
    }

    /// VTK: `vtkMatrix4x4::MultiplyPoint(const double[4])`.
    pub fn multiply_double_point(&mut self, in_point: &[f64; 4]) -> &[f64; 4] {
        Self::multiply_point_elements(&self.element, in_point, &mut self.double_point);
        &self.double_point
    }

    /// VTK: `vtkMatrix4x4::MultiplyFloatPoint`.
    pub fn multiply_float_point(&mut self, in_point: &[f32; 4]) -> &[f32; 4] {
        Self::multiply_point_elements_f32(&self.element, in_point, &mut self.float_point);
        &self.float_point
    }

    /// VTK: `vtkMatrix4x4::Multiply4x4(const vtkMatrix4x4*, const vtkMatrix4x4*, vtkMatrix4x4*)`.
    pub fn multiply_4x4(a: &Self, b: &Self, c: &mut Self) {
        Self::multiply_4x4_elements(a.get_data(), b.get_data(), &mut c.element);
    }

    /// VTK: `vtkMatrix4x4::Multiply4x4(const double[16], const double[16], double[16])`.
    pub fn multiply_4x4_elements(a: &[f64; 16], b: &[f64; 16], c: &mut [f64; 16]) {
        let mut tmp = [0.0; 16];
        for i in (0..16).step_by(4) {
            for j in 0..4 {
                tmp[i + j] =
                    a[i] * b[j] + a[i + 1] * b[j + 4] + a[i + 2] * b[j + 8] + a[i + 3] * b[j + 12];
            }
        }
        c.copy_from_slice(&tmp);
    }

    /// VTK: `vtkMatrix4x4::Multiply4x4(const double[16], const double[16], float[16])`.
    pub fn multiply_4x4_elements_f32(a: &[f64; 16], b: &[f64; 16], c: &mut [f32; 16]) {
        for i in (0..16).step_by(4) {
            for j in 0..4 {
                c[i + j] = (a[i] * b[j]
                    + a[i + 1] * b[j + 4]
                    + a[i + 2] * b[j + 8]
                    + a[i + 3] * b[j + 12]) as f32;
            }
        }
    }

    /// VTK: `vtkMatrix4x4::MultiplyAndTranspose4x4`.
    pub fn multiply_and_transpose_4x4(a: &[f64; 16], b: &[f64; 16], c: &mut [f32; 16]) {
        for i in 0..4 {
            for j in 0..4 {
                let it4 = i * 4;
                c[i + j * 4] = (a[it4] * b[j]
                    + a[it4 + 1] * b[j + 4]
                    + a[it4 + 2] * b[j + 8]
                    + a[it4 + 3] * b[j + 12]) as f32;
            }
        }
    }

    /// VTK: `vtkMatrix4x4::Adjoint(const vtkMatrix4x4*, vtkMatrix4x4*)`.
    pub fn adjoint(in_matrix: &Self, out_matrix: &mut Self) {
        Self::adjoint_elements(in_matrix.get_data(), &mut out_matrix.element);
    }

    /// VTK: `vtkMatrix4x4::Adjoint(const double[16], double[16])`.
    pub fn adjoint_elements(elem: &[f64; 16], out_elem: &mut [f64; 16]) {
        let a1 = elem[0];
        let b1 = elem[1];
        let c1 = elem[2];
        let d1 = elem[3];
        let a2 = elem[4];
        let b2 = elem[5];
        let c2 = elem[6];
        let d2 = elem[7];
        let a3 = elem[8];
        let b3 = elem[9];
        let c3 = elem[10];
        let d3 = elem[11];
        let a4 = elem[12];
        let b4 = elem[13];
        let c4 = elem[14];
        let d4 = elem[15];

        out_elem[0] = determinant3x3_from_values(b2, b3, b4, c2, c3, c4, d2, d3, d4);
        out_elem[4] = -determinant3x3_from_values(a2, a3, a4, c2, c3, c4, d2, d3, d4);
        out_elem[8] = determinant3x3_from_values(a2, a3, a4, b2, b3, b4, d2, d3, d4);
        out_elem[12] = -determinant3x3_from_values(a2, a3, a4, b2, b3, b4, c2, c3, c4);

        out_elem[1] = -determinant3x3_from_values(b1, b3, b4, c1, c3, c4, d1, d3, d4);
        out_elem[5] = determinant3x3_from_values(a1, a3, a4, c1, c3, c4, d1, d3, d4);
        out_elem[9] = -determinant3x3_from_values(a1, a3, a4, b1, b3, b4, d1, d3, d4);
        out_elem[13] = determinant3x3_from_values(a1, a3, a4, b1, b3, b4, c1, c3, c4);

        out_elem[2] = determinant3x3_from_values(b1, b2, b4, c1, c2, c4, d1, d2, d4);
        out_elem[6] = -determinant3x3_from_values(a1, a2, a4, c1, c2, c4, d1, d2, d4);
        out_elem[10] = determinant3x3_from_values(a1, a2, a4, b1, b2, b4, d1, d2, d4);
        out_elem[14] = -determinant3x3_from_values(a1, a2, a4, b1, b2, b4, c1, c2, c4);

        out_elem[3] = -determinant3x3_from_values(b1, b2, b3, c1, c2, c3, d1, d2, d3);
        out_elem[7] = determinant3x3_from_values(a1, a2, a3, c1, c2, c3, d1, d2, d3);
        out_elem[11] = -determinant3x3_from_values(a1, a2, a3, b1, b2, b3, d1, d2, d3);
        out_elem[15] = determinant3x3_from_values(a1, a2, a3, b1, b2, b3, c1, c2, c3);
    }

    /// VTK: `vtkMatrix4x4::Determinant`.
    pub fn determinant(&self) -> f64 {
        Self::determinant_elements(&self.element)
    }

    /// VTK: `vtkMatrix4x4::Determinant(const double[16])`.
    pub fn determinant_elements(elem: &[f64; 16]) -> f64 {
        let a1 = elem[0];
        let b1 = elem[1];
        let c1 = elem[2];
        let d1 = elem[3];
        let a2 = elem[4];
        let b2 = elem[5];
        let c2 = elem[6];
        let d2 = elem[7];
        let a3 = elem[8];
        let b3 = elem[9];
        let c3 = elem[10];
        let d3 = elem[11];
        let a4 = elem[12];
        let b4 = elem[13];
        let c4 = elem[14];
        let d4 = elem[15];

        a1 * determinant3x3_from_values(b2, b3, b4, c2, c3, c4, d2, d3, d4)
            - b1 * determinant3x3_from_values(a2, a3, a4, c2, c3, c4, d2, d3, d4)
            + c1 * determinant3x3_from_values(a2, a3, a4, b2, b3, b4, d2, d3, d4)
            - d1 * determinant3x3_from_values(a2, a3, a4, b2, b3, b4, c2, c3, c4)
    }

    /// VTK: `vtkMatrix4x4::SetElement`.
    pub fn set_element(&mut self, i: i32, j: i32, value: f64) {
        let index = matrix_index(i, j);
        if self.element[index] != value {
            self.element[index] = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkMatrix4x4::GetElement`.
    pub fn get_element(&self, i: i32, j: i32) -> f64 {
        self.element[matrix_index(i, j)]
    }

    /// VTK: `vtkMatrix4x4::GetData`.
    pub fn get_data(&self) -> &[f64; 16] {
        &self.element
    }

    /// VTK: `vtkMatrix4x4::GetData`.
    pub fn get_data_mut(&mut self) -> &mut [f64; 16] {
        &mut self.element
    }

    /// VTK: `vtkMatrix4x4::SetData`.
    pub fn set_data(&mut self, data: &[f64; 16]) {
        self.deep_copy_from_elements(data);
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkMatrix4x4::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkMatrix4x4" || Object::is_type_of(name)
    }

    /// VTK: `vtkMatrix4x4::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Self::new()
    }
}

fn matrix_index(i: i32, j: i32) -> usize {
    (4 * i + j) as usize
}
