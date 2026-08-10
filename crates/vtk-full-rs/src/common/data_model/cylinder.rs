use crate::common::core::{Object, VtkMTimeType, VTK_DOUBLE_MAX};

/// VTK: `vtkCylinder`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cylinder {
    object: Object,
    radius: f64,
    center: [f64; 3],
    axis: [f64; 3],
}

impl Cylinder {
    /// VTK: `vtkCylinder::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCylinder"),
            radius: 0.5,
            center: [0.0, 0.0, 0.0],
            axis: [0.0, 1.0, 0.0],
        }
    }

    /// VTK: `vtkCylinder::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let x2c = [
            x[0] - self.center[0],
            x[1] - self.center[1],
            x[2] - self.center[2],
        ];
        let proj = dot(self.axis, x2c);
        dot(x2c, x2c) - proj * proj - self.radius * self.radius
    }

    /// VTK: `vtkCylinder::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3], g: &mut [f64; 3]) {
        let t = self.axis[0] * (x[0] - self.center[0])
            + self.axis[1] * (x[1] - self.center[1])
            + self.axis[2] * (x[2] - self.center[2]);

        let cp = [
            self.center[0] + t * self.axis[0],
            self.center[1] + t * self.axis[1],
            self.center[2] + t * self.axis[2],
        ];

        g[0] = 2.0 * (x[0] - cp[0]);
        g[1] = 2.0 * (x[1] - cp[1]);
        g[2] = 2.0 * (x[2] - cp[2]);
    }

    /// VTK: `vtkCylinder::SetRadius`.
    pub fn set_radius(&mut self, radius: f64) {
        let radius = if radius < 0.0 {
            0.0
        } else if radius > VTK_DOUBLE_MAX {
            VTK_DOUBLE_MAX
        } else {
            radius
        };
        if self.radius != radius {
            self.radius = radius;
            self.modified();
        }
    }

    /// VTK: `vtkCylinder::GetRadius`.
    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    /// VTK: `vtkCylinder::SetCenter`.
    pub fn set_center(&mut self, x: f64, y: f64, z: f64) {
        let center = [x, y, z];
        if self.center != center {
            self.center = center;
            self.modified();
        }
    }

    /// VTK: `vtkCylinder::SetCenter`.
    pub fn set_center_array(&mut self, center: [f64; 3]) {
        self.set_center(center[0], center[1], center[2]);
    }

    /// VTK: `vtkCylinder::GetCenter`.
    pub fn get_center(&self) -> [f64; 3] {
        self.center
    }

    /// VTK: `vtkCylinder::SetAxis`.
    pub fn set_axis(&mut self, ax: f64, ay: f64, az: f64) {
        self.set_axis_array([ax, ay, az]);
    }

    /// VTK: `vtkCylinder::SetAxis`.
    pub fn set_axis_array(&mut self, mut axis: [f64; 3]) {
        if normalize(&mut axis) < f64::EPSILON {
            return;
        }

        if self.axis != axis {
            self.modified();
            self.axis = axis;
        }
    }

    /// VTK: `vtkCylinder::GetAxis`.
    pub fn get_axis(&self) -> [f64; 3] {
        self.axis
    }

    /// VTK: `vtkCylinder::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Center: ( {}, {}, {} )Axis: ( {}, {}, {} )Radius: {}\n",
            self.center[0],
            self.center[1],
            self.center[2],
            self.axis[0],
            self.axis[1],
            self.axis[2],
            self.radius
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

impl Default for Cylinder {
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
