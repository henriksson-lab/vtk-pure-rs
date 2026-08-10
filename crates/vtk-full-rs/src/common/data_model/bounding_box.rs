use crate::common::core::{Points, VtkIdType};
use std::sync::atomic::{AtomicU8, Ordering};

/// Axis-aligned bounding box in 3D space.
///
/// VTK origin: `VTK/Common/DataModel/vtkBoundingBox.cxx`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
}

/// VTK: `vtkBoundingBox::ContainsLine` return value plus out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBoxLineContainment {
    pub contained: bool,
    pub t: f64,
    pub x_int: [f64; 3],
    pub plane: i32,
}

/// VTK: `vtkBoundingBox::ComputeDivisions` return value plus out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBoxDivisions {
    pub number_of_bins: VtkIdType,
    pub bounds: [f64; 6],
    pub divisions: [i32; 3],
}

fn sign(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn opposite_sign(a: f64, b: f64) -> bool {
    (a <= 0.0 && b >= 0.0) || (a >= 0.0 && b <= 0.0)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl BoundingBox {
    /// An empty inverted bounding box that expands on the first point added.
    pub(crate) fn empty() -> Self {
        Self {
            x_min: f64::INFINITY,
            x_max: f64::NEG_INFINITY,
            y_min: f64::INFINITY,
            y_max: f64::NEG_INFINITY,
            z_min: f64::INFINITY,
            z_max: f64::NEG_INFINITY,
        }
    }

    pub(crate) fn from_corners(min: [f64; 3], max: [f64; 3]) -> Self {
        Self {
            x_min: min[0],
            x_max: max[0],
            y_min: min[1],
            y_max: max[1],
            z_min: min[2],
            z_max: max[2],
        }
    }

    /// VTK: `vtkBoundingBox::vtkBoundingBox()`.
    pub fn new() -> Self {
        Self::empty()
    }

    /// VTK: `vtkBoundingBox::vtkBoundingBox(const double bounds[6])`.
    pub fn new_with_bounds(bounds: [f64; 6]) -> Self {
        Self::from_bounds(bounds)
    }

    /// VTK: `vtkBoundingBox::vtkBoundingBox(double xMin, double xMax, double yMin, double yMax, double zMin, double zMax)`.
    pub fn new_with_extents(
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_min: f64,
        z_max: f64,
    ) -> Self {
        let mut bbox = Self::empty();
        bbox.set_bounds([x_min, x_max, y_min, y_max, z_min, z_max]);
        bbox
    }

    /// VTK: `vtkBoundingBox::vtkBoundingBox(const double min[3], const double max[3])`.
    pub fn new_with_min_max(min: [f64; 3], max: [f64; 3]) -> Self {
        Self::from_corners(min, max)
    }

    /// VTK: `vtkBoundingBox::vtkBoundingBox(double center[3], double delta)`.
    pub fn new_with_center_delta(center: [f64; 3], delta: f64) -> Self {
        let mut bbox = Self::empty();
        bbox.add_point(center);
        bbox.inflate(delta);
        bbox
    }

    /// VTK: `vtkBoundingBox::SetBounds`.
    pub(crate) fn from_bounds(bounds: [f64; 6]) -> Self {
        Self {
            x_min: bounds[0],
            x_max: bounds[1],
            y_min: bounds[2],
            y_max: bounds[3],
            z_min: bounds[4],
            z_max: bounds[5],
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.x_min > self.x_max || self.y_min > self.y_max || self.z_min > self.z_max
    }

    /// VTK: `vtkBoundingBox::IsValid`.
    pub fn is_valid(&self) -> bool {
        !self.is_empty()
    }

    /// VTK: `vtkBoundingBox::IsValid(const double bounds[6])`.
    pub fn is_valid_bounds(bounds: [f64; 6]) -> bool {
        bounds[0] <= bounds[1] && bounds[2] <= bounds[3] && bounds[4] <= bounds[5]
    }

    /// VTK: `vtkBoundingBox::Reset`.
    pub fn reset(&mut self) {
        *self = Self::empty();
    }

    /// VTK: `vtkBoundingBox::AddPoint`.
    pub fn add_point(&mut self, point: [f64; 3]) {
        self.x_min = self.x_min.min(point[0]);
        self.x_max = self.x_max.max(point[0]);
        self.y_min = self.y_min.min(point[1]);
        self.y_max = self.y_max.max(point[1]);
        self.z_min = self.z_min.min(point[2]);
        self.z_max = self.z_max.max(point[2]);
    }

    /// VTK: `vtkBoundingBox::AddBounds`.
    pub fn add_bounds(&mut self, bounds: [f64; 6]) {
        if !Self::is_valid_bounds(bounds) {
            return;
        }
        if self.is_empty() {
            self.set_bounds(bounds);
            return;
        }
        let other = Self::from_bounds(bounds);
        self.add_point(other.get_min_point());
        self.add_point(other.get_max_point());
    }

    /// VTK: `vtkBoundingBox::AddBox`.
    pub fn add_box(&mut self, other: &BoundingBox) {
        self.add_bounds(other.get_bounds());
    }

    /// VTK: `vtkBoundingBox::SetBounds`.
    pub fn set_bounds(&mut self, bounds: [f64; 6]) {
        *self = Self::from_bounds(bounds);
    }

    /// VTK: `vtkBoundingBox::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        [
            self.x_min, self.x_max, self.y_min, self.y_max, self.z_min, self.z_max,
        ]
    }

    /// VTK: `vtkBoundingBox::GetBound`.
    pub fn get_bound(&self, index: usize) -> f64 {
        self.get_bounds()[index]
    }

    /// VTK: `vtkBoundingBox::SetMinPoint`.
    pub fn set_min_point(&mut self, point: [f64; 3]) {
        self.x_min = point[0];
        self.x_max = self.x_max.max(point[0]);
        self.y_min = point[1];
        self.y_max = self.y_max.max(point[1]);
        self.z_min = point[2];
        self.z_max = self.z_max.max(point[2]);
    }

    /// VTK: `vtkBoundingBox::SetMaxPoint`.
    pub fn set_max_point(&mut self, point: [f64; 3]) {
        self.x_max = point[0];
        self.x_min = self.x_min.min(point[0]);
        self.y_max = point[1];
        self.y_min = self.y_min.min(point[1]);
        self.z_max = point[2];
        self.z_min = self.z_min.min(point[2]);
    }

    /// VTK: `vtkBoundingBox::GetMinPoint`.
    pub fn get_min_point(&self) -> [f64; 3] {
        [self.x_min, self.y_min, self.z_min]
    }

    /// VTK: `vtkBoundingBox::GetMaxPoint`.
    pub fn get_max_point(&self) -> [f64; 3] {
        [self.x_max, self.y_max, self.z_max]
    }

    fn center(&self) -> [f64; 3] {
        [
            (self.x_min + self.x_max) * 0.5,
            (self.y_min + self.y_max) * 0.5,
            (self.z_min + self.z_max) * 0.5,
        ]
    }

    /// VTK: `vtkBoundingBox::GetCenter`.
    pub fn get_center(&self) -> [f64; 3] {
        self.center()
    }

    /// VTK: `vtkBoundingBox::GetDiagonalLength`.
    pub fn get_diagonal_length(&self) -> f64 {
        let dx = self.x_max - self.x_min;
        let dy = self.y_max - self.y_min;
        let dz = self.z_max - self.z_min;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// VTK: `vtkBoundingBox::GetDiagonalLength2`.
    pub fn get_diagonal_length2(&self) -> f64 {
        let dx = self.x_max - self.x_min;
        let dy = self.y_max - self.y_min;
        let dz = self.z_max - self.z_min;
        dx * dx + dy * dy + dz * dz
    }

    fn size(&self) -> [f64; 3] {
        [
            self.x_max - self.x_min,
            self.y_max - self.y_min,
            self.z_max - self.z_min,
        ]
    }

    /// VTK: `vtkBoundingBox::GetLengths`.
    pub fn get_lengths(&self) -> [f64; 3] {
        self.size()
    }

    /// VTK: `vtkBoundingBox::GetLength`.
    pub fn get_length(&self, axis: usize) -> f64 {
        self.size()[axis]
    }

    /// VTK: `vtkBoundingBox::GetMaxLength`.
    pub fn get_max_length(&self) -> f64 {
        let s = self.size();
        s[0].max(s[1]).max(s[2])
    }

    fn contains(&self, point: [f64; 3]) -> bool {
        point[0] >= self.x_min
            && point[0] <= self.x_max
            && point[1] >= self.y_min
            && point[1] <= self.y_max
            && point[2] >= self.z_min
            && point[2] <= self.z_max
    }

    /// VTK: `vtkBoundingBox::Contains`.
    pub fn contains_box(&self, other: &BoundingBox) -> bool {
        other.is_subset_of(self)
    }

    /// VTK: `vtkBoundingBox::ContainsPoint`.
    pub fn contains_point(&self, point: [f64; 3]) -> bool {
        self.contains(point)
    }

    /// VTK: `vtkBoundingBox::ClampPoint`.
    pub fn clamp_point(&self, point: &mut [f64; 3]) {
        if point[0] < self.x_min {
            point[0] = self.x_min;
        } else if point[0] > self.x_max {
            point[0] = self.x_max;
        }
        if point[1] < self.y_min {
            point[1] = self.y_min;
        } else if point[1] > self.y_max {
            point[1] = self.y_max;
        }
        if point[2] < self.z_min {
            point[2] = self.z_min;
        } else if point[2] > self.z_max {
            point[2] = self.z_max;
        }
    }

    /// VTK: `vtkBoundingBox::GetCorner`.
    pub fn get_corner(&self, index: usize) -> [f64; 3] {
        if index > 7 {
            return [f64::MAX; 3];
        }
        [
            if index & 1 == 0 {
                self.x_min
            } else {
                self.x_max
            },
            if index & 2 == 0 {
                self.y_min
            } else {
                self.y_max
            },
            if index & 4 == 0 {
                self.z_min
            } else {
                self.z_max
            },
        ]
    }

    /// VTK: `vtkBoundingBox::Intersects`.
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
            && self.z_min <= other.z_max
            && self.z_max >= other.z_min
    }

    /// VTK: `vtkBoundingBox::IsSubsetOf`.
    pub fn is_subset_of(&self, other: &BoundingBox) -> bool {
        self.x_max < other.x_max
            && self.x_min > other.x_min
            && self.y_max < other.y_max
            && self.y_min > other.y_min
            && self.z_max < other.z_max
            && self.z_min > other.z_min
    }

    /// VTK: `vtkBoundingBox::IntersectBox`.
    pub fn intersect_box(&mut self, other: &BoundingBox) -> bool {
        if !self.is_valid() || !other.is_valid() || !self.intersects(other) {
            return false;
        }
        let x_min = self.x_min.max(other.x_min);
        let x_max = self.x_max.min(other.x_max);
        let y_min = self.y_min.max(other.y_min);
        let y_max = self.y_max.min(other.y_max);
        let z_min = self.z_min.max(other.z_min);
        let z_max = self.z_max.min(other.z_max);
        self.set_bounds([x_min, x_max, y_min, y_max, z_min, z_max]);
        true
    }

    /// VTK: `vtkBoundingBox::Inflate`.
    pub fn inflate(&mut self, amount: f64) {
        self.inflate_with_deltas([amount, amount, amount]);
    }

    /// VTK: `vtkBoundingBox::Inflate(double deltaX, double deltaY, double deltaZ)`.
    pub fn inflate_with_deltas(&mut self, deltas: [f64; 3]) {
        self.x_min -= deltas[0];
        self.x_max += deltas[0];
        self.y_min -= deltas[1];
        self.y_max += deltas[1];
        self.z_min -= deltas[2];
        self.z_max += deltas[2];
    }

    /// VTK: `vtkBoundingBox::Inflate()`.
    pub fn inflate_to_non_zero_volume(&mut self) {
        let lengths = self.get_lengths();
        let mut max_length = 0.0;
        let mut has_non_zero = false;
        for length in lengths {
            if length > max_length {
                max_length = length;
            }
            has_non_zero |= length > 0.0;
        }
        if !has_non_zero {
            self.inflate(0.5);
            return;
        }
        let delta = 0.005 * max_length;
        if lengths[0] <= 0.0 {
            self.x_min -= delta;
            self.x_max += delta;
        }
        if lengths[1] <= 0.0 {
            self.y_min -= delta;
            self.y_max += delta;
        }
        if lengths[2] <= 0.0 {
            self.z_min -= delta;
            self.z_max += delta;
        }
    }

    /// VTK: `vtkBoundingBox::InflateSlice`.
    pub fn inflate_slice(&mut self, amount: f64) {
        let min_width = 2.0 * amount;
        if self.get_length(0) < min_width {
            self.x_min -= amount;
            self.x_max += amount;
        }
        if self.get_length(1) < min_width {
            self.y_min -= amount;
            self.y_max += amount;
        }
        if self.get_length(2) < min_width {
            self.z_min -= amount;
            self.z_max += amount;
        }
    }

    /// VTK: `vtkBoundingBox::Translate`.
    pub fn translate(&mut self, delta: [f64; 3]) {
        self.x_min += delta[0];
        self.x_max += delta[0];
        self.y_min += delta[1];
        self.y_max += delta[1];
        self.z_min += delta[2];
        self.z_max += delta[2];
    }

    /// VTK: `vtkBoundingBox::Scale`.
    pub fn scale(&mut self, factors: [f64; 3]) {
        if !self.is_valid() {
            return;
        }
        *self = BoundingBox::from_corners(
            [
                self.x_min * factors[0],
                self.y_min * factors[1],
                self.z_min * factors[2],
            ],
            [
                self.x_max * factors[0],
                self.y_max * factors[1],
                self.z_max * factors[2],
            ],
        );
        if self.x_min > self.x_max {
            std::mem::swap(&mut self.x_min, &mut self.x_max);
        }
        if self.y_min > self.y_max {
            std::mem::swap(&mut self.y_min, &mut self.y_max);
        }
        if self.z_min > self.z_max {
            std::mem::swap(&mut self.z_min, &mut self.z_max);
        }
    }

    /// VTK: `vtkBoundingBox::ScaleAboutCenter`.
    pub fn scale_about_center(&mut self, factors: [f64; 3]) {
        if !self.is_valid() {
            return;
        }
        let center = self.get_center();
        self.x_min = center[0] + factors[0] * (self.x_min - center[0]);
        self.x_max = center[0] + factors[0] * (self.x_max - center[0]);
        self.y_min = center[1] + factors[1] * (self.y_min - center[1]);
        self.y_max = center[1] + factors[1] * (self.y_max - center[1]);
        self.z_min = center[2] + factors[2] * (self.z_min - center[2]);
        self.z_max = center[2] + factors[2] * (self.z_max - center[2]);
    }

    /// VTK: `vtkBoundingBox::InsideSphere`.
    pub fn inside_sphere(&self, center: [f64; 3], radius2: f64) -> bool {
        Self::inside_sphere_with_bounds(self.get_min_point(), self.get_max_point(), center, radius2)
    }

    /// VTK: `vtkBoundingBox::InsideSphere`.
    pub fn inside_sphere_with_bounds(
        min: [f64; 3],
        max: [f64; 3],
        center: [f64; 3],
        radius2: f64,
    ) -> bool {
        let mut dmin = 0.0;
        let mut dmax = 0.0;
        for axis in 0..3 {
            let a = (center[axis] - min[axis]) * (center[axis] - min[axis]);
            let b = (center[axis] - max[axis]) * (center[axis] - max[axis]);
            dmax += a.max(b);
            if min[axis] <= center[axis] && center[axis] <= max[axis] {
                dmin += a.min(b);
            }
        }
        !(dmin <= radius2 && radius2 <= dmax)
    }

    /// VTK: `vtkBoundingBox::IntersectsSphere`.
    pub fn intersects_sphere(&self, center: [f64; 3], radius: f64) -> bool {
        center[0] >= self.x_min - radius
            && center[0] <= self.x_max + radius
            && center[1] >= self.y_min - radius
            && center[1] <= self.y_max + radius
            && center[2] >= self.z_min - radius
            && center[2] <= self.z_max + radius
    }

    /// VTK: `vtkBoundingBox::IntersectsSphere`.
    pub fn intersects_sphere_with_bounds(
        min: [f64; 3],
        max: [f64; 3],
        center: [f64; 3],
        radius2: f64,
    ) -> bool {
        let mut distance2 = 0.0;
        for axis in 0..3 {
            if center[axis] < min[axis] {
                distance2 += (center[axis] - min[axis]) * (center[axis] - min[axis]);
            } else if center[axis] > max[axis] {
                distance2 += (center[axis] - max[axis]) * (center[axis] - max[axis]);
            }
        }
        distance2 <= radius2
    }

    /// VTK: `vtkBoundingBox::IntersectsSphere2`.
    pub fn intersects_sphere2(&self, center: [f64; 3], radius2: f64) -> bool {
        if self.is_empty() {
            return false;
        }
        Self::intersects_sphere_with_bounds(
            self.get_min_point(),
            self.get_max_point(),
            center,
            radius2,
        )
    }

    /// VTK: `vtkBoundingBox::IntersectsLine`.
    pub fn intersects_line(&self, p0: [f64; 3], p1: [f64; 3]) -> bool {
        if self.is_empty() {
            return false;
        }
        let d = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let mins = self.get_min_point();
        let maxs = self.get_max_point();
        let mut tmin: f64 = 0.0;
        let mut tmax: f64 = 1.0;

        for axis in 0..3 {
            if d[axis].abs() < f64::EPSILON {
                if p0[axis] < mins[axis] || p0[axis] > maxs[axis] {
                    return false;
                }
            } else {
                let inv = 1.0 / d[axis];
                let mut t1 = (mins[axis] - p0[axis]) * inv;
                let mut t2 = (maxs[axis] - p0[axis]) * inv;
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                tmin = tmin.max(t1);
                tmax = tmax.min(t2);
                if tmin > tmax {
                    return false;
                }
            }
        }

        true
    }

    /// VTK: `vtkBoundingBox::ContainsLine`.
    pub fn contains_line(
        center: [f64; 3],
        size: [f64; 3],
        line_end: [f64; 3],
    ) -> BoundingBoxLineContainment {
        let mut t = f64::MAX;
        let mut t_min = f64::MAX;
        let mut plane = 0;
        let half_box = [size[0] / 2.0, size[1] / 2.0, size[2] / 2.0];
        let mut v = [0.0; 3];

        for axis in 0..3 {
            v[axis] = line_end[axis] - center[axis];
            if v[axis] < -half_box[axis] {
                t = -half_box[axis] / v[axis];
                if t < t_min {
                    t_min = t;
                    plane = 2 * axis as i32;
                }
            } else if v[axis] > half_box[axis] {
                t = half_box[axis] / v[axis];
                if t < t_min {
                    t_min = t;
                    plane = 2 * axis as i32 + 1;
                }
            }
        }

        if t_min == f64::MAX {
            BoundingBoxLineContainment {
                contained: true,
                t,
                x_int: [0.0; 3],
                plane,
            }
        } else {
            BoundingBoxLineContainment {
                contained: false,
                t: t_min,
                x_int: [
                    center[0] + t_min * v[0],
                    center[1] + t_min * v[1],
                    center[2] + t_min * v[2],
                ],
                plane,
            }
        }
    }

    /// VTK: `vtkBoundingBox::IntersectPlane`.
    pub fn intersect_plane(&mut self, origin: [f64; 3], normal: [f64; 3]) -> bool {
        assert!(
            self.is_valid(),
            "vtkBoundingBox::IntersectPlane requires a valid box"
        );

        const INDEX: [[usize; 8]; 3] = [
            [0, 1, 2, 3, 4, 5, 6, 7],
            [0, 1, 4, 5, 2, 3, 6, 7],
            [0, 2, 4, 6, 1, 3, 5, 7],
        ];

        let min = self.get_min_point();
        let max = self.get_max_point();
        let mut d = [0.0; 8];
        let mut index = 0;
        for ix in 0..=1 {
            for iy in 0..=1 {
                for iz in 0..=1 {
                    let x = [
                        if ix == 0 { min[0] } else { max[0] },
                        if iy == 0 { min[1] } else { max[1] },
                        if iz == 0 { min[2] } else { max[2] },
                    ];
                    d[index] = (x[0] - origin[0]) * normal[0]
                        + (x[1] - origin[1]) * normal[1]
                        + (x[2] - origin[2]) * normal[2];
                    index += 1;
                }
            }
        }

        let mut dir = None;
        for candidate in (0..3).rev() {
            let order = INDEX[candidate];
            if opposite_sign(d[order[0]], d[order[4]])
                && opposite_sign(d[order[1]], d[order[5]])
                && opposite_sign(d[order[2]], d[order[6]])
                && opposite_sign(d[order[3]], d[order[7]])
            {
                dir = Some(candidate);
                break;
            }
        }

        let Some(dir) = dir else {
            return false;
        };

        let sign = sign(normal[dir]);
        let size = ((max[dir] - min[dir]) * normal[dir]).abs();
        let mut t = if sign > 0.0 { 1.0 } else { 0.0 };
        for i in 0..4 {
            if size == 0.0 {
                continue;
            }
            let ti = d[INDEX[dir][i]].abs() / size;
            if sign > 0.0 && ti < t {
                t = ti;
            }
            if sign < 0.0 && ti > t {
                t = ti;
            }
        }

        let bound = (1.0 - t) * min[dir] + t * max[dir];
        match dir {
            0 if sign > 0.0 => self.x_min = bound,
            0 => self.x_max = bound,
            1 if sign > 0.0 => self.y_min = bound,
            1 => self.y_max = bound,
            2 if sign > 0.0 => self.z_min = bound,
            _ => self.z_max = bound,
        }
        true
    }

    /// VTK: `vtkBoundingBox::GetDistance`.
    pub fn get_distance(&self, point: [f64; 3]) -> [f64; 3] {
        let mut distance = [0.0; 3];
        for axis in 0..3 {
            let min = self.get_min_point()[axis];
            let max = self.get_max_point()[axis];
            if point[axis] < min {
                distance[axis] = point[axis] - min;
            } else if point[axis] > max {
                distance[axis] = point[axis] - max;
            }
        }
        distance
    }

    /// VTK: `vtkBoundingBox::ComputeBounds(vtkPoints*, double[6])`.
    pub fn compute_bounds(points: &Points) -> BoundingBox {
        Self::compute_bounds_from_points_iter(
            (0..points.get_number_of_points()).map(|point_id| points.get_point(point_id)),
        )
    }

    /// VTK: `vtkBoundingBox::ComputeBounds(vtkPoints*, const unsigned char*, double[6])`.
    pub fn compute_bounds_with_point_uses(points: &Points, point_uses: &[u8]) -> BoundingBox {
        Self::compute_bounds_from_points_iter((0..points.get_number_of_points()).filter_map(
            |point_id| {
                point_uses
                    .get(point_id as usize)
                    .copied()
                    .is_some_and(|use_point| use_point != 0)
                    .then(|| points.get_point(point_id))
            },
        ))
    }

    /// VTK: `vtkBoundingBox::ComputeBounds(vtkPoints*, const std::atomic<unsigned char>*, double[6])`.
    pub fn compute_bounds_with_atomic_point_uses(
        points: &Points,
        point_uses: &[AtomicU8],
    ) -> BoundingBox {
        Self::compute_bounds_from_points_iter((0..points.get_number_of_points()).filter_map(
            |point_id| {
                point_uses
                    .get(point_id as usize)
                    .is_some_and(|use_point| use_point.load(Ordering::SeqCst) != 0)
                    .then(|| points.get_point(point_id))
            },
        ))
    }

    /// VTK: `vtkBoundingBox::ComputeBounds(vtkPoints*, TIter, vtkIdType, double[6])`.
    pub fn compute_bounds_with_point_ids<I>(points: &Points, point_ids: I) -> BoundingBox
    where
        I: IntoIterator<Item = VtkIdType>,
    {
        Self::compute_bounds_from_points_iter(
            point_ids
                .into_iter()
                .map(|point_id| points.get_point(point_id)),
        )
    }

    /// VTK: `vtkBoundingBox::ComputeBounds(vtkPoints*)`.
    pub fn compute_bounds_instance(&mut self, points: &Points) {
        *self = Self::compute_bounds(points);
    }

    /// VTK: `vtkBoundingBox::ComputeBounds(vtkPoints*, unsigned char*)`.
    pub fn compute_bounds_with_point_uses_instance(&mut self, points: &Points, point_uses: &[u8]) {
        *self = Self::compute_bounds_with_point_uses(points, point_uses);
    }

    fn compute_bounds_from_points_iter(points: impl IntoIterator<Item = [f64; 3]>) -> BoundingBox {
        let mut bounds = BoundingBox::empty();
        for point in points {
            bounds.add_point(point);
        }
        bounds
    }

    /// VTK: `vtkBoundingBox::ComputeLocalBounds`.
    pub fn compute_local_bounds(
        points: &Points,
        u: [f64; 3],
        v: [f64; 3],
        w: [f64; 3],
    ) -> BoundingBox {
        let mut bounds = BoundingBox::empty();
        for point_id in 0..points.get_number_of_points() {
            let point = points.get_point(point_id);
            bounds.add_point([dot(point, u), dot(point, v), dot(point, w)]);
        }
        bounds
    }

    /// VTK: `vtkBoundingBox::ComputeInnerDimension`.
    pub fn compute_inner_dimension(&self) -> i32 {
        let min = self.get_min_point();
        let max = self.get_max_point();
        let mut dim = 3;
        for axis in 0..3 {
            let thickness = max[axis] - min[axis];
            if thickness.abs() <= max[axis].abs().max(min[axis].abs()) * f64::EPSILON {
                dim -= 1;
            }
        }
        dim
    }

    /// VTK: `vtkBoundingBox::ClampDivisions`.
    pub fn clamp_divisions(target_bins: VtkIdType, divisions: &mut [i32; 3]) {
        for value in divisions.iter_mut() {
            *value = (*value).max(1);
        }

        let target_bins = target_bins.max(1);
        let mut number_of_bins =
            divisions[0] as VtkIdType * divisions[1] as VtkIdType * divisions[2] as VtkIdType;
        while number_of_bins > target_bins {
            for value in divisions.iter_mut() {
                if *value > 1 {
                    *value -= 1;
                }
            }
            number_of_bins =
                divisions[0] as VtkIdType * divisions[1] as VtkIdType * divisions[2] as VtkIdType;
        }
    }

    /// VTK: `vtkBoundingBox::ComputeDivisions`.
    pub fn compute_divisions(&self, total_bins: VtkIdType) -> BoundingBoxDivisions {
        let total_bins = total_bins.max(1);
        let lengths = self.size();
        let total_length = lengths[0] + lengths[1] + lengths[2];
        let zero_detection_tolerance = total_length * (0.001 / 3.0);

        let mut number_of_non_zero_axes = 0;
        let mut non_zero = [false; 3];
        let mut max_axis = 0;
        let mut max_length = 0.0;
        for axis in 0..3 {
            if lengths[axis] > max_length {
                max_axis = axis;
                max_length = lengths[axis];
            }
            if lengths[axis] > zero_detection_tolerance {
                non_zero[axis] = true;
                number_of_non_zero_axes += 1;
            }
        }

        if number_of_non_zero_axes < 1 {
            return BoundingBoxDivisions {
                number_of_bins: 1,
                bounds: [
                    self.x_min - 0.5,
                    self.x_max + 0.5,
                    self.y_min - 0.5,
                    self.y_max + 0.5,
                    self.z_min - 0.5,
                    self.z_max + 0.5,
                ],
                divisions: [1, 1, 1],
            };
        }

        let mut f = total_bins as f64;
        for axis in 0..3 {
            if non_zero[axis] {
                f /= lengths[axis] / total_length;
            }
        }
        f = f.powf(1.0 / number_of_non_zero_axes as f64);

        let mut divisions = [
            if non_zero[0] {
                (f * lengths[0] / total_length).floor() as i32
            } else {
                1
            },
            if non_zero[1] {
                (f * lengths[1] / total_length).floor() as i32
            } else {
                1
            },
            if non_zero[2] {
                (f * lengths[2] / total_length).floor() as i32
            } else {
                1
            },
        ];
        for value in &mut divisions {
            *value = (*value).max(1);
        }
        Self::clamp_divisions(total_bins, &mut divisions);

        let delta = 0.5 * lengths[max_axis] / divisions[max_axis] as f64;
        let mut bounds = [0.0; 6];
        for axis in 0..3 {
            if non_zero[axis] {
                bounds[2 * axis] = self.get_min_point()[axis];
                bounds[2 * axis + 1] = self.get_max_point()[axis];
            } else {
                bounds[2 * axis] = self.get_min_point()[axis] - delta;
                bounds[2 * axis + 1] = self.get_max_point()[axis] + delta;
            }
        }

        BoundingBoxDivisions {
            number_of_bins: divisions[0] as VtkIdType
                * divisions[1] as VtkIdType
                * divisions[2] as VtkIdType,
            bounds,
            divisions,
        }
    }
}
