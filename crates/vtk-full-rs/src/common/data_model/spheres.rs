use super::Sphere;
use crate::common::core::{AnyArray, Object, Points, VtkMTimeType, VTK_DOUBLE_MAX};

/// VTK: `vtkSpheres`.
#[derive(Debug, Clone, PartialEq)]
pub struct Spheres {
    object: Object,
    centers: Option<Points>,
    radii: Option<AnyArray>,
    sphere: Sphere,
}

impl Spheres {
    /// VTK: `vtkSpheres::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkSpheres"),
            centers: None,
            radii: None,
            sphere: Sphere::new(),
        }
    }

    /// VTK: `vtkSpheres::SetCenters`.
    pub fn set_centers(&mut self, centers: Option<&Points>) {
        if option_points_storage_eq(self.centers.as_ref(), centers) {
            return;
        }

        self.centers = centers.cloned();
        self.modified();
    }

    /// VTK: `vtkSpheres::GetCenters`.
    pub fn get_centers(&self) -> Option<&Points> {
        self.centers.as_ref()
    }

    /// VTK: `vtkSpheres::SetRadii`.
    pub fn set_radii(&mut self, radii: Option<&AnyArray>) {
        if radii.is_some_and(|radii| radii.get_number_of_components() != 1) {
            return;
        }

        if option_array_storage_eq(self.radii.as_ref(), radii) {
            return;
        }

        self.radii = radii.map(AnyArray::shallow_clone);
        self.modified();
    }

    /// VTK: `vtkSpheres::GetRadii`.
    pub fn get_radii(&self) -> Option<&AnyArray> {
        self.radii.as_ref()
    }

    /// VTK: `vtkSpheres::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let Some(centers) = &self.centers else {
            return VTK_DOUBLE_MAX;
        };
        let Some(radii) = &self.radii else {
            return VTK_DOUBLE_MAX;
        };

        let num_spheres = centers.get_number_of_points();
        if num_spheres != radii.get_number_of_tuples() {
            return VTK_DOUBLE_MAX;
        }

        let mut min_val = VTK_DOUBLE_MAX;
        for i in 0..num_spheres {
            let radius = radius_at(radii, i);
            let center = centers.get_point(i);
            let val = Sphere::evaluate(center, radius, x);
            min_val = min_val.min(val);
        }
        min_val
    }

    /// VTK: `vtkSpheres::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3], n: &mut [f64; 3]) {
        let Some(centers) = &self.centers else {
            return;
        };
        let Some(radii) = &self.radii else {
            return;
        };

        let num_spheres = centers.get_number_of_points();
        if num_spheres != radii.get_number_of_tuples() {
            return;
        }

        let mut min_val = VTK_DOUBLE_MAX;
        for i in 0..num_spheres {
            let radius = radius_at(radii, i);
            let center = centers.get_point(i);
            let val = Sphere::evaluate(center, radius, x);
            if val < min_val {
                min_val = val;
                n[0] = x[0] - center[0];
                n[1] = x[1] - center[1];
                n[2] = x[2] - center[2];
            }
        }
    }

    /// VTK: `vtkSpheres::GetNumberOfSpheres`.
    pub fn get_number_of_spheres(&self) -> i32 {
        match (&self.centers, &self.radii) {
            (Some(centers), Some(radii)) => centers
                .get_number_of_points()
                .min(radii.get_number_of_tuples())
                as i32,
            _ => 0,
        }
    }

    /// VTK: `vtkSpheres::GetSphere(int)`.
    pub fn get_sphere(&mut self, i: i32) -> Option<&Sphere> {
        if i < 0 || i >= self.get_number_of_spheres() {
            return None;
        }

        let radius = radius_at(self.radii.as_ref().expect("range checked radii"), i as i64);
        let center = self
            .centers
            .as_ref()
            .expect("range checked centers")
            .get_point(i as i64);
        self.sphere.set_radius(radius);
        self.sphere.set_center_array(center);
        Some(&self.sphere)
    }

    /// VTK: `vtkSpheres::GetSphere(int, vtkSphere*)`.
    pub fn get_sphere_into(&self, i: i32, sphere: &mut Sphere) {
        if i < 0 || i >= self.get_number_of_spheres() {
            return;
        }

        let radius = radius_at(self.radii.as_ref().expect("range checked radii"), i as i64);
        let center = self
            .centers
            .as_ref()
            .expect("range checked centers")
            .get_point(i as i64);
        sphere.set_radius(radius);
        sphere.set_center_array(center);
    }

    /// VTK: `vtkSpheres::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = String::new();
        if let Some(centers) = &self.centers {
            let num_spheres = centers.get_number_of_points();
            if num_spheres > 0 {
                result.push_str(&format!("Number of Spheres: {num_spheres}\n"));
            } else {
                result.push_str("No Spheres Defined.\n");
            }
        } else {
            result.push_str("No Spheres Defined.\n");
        }

        if self.radii.is_some() {
            result.push_str("Radii: (defined)\n");
        } else {
            result.push_str("Radii: (none)\n");
        }
        result
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

impl Default for Spheres {
    fn default() -> Self {
        Self::new()
    }
}

fn radius_at(radii: &AnyArray, i: i64) -> f64 {
    radii
        .numeric_tuple_as_f64_checked(i as usize)
        .expect("vtkSpheres radii must be a numeric data array")[0]
}

fn option_points_storage_eq(left: Option<&Points>, right: Option<&Points>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.shares_storage_with(right),
        (None, None) => true,
        _ => false,
    }
}

fn option_array_storage_eq(left: Option<&AnyArray>, right: Option<&AnyArray>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.shares_storage_with(right),
        (None, None) => true,
        _ => false,
    }
}
