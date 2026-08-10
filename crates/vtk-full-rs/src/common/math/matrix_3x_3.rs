use crate::common::core::{
    math::{determinant2x2, determinant3x3_from_values},
    object::Object,
    vtk_type::VtkMTimeType,
};

/// VTK: `vtkMatrix3x3`.
#[derive(Debug, Clone)]
pub struct Matrix3x3 {
    object: Object,
    element: [f64; 9],
}

impl Matrix3x3 {
    /// VTK: `vtkMatrix3x3::New`.
    pub fn new() -> Self {
        let mut element = [0.0; 9];
        Self::identity_elements(&mut element);
        Self {
            object: Object::with_class_name("vtkMatrix3x3"),
            element,
        }
    }

    /// VTK: `vtkMatrix3x3::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut out = format!("{}\nElements:\n", self.object.get_class_name());
        for i in 0..3 {
            out.push('\t');
            for j in 0..3 {
                if j > 0 {
                    out.push('\t');
                }
                out.push_str(&self.element[3 * i + j].to_string());
            }
            if i < 2 {
                out.push('\n');
            }
        }
        out
    }

    /// VTK: `vtkMatrix3x3::DeepCopy(vtkMatrix3x3*)`.
    pub fn deep_copy(&mut self, source: &Self) {
        Self::deep_copy_elements(&mut self.element, source.get_data());
        self.object.modified();
    }

    /// VTK: `vtkMatrix3x3::DeepCopy(const double[9])`.
    pub fn deep_copy_from_elements(&mut self, elements: &[f64; 9]) {
        Self::deep_copy_elements(&mut self.element, elements);
        self.object.modified();
    }

    /// VTK: `vtkMatrix3x3::DeepCopy(double[9], const double[9])`.
    pub fn deep_copy_elements(elements: &mut [f64; 9], new_elements: &[f64; 9]) {
        elements.copy_from_slice(new_elements);
    }

    /// VTK: `vtkMatrix3x3::Zero`.
    pub fn zero(&mut self) {
        Self::zero_elements(&mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix3x3::Zero(double[9])`.
    pub fn zero_elements(elements: &mut [f64; 9]) {
        for element in elements {
            *element = 0.0;
        }
    }

    /// VTK: `vtkMatrix3x3::Identity`.
    pub fn identity(&mut self) {
        Self::identity_elements(&mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix3x3::Identity(double[9])`.
    pub fn identity_elements(elements: &mut [f64; 9]) {
        elements[0] = 1.0;
        elements[4] = 1.0;
        elements[8] = 1.0;
        elements[1] = 0.0;
        elements[2] = 0.0;
        elements[3] = 0.0;
        elements[5] = 0.0;
        elements[6] = 0.0;
        elements[7] = 0.0;
    }

    /// VTK: `vtkMatrix3x3::Invert`.
    pub fn invert(&mut self) {
        let input = self.element;
        Self::invert_elements(&input, &mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix3x3::Invert(vtkMatrix3x3*, vtkMatrix3x3*)`.
    pub fn invert_matrix(in_matrix: &Self, out_matrix: &mut Self) {
        Self::invert_elements(in_matrix.get_data(), &mut out_matrix.element);
        out_matrix.object.modified();
    }

    /// VTK: `vtkMatrix3x3::Invert(const double[9], double[9])`.
    pub fn invert_elements(in_elements: &[f64; 9], out_elements: &mut [f64; 9]) {
        let det = Self::determinant_elements(in_elements);
        if det == 0.0 {
            return;
        }

        Self::adjoint_elements(in_elements, out_elements);
        for element in out_elements {
            *element /= det;
        }
    }

    /// VTK: `vtkMatrix3x3::Transpose`.
    pub fn transpose(&mut self) {
        let input = self.element;
        Self::transpose_elements(&input, &mut self.element);
        self.object.modified();
    }

    /// VTK: `vtkMatrix3x3::Transpose(vtkMatrix3x3*, vtkMatrix3x3*)`.
    pub fn transpose_matrix(in_matrix: &Self, out_matrix: &mut Self) {
        Self::transpose_elements(in_matrix.get_data(), &mut out_matrix.element);
        out_matrix.object.modified();
    }

    /// VTK: `vtkMatrix3x3::Transpose(const double[9], double[9])`.
    pub fn transpose_elements(in_elements: &[f64; 9], out_elements: &mut [f64; 9]) {
        for i in 0..3 {
            for j in i..3 {
                let temp = in_elements[3 * i + j];
                out_elements[3 * i + j] = in_elements[3 * j + i];
                out_elements[3 * j + i] = temp;
            }
        }
    }

    /// VTK: `vtkMatrix3x3::MultiplyPoint(const double[3], double[3])`.
    pub fn multiply_point(&self, in_point: &[f64; 3], out_point: &mut [f64; 3]) {
        Self::multiply_point_elements(self.get_data(), in_point, out_point);
    }

    /// VTK: `vtkMatrix3x3::MultiplyPoint(const float[3], float[3])`.
    pub fn multiply_point_f32(&self, in_point: &[f32; 3], out_point: &mut [f32; 3]) {
        Self::multiply_point_elements_f32(self.get_data(), in_point, out_point);
    }

    /// VTK: `vtkMatrix3x3::MultiplyPoint(const double[9], const double[3], double[3])`.
    pub fn multiply_point_elements(
        elements: &[f64; 9],
        in_point: &[f64; 3],
        out_point: &mut [f64; 3],
    ) {
        let v1 = in_point[0];
        let v2 = in_point[1];
        let v3 = in_point[2];

        out_point[0] = v1 * elements[0] + v2 * elements[1] + v3 * elements[2];
        out_point[1] = v1 * elements[3] + v2 * elements[4] + v3 * elements[5];
        out_point[2] = v1 * elements[6] + v2 * elements[7] + v3 * elements[8];
    }

    /// VTK: `vtkMatrix3x3::MultiplyPoint(const double[9], const float[3], float[3])`.
    pub fn multiply_point_elements_f32(
        elements: &[f64; 9],
        in_point: &[f32; 3],
        out_point: &mut [f32; 3],
    ) {
        let v1 = in_point[0] as f64;
        let v2 = in_point[1] as f64;
        let v3 = in_point[2] as f64;

        out_point[0] = (v1 * elements[0] + v2 * elements[1] + v3 * elements[2]) as f32;
        out_point[1] = (v1 * elements[3] + v2 * elements[4] + v3 * elements[5]) as f32;
        out_point[2] = (v1 * elements[6] + v2 * elements[7] + v3 * elements[8]) as f32;
    }

    /// VTK: `vtkMatrix3x3::Multiply3x3(vtkMatrix3x3*, vtkMatrix3x3*, vtkMatrix3x3*)`.
    pub fn multiply_3x3(a: &Self, b: &Self, c: &mut Self) {
        Self::multiply_3x3_elements(a.get_data(), b.get_data(), &mut c.element);
    }

    /// VTK: `vtkMatrix3x3::Multiply3x3(const double[9], const double[9], double[9])`.
    pub fn multiply_3x3_elements(a: &[f64; 9], b: &[f64; 9], c: &mut [f64; 9]) {
        let mut accum = [0.0; 9];
        for i in (0..9).step_by(3) {
            for k in 0..3 {
                accum[i + k] = a[i] * b[k] + a[i + 1] * b[k + 3] + a[i + 2] * b[k + 6];
            }
        }
        c.copy_from_slice(&accum);
    }

    /// VTK: `vtkMatrix3x3::Adjoint(vtkMatrix3x3*, vtkMatrix3x3*)`.
    pub fn adjoint(in_matrix: &Self, out_matrix: &mut Self) {
        Self::adjoint_elements(in_matrix.get_data(), &mut out_matrix.element);
    }

    /// VTK: `vtkMatrix3x3::Adjoint(const double[9], double[9])`.
    pub fn adjoint_elements(in_elements: &[f64; 9], out_elements: &mut [f64; 9]) {
        let a1 = in_elements[0];
        let b1 = in_elements[1];
        let c1 = in_elements[2];
        let a2 = in_elements[3];
        let b2 = in_elements[4];
        let c2 = in_elements[5];
        let a3 = in_elements[6];
        let b3 = in_elements[7];
        let c3 = in_elements[8];

        out_elements[0] = determinant2x2(b2, b3, c2, c3);
        out_elements[3] = -determinant2x2(a2, a3, c2, c3);
        out_elements[6] = determinant2x2(a2, a3, b2, b3);

        out_elements[1] = -determinant2x2(b1, b3, c1, c3);
        out_elements[4] = determinant2x2(a1, a3, c1, c3);
        out_elements[7] = -determinant2x2(a1, a3, b1, b3);

        out_elements[2] = determinant2x2(b1, b2, c1, c2);
        out_elements[5] = -determinant2x2(a1, a2, c1, c2);
        out_elements[8] = determinant2x2(a1, a2, b1, b2);
    }

    /// VTK: `vtkMatrix3x3::Determinant`.
    pub fn determinant(&self) -> f64 {
        Self::determinant_elements(&self.element)
    }

    /// VTK: `vtkMatrix3x3::Determinant(const double[9])`.
    pub fn determinant_elements(elements: &[f64; 9]) -> f64 {
        determinant3x3_from_values(
            elements[0],
            elements[1],
            elements[2],
            elements[3],
            elements[4],
            elements[5],
            elements[6],
            elements[7],
            elements[8],
        )
    }

    /// VTK: `vtkMatrix3x3::SetElement`.
    pub fn set_element(&mut self, i: i32, j: i32, value: f64) {
        let index = matrix_index(i, j);
        if self.element[index] != value {
            self.element[index] = value;
            self.object.modified();
        }
    }

    /// VTK: `vtkMatrix3x3::GetElement`.
    pub fn get_element(&self, i: i32, j: i32) -> f64 {
        self.element[matrix_index(i, j)]
    }

    /// VTK: `vtkMatrix3x3::IsIdentity`.
    pub fn is_identity(&self) -> bool {
        let m = self.element;
        m[0] == 1.0
            && m[4] == 1.0
            && m[8] == 1.0
            && m[1] == 0.0
            && m[2] == 0.0
            && m[3] == 0.0
            && m[5] == 0.0
            && m[6] == 0.0
            && m[7] == 0.0
    }

    /// VTK: `vtkMatrix3x3::GetData`.
    pub fn get_data(&self) -> &[f64; 9] {
        &self.element
    }

    /// VTK: `vtkMatrix3x3::GetData`.
    pub fn get_data_mut(&mut self) -> &mut [f64; 9] {
        &mut self.element
    }

    /// VTK: `vtkMatrix3x3::SetData`.
    pub fn set_data(&mut self, data: &[f64; 9]) {
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

    /// VTK: `vtkMatrix3x3::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkMatrix3x3" || Object::is_type_of(name)
    }

    /// VTK: `vtkMatrix3x3::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Self::new()
    }
}

fn matrix_index(i: i32, j: i32) -> usize {
    (3 * i + j) as usize
}
