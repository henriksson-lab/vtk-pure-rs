use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkCoordinateFrame`.
#[derive(Debug, Clone, PartialEq)]
pub struct CoordinateFrame {
    object: Object,
    origin: [f64; 3],
    x_axis: [f64; 3],
    y_axis: [f64; 3],
    z_axis: [f64; 3],
}

impl CoordinateFrame {
    /// VTK: `vtkCoordinateFrame::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkCoordinateFrame"),
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        }
    }

    /// VTK: `vtkCoordinateFrame::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let dw = [
            x[0] - self.origin[0],
            x[1] - self.origin[1],
            x[2] - self.origin[2],
        ];

        let xx = dot3(dw, self.x_axis);
        let yy = dot3(dw, self.y_axis);
        let zz = dot3(dw, self.z_axis);

        let x2 = xx * xx;
        let y2 = yy * yy;
        let z2 = zz * zz;
        let r2 = x2 + y2 + z2;

        let c40 = 0.105_785_546_915_204_31;
        let c44 = 0.625_835_735_449_176_14;
        let y40 = c40 * (35.0 * z2 * z2 - 30.0 * z2 * r2 + 3.0 * r2 * r2) / r2 / r2;
        let y44 = c44 * (x2 * (x2 - 3.0 * y2) - y2 * (3.0 * x2 - y2)) / r2 / r2;

        let w40 = 0.763_762_615_825_973_4;
        let w44 = 0.645_497_224_367_902_8;
        w40 * y40 + w44 * y44
    }

    /// VTK: `vtkCoordinateFrame::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        let fxyz = self.evaluate_function(x);
        let fxdx = self.evaluate_function([x[0] + 1.0e-6, x[1], x[2]]);
        let fydy = self.evaluate_function([x[0], x[1] + 1.0e-6, x[2]]);
        let fzdz = self.evaluate_function([x[0], x[1], x[2] + 1.0e-6]);
        [
            (fxdx - fxyz) / 1.0e-6,
            (fydy - fxyz) / 1.0e-6,
            (fzdz - fxyz) / 1.0e-6,
        ]
    }

    /// VTK: `vtkCoordinateFrame::SetOrigin`.
    pub fn set_origin(&mut self, x: f64, y: f64, z: f64) {
        let origin = [x, y, z];
        if self.origin != origin {
            self.origin = origin;
            self.modified();
        }
    }

    /// VTK: `vtkCoordinateFrame::SetOrigin`.
    pub fn set_origin_array(&mut self, origin: [f64; 3]) {
        self.set_origin(origin[0], origin[1], origin[2]);
    }

    /// VTK: `vtkCoordinateFrame::GetOrigin`.
    pub fn get_origin(&self) -> [f64; 3] {
        self.origin
    }

    /// VTK: `vtkCoordinateFrame::SetXAxis`.
    pub fn set_x_axis(&mut self, x: f64, y: f64, z: f64) {
        let axis = [x, y, z];
        if self.x_axis != axis {
            self.x_axis = axis;
            self.modified();
        }
    }

    /// VTK: `vtkCoordinateFrame::SetXAxis`.
    pub fn set_x_axis_array(&mut self, axis: [f64; 3]) {
        self.set_x_axis(axis[0], axis[1], axis[2]);
    }

    /// VTK: `vtkCoordinateFrame::GetXAxis`.
    pub fn get_x_axis(&self) -> [f64; 3] {
        self.x_axis
    }

    /// VTK: `vtkCoordinateFrame::SetYAxis`.
    pub fn set_y_axis(&mut self, x: f64, y: f64, z: f64) {
        let axis = [x, y, z];
        if self.y_axis != axis {
            self.y_axis = axis;
            self.modified();
        }
    }

    /// VTK: `vtkCoordinateFrame::SetYAxis`.
    pub fn set_y_axis_array(&mut self, axis: [f64; 3]) {
        self.set_y_axis(axis[0], axis[1], axis[2]);
    }

    /// VTK: `vtkCoordinateFrame::GetYAxis`.
    pub fn get_y_axis(&self) -> [f64; 3] {
        self.y_axis
    }

    /// VTK: `vtkCoordinateFrame::SetZAxis`.
    pub fn set_z_axis(&mut self, x: f64, y: f64, z: f64) {
        let axis = [x, y, z];
        if self.z_axis != axis {
            self.z_axis = axis;
            self.modified();
        }
    }

    /// VTK: `vtkCoordinateFrame::SetZAxis`.
    pub fn set_z_axis_array(&mut self, axis: [f64; 3]) {
        self.set_z_axis(axis[0], axis[1], axis[2]);
    }

    /// VTK: `vtkCoordinateFrame::GetZAxis`.
    pub fn get_z_axis(&self) -> [f64; 3] {
        self.z_axis
    }

    /// VTK: `vtkCoordinateFrame::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Origin: {} {} {}\nXAxis: {} {} {}\nYAxis: {} {} {}\nZAxis: {} {} {}\n",
            self.origin[0],
            self.origin[1],
            self.origin[2],
            self.x_axis[0],
            self.x_axis[1],
            self.x_axis[2],
            self.y_axis[0],
            self.y_axis[1],
            self.y_axis[2],
            self.z_axis[0],
            self.z_axis[1],
            self.z_axis[2]
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

impl Default for CoordinateFrame {
    fn default() -> Self {
        Self::new()
    }
}

fn dot3(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}
