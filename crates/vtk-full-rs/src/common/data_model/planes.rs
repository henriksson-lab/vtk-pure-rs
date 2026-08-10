use super::Plane;
use crate::common::core::{
    AnyArray, DoubleArray, Object, Points, VtkMTimeType, VTK_DOUBLE, VTK_DOUBLE_MAX,
};

/// VTK: `vtkPlanes`.
#[derive(Debug, Clone, PartialEq)]
pub struct Planes {
    object: Object,
    points: Option<Points>,
    normals: Option<AnyArray>,
    plane: Plane,
    planes: [f64; 24],
    bounds: [f64; 6],
}

impl Planes {
    /// VTK: `vtkPlanes::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPlanes"),
            points: None,
            normals: None,
            plane: Plane::new(),
            planes: [0.0; 24],
            bounds: [0.0; 6],
        }
    }

    /// VTK: `vtkPlanes::SetPoints`.
    pub fn set_points(&mut self, points: Option<&Points>) {
        if option_points_storage_eq(self.points.as_ref(), points) {
            return;
        }

        self.points = points.cloned();
        self.modified();
    }

    /// VTK: `vtkPlanes::GetPoints`.
    pub fn get_points(&self) -> Option<&Points> {
        self.points.as_ref()
    }

    /// VTK: `vtkPlanes::SetNormals`.
    pub fn set_normals(&mut self, normals: Option<&AnyArray>) {
        if normals.is_some_and(|normals| {
            !normals.is_data_array() || normals.get_number_of_components() != 3
        }) {
            return;
        }

        if option_array_storage_eq(self.normals.as_ref(), normals) {
            return;
        }

        self.normals = normals.map(AnyArray::shallow_clone);
        self.modified();
    }

    /// VTK: `vtkPlanes::GetNormals`.
    pub fn get_normals(&self) -> Option<&AnyArray> {
        self.normals.as_ref()
    }

    /// VTK: `vtkPlanes::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        let Some(points) = &self.points else {
            return VTK_DOUBLE_MAX;
        };
        let Some(normals) = &self.normals else {
            return VTK_DOUBLE_MAX;
        };

        let num_planes = points.get_number_of_points();
        if num_planes != normals.get_number_of_tuples() {
            return VTK_DOUBLE_MAX;
        }

        let mut max_val = -VTK_DOUBLE_MAX;
        for i in 0..num_planes {
            let normal = normal_at(normals, i);
            let point = points.get_point(i);
            let val = Plane::evaluate(normal, point, x);
            max_val = max_val.max(val);
        }
        max_val
    }

    /// VTK: `vtkPlanes::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3], n: &mut [f64; 3]) {
        let Some(points) = &self.points else {
            return;
        };
        let Some(normals) = &self.normals else {
            return;
        };

        let num_planes = points.get_number_of_points();
        if num_planes != normals.get_number_of_tuples() {
            return;
        }

        let mut max_val = -VTK_DOUBLE_MAX;
        for i in 0..num_planes {
            let normal = normal_at(normals, i);
            let point = points.get_point(i);
            let val = Plane::evaluate(normal, point, x);
            if val > max_val {
                max_val = val;
                *n = normal;
            }
        }
    }

    /// VTK: `vtkPlanes::SetFrustumPlanes`.
    pub fn set_frustum_planes(&mut self, planes: [f64; 24]) {
        if self.planes == planes {
            return;
        }

        self.modified();

        let mut points = Points::new_with_data_type(VTK_DOUBLE);
        points.set_number_of_points(6);
        let mut normals = double_normals_array(6);

        for i in 0..6 {
            let plane = &planes[(4 * i)..(4 * i + 4)];
            let n = [-plane[0], -plane[1], -plane[2]];
            let mut x = [0.0, 0.0, 0.0];
            if n[0] != 0.0 {
                x[0] = plane[3] / n[0];
            } else if n[1] != 0.0 {
                x[1] = plane[3] / n[1];
            } else {
                x[2] = plane[3] / n[2];
            }
            points.set_point(i as i64, x);
            set_normal_tuple(&mut normals, i, n);
        }

        self.set_points(Some(&points));
        self.set_normals(Some(&normals));
    }

    /// VTK: `vtkPlanes::SetBounds`.
    pub fn set_bounds(&mut self, bounds: [f64; 6]) {
        if self.bounds == bounds {
            return;
        }

        self.modified();

        let mut points = Points::new();
        points.set_number_of_points(6);
        let mut normals = double_normals_array(6);

        let plane_data = [
            ([bounds[0], 0.0, 0.0], [-1.0, 0.0, 0.0]),
            ([bounds[1], 0.0, 0.0], [1.0, 0.0, 0.0]),
            ([0.0, bounds[2], 0.0], [0.0, -1.0, 0.0]),
            ([0.0, bounds[3], 0.0], [0.0, 1.0, 0.0]),
            ([0.0, 0.0, bounds[4]], [0.0, 0.0, -1.0]),
            ([0.0, 0.0, bounds[5]], [0.0, 0.0, 1.0]),
        ];

        for (i, (point, normal)) in plane_data.into_iter().enumerate() {
            points.set_point(i as i64, point);
            set_normal_tuple(&mut normals, i, normal);
        }

        self.bounds = bounds;
        self.set_points(Some(&points));
        self.set_normals(Some(&normals));
    }

    /// VTK: `vtkPlanes::SetBounds`.
    pub fn set_bounds_components(
        &mut self,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        zmin: f64,
        zmax: f64,
    ) {
        self.set_bounds([xmin, xmax, ymin, ymax, zmin, zmax]);
    }

    /// VTK: `vtkPlanes::GetNumberOfPlanes`.
    pub fn get_number_of_planes(&self) -> i32 {
        match (&self.points, &self.normals) {
            (Some(points), Some(normals)) => points
                .get_number_of_points()
                .min(normals.get_number_of_tuples())
                as i32,
            _ => 0,
        }
    }

    /// VTK: `vtkPlanes::GetPlane(int)`.
    pub fn get_plane(&mut self, i: i32) -> Option<&Plane> {
        if i < 0 || i >= self.get_number_of_planes() {
            return None;
        }

        let normal = normal_at(
            self.normals.as_ref().expect("range checked normals"),
            i as i64,
        );
        let point = self
            .points
            .as_ref()
            .expect("range checked points")
            .get_point(i as i64);
        self.plane.set_normal_array(normal);
        self.plane.set_origin_array(point);
        Some(&self.plane)
    }

    /// VTK: `vtkPlanes::GetPlane(int, vtkPlane*)`.
    pub fn get_plane_into(&self, i: i32, plane: &mut Plane) {
        if i < 0 || i >= self.get_number_of_planes() {
            return;
        }

        let normal = normal_at(
            self.normals.as_ref().expect("range checked normals"),
            i as i64,
        );
        let point = self
            .points
            .as_ref()
            .expect("range checked points")
            .get_point(i as i64);
        plane.set_normal_array(normal);
        plane.set_origin_array(point);
    }

    /// VTK: `vtkPlanes::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = String::new();
        if let Some(points) = &self.points {
            let num_planes = points.get_number_of_points();
            if num_planes > 0 {
                result.push_str(&format!("Number of Planes: {num_planes}\n"));
            } else {
                result.push_str("No Planes Defined.\n");
            }
        } else {
            result.push_str("No Planes Defined.\n");
        }

        if self.normals.is_some() {
            result.push_str("Normals: (defined)\n");
        } else {
            result.push_str("Normals: (none)\n");
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

impl Default for Planes {
    fn default() -> Self {
        Self::new()
    }
}

fn double_normals_array(tuple_count: i64) -> AnyArray {
    let mut normals = AnyArray::Double(DoubleArray::new());
    normals.set_number_of_components(3);
    normals.set_number_of_tuples(tuple_count);
    normals
}

fn set_normal_tuple(normals: &mut AnyArray, i: usize, normal: [f64; 3]) {
    normals
        .insert_numeric_tuple_from_f64_checked(i, &normal)
        .expect("vtkPlanes generated normals must be numeric");
}

fn normal_at(normals: &AnyArray, i: i64) -> [f64; 3] {
    let tuple = normals
        .numeric_tuple_as_f64_checked(i as usize)
        .expect("vtkPlanes normals must be a numeric data array");
    [tuple[0], tuple[1], tuple[2]]
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
