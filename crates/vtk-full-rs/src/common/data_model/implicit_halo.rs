use crate::common::core::{math::distance2_between_points, Object, VtkMTimeType};

/// VTK: `vtkImplicitHalo`.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplicitHalo {
    object: Object,
    radius: f64,
    center: [f64; 3],
    fade_out: f64,
}

impl ImplicitHalo {
    /// VTK: `vtkImplicitHalo::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkImplicitHalo"),
            radius: 1.0,
            center: [0.0, 0.0, 0.0],
            fade_out: 0.01,
        }
    }

    /// VTK: `vtkImplicitHalo::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let distance = distance2_between_points(self.center, x).sqrt();
        if distance > self.radius {
            0.0
        } else {
            let small_radius = self.radius * (1.0 - self.fade_out);
            if distance <= small_radius {
                1.0
            } else {
                (1.0 - distance / self.radius) / self.fade_out
            }
        }
    }

    /// VTK: `vtkImplicitHalo::EvaluateGradient`.
    pub fn evaluate_gradient(&self, _x: [f64; 3]) -> [f64; 3] {
        panic!("vtkImplicitHalo::EvaluateGradient is not implemented in VTK")
    }

    /// VTK: `vtkImplicitHalo::SetRadius`.
    pub fn set_radius(&mut self, radius: f64) {
        if self.radius != radius {
            self.radius = radius;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitHalo::GetRadius`.
    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    /// VTK: `vtkImplicitHalo::SetCenter`.
    pub fn set_center(&mut self, x: f64, y: f64, z: f64) {
        let center = [x, y, z];
        if self.center != center {
            self.center = center;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitHalo::SetCenter`.
    pub fn set_center_array(&mut self, center: [f64; 3]) {
        self.set_center(center[0], center[1], center[2]);
    }

    /// VTK: `vtkImplicitHalo::GetCenter`.
    pub fn get_center(&self) -> [f64; 3] {
        self.center
    }

    /// VTK: `vtkImplicitHalo::SetFadeOut`.
    pub fn set_fade_out(&mut self, fade_out: f64) {
        if self.fade_out != fade_out {
            self.fade_out = fade_out;
            self.modified();
        }
    }

    /// VTK: `vtkImplicitHalo::GetFadeOut`.
    pub fn get_fade_out(&self) -> f64 {
        self.fade_out
    }

    /// VTK: `vtkImplicitHalo::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Center: {},{},{}\nRadius: {}\nFadeOut: {}\n",
            self.center[0], self.center[1], self.center[2], self.radius, self.fade_out
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

impl Default for ImplicitHalo {
    fn default() -> Self {
        Self::new()
    }
}
