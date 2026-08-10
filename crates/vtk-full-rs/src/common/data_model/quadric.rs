use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkQuadric`.
#[derive(Debug, Clone, PartialEq)]
pub struct Quadric {
    object: Object,
    coefficients: [f64; 10],
}

impl Quadric {
    /// VTK: `vtkQuadric::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkQuadric"),
            coefficients: [1.0; 10],
        }
    }

    /// VTK: `vtkQuadric::SetCoefficients`.
    pub fn set_coefficients(&mut self, coefficients: [f64; 10]) {
        if self.coefficients != coefficients {
            self.coefficients = coefficients;
            self.modified();
        }
    }

    /// VTK: `vtkQuadric::SetCoefficients`.
    pub fn set_coefficients_components(
        &mut self,
        a0: f64,
        a1: f64,
        a2: f64,
        a3: f64,
        a4: f64,
        a5: f64,
        a6: f64,
        a7: f64,
        a8: f64,
        a9: f64,
    ) {
        self.set_coefficients([a0, a1, a2, a3, a4, a5, a6, a7, a8, a9]);
    }

    /// VTK: `vtkQuadric::GetCoefficients`.
    pub fn get_coefficients(&self) -> [f64; 10] {
        self.coefficients
    }

    /// VTK: `vtkQuadric::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let a = self.coefficients;
        a[0] * x[0] * x[0]
            + a[1] * x[1] * x[1]
            + a[2] * x[2] * x[2]
            + a[3] * x[0] * x[1]
            + a[4] * x[1] * x[2]
            + a[5] * x[0] * x[2]
            + a[6] * x[0]
            + a[7] * x[1]
            + a[8] * x[2]
            + a[9]
    }

    /// VTK: `vtkQuadric::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3], g: &mut [f64; 3]) {
        let a = self.coefficients;
        g[0] = 2.0 * a[0] * x[0] + a[3] * x[1] + a[5] * x[2] + a[6];
        g[1] = 2.0 * a[1] * x[1] + a[3] * x[0] + a[4] * x[2] + a[7];
        g[2] = 2.0 * a[2] * x[2] + a[4] * x[1] + a[5] * x[0] + a[8];
    }

    /// VTK: `vtkQuadric::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Coefficients: \n\ta0: {}\n\ta1: {}\n\ta2: {}\n\ta3: {}\n\ta4: {}\n\ta5: {}\n\ta6: {}\n\ta7: {}\n\ta8: {}\n\ta9: {}\n",
            self.coefficients[0],
            self.coefficients[1],
            self.coefficients[2],
            self.coefficients[3],
            self.coefficients[4],
            self.coefficients[5],
            self.coefficients[6],
            self.coefficients[7],
            self.coefficients[8],
            self.coefficients[9]
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl Default for Quadric {
    fn default() -> Self {
        Self::new()
    }
}
