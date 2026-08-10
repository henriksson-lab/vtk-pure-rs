use crate::common::core::{
    math::{add, dot, multiply_scalar, normalize, subtract},
    AnyArray, Object, Points, VTK_DOUBLE_MAX,
};

const VTK_PLANE_TOL: f64 = 1.0e-6;

/// VTK: `vtkPlane`.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    object: Object,
    normal: [f64; 3],
    origin: [f64; 3],
    offset: f64,
    axis_aligned: bool,
    internal_normal: [f64; 3],
    internal_origin: [f64; 3],
}

impl Plane {
    /// VTK: `vtkPlane::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPlane"),
            normal: [0.0, 0.0, 1.0],
            origin: [0.0, 0.0, 0.0],
            offset: 0.0,
            axis_aligned: false,
            internal_normal: [0.0, 0.0, 1.0],
            internal_origin: [0.0, 0.0, 0.0],
        }
    }

    /// VTK: `vtkPlane::SetNormal`.
    pub fn set_normal(&mut self, x: f64, y: f64, z: f64) {
        let normal = [x, y, z];
        if self.normal != normal {
            self.normal = normal;
            self.modified();
            self.internal_updates();
        }
    }

    /// VTK: `vtkPlane::SetNormal`.
    pub fn set_normal_array(&mut self, normal: [f64; 3]) {
        self.set_normal(normal[0], normal[1], normal[2]);
    }

    /// VTK: `vtkPlane::GetNormal`.
    pub fn get_normal(&self) -> [f64; 3] {
        self.normal
    }

    /// VTK: `vtkPlane::SetOrigin`.
    pub fn set_origin(&mut self, x: f64, y: f64, z: f64) {
        let origin = [x, y, z];
        if self.origin != origin {
            self.origin = origin;
            self.modified();
            self.internal_updates();
        }
    }

    /// VTK: `vtkPlane::SetOrigin`.
    pub fn set_origin_array(&mut self, origin: [f64; 3]) {
        self.set_origin(origin[0], origin[1], origin[2]);
    }

    /// VTK: `vtkPlane::GetOrigin`.
    pub fn get_origin(&self) -> [f64; 3] {
        self.origin
    }

    /// VTK: `vtkPlane::SetOffset`.
    pub fn set_offset(&mut self, offset: f64) {
        if self.offset != offset {
            self.offset = offset;
            self.modified();
            self.internal_updates();
        }
    }

    /// VTK: `vtkPlane::GetOffset`.
    pub fn get_offset(&self) -> f64 {
        self.offset
    }

    /// VTK: `vtkPlane::SetAxisAligned`.
    pub fn set_axis_aligned(&mut self, axis_aligned: bool) {
        if self.axis_aligned != axis_aligned {
            self.axis_aligned = axis_aligned;
            self.modified();
            self.internal_updates();
        }
    }

    /// VTK: `vtkPlane::GetAxisAligned`.
    pub fn get_axis_aligned(&self) -> bool {
        self.axis_aligned
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkPlane::ComputeInternalNormal`.
    fn compute_internal_normal(&mut self) {
        if self.axis_aligned {
            self.internal_normal[0] = if self.normal[0].abs() >= self.normal[1].abs()
                && self.normal[0].abs() >= self.normal[2].abs()
            {
                1.0
            } else {
                0.0
            };
            self.internal_normal[1] = if self.normal[1].abs() >= self.normal[0].abs()
                && self.normal[1].abs() >= self.normal[2].abs()
            {
                1.0
            } else {
                0.0
            };
            self.internal_normal[2] = if self.normal[2].abs() >= self.normal[0].abs()
                && self.normal[2].abs() >= self.normal[1].abs()
            {
                1.0
            } else {
                0.0
            };
        } else {
            self.internal_normal = self.normal;
        }
    }

    /// VTK: `vtkPlane::ComputeInternalOrigin`.
    fn compute_internal_origin(&mut self) {
        self.internal_origin = self.origin;
        if self.offset != 0.0 {
            for i in 0..3 {
                self.internal_origin[i] += self.offset * self.internal_normal[i];
            }
        }
    }

    /// VTK: `vtkPlane::InternalUpdates`.
    fn internal_updates(&mut self) {
        self.compute_internal_normal();
        self.compute_internal_origin();
    }

    /// VTK: `vtkPlane::DeepCopy`.
    pub fn deep_copy(&mut self, plane: &Self) {
        self.set_normal_array(plane.get_normal());
        self.set_origin_array(plane.get_origin());
        self.set_axis_aligned(plane.get_axis_aligned());
        self.set_offset(plane.get_offset());
    }

    /// VTK: `vtkPlane::DistanceToPlane`.
    pub fn distance_to_plane_instance(&self, x: [f64; 3]) -> f64 {
        Self::distance_to_plane(x, self.get_normal(), self.get_origin())
    }

    /// VTK: `vtkPlane::ProjectPoint`.
    pub fn project_point(x: [f64; 3], origin: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
        let xo = subtract(x, origin);
        let t = dot(normal, xo);
        [
            x[0] - t * normal[0],
            x[1] - t * normal[1],
            x[2] - t * normal[2],
        ]
    }

    /// VTK: `vtkPlane::ProjectPoint`.
    pub fn project_point_instance(&self, x: [f64; 3]) -> [f64; 3] {
        Self::project_point(x, self.get_origin(), self.get_normal())
    }

    /// VTK: `vtkPlane::ProjectVector`.
    pub fn project_vector(v: [f64; 3], _origin: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
        let t = dot(v, normal);
        let mut n2 = dot(normal, normal);
        if n2 == 0.0 {
            n2 = 1.0;
        }
        [
            v[0] - t * normal[0] / n2,
            v[1] - t * normal[1] / n2,
            v[2] - t * normal[2] / n2,
        ]
    }

    /// VTK: `vtkPlane::ProjectVector`.
    pub fn project_vector_instance(&self, v: [f64; 3]) -> [f64; 3] {
        Self::project_vector(v, self.get_origin(), self.get_normal())
    }

    /// VTK: `vtkPlane::Push`.
    pub fn push(&mut self, distance: f64) {
        if distance == 0.0 {
            return;
        }
        self.compute_internal_normal();
        for i in 0..3 {
            self.origin[i] += distance * self.internal_normal[i];
        }
        self.compute_internal_origin();
        self.modified();
    }

    /// VTK: `vtkPlane::GeneralizedProjectPoint`.
    pub fn generalized_project_point(x: [f64; 3], origin: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
        let xo = subtract(x, origin);
        let t = dot(normal, xo);
        let n2 = dot(normal, normal);
        if n2 != 0.0 {
            [
                x[0] - t * normal[0] / n2,
                x[1] - t * normal[1] / n2,
                x[2] - t * normal[2] / n2,
            ]
        } else {
            x
        }
    }

    /// VTK: `vtkPlane::GeneralizedProjectPoint`.
    pub fn generalized_project_point_instance(&self, x: [f64; 3]) -> [f64; 3] {
        Self::generalized_project_point(x, self.get_origin(), self.get_normal())
    }

    /// VTK: `vtkPlane::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        self.internal_normal[0] * (x[0] - self.internal_origin[0])
            + self.internal_normal[1] * (x[1] - self.internal_origin[1])
            + self.internal_normal[2] * (x[2] - self.internal_origin[2])
    }

    /// VTK: `vtkPlane::EvaluateFunction(vtkDataArray*, vtkDataArray*)`.
    pub fn evaluate_function_array(&self, input: &AnyArray, output: &mut AnyArray) -> bool {
        if input.get_number_of_components() != 3 || output.get_number_of_components() != 1 {
            return false;
        }
        let tuple_count = input.get_number_of_tuples();
        output.set_number_of_tuples(tuple_count);
        for point_id in 0..tuple_count {
            let Some(tuple) = input.component_tuple_values_as_f64(point_id as usize) else {
                return false;
            };
            let value = self.internal_normal[0] * (tuple[0] - self.internal_origin[0])
                + self.internal_normal[1] * (tuple[1] - self.internal_origin[1])
                + self.internal_normal[2] * (tuple[2] - self.internal_origin[2]);
            if output
                .insert_numeric_tuple_from_f64_checked(point_id as usize, &[value])
                .is_err()
            {
                return false;
            }
        }
        true
    }

    /// VTK: `vtkPlane::EvaluateGradient`.
    pub fn evaluate_gradient(&self, _x: [f64; 3]) -> [f64; 3] {
        self.internal_normal
    }

    /// VTK: `vtkPlane::Evaluate`.
    pub fn evaluate(normal: [f64; 3], origin: [f64; 3], x: [f64; 3]) -> f64 {
        normal[0] * (x[0] - origin[0])
            + normal[1] * (x[1] - origin[1])
            + normal[2] * (x[2] - origin[2])
    }

    /// VTK: `vtkPlane::DistanceToPlane`.
    pub fn distance_to_plane(x: [f64; 3], n: [f64; 3], p0: [f64; 3]) -> f64 {
        (n[0] * (x[0] - p0[0]) + n[1] * (x[1] - p0[1]) + n[2] * (x[2] - p0[2])).abs()
    }

    /// VTK: `vtkPlane::IntersectWithLine`.
    pub fn intersect_with_line(
        p1: [f64; 3],
        p2: [f64; 3],
        n: [f64; 3],
        p0: [f64; 3],
    ) -> (i32, f64, [f64; 3]) {
        let p21 = subtract(p2, p1);
        let num = dot(n, p0) - (n[0] * p1[0] + n[1] * p1[1] + n[2] * p1[2]);
        let den = n[0] * p21[0] + n[1] * p21[1] + n[2] * p21[2];
        let fabsden = if den < 0.0 { -den } else { den };
        let fabstolerance = if num < 0.0 {
            -num * VTK_PLANE_TOL
        } else {
            num * VTK_PLANE_TOL
        };
        if fabsden <= fabstolerance {
            return (0, VTK_DOUBLE_MAX, [0.0; 3]);
        }

        let t = num / den;
        let x = [p1[0] + t * p21[0], p1[1] + t * p21[1], p1[2] + t * p21[2]];
        if (0.0..=1.0).contains(&t) {
            (1, t, x)
        } else {
            (0, t, x)
        }
    }

    /// VTK: `vtkPlane::IntersectWithLine`.
    pub fn intersect_with_line_instance(&self, p1: [f64; 3], p2: [f64; 3]) -> (i32, f64, [f64; 3]) {
        Self::intersect_with_line(p1, p2, self.get_normal(), self.get_origin())
    }

    /// VTK: `vtkPlane::IntersectWithFinitePlane`.
    pub fn intersect_with_finite_plane(
        n: [f64; 3],
        o: [f64; 3],
        p_origin: [f64; 3],
        px: [f64; 3],
        py: [f64; 3],
    ) -> (i32, [f64; 3], [f64; 3]) {
        let mut num_ints = 0;
        let mut x0 = [0.0; 3];
        let mut x1 = [0.0; 3];

        let mut xr0 = p_origin;
        let mut xr1 = px;
        let (hit, _t, x) = Self::intersect_with_line(xr0, xr1, n, o);
        if hit != 0 {
            x0 = x;
            num_ints += 1;
        }

        xr1 = py;
        let (hit, _t, x) = Self::intersect_with_line(xr0, xr1, n, o);
        if hit != 0 {
            if num_ints == 0 {
                x0 = x;
            } else {
                x1 = x;
            }
            num_ints += 1;
        }
        if num_ints == 2 {
            return (1, x0, x1);
        }

        xr0 = add(px, subtract(py, p_origin));
        let (hit, _t, x) = Self::intersect_with_line(xr0, xr1, n, o);
        if hit != 0 {
            if num_ints == 0 {
                x0 = x;
            } else {
                x1 = x;
            }
            num_ints += 1;
        }
        if num_ints == 2 {
            return (1, x0, x1);
        }

        xr1 = px;
        let (hit, _t, x) = Self::intersect_with_line(xr0, xr1, n, o);
        if hit != 0 {
            if num_ints == 0 {
                x0 = x;
            } else {
                x1 = x;
            }
            num_ints += 1;
        }
        if num_ints == 2 {
            (1, x0, x1)
        } else {
            (0, x0, x1)
        }
    }

    /// VTK: `vtkPlane::IntersectWithFinitePlane`.
    pub fn intersect_with_finite_plane_instance(
        &self,
        p_origin: [f64; 3],
        px: [f64; 3],
        py: [f64; 3],
    ) -> (i32, [f64; 3], [f64; 3]) {
        Self::intersect_with_finite_plane(self.get_normal(), self.get_origin(), p_origin, px, py)
    }

    /// VTK: `vtkPlane::ComputeBestFittingPlane`.
    pub fn compute_best_fitting_plane(pts: &Points) -> (bool, [f64; 3], [f64; 3]) {
        let mut origin = [0.0; 3];
        let mut normal = [0.0, 0.0, 1.0];

        let npts = pts.get_number_of_points();
        if npts < 3 {
            return (false, origin, normal);
        }

        for pt_id in 0..npts {
            origin = add(origin, pts.get_point(pt_id));
        }
        multiply_scalar(&mut origin, 1.0 / npts as f64);

        let mut covariance = [0.0; 6];
        for pt_id in 0..npts {
            let r = subtract(pts.get_point(pt_id), origin);
            covariance[0] += r[0] * r[0];
            covariance[1] += r[0] * r[1];
            covariance[2] += r[0] * r[2];
            covariance[3] += r[1] * r[1];
            covariance[4] += r[1] * r[2];
            covariance[5] += r[2] * r[2];
        }
        for value in &mut covariance {
            *value /= npts as f64;
        }

        let [xx, xy, xz, yy, yz, zz] = covariance;
        let mut weighted_dir = [0.0; 3];

        let det_x = yy * zz - yz * yz;
        let mut axis_dir = [det_x, xz * yz - xy * zz, xy * yz - xz * yy];
        let mut weight = det_x * det_x;
        if dot(weighted_dir, axis_dir) < 0.0 {
            weight = -weight;
        }
        multiply_scalar(&mut axis_dir, weight);
        weighted_dir = add(weighted_dir, axis_dir);

        let det_y = xx * zz - xz * xz;
        let mut axis_dir = [xz * yz - xy * zz, det_y, xy * xz - yz * xx];
        let mut weight = det_y * det_y;
        if dot(weighted_dir, axis_dir) < 0.0 {
            weight = -weight;
        }
        multiply_scalar(&mut axis_dir, weight);
        weighted_dir = add(weighted_dir, axis_dir);

        let det_z = xx * yy - xy * xy;
        let mut axis_dir = [xy * yz - xz * yy, xy * xz - yz * xx, det_z];
        let mut weight = det_z * det_z;
        if dot(weighted_dir, axis_dir) < 0.0 {
            weight = -weight;
        }
        multiply_scalar(&mut axis_dir, weight);
        weighted_dir = add(weighted_dir, axis_dir);

        let nrm = normalize(&mut weighted_dir);
        if !nrm.is_finite() || nrm == 0.0 {
            return (false, origin, normal);
        }

        normal = weighted_dir;
        (true, origin, normal)
    }

    /// VTK: `vtkPlane::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Normal: ({}, {}, {})\nOrigin: ({}, {}, {})\nOffset: {}\nAxisAligned: {}\n",
            self.normal[0],
            self.normal[1],
            self.normal[2],
            self.origin[0],
            self.origin[1],
            self.origin[2],
            self.offset,
            if self.axis_aligned { "On" } else { "Off" }
        )
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self::new()
    }
}
