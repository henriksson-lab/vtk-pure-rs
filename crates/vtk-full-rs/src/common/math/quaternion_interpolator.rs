use crate::common::core::{object::Object, vtk_type::VtkMTimeType};

use super::quaternion::Quaterniond;

/// VTK: `TimedQuaternion` internal helper in `vtkQuaternionInterpolator.cxx`.
#[derive(Debug, Clone, Copy)]
struct TimedQuaternion {
    time: f64,
    q: Quaterniond,
}

impl TimedQuaternion {
    fn new(time: f64, q: Quaterniond) -> Self {
        Self { time, q }
    }
}

/// VTK: `vtkQuaternionInterpolator`.
#[derive(Debug, Clone)]
pub struct QuaternionInterpolator {
    object: Object,
    interpolation_type: i32,
    search_method: i32,
    quaternion_list: Vec<TimedQuaternion>,
}

impl QuaternionInterpolator {
    /// VTK: `vtkQuaternionInterpolator::BinarySearch`.
    pub const BINARY_SEARCH: i32 = 0;

    /// VTK: `vtkQuaternionInterpolator::LinearSearch`.
    pub const LINEAR_SEARCH: i32 = 1;

    /// VTK: `vtkQuaternionInterpolator::MaxEnum`.
    pub const MAX_ENUM: i32 = 3;

    /// VTK: `vtkQuaternionInterpolator::INTERPOLATION_TYPE_LINEAR`.
    pub const INTERPOLATION_TYPE_LINEAR: i32 = 0;

    /// VTK: `vtkQuaternionInterpolator::INTERPOLATION_TYPE_SPLINE`.
    pub const INTERPOLATION_TYPE_SPLINE: i32 = 1;

    /// VTK: `vtkQuaternionInterpolator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkQuaternionInterpolator"),
            interpolation_type: Self::INTERPOLATION_TYPE_SPLINE,
            search_method: Self::BINARY_SEARCH,
            quaternion_list: Vec::new(),
        }
    }

    /// VTK: `vtkQuaternionInterpolator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nQuaternionList: {} quaternions to interpolate\nInterpolationType: {}",
            self.object.get_class_name(),
            self.quaternion_list.len(),
            if self.interpolation_type == Self::INTERPOLATION_TYPE_LINEAR {
                "Linear"
            } else {
                "Spline"
            }
        )
    }

    /// VTK: `vtkQuaternionInterpolator::GetNumberOfQuaternions`.
    pub fn get_number_of_quaternions(&self) -> i32 {
        self.quaternion_list.len() as i32
    }

    /// VTK: `vtkQuaternionInterpolator::GetMinimumT`.
    pub fn get_minimum_t(&self) -> f64 {
        if !self.quaternion_list.is_empty() {
            self.quaternion_list[0].time
        } else {
            0.0
        }
    }

    /// VTK: `vtkQuaternionInterpolator::GetMaximumT`.
    pub fn get_maximum_t(&self) -> f64 {
        if !self.quaternion_list.is_empty() {
            self.quaternion_list[self.quaternion_list.len() - 1].time
        } else {
            0.0
        }
    }

    /// VTK: `vtkQuaternionInterpolator::Initialize`.
    pub fn initialize(&mut self) {
        self.quaternion_list.clear();
    }

    /// VTK: `vtkQuaternionInterpolator::AddQuaternion(double, double[4])`.
    pub fn add_quaternion_from_array(&mut self, t: f64, q: &[f64; 4]) {
        let quat = Quaterniond::from_array(q);
        self.add_quaternion(t, &quat);
    }

    /// VTK: `vtkQuaternionInterpolator::AddQuaternion(double, const vtkQuaterniond&)`.
    pub fn add_quaternion(&mut self, t: f64, q: &Quaterniond) {
        let size = self.quaternion_list.len();

        if size == 0 || t < self.quaternion_list[0].time {
            self.quaternion_list.insert(0, TimedQuaternion::new(t, *q));
            return;
        } else if t > self.quaternion_list[size - 1].time {
            self.quaternion_list.push(TimedQuaternion::new(t, *q));
            return;
        } else if size == 1 && t == self.quaternion_list[0].time {
            self.quaternion_list[0] = TimedQuaternion::new(t, *q);
            return;
        }

        for i in 0..(size - 1) {
            if t == self.quaternion_list[i].time {
                self.quaternion_list[i] = TimedQuaternion::new(t, *q);
                break;
            } else if t > self.quaternion_list[i].time && t < self.quaternion_list[i + 1].time {
                self.quaternion_list
                    .insert(i + 1, TimedQuaternion::new(t, *q));
                break;
            }
        }

        self.object.modified();
    }

    /// VTK: `vtkQuaternionInterpolator::RemoveQuaternion`.
    pub fn remove_quaternion(&mut self, t: f64) {
        if t < self.quaternion_list[0].time
            || t > self.quaternion_list[self.quaternion_list.len() - 1].time
        {
            return;
        }

        if let Some(index) = self.quaternion_list.iter().position(|item| item.time == t) {
            self.quaternion_list.remove(index);
        }

        self.object.modified();
    }

    /// VTK: `vtkQuaternionInterpolator::InterpolateQuaternion(double, double[4])`.
    pub fn interpolate_quaternion_to_array(&self, t: f64, q: &mut [f64; 4]) {
        let mut quat = Quaterniond::from_array(q);
        self.interpolate_quaternion(t, &mut quat);
        for i in 0..4 {
            q[i] = quat[i];
        }
    }

    /// VTK: `vtkQuaternionInterpolator::InterpolateQuaternion(double, vtkQuaterniond&)`.
    pub fn interpolate_quaternion(&self, t: f64, q: &mut Quaterniond) {
        if t <= self.quaternion_list[0].time {
            *q = self.quaternion_list[0].q;
            return;
        } else if t >= self.quaternion_list[self.quaternion_list.len() - 1].time {
            *q = self.quaternion_list[self.quaternion_list.len() - 1].q;
            return;
        }

        let num_quats = self.get_number_of_quaternions();
        if self.interpolation_type == Self::INTERPOLATION_TYPE_LINEAR || num_quats < 3 {
            if self.search_method == Self::BINARY_SEARCH {
                let up_bound = self.upper_bound(t);
                if up_bound == 0 {
                    *q = self.quaternion_list[0].q;
                    return;
                }

                let low_bound = up_bound - 1;
                let t_norm = (t - self.quaternion_list[low_bound].time)
                    / (self.quaternion_list[up_bound].time - self.quaternion_list[low_bound].time);
                *q = self.quaternion_list[low_bound]
                    .q
                    .slerp(t_norm, &self.quaternion_list[up_bound].q);
            } else {
                for i in 0..(self.quaternion_list.len() - 1) {
                    let iter = self.quaternion_list[i];
                    let next = self.quaternion_list[i + 1];
                    if iter.time <= t && t <= next.time {
                        let t_norm = (t - iter.time) / (next.time - iter.time);
                        *q = iter.q.slerp(t_norm, &next.q);
                        break;
                    }
                }
            }
        } else {
            self.interpolate_spline(t, q);
        }
    }

    /// VTK: `vtkQuaternionInterpolator::GetSearchMethod`.
    pub fn get_search_method(&self) -> i32 {
        self.search_method
    }

    /// VTK: `vtkQuaternionInterpolator::SetSearchMethod`.
    pub fn set_search_method(&mut self, type_: i32) {
        if type_ < 0 || type_ >= Self::MAX_ENUM {
            self.search_method = Self::BINARY_SEARCH;
        }

        self.search_method = type_;
    }

    /// VTK: `vtkQuaternionInterpolator::SetInterpolationType`.
    pub fn set_interpolation_type(&mut self, interpolation_type: i32) {
        let clamped = interpolation_type.clamp(
            Self::INTERPOLATION_TYPE_LINEAR,
            Self::INTERPOLATION_TYPE_SPLINE,
        );
        if self.interpolation_type != clamped {
            self.interpolation_type = clamped;
            self.object.modified();
        }
    }

    /// VTK: `vtkQuaternionInterpolator::GetInterpolationType`.
    pub fn get_interpolation_type(&self) -> i32 {
        self.interpolation_type
    }

    /// VTK: `vtkQuaternionInterpolator::SetInterpolationTypeToLinear`.
    pub fn set_interpolation_type_to_linear(&mut self) {
        self.set_interpolation_type(Self::INTERPOLATION_TYPE_LINEAR);
    }

    /// VTK: `vtkQuaternionInterpolator::SetInterpolationTypeToSpline`.
    pub fn set_interpolation_type_to_spline(&mut self) {
        self.set_interpolation_type(Self::INTERPOLATION_TYPE_SPLINE);
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkQuaternionInterpolator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkQuaternionInterpolator" || Object::is_type_of(name)
    }

    /// VTK: `vtkQuaternionInterpolator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    fn upper_bound(&self, t: f64) -> usize {
        self.quaternion_list
            .partition_point(|timed_quaternion| timed_quaternion.time <= t)
    }

    fn interpolate_spline(&self, t: f64, q: &mut Quaterniond) {
        let mut iter = 0;
        let mut next_iter = 1;
        let mut t_norm = 0.0;
        let i;

        if self.search_method == Self::BINARY_SEARCH {
            let up_bound = self.upper_bound(t);
            let low_bound = up_bound - 1;
            t_norm = (t - self.quaternion_list[low_bound].time)
                / (self.quaternion_list[up_bound].time - self.quaternion_list[low_bound].time);
            iter = low_bound;
            next_iter = up_bound;
            i = iter;
        } else {
            let mut found_i = 0;
            while next_iter != self.quaternion_list.len() {
                if self.quaternion_list[iter].time <= t && t <= self.quaternion_list[next_iter].time
                {
                    t_norm = (t - self.quaternion_list[iter].time)
                        / (self.quaternion_list[next_iter].time - self.quaternion_list[iter].time);
                    break;
                }
                iter += 1;
                next_iter += 1;
                found_i += 1;
            }
            i = found_i;
        }

        let iter1;
        let iter2;
        let ai;
        let bi;
        if i == 0 {
            iter1 = iter;
            iter2 = next_iter;
            let iter3 = next_iter + 1;

            ai = self.quaternion_list[iter1].q.normalized();
            let q1 = self.quaternion_list[iter1].q.normalized();
            bi = q1.inner_point(
                &self.quaternion_list[iter2].q.normalized(),
                &self.quaternion_list[iter3].q.normalized(),
            );
        } else if i == (self.get_number_of_quaternions() as usize - 2) {
            let iter0 = iter - 1;
            iter1 = iter;
            iter2 = next_iter;

            let q0 = self.quaternion_list[iter0].q.normalized();
            ai = q0.inner_point(
                &self.quaternion_list[iter1].q.normalized(),
                &self.quaternion_list[iter2].q.normalized(),
            );

            bi = self.quaternion_list[iter2].q.normalized();
        } else {
            let iter0 = iter - 1;
            iter1 = iter;
            iter2 = next_iter;
            let iter3 = next_iter + 1;

            let q0 = self.quaternion_list[iter0].q.normalized();
            ai = q0.inner_point(
                &self.quaternion_list[iter1].q.normalized(),
                &self.quaternion_list[iter2].q.normalized(),
            );

            let q1 = self.quaternion_list[iter1].q.normalized();
            bi = q1.inner_point(
                &self.quaternion_list[iter2].q.normalized(),
                &self.quaternion_list[iter3].q.normalized(),
            );
        }

        let q1 = self.quaternion_list[iter1].q.normalized();
        let qc = q1.slerp(t_norm, &self.quaternion_list[iter2].q.normalized());
        let qd = ai.slerp(t_norm, &bi);
        *q = qc.slerp(2.0 * t_norm * (1.0 - t_norm), &qd);
        q.normalize_with_angle_in_degrees();
    }
}

impl Default for QuaternionInterpolator {
    fn default() -> Self {
        Self::new()
    }
}
