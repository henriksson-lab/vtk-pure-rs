use std::ops::{Add, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub};

use crate::common::core::math::{degrees_from_radians, jacobi_n};

/// Scalar types supported by VTK's concrete quaternion wrappers.
pub trait QuaternionScalar:
    Copy
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + From<f32>
    + Into<f64>
{
    fn from_f64(value: f64) -> Self;
}

impl QuaternionScalar for f32 {
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl QuaternionScalar for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }
}

/// VTK: `vtkQuaternion<T>`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion<T: QuaternionScalar> {
    data: [T; 4],
}

/// VTK: `vtkQuaternionf`.
pub type Quaternionf = Quaternion<f32>;

/// VTK: `vtkQuaterniond`.
pub type Quaterniond = Quaternion<f64>;

impl<T: QuaternionScalar> Quaternion<T> {
    /// VTK: `vtkQuaternion<T>::vtkQuaternion()`.
    pub fn new() -> Self {
        Self::identity()
    }

    /// VTK: `vtkQuaternion<T>::vtkQuaternion(const T&)`.
    pub fn from_scalar(scalar: T) -> Self {
        Self { data: [scalar; 4] }
    }

    /// VTK: `vtkQuaternion<T>::vtkQuaternion(const T*)`.
    pub fn from_array(init: &[T; 4]) -> Self {
        Self { data: *init }
    }

    /// VTK: `vtkQuaternion<T>::vtkQuaternion(const T&, const T&, const T&, const T&)`.
    pub fn from_components(w: T, x: T, y: T, z: T) -> Self {
        Self { data: [w, x, y, z] }
    }

    /// VTK: `vtkTuple<T, 4>::GetSize`.
    pub fn get_size(&self) -> i32 {
        4
    }

    /// VTK: `vtkTuple<T, 4>::GetData`.
    pub fn get_data(&self) -> &[T; 4] {
        &self.data
    }

    /// VTK: `vtkTuple<T, 4>::GetData`.
    pub fn get_data_mut(&mut self) -> &mut [T; 4] {
        &mut self.data
    }

    /// VTK: `vtkTuple<T, 4>::Compare`.
    pub fn compare(&self, other: &Self, tol: T) -> bool {
        for i in 0..4 {
            let delta = (self.data[i].into() - other.data[i].into()).abs();
            if delta >= tol.into() {
                return false;
            }
        }
        true
    }

    /// VTK: `vtkQuaternion<T>::SquaredNorm`.
    pub fn squared_norm(&self) -> T {
        let mut result = 0.0;
        for value in self.data {
            let value = value.into();
            result += value * value;
        }
        T::from_f64(result)
    }

    /// VTK: `vtkQuaternion<T>::Norm`.
    pub fn norm(&self) -> T {
        T::from_f64(self.squared_norm().into().sqrt())
    }

    /// VTK: `vtkQuaternion<T>::ToIdentity`.
    pub fn to_identity(&mut self) {
        self.set(
            T::from_f64(1.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
        );
    }

    /// VTK: `vtkQuaternion<T>::Identity`.
    pub fn identity() -> Self {
        Self::from_components(
            T::from_f64(1.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
        )
    }

    /// VTK: `vtkQuaternion<T>::Normalize`.
    pub fn normalize(&mut self) -> T {
        let norm = self.norm();
        if norm != T::from_f64(0.0) {
            for value in &mut self.data {
                *value = *value / norm;
            }
        }
        norm
    }

    /// VTK: `vtkQuaternion<T>::Normalized`.
    pub fn normalized(&self) -> Self {
        let mut temp = *self;
        temp.normalize();
        temp
    }

    /// VTK: `vtkQuaternion<T>::Conjugate`.
    pub fn conjugate(&mut self) {
        for i in 1..4 {
            self.data[i] = self.data[i] * T::from_f64(-1.0);
        }
    }

    /// VTK: `vtkQuaternion<T>::Conjugated`.
    pub fn conjugated(&self) -> Self {
        let mut ret = *self;
        ret.conjugate();
        ret
    }

    /// VTK: `vtkQuaternion<T>::Invert`.
    pub fn invert(&mut self) {
        let square_norm = self.squared_norm();
        if square_norm != T::from_f64(0.0) {
            self.conjugate();
            for value in &mut self.data {
                *value = *value / square_norm;
            }
        }
    }

    /// VTK: `vtkQuaternion<T>::Inverse`.
    pub fn inverse(&self) -> Self {
        let mut ret = *self;
        ret.invert();
        ret
    }

    /// VTK: `vtkQuaternion<T>::Cast`.
    pub fn cast<U: QuaternionScalar>(&self) -> Quaternion<U> {
        Quaternion::from_components(
            U::from_f64(self.data[0].into()),
            U::from_f64(self.data[1].into()),
            U::from_f64(self.data[2].into()),
            U::from_f64(self.data[3].into()),
        )
    }

    /// VTK: `vtkQuaternion<T>::Set(const T&, const T&, const T&, const T&)`.
    pub fn set(&mut self, w: T, x: T, y: T, z: T) {
        self.data = [w, x, y, z];
    }

    /// VTK: `vtkQuaternion<T>::Set(T[4])`.
    pub fn set_array(&mut self, quat: &[T; 4]) {
        self.data.copy_from_slice(quat);
    }

    /// VTK: `vtkQuaternion<T>::Get(T[4])`.
    pub fn get(&self, quat: &mut [T; 4]) {
        quat.copy_from_slice(&self.data);
    }

    /// VTK: `vtkQuaternion<T>::SetW`.
    pub fn set_w(&mut self, w: T) {
        self.data[0] = w;
    }

    /// VTK: `vtkQuaternion<T>::GetW`.
    pub fn get_w(&self) -> T {
        self.data[0]
    }

    /// VTK: `vtkQuaternion<T>::SetX`.
    pub fn set_x(&mut self, x: T) {
        self.data[1] = x;
    }

    /// VTK: `vtkQuaternion<T>::GetX`.
    pub fn get_x(&self) -> T {
        self.data[1]
    }

    /// VTK: `vtkQuaternion<T>::SetY`.
    pub fn set_y(&mut self, y: T) {
        self.data[2] = y;
    }

    /// VTK: `vtkQuaternion<T>::GetY`.
    pub fn get_y(&self) -> T {
        self.data[2]
    }

    /// VTK: `vtkQuaternion<T>::SetZ`.
    pub fn set_z(&mut self, z: T) {
        self.data[3] = z;
    }

    /// VTK: `vtkQuaternion<T>::GetZ`.
    pub fn get_z(&self) -> T {
        self.data[3]
    }

    /// VTK: `vtkQuaternion<T>::GetRotationAngleAndAxis`.
    pub fn get_rotation_angle_and_axis(&self, axis: &mut [T; 3]) -> T {
        let mut w = self.get_w().into();
        let x = self.get_x().into();
        let y = self.get_y().into();
        let z = self.get_z().into();
        let f = (x * x + y * y + z * z).sqrt();
        if f != 0.0 {
            axis[0] = T::from_f64(x / f);
            axis[1] = T::from_f64(y / f);
            axis[2] = T::from_f64(z / f);
        } else {
            w = 1.0;
            axis[0] = T::from_f64(0.0);
            axis[1] = T::from_f64(0.0);
            axis[2] = T::from_f64(0.0);
        }

        T::from_f64(2.0 * f.atan2(w))
    }

    /// VTK: `vtkQuaternion<T>::SetRotationAngleAndAxis(T, T[3])`.
    pub fn set_rotation_angle_and_axis(&mut self, angle: T, axis: &[T; 3]) {
        self.set_rotation_angle_and_axis_components(angle, axis[0], axis[1], axis[2]);
    }

    /// VTK: `vtkQuaternion<T>::SetRotationAngleAndAxis(const T&, const T&, const T&, const T&)`.
    pub fn set_rotation_angle_and_axis_components(&mut self, angle: T, x: T, y: T, z: T) {
        let angle = angle.into();
        let x = x.into();
        let y = y.into();
        let z = z.into();
        let axis_norm = x * x + y * y + z * z;
        if axis_norm != 0.0 {
            let f = (0.5 * angle).sin();
            self.set_w(T::from_f64((0.5 * angle).cos()));
            self.set_x(T::from_f64((x / axis_norm) * f));
            self.set_y(T::from_f64((y / axis_norm) * f));
            self.set_z(T::from_f64((z / axis_norm) * f));
        } else {
            self.set(
                T::from_f64(1.0),
                T::from_f64(0.0),
                T::from_f64(0.0),
                T::from_f64(0.0),
            );
        }
    }

    /// VTK: `vtkQuaternion<T>::ToMatrix3x3`.
    pub fn to_matrix3x3(&self, a: &mut [[T; 3]; 3]) {
        let ww = self.data[0].into() * self.data[0].into();
        let wx = self.data[0].into() * self.data[1].into();
        let wy = self.data[0].into() * self.data[2].into();
        let wz = self.data[0].into() * self.data[3].into();

        let xx = self.data[1].into() * self.data[1].into();
        let yy = self.data[2].into() * self.data[2].into();
        let zz = self.data[3].into() * self.data[3].into();

        let xy = self.data[1].into() * self.data[2].into();
        let xz = self.data[1].into() * self.data[3].into();
        let yz = self.data[2].into() * self.data[3].into();

        let rr = xx + yy + zz;
        if ww + rr == 0.0 {
            for row in a.iter_mut() {
                for value in row.iter_mut() {
                    *value = T::from_f64(0.0);
                }
            }
            return;
        }

        let mut f = 1.0 / (ww + rr);
        let s = (ww - rr) * f;
        f *= 2.0;

        a[0][0] = T::from_f64(xx * f + s);
        a[1][0] = T::from_f64((xy + wz) * f);
        a[2][0] = T::from_f64((xz - wy) * f);
        a[0][1] = T::from_f64((xy - wz) * f);
        a[1][1] = T::from_f64(yy * f + s);
        a[2][1] = T::from_f64((yz + wx) * f);
        a[0][2] = T::from_f64((xz + wy) * f);
        a[1][2] = T::from_f64((yz - wx) * f);
        a[2][2] = T::from_f64(zz * f + s);
    }

    /// VTK: `vtkQuaternion<T>::FromMatrix3x3`.
    pub fn from_matrix3x3(&mut self, a: &[[T; 3]; 3]) {
        let a = [
            [a[0][0].into(), a[0][1].into(), a[0][2].into()],
            [a[1][0].into(), a[1][1].into(), a[1][2].into()],
            [a[2][0].into(), a[2][1].into(), a[2][2].into()],
        ];
        let n = [
            [
                a[0][0] + a[1][1] + a[2][2],
                a[2][1] - a[1][2],
                a[0][2] - a[2][0],
                a[1][0] - a[0][1],
            ],
            [
                a[2][1] - a[1][2],
                a[0][0] - a[1][1] - a[2][2],
                a[1][0] + a[0][1],
                a[0][2] + a[2][0],
            ],
            [
                a[0][2] - a[2][0],
                a[1][0] + a[0][1],
                -a[0][0] + a[1][1] - a[2][2],
                a[2][1] + a[1][2],
            ],
            [
                a[1][0] - a[0][1],
                a[0][2] + a[2][0],
                a[2][1] + a[1][2],
                -a[0][0] - a[1][1] + a[2][2],
            ],
        ];
        let rows: [&[f64]; 4] = [&n[0], &n[1], &n[2], &n[3]];
        let (_success, _mutated_n, _eigenvalues, eigenvectors) = jacobi_n(&rows, 4);
        for (i, value) in self.data.iter_mut().enumerate() {
            *value = T::from_f64(eigenvectors[i][0]);
        }
    }

    /// VTK: `vtkQuaternion<T>::Slerp`.
    pub fn slerp(&self, t: T, q1: &Self) -> Self {
        let mut cos_theta = self.get_w().into() * q1.get_w().into()
            + self.get_x().into() * q1.get_x().into()
            + self.get_y().into() * q1.get_y().into()
            + self.get_z().into() * q1.get_z().into();

        let mut q_closest = *q1;
        if cos_theta < 0.0 {
            cos_theta = -cos_theta;
            q_closest = q_closest * T::from_f64(-1.0);
        }

        let t = t.into();
        let (t1, t2) = if (1.0 - cos_theta.abs()) < 1e-6 {
            (1.0 - t, t)
        } else {
            let theta = cos_theta.acos();
            (
                ((1.0 - t) * theta).sin() / theta.sin(),
                (t * theta).sin() / theta.sin(),
            )
        };

        (*self) * T::from_f64(t1) + q_closest * T::from_f64(t2)
    }

    /// VTK: `vtkQuaternion<T>::InnerPoint`.
    pub fn inner_point(&self, q1: &Self, q2: &Self) -> Self {
        let q_inv = q1.inverse();
        let q_l = q_inv * *q2;
        let q_r = q_inv * *self;

        let q_l_log = q_l.unit_log();
        let q_r_log = q_r.unit_log();
        let mut q_sum = q_l_log + q_r_log;
        let w = q_sum.get_w();
        q_sum /= T::from_f64(-4.0);
        q_sum.set_w(w);

        let q_exp = q_sum.unit_exp();
        *q1 * q_exp
    }

    /// VTK: `vtkQuaternion<T>::ToUnitLog`.
    pub fn to_unit_log(&mut self) {
        let mut axis = [T::from_f64(0.0); 3];
        let angle = T::from_f64(0.5) * self.get_rotation_angle_and_axis(&mut axis);
        self.set(
            T::from_f64(0.0),
            angle * axis[0],
            angle * axis[1],
            angle * axis[2],
        );
    }

    /// VTK: `vtkQuaternion<T>::UnitLog`.
    pub fn unit_log(&self) -> Self {
        let mut unit_log = *self;
        unit_log.to_unit_log();
        unit_log
    }

    /// VTK: `vtkQuaternion<T>::ToUnitExp`.
    pub fn to_unit_exp(&mut self) {
        let mut x = self.get_x().into();
        let mut y = self.get_y().into();
        let mut z = self.get_z().into();
        let angle = (x * x + y * y + z * z).sqrt();
        let sin_angle = angle.sin();
        let cos_angle = angle.cos();
        if angle != 0.0 {
            x /= angle;
            y /= angle;
            z /= angle;
        }

        self.set(
            T::from_f64(cos_angle),
            T::from_f64(sin_angle * x),
            T::from_f64(sin_angle * y),
            T::from_f64(sin_angle * z),
        );
    }

    /// VTK: `vtkQuaternion<T>::UnitExp`.
    pub fn unit_exp(&self) -> Self {
        let mut unit_exp = *self;
        unit_exp.to_unit_exp();
        unit_exp
    }

    /// VTK: `vtkQuaternion<T>::NormalizeWithAngleInDegrees`.
    pub fn normalize_with_angle_in_degrees(&mut self) {
        self.normalize();
        self.set_w(T::from_f64(degrees_from_radians(self.get_w().into())));
    }

    /// VTK: `vtkQuaternion<T>::NormalizedWithAngleInDegrees`.
    pub fn normalized_with_angle_in_degrees(&self) -> Self {
        let mut unit_vtk = *self;
        unit_vtk.normalize();
        unit_vtk.set_w(T::from_f64(degrees_from_radians(unit_vtk.get_w().into())));
        unit_vtk
    }
}

impl<T: QuaternionScalar> Default for Quaternion<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: QuaternionScalar> Index<usize> for Quaternion<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: QuaternionScalar> IndexMut<usize> for Quaternion<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T: QuaternionScalar> Add for Quaternion<T> {
    type Output = Self;

    /// VTK: `vtkQuaternion<T>::operator+`.
    fn add(self, q: Self) -> Self::Output {
        Self::from_components(
            self.data[0] + q.data[0],
            self.data[1] + q.data[1],
            self.data[2] + q.data[2],
            self.data[3] + q.data[3],
        )
    }
}

impl<T: QuaternionScalar> Sub for Quaternion<T> {
    type Output = Self;

    /// VTK: `vtkQuaternion<T>::operator-`.
    fn sub(self, q: Self) -> Self::Output {
        Self::from_components(
            self.data[0] - q.data[0],
            self.data[1] - q.data[1],
            self.data[2] - q.data[2],
            self.data[3] - q.data[3],
        )
    }
}

impl<T: QuaternionScalar> Mul for Quaternion<T> {
    type Output = Self;

    /// VTK: `vtkQuaternion<T>::operator*(const vtkQuaternion<T>&)`.
    fn mul(self, q: Self) -> Self::Output {
        let ww = self.data[0] * q.data[0];
        let wx = self.data[0] * q.data[1];
        let wy = self.data[0] * q.data[2];
        let wz = self.data[0] * q.data[3];

        let xw = self.data[1] * q.data[0];
        let xx = self.data[1] * q.data[1];
        let xy = self.data[1] * q.data[2];
        let xz = self.data[1] * q.data[3];

        let yw = self.data[2] * q.data[0];
        let yx = self.data[2] * q.data[1];
        let yy = self.data[2] * q.data[2];
        let yz = self.data[2] * q.data[3];

        let zw = self.data[3] * q.data[0];
        let zx = self.data[3] * q.data[1];
        let zy = self.data[3] * q.data[2];
        let zz = self.data[3] * q.data[3];

        Self::from_components(
            ww - xx - yy - zz,
            wx + xw + yz - zy,
            wy - xz + yw + zx,
            wz + xy - yx + zw,
        )
    }
}

impl<T: QuaternionScalar> Mul<T> for Quaternion<T> {
    type Output = Self;

    /// VTK: `vtkQuaternion<T>::operator*(const T&)`.
    fn mul(self, scalar: T) -> Self::Output {
        Self::from_components(
            self.data[0] * scalar,
            self.data[1] * scalar,
            self.data[2] * scalar,
            self.data[3] * scalar,
        )
    }
}

impl<T: QuaternionScalar> MulAssign<T> for Quaternion<T> {
    /// VTK: `vtkQuaternion<T>::operator*=`.
    fn mul_assign(&mut self, scalar: T) {
        for value in &mut self.data {
            *value = *value * scalar;
        }
    }
}

impl<T: QuaternionScalar> Div for Quaternion<T> {
    type Output = Self;

    /// VTK: `vtkQuaternion<T>::operator/(const vtkQuaternion<T>&)`.
    fn div(self, q: Self) -> Self::Output {
        self * q.inverse()
    }
}

impl<T: QuaternionScalar> Div<T> for Quaternion<T> {
    type Output = Self;

    /// VTK: `vtkQuaternion<T>::operator/(const T&)`.
    fn div(self, scalar: T) -> Self::Output {
        Self::from_components(
            self.data[0] / scalar,
            self.data[1] / scalar,
            self.data[2] / scalar,
            self.data[3] / scalar,
        )
    }
}

impl<T: QuaternionScalar> DivAssign<T> for Quaternion<T> {
    /// VTK: `vtkQuaternion<T>::operator/=`.
    fn div_assign(&mut self, scalar: T) {
        for value in &mut self.data {
            *value = *value / scalar;
        }
    }
}
