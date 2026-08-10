use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkCone`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cone {
    object: Object,
    angle: f64,
    origin: [f64; 3],
    axis: [f64; 3],
    is_double_cone: bool,
}

impl Cone {
    /// VTK: `vtkCone::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCone"),
            angle: 45.0,
            origin: [0.0, 0.0, 0.0],
            axis: [1.0, 0.0, 0.0],
            is_double_cone: true,
        }
    }

    /// VTK: `vtkCone::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        if !self.is_double_cone && x[0] < 0.0 {
            return -x[0];
        }

        let tan_theta = self.angle.to_radians().tan();
        x[1] * x[1] + x[2] * x[2] - x[0] * x[0] * tan_theta * tan_theta
    }

    /// VTK: `vtkCone::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        if !self.is_double_cone && x[0] < 0.0 {
            return [0.0, 0.0, 0.0];
        }

        let tan_theta = self.angle.to_radians().tan();
        [-2.0 * x[0] * tan_theta * tan_theta, 2.0 * x[1], 2.0 * x[2]]
    }

    /// VTK: `vtkCone::SetAngle`.
    pub fn set_angle(&mut self, angle: f64) {
        let angle = angle.clamp(0.0, 89.0);
        if self.angle != angle {
            self.angle = angle;
            self.modified();
        }
    }

    /// VTK: `vtkCone::GetAngle`.
    pub fn get_angle(&self) -> f64 {
        self.angle
    }

    /// VTK: `vtkCone::SetOrigin`.
    pub fn set_origin(&mut self, x: f64, y: f64, z: f64) {
        let origin = [x, y, z];
        if self.origin != origin {
            self.origin = origin;
            self.update_transform();
        }
    }

    /// VTK: `vtkCone::SetOrigin`.
    pub fn set_origin_array(&mut self, origin: [f64; 3]) {
        self.set_origin(origin[0], origin[1], origin[2]);
    }

    /// VTK: `vtkCone::GetOrigin`.
    pub fn get_origin(&self) -> [f64; 3] {
        self.origin
    }

    /// VTK: `vtkCone::SetAxis`.
    pub fn set_axis(&mut self, x: f64, y: f64, z: f64) {
        self.set_axis_array([x, y, z]);
    }

    /// VTK: `vtkCone::SetAxis`.
    pub fn set_axis_array(&mut self, mut axis: [f64; 3]) {
        if normalize(&mut axis) < f64::EPSILON {
            return;
        }

        if self.axis != axis {
            self.axis = axis;
            self.update_transform();
        }
    }

    /// VTK: `vtkCone::GetAxis`.
    pub fn get_axis(&self) -> [f64; 3] {
        self.axis
    }

    /// VTK: `vtkCone::SetIsDoubleCone`.
    pub fn set_is_double_cone(&mut self, is_double_cone: bool) {
        if self.is_double_cone != is_double_cone {
            self.is_double_cone = is_double_cone;
            self.modified();
        }
    }

    /// VTK: `vtkCone::GetIsDoubleCone`.
    pub fn get_is_double_cone(&self) -> bool {
        self.is_double_cone
    }

    /// VTK: `vtkCone::IsDoubleConeOn`.
    pub fn is_double_cone_on(&mut self) {
        self.set_is_double_cone(true);
    }

    /// VTK: `vtkCone::IsDoubleConeOff`.
    pub fn is_double_cone_off(&mut self) {
        self.set_is_double_cone(false);
    }

    /// VTK: `vtkCone::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "IsDoubleCone: {}\nAngle: {}\nAxis: {} {} {}\nOrigin: {} {} {}\n",
            self.is_double_cone,
            self.angle,
            self.axis[0],
            self.axis[1],
            self.axis[2],
            self.origin[0],
            self.origin[1],
            self.origin[2]
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

    /// VTK: `vtkCone::UpdateTransform`.
    fn update_transform(&mut self) {
        self.modified();
    }
}

impl Default for Cone {
    fn default() -> Self {
        Self::new()
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(a: &mut [f64; 3]) -> f64 {
    let norm = dot(*a, *a).sqrt();
    if norm != 0.0 {
        a[0] /= norm;
        a[1] /= norm;
        a[2] /= norm;
    }
    norm
}
