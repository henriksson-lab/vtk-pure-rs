use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::common::core::{Object, VtkIdType, VtkMTimeType};

/// VTK: `vtkSphere`.
#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    object: Object,
    radius: f64,
    center: [f64; 3],
}

impl Sphere {
    /// VTK: `vtkSphere::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkSphere"),
            radius: 0.5,
            center: [0.0, 0.0, 0.0],
        }
    }

    /// VTK: `vtkSphere::EvaluateFunction`.
    pub fn evaluate_function(&self, x: [f64; 3]) -> f64 {
        Self::evaluate(self.center, self.radius, x)
    }

    /// VTK: `vtkSphere::EvaluateGradient`.
    pub fn evaluate_gradient(&self, x: [f64; 3]) -> [f64; 3] {
        [
            2.0 * (x[0] - self.center[0]),
            2.0 * (x[1] - self.center[1]),
            2.0 * (x[2] - self.center[2]),
        ]
    }

    /// VTK: `vtkSphere::SetRadius`.
    pub fn set_radius(&mut self, radius: f64) {
        if self.radius != radius {
            self.radius = radius;
            self.modified();
        }
    }

    /// VTK: `vtkSphere::GetRadius`.
    pub fn get_radius(&self) -> f64 {
        self.radius
    }

    /// VTK: `vtkSphere::SetCenter`.
    pub fn set_center(&mut self, x: f64, y: f64, z: f64) {
        let center = [x, y, z];
        if self.center != center {
            self.center = center;
            self.modified();
        }
    }

    /// VTK: `vtkSphere::SetCenter`.
    pub fn set_center_array(&mut self, center: [f64; 3]) {
        self.set_center(center[0], center[1], center[2]);
    }

    /// VTK: `vtkSphere::GetCenter`.
    pub fn get_center(&self) -> [f64; 3] {
        self.center
    }

    /// VTK: `vtkSphere::Evaluate`.
    pub fn evaluate(center: [f64; 3], radius: f64, x: [f64; 3]) -> f64 {
        (x[0] - center[0]) * (x[0] - center[0])
            + (x[1] - center[1]) * (x[1] - center[1])
            + (x[2] - center[2]) * (x[2] - center[2])
            - radius * radius
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(float*, vtkIdType, float[4], vtkIdType[2])`.
    pub fn compute_bounding_sphere_f32(
        pts: &[f32],
        num_pts: VtkIdType,
        sphere: &mut [f32; 4],
        hints: Option<[VtkIdType; 2]>,
    ) {
        compute_bounding_sphere_points(pts, num_pts, sphere, hints);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(double*, vtkIdType, double[4], vtkIdType[2])`.
    pub fn compute_bounding_sphere_f64(
        pts: &[f64],
        num_pts: VtkIdType,
        sphere: &mut [f64; 4],
        hints: Option<[VtkIdType; 2]>,
    ) {
        compute_bounding_sphere_points(pts, num_pts, sphere, hints);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(float*, vtkIdType, float[4])`.
    pub fn compute_bounding_sphere_f32_no_hints(
        pts: &[f32],
        num_pts: VtkIdType,
        sphere: &mut [f32; 4],
    ) {
        Self::compute_bounding_sphere_f32(pts, num_pts, sphere, None);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(double*, vtkIdType, double[4])`.
    pub fn compute_bounding_sphere_f64_no_hints(
        pts: &[f64],
        num_pts: VtkIdType,
        sphere: &mut [f64; 4],
    ) {
        Self::compute_bounding_sphere_f64(pts, num_pts, sphere, None);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(float**, vtkIdType, float[4], vtkIdType[2])`.
    pub fn compute_bounding_sphere_from_spheres_f32(
        spheres: &[[f32; 4]],
        num_spheres: VtkIdType,
        sphere: &mut [f32; 4],
        hints: Option<[VtkIdType; 2]>,
    ) {
        compute_bounding_sphere_spheres(spheres, num_spheres, sphere, hints);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(double**, vtkIdType, double[4], vtkIdType[2])`.
    pub fn compute_bounding_sphere_from_spheres_f64(
        spheres: &[[f64; 4]],
        num_spheres: VtkIdType,
        sphere: &mut [f64; 4],
        hints: Option<[VtkIdType; 2]>,
    ) {
        compute_bounding_sphere_spheres(spheres, num_spheres, sphere, hints);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(float**, vtkIdType, float[4])`.
    pub fn compute_bounding_sphere_from_spheres_f32_no_hints(
        spheres: &[[f32; 4]],
        num_spheres: VtkIdType,
        sphere: &mut [f32; 4],
    ) {
        Self::compute_bounding_sphere_from_spheres_f32(spheres, num_spheres, sphere, None);
    }

    /// VTK: `vtkSphere::ComputeBoundingSphere(double**, vtkIdType, double[4])`.
    pub fn compute_bounding_sphere_from_spheres_f64_no_hints(
        spheres: &[[f64; 4]],
        num_spheres: VtkIdType,
        sphere: &mut [f64; 4],
    ) {
        Self::compute_bounding_sphere_from_spheres_f64(spheres, num_spheres, sphere, None);
    }

    /// VTK: `vtkSphere::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Radius: {}\nCenter: ({}, {}, {})\n",
            self.radius, self.center[0], self.center[1], self.center[2]
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

impl Default for Sphere {
    fn default() -> Self {
        Self::new()
    }
}

trait SphereScalar:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn two() -> Self;
    fn four() -> Self;
    fn float_max() -> Self;
    fn sqrt(self) -> Self;
    fn max(self, other: Self) -> Self;
}

impl SphereScalar for f32 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn two() -> Self {
        2.0
    }

    fn four() -> Self {
        4.0
    }

    fn float_max() -> Self {
        f32::MAX
    }

    fn sqrt(self) -> Self {
        self.sqrt()
    }

    fn max(self, other: Self) -> Self {
        self.max(other)
    }
}

impl SphereScalar for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }

    fn two() -> Self {
        2.0
    }

    fn four() -> Self {
        4.0
    }

    fn float_max() -> Self {
        f32::MAX as f64
    }

    fn sqrt(self) -> Self {
        self.sqrt()
    }

    fn max(self, other: Self) -> Self {
        self.max(other)
    }
}

fn compute_bounding_sphere_points<T: SphereScalar>(
    pts: &[T],
    num_pts: VtkIdType,
    sphere: &mut [T; 4],
    hints: Option<[VtkIdType; 2]>,
) {
    sphere[0] = T::zero();
    sphere[1] = T::zero();
    sphere[2] = T::zero();
    sphere[3] = T::zero();
    if num_pts < 1 || pts.len() < (num_pts as usize).saturating_mul(3) {
        return;
    }

    let (d1, d2) = if let Some(hints) = hints {
        let Some(d1) = point_at(pts, num_pts, hints[0]) else {
            return;
        };
        let Some(d2) = point_at(pts, num_pts, hints[1]) else {
            return;
        };
        (d1, d2)
    } else {
        initial_point_diameter(pts, num_pts)
    };

    sphere[0] = (d1[0] + d2[0]) / T::two();
    sphere[1] = (d1[1] + d2[1]) / T::two();
    sphere[2] = (d1[2] + d2[2]) / T::two();
    let mut r2 = distance2_between_points(d1, d2) / T::four();
    sphere[3] = r2.sqrt();

    for i in 0..num_pts {
        let Some(point) = point_at(pts, num_pts, i) else {
            return;
        };
        let dist2 = distance2_between_points(point, [sphere[0], sphere[1], sphere[2]]);
        if dist2 > r2 {
            let dist = dist2.sqrt();
            sphere[3] = (sphere[3] + dist) / T::two();
            r2 = sphere[3] * sphere[3];
            let delta = dist - sphere[3];
            sphere[0] = (sphere[3] * sphere[0] + delta * point[0]) / dist;
            sphere[1] = (sphere[3] * sphere[1] + delta * point[1]) / dist;
            sphere[2] = (sphere[3] * sphere[2] + delta * point[2]) / dist;
        }
    }
}

fn compute_bounding_sphere_spheres<T: SphereScalar>(
    spheres: &[[T; 4]],
    num_spheres: VtkIdType,
    sphere: &mut [T; 4],
    hints: Option<[VtkIdType; 2]>,
) {
    if num_spheres < 1 || spheres.is_empty() {
        sphere[0] = T::zero();
        sphere[1] = T::zero();
        sphere[2] = T::zero();
        sphere[3] = T::zero();
        return;
    }
    if num_spheres == 1 {
        if let Some(input) = spheres.first() {
            *sphere = *input;
        } else {
            sphere[0] = T::zero();
            sphere[1] = T::zero();
            sphere[2] = T::zero();
            sphere[3] = T::zero();
        }
        return;
    }
    if spheres.len() < num_spheres as usize {
        sphere[0] = T::zero();
        sphere[1] = T::zero();
        sphere[2] = T::zero();
        sphere[3] = T::zero();
        return;
    }

    let (mut s1, mut s2) = if let Some(hints) = hints {
        let Some(s1) = sphere_at(spheres, num_spheres, hints[0]) else {
            return;
        };
        let Some(s2) = sphere_at(spheres, num_spheres, hints[1]) else {
            return;
        };
        (s1, s2)
    } else {
        initial_sphere_diameter(spheres, num_spheres)
    };

    let mut v = [T::zero(); 3];
    let mut r2 = distance2_between_points([s1[0], s1[1], s1[2]], [s2[0], s2[1], s2[2]]) / T::four();
    sphere[3] = if r2 > T::zero() { r2.sqrt() } else { s1[3] };
    let t1 = -s1[3] / (T::two() * sphere[3]);
    let t2 = T::one() + s2[3] / (T::two() * sphere[3]);
    for i in 0..3 {
        v[i] = s2[i] - s1[i];
        let tmp = s1[i] + t1 * v[i];
        s2[i] = s1[i] + t2 * v[i];
        s1[i] = tmp;
        sphere[i] = (s1[i] + s2[i]) / T::two();
    }
    r2 = distance2_between_points([s1[0], s1[1], s1[2]], [s2[0], s2[1], s2[2]]) / T::four();
    if r2 > T::zero() {
        sphere[3] = r2.sqrt();
    } else {
        sphere[3] = s1[3];
        r2 = sphere[3] * sphere[3];
    }

    for sphere_i in spheres.iter().take(num_spheres as usize).copied() {
        let sphere_radius2 = sphere_i[3] * sphere_i[3];
        let mut dist2 = distance2_between_points(
            [sphere_i[0], sphere_i[1], sphere_i[2]],
            [sphere[0], sphere[1], sphere[2]],
        );
        if dist2 <= T::zero() {
            dist2 = sphere_i[3];
        }
        let fac = if sphere_radius2 > dist2 {
            T::two() * sphere_radius2
        } else {
            T::two() * dist2
        };
        if dist2 + fac + sphere_radius2 > r2 {
            let dist = dist2.sqrt();
            if (dist + sphere_i[3]) * (dist + sphere_i[3]) > r2 {
                for j in 0..3 {
                    v[j] = sphere_i[j] - sphere[j];
                    s1[j] = sphere[j] - (sphere[3] / dist) * v[j];
                    s2[j] = sphere[j] + (T::one() + sphere_i[3] / dist) * v[j];
                    sphere[j] = (s1[j] + s2[j]) / T::two();
                }
                r2 = distance2_between_points([s1[0], s1[1], s1[2]], [s2[0], s2[1], s2[2]])
                    / T::four();
                if r2 > T::zero() {
                    sphere[3] = r2.sqrt();
                } else {
                    sphere[3] = s1[3].max(sphere[3]);
                    r2 = sphere[3] * sphere[3];
                }
            }
        }
    }
}

fn initial_point_diameter<T: SphereScalar>(pts: &[T], num_pts: VtkIdType) -> ([T; 3], [T; 3]) {
    let mut x_min = [T::float_max(); 3];
    let mut y_min = [T::float_max(); 3];
    let mut z_min = [T::float_max(); 3];
    let mut x_max = [-T::float_max(); 3];
    let mut y_max = [-T::float_max(); 3];
    let mut z_max = [-T::float_max(); 3];

    for i in 0..num_pts {
        let point = point_at(pts, num_pts, i).expect("point length prechecked");
        if point[0] < x_min[0] {
            x_min = point;
        }
        if point[0] > x_max[0] {
            x_max = point;
        }
        if point[1] < y_min[1] {
            y_min = point;
        }
        if point[1] > y_max[1] {
            y_max = point;
        }
        if point[2] < z_min[2] {
            z_min = point;
        }
        if point[2] > z_max[2] {
            z_max = point;
        }
    }

    let x_span = distance2_between_points(x_min, x_max);
    let y_span = distance2_between_points(y_min, y_max);
    let z_span = distance2_between_points(z_min, z_max);

    if x_span > y_span {
        if x_span > z_span {
            (x_min, x_max)
        } else {
            (z_min, z_max)
        }
    } else if y_span > z_span {
        (y_min, y_max)
    } else {
        (z_min, z_max)
    }
}

fn initial_sphere_diameter<T: SphereScalar>(
    spheres: &[[T; 4]],
    num_spheres: VtkIdType,
) -> ([T; 4], [T; 4]) {
    let mut x_min = [T::float_max(), T::float_max(), T::float_max(), T::zero()];
    let mut y_min = [T::float_max(), T::float_max(), T::float_max(), T::zero()];
    let mut z_min = [T::float_max(), T::float_max(), T::float_max(), T::zero()];
    let mut x_max = [-T::float_max(), -T::float_max(), -T::float_max(), T::zero()];
    let mut y_max = [-T::float_max(), -T::float_max(), -T::float_max(), T::zero()];
    let mut z_max = [-T::float_max(), -T::float_max(), -T::float_max(), T::zero()];

    for sphere in spheres.iter().take(num_spheres as usize).copied() {
        if sphere[0] - sphere[3] < x_min[0] - x_min[3] {
            x_min = sphere;
        }
        if sphere[0] + sphere[3] > x_max[0] + x_max[3] {
            x_max = sphere;
        }
        if sphere[1] - sphere[3] < y_min[1] - y_min[3] {
            y_min = sphere;
        }
        if sphere[1] + sphere[3] > y_max[1] + y_max[3] {
            y_max = sphere;
        }
        if sphere[2] - sphere[3] < z_min[2] - z_min[3] {
            z_min = sphere;
        }
        if sphere[2] + sphere[3] > z_max[2] + z_max[3] {
            z_max = sphere;
        }
    }

    let x_span = sphere_span(x_min, x_max);
    let y_span = sphere_span(y_min, y_max);
    let z_span = sphere_span(z_min, z_max);

    if x_span > y_span {
        if x_span > z_span {
            (x_min, x_max)
        } else {
            (z_min, z_max)
        }
    } else if y_span > z_span {
        (y_min, y_max)
    } else {
        (z_min, z_max)
    }
}

fn point_at<T: Copy>(pts: &[T], num_pts: VtkIdType, index: VtkIdType) -> Option<[T; 3]> {
    if index < 0 || index >= num_pts {
        return None;
    }
    let start = 3usize.checked_mul(index as usize)?;
    Some([*pts.get(start)?, *pts.get(start + 1)?, *pts.get(start + 2)?])
}

fn sphere_at<T: Copy>(
    spheres: &[[T; 4]],
    num_spheres: VtkIdType,
    index: VtkIdType,
) -> Option<[T; 4]> {
    if index < 0 || index >= num_spheres {
        return None;
    }
    spheres.get(index as usize).copied()
}

fn distance2_between_points<T: SphereScalar>(a: [T; 3], b: [T; 3]) -> T {
    (a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1]) + (a[2] - b[2]) * (a[2] - b[2])
}

fn sphere_span<T: SphereScalar>(min_sphere: [T; 4], max_sphere: [T; 4]) -> T {
    let dx = (max_sphere[0] + max_sphere[3]) - (min_sphere[0] - min_sphere[3]);
    let dy = (max_sphere[1] + max_sphere[3]) - (min_sphere[1] - min_sphere[3]);
    let dz = (max_sphere[2] + max_sphere[3]) - (min_sphere[2] - min_sphere[3]);
    dx * dx + dy * dy + dz * dz
}
