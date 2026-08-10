use crate::common::core::{Object, VtkMTimeType};

pub const VTK_MIN_SUPERQUADRIC_THICKNESS: f64 = 1e-4;

const MAX_FVAL: f64 = 1e12;
const VTK_MIN_SUPERQUADRIC_ROUNDNESS: f64 = 1e-24;

/// VTK: `vtkSuperquadric`.
#[derive(Debug, Clone, PartialEq)]
pub struct Superquadric {
    object: Object,
    toroidal: bool,
    thickness: f64,
    size: f64,
    phi_roundness: f64,
    theta_roundness: f64,
    center: [f64; 3],
    scale: [f64; 3],
}

impl Superquadric {
    /// VTK: `vtkSuperquadric::New`.
    pub fn new() -> Self {
        let mut superquadric = Self {
            object: Object::with_class_name("vtkSuperquadric"),
            toroidal: false,
            thickness: 0.3333,
            size: 0.5,
            phi_roundness: 0.0,
            theta_roundness: 0.0,
            center: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };
        superquadric.set_phi_roundness(1.0);
        superquadric.set_theta_roundness(1.0);
        superquadric
    }

    /// VTK: `vtkSuperquadric::EvaluateFunction`.
    pub fn evaluate_function(&self, xyz: [f64; 3]) -> f64 {
        let e = self.theta_roundness;
        let n = self.phi_roundness;
        let mut s = [
            self.scale[0] * self.size,
            self.scale[1] * self.size,
            self.scale[2] * self.size,
        ];

        let value = if self.toroidal {
            let alpha = 1.0 / self.thickness;
            s[0] /= alpha + 1.0;
            s[1] /= alpha + 1.0;
            s[2] /= alpha + 1.0;

            let p = [
                (xyz[0] - self.center[0]) / s[0],
                (xyz[1] - self.center[1]) / s[1],
                (xyz[2] - self.center[2]) / s[2],
            ];
            let tval = (p[2].abs().powf(2.0 / e) + p[0].abs().powf(2.0 / e)).powf(e / 2.0);
            (tval - alpha).abs().powf(2.0 / n) + p[1].abs().powf(2.0 / n) - 1.0
        } else {
            let p = [
                (xyz[0] - self.center[0]) / s[0],
                (xyz[1] - self.center[1]) / s[1],
                (xyz[2] - self.center[2]) / s[2],
            ];
            (p[2].abs().powf(2.0 / e) + p[0].abs().powf(2.0 / e)).powf(e / n)
                + p[1].abs().powf(2.0 / n)
                - 1.0
        };

        value.clamp(-MAX_FVAL, MAX_FVAL)
    }

    /// VTK: `vtkSuperquadric::EvaluateGradient`.
    pub fn evaluate_gradient(&self, _xyz: [f64; 3]) -> [f64; 3] {
        [0.0, 0.0, 0.0]
    }

    /// VTK: `vtkSuperquadric::SetCenter`.
    pub fn set_center(&mut self, x: f64, y: f64, z: f64) {
        let center = [x, y, z];
        if self.center != center {
            self.center = center;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::SetCenter`.
    pub fn set_center_array(&mut self, center: [f64; 3]) {
        self.set_center(center[0], center[1], center[2]);
    }

    /// VTK: `vtkSuperquadric::GetCenter`.
    pub fn get_center(&self) -> [f64; 3] {
        self.center
    }

    /// VTK: `vtkSuperquadric::SetScale`.
    pub fn set_scale(&mut self, x: f64, y: f64, z: f64) {
        let scale = [x, y, z];
        if self.scale != scale {
            self.scale = scale;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::SetScale`.
    pub fn set_scale_array(&mut self, scale: [f64; 3]) {
        self.set_scale(scale[0], scale[1], scale[2]);
    }

    /// VTK: `vtkSuperquadric::GetScale`.
    pub fn get_scale(&self) -> [f64; 3] {
        self.scale
    }

    /// VTK: `vtkSuperquadric::GetThickness`.
    pub fn get_thickness(&self) -> f64 {
        self.thickness
    }

    /// VTK: `vtkSuperquadric::SetThickness`.
    pub fn set_thickness(&mut self, thickness: f64) {
        let thickness = thickness.clamp(VTK_MIN_SUPERQUADRIC_THICKNESS, 1.0);
        if self.thickness != thickness {
            self.thickness = thickness;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::GetPhiRoundness`.
    pub fn get_phi_roundness(&self) -> f64 {
        self.phi_roundness
    }

    /// VTK: `vtkSuperquadric::SetPhiRoundness`.
    pub fn set_phi_roundness(&mut self, e: f64) {
        let e = e.max(VTK_MIN_SUPERQUADRIC_ROUNDNESS);
        if self.phi_roundness != e {
            self.phi_roundness = e;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::GetThetaRoundness`.
    pub fn get_theta_roundness(&self) -> f64 {
        self.theta_roundness
    }

    /// VTK: `vtkSuperquadric::SetThetaRoundness`.
    pub fn set_theta_roundness(&mut self, e: f64) {
        let e = e.max(VTK_MIN_SUPERQUADRIC_ROUNDNESS);
        if self.theta_roundness != e {
            self.theta_roundness = e;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::SetSize`.
    pub fn set_size(&mut self, size: f64) {
        if self.size != size {
            self.size = size;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::GetSize`.
    pub fn get_size(&self) -> f64 {
        self.size
    }

    /// VTK: `vtkSuperquadric::SetToroidal`.
    pub fn set_toroidal(&mut self, toroidal: bool) {
        if self.toroidal != toroidal {
            self.toroidal = toroidal;
            self.modified();
        }
    }

    /// VTK: `vtkSuperquadric::GetToroidal`.
    pub fn get_toroidal(&self) -> bool {
        self.toroidal
    }

    /// VTK: `vtkSuperquadric::ToroidalOn`.
    pub fn toroidal_on(&mut self) {
        self.set_toroidal(true);
    }

    /// VTK: `vtkSuperquadric::ToroidalOff`.
    pub fn toroidal_off(&mut self) {
        self.set_toroidal(false);
    }

    /// VTK: `vtkSuperquadric::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Toroidal: {}\nSize: {}\nThickness: {}\nThetaRoundness: {}\nPhiRoundness: {}\nCenter: ({}, {}, {})\nScale: ({}, {}, {})\n",
            if self.toroidal { "On" } else { "Off" },
            self.size,
            self.thickness,
            self.theta_roundness,
            self.phi_roundness,
            self.center[0],
            self.center[1],
            self.center[2],
            self.scale[0],
            self.scale[1],
            self.scale[2]
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

impl Default for Superquadric {
    fn default() -> Self {
        Self::new()
    }
}
