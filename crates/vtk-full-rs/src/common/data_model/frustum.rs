use crate::common::core::{Object, VtkMTimeType, VTK_DOUBLE_MAX};
use crate::common::data_model::Plane;

/// VTK: `vtkFrustum`.
#[derive(Debug, Clone, PartialEq)]
pub struct Frustum {
    object: Object,
    near_plane_distance: f64,
    vertical_angle: f64,
    horizontal_angle: f64,
    near_plane: Plane,
    bottom_plane: Plane,
    top_plane: Plane,
    right_plane: Plane,
    left_plane: Plane,
}

impl Frustum {
    /// VTK: `vtkFrustum::New`.
    pub fn new() -> Self {
        let mut frustum = Self {
            object: Object::with_class_name("vtkFrustum"),
            near_plane_distance: 0.5,
            vertical_angle: 30.0,
            horizontal_angle: 30.0,
            near_plane: Plane::new(),
            bottom_plane: Plane::new(),
            top_plane: Plane::new(),
            right_plane: Plane::new(),
            left_plane: Plane::new(),
        };
        frustum.near_plane.set_normal(0.0, -1.0, 0.0);
        frustum
            .near_plane
            .set_origin(0.0, frustum.near_plane_distance, 0.0);
        frustum.calculate_horizontal_planes_normal();
        frustum.calculate_vertical_planes_normal();
        frustum
    }

    /// VTK: `vtkFrustum::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let mut value = -VTK_DOUBLE_MAX;
        for plane in self.planes() {
            let v = plane.evaluate_function(x);
            if v > value {
                value = v;
            }
        }
        value
    }

    /// VTK: `vtkFrustum::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        let mut value = -VTK_DOUBLE_MAX;
        let mut gradient = [0.0; 3];
        for plane in self.planes() {
            let v = plane.evaluate_function(x);
            if v > value {
                value = v;
                gradient = plane.evaluate_gradient(x);
            }
        }
        gradient
    }

    /// VTK: `vtkFrustum::GetNearPlaneDistance`.
    pub fn get_near_plane_distance(&self) -> f64 {
        self.near_plane_distance
    }

    /// VTK: `vtkFrustum::SetNearPlaneDistance`.
    pub fn set_near_plane_distance(&mut self, distance: f64) {
        let distance = distance.max(0.0);
        if self.near_plane_distance == distance {
            return;
        }
        self.near_plane_distance = distance;
        self.near_plane.set_origin(0.0, distance, 0.0);
        self.modified();
    }

    /// VTK: `vtkFrustum::GetHorizontalAngle`.
    pub fn get_horizontal_angle(&self) -> f64 {
        self.horizontal_angle
    }

    /// VTK: `vtkFrustum::SetHorizontalAngle`.
    pub fn set_horizontal_angle(&mut self, angle_in_degrees: f64) {
        let angle_in_degrees = angle_in_degrees.clamp(1.0, 89.0);
        if self.horizontal_angle == angle_in_degrees {
            return;
        }
        self.horizontal_angle = angle_in_degrees;
        self.calculate_horizontal_planes_normal();
        self.modified();
    }

    /// VTK: `vtkFrustum::GetVerticalAngle`.
    pub fn get_vertical_angle(&self) -> f64 {
        self.vertical_angle
    }

    /// VTK: `vtkFrustum::SetVerticalAngle`.
    pub fn set_vertical_angle(&mut self, angle_in_degrees: f64) {
        let angle_in_degrees = angle_in_degrees.clamp(1.0, 89.0);
        if self.vertical_angle == angle_in_degrees {
            return;
        }
        self.vertical_angle = angle_in_degrees;
        self.calculate_vertical_planes_normal();
        self.modified();
    }

    /// VTK: `vtkFrustum::GetTopPlane`.
    pub fn get_top_plane(&self) -> &Plane {
        &self.top_plane
    }

    /// VTK: `vtkFrustum::GetBottomPlane`.
    pub fn get_bottom_plane(&self) -> &Plane {
        &self.bottom_plane
    }

    /// VTK: `vtkFrustum::GetRightPlane`.
    pub fn get_right_plane(&self) -> &Plane {
        &self.right_plane
    }

    /// VTK: `vtkFrustum::GetLeftPlane`.
    pub fn get_left_plane(&self) -> &Plane {
        &self.left_plane
    }

    /// VTK: `vtkFrustum::GetNearPlane`.
    pub fn get_near_plane(&self) -> &Plane {
        &self.near_plane
    }

    /// VTK: `vtkFrustum::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Near Plane Distance: {}\nHorizontal Angle: {}\nVertical Angle: {}\n",
            self.near_plane_distance, self.horizontal_angle, self.vertical_angle
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

    /// VTK: `vtkFrustum::CalculateHorizontalPlanesNormal`.
    fn calculate_horizontal_planes_normal(&mut self) {
        let angle_radians = self.horizontal_angle.to_radians();
        let cos_angle = angle_radians.cos();
        let sin_angle = angle_radians.sin();
        self.right_plane.set_normal(-cos_angle, -sin_angle, 0.0);
        self.left_plane.set_normal(cos_angle, -sin_angle, 0.0);
    }

    /// VTK: `vtkFrustum::CalculateVerticalPlanesNormal`.
    fn calculate_vertical_planes_normal(&mut self) {
        let angle_radians = self.vertical_angle.to_radians();
        let cos_angle = angle_radians.cos();
        let sin_angle = angle_radians.sin();
        self.top_plane.set_normal(0.0, -sin_angle, -cos_angle);
        self.bottom_plane.set_normal(0.0, -sin_angle, cos_angle);
    }

    fn planes(&self) -> [&Plane; 5] {
        [
            &self.near_plane,
            &self.bottom_plane,
            &self.top_plane,
            &self.right_plane,
            &self.left_plane,
        ]
    }
}

impl Default for Frustum {
    fn default() -> Self {
        Self::new()
    }
}
