/// Easing functions for animation interpolation.
#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
}

impl Easing {
    /// Evaluate the easing function at parameter t in [0, 1].
    pub fn eval(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Easing::EaseInCubic => t * t * t,
            Easing::EaseOutCubic => {
                let t1 = t - 1.0;
                t1 * t1 * t1 + 1.0
            }
        }
    }
}

/// A keyframe with a time and a value.
#[derive(Debug, Clone)]
pub struct Keyframe<T: Clone> {
    pub time: f64,
    pub value: T,
    pub easing: Easing,
}

/// A track of keyframes for animating a single property.
#[derive(Debug, Clone)]
pub struct Track<T: Clone> {
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone> Track<T> {
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    /// Add a keyframe at the given time.
    pub fn add(&mut self, time: f64, value: T, easing: Easing) {
        self.keyframes.push(Keyframe {
            time,
            value,
            easing,
        });
        self.keyframes
            .sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Number of keyframes.
    pub fn len(&self) -> usize {
        self.keyframes.len()
    }

    /// Whether the track is empty.
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }

    /// Total duration (time of last keyframe).
    pub fn duration(&self) -> f64 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }
}

impl Track<f64> {
    /// Evaluate the track at a given time, interpolating between keyframes.
    pub fn eval(&self, time: f64) -> f64 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        if time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value;
        }
        for i in 0..self.keyframes.len() - 1 {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            if time >= k0.time && time <= k1.time {
                let t = (time - k0.time) / (k1.time - k0.time);
                let eased = k1.easing.eval(t);
                return k0.value + eased * (k1.value - k0.value);
            }
        }
        self.keyframes.last().unwrap().value
    }
}

impl Track<[f64; 3]> {
    /// Evaluate a 3D vector track at a given time.
    pub fn eval(&self, time: f64) -> [f64; 3] {
        if self.keyframes.is_empty() {
            return [0.0; 3];
        }
        if time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }
        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value;
        }
        for i in 0..self.keyframes.len() - 1 {
            let k0 = &self.keyframes[i];
            let k1 = &self.keyframes[i + 1];
            if time >= k0.time && time <= k1.time {
                let t = (time - k0.time) / (k1.time - k0.time);
                let eased = k1.easing.eval(t);
                return [
                    k0.value[0] + eased * (k1.value[0] - k0.value[0]),
                    k0.value[1] + eased * (k1.value[1] - k0.value[1]),
                    k0.value[2] + eased * (k1.value[2] - k0.value[2]),
                ];
            }
        }
        self.keyframes.last().unwrap().value
    }
}

/// Camera animation with position and focal point tracks.
#[derive(Debug, Clone)]
pub struct CameraAnimation {
    pub position: Track<[f64; 3]>,
    pub focal_point: Track<[f64; 3]>,
}

impl CameraAnimation {
    pub fn new() -> Self {
        Self {
            position: Track::new(),
            focal_point: Track::new(),
        }
    }

    /// Apply the animation to a camera at the given time.
    pub fn apply(&self, camera: &mut crate::render::Camera, time: f64) {
        if !self.position.is_empty() {
            let p = self.position.eval(time);
            camera.position = glam::DVec3::new(p[0], p[1], p[2]);
        }
        if !self.focal_point.is_empty() {
            let f = self.focal_point.eval(time);
            camera.focal_point = glam::DVec3::new(f[0], f[1], f[2]);
        }
    }

    /// Total duration.
    pub fn duration(&self) -> f64 {
        self.position.duration().max(self.focal_point.duration())
    }

    /// Create a turntable (orbit) animation around a center point.
    ///
    /// The camera orbits the focal point at a fixed distance and elevation.
    pub fn turntable(
        center: [f64; 3],
        distance: f64,
        elevation: f64,
        duration: f64,
        num_keyframes: usize,
    ) -> Self {
        let mut anim = Self::new();
        for i in 0..=num_keyframes {
            let t = i as f64 / num_keyframes as f64;
            let angle = t * 2.0 * std::f64::consts::PI;
            let x = center[0] + distance * elevation.to_radians().cos() * angle.cos();
            let y = center[1] + distance * elevation.to_radians().sin();
            let z = center[2] + distance * elevation.to_radians().cos() * angle.sin();
            anim.position.add(t * duration, [x, y, z], Easing::Linear);
            anim.focal_point.add(t * duration, center, Easing::Linear);
        }
        anim
    }

    /// Create a zoom animation (dolly in/out).
    pub fn zoom(start_pos: [f64; 3], end_pos: [f64; 3], focal: [f64; 3], duration: f64) -> Self {
        let mut anim = Self::new();
        anim.position.add(0.0, start_pos, Easing::EaseInOutQuad);
        anim.position.add(duration, end_pos, Easing::EaseInOutQuad);
        anim.focal_point.add(0.0, focal, Easing::Linear);
        anim.focal_point.add(duration, focal, Easing::Linear);
        anim
    }
}

impl Default for CameraAnimation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_linear() {
        assert!((Easing::Linear.eval(0.5) - 0.5).abs() < 1e-12);
        assert!((Easing::Linear.eval(0.0)).abs() < 1e-12);
        assert!((Easing::Linear.eval(1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn easing_ease_in_quad() {
        assert!((Easing::EaseInQuad.eval(0.5) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn track_f64_interpolation() {
        let mut track = Track::<f64>::new();
        track.add(0.0, 0.0, Easing::Linear);
        track.add(1.0, 10.0, Easing::Linear);

        assert!((track.eval(0.0)).abs() < 1e-12);
        assert!((track.eval(0.5) - 5.0).abs() < 1e-12);
        assert!((track.eval(1.0) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn track_vec3_interpolation() {
        let mut track = Track::<[f64; 3]>::new();
        track.add(0.0, [0.0, 0.0, 0.0], Easing::Linear);
        track.add(2.0, [6.0, 4.0, 2.0], Easing::Linear);

        let v = track.eval(1.0);
        assert!((v[0] - 3.0).abs() < 1e-12);
        assert!((v[1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn track_eased() {
        let mut track = Track::<f64>::new();
        track.add(0.0, 0.0, Easing::Linear);
        track.add(1.0, 1.0, Easing::EaseInQuad);

        // EaseInQuad at t=0.5 should give 0.25
        assert!((track.eval(0.5) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn camera_animation() {
        let mut anim = CameraAnimation::new();
        anim.position.add(0.0, [0.0, 0.0, 5.0], Easing::Linear);
        anim.position.add(1.0, [5.0, 0.0, 5.0], Easing::Linear);

        let mut camera = crate::render::Camera::default();
        anim.apply(&mut camera, 0.5);
        assert!((camera.position.x - 2.5).abs() < 1e-12);
    }

    #[test]
    fn duration() {
        let mut anim = CameraAnimation::new();
        anim.position.add(0.0, [0.0, 0.0, 0.0], Easing::Linear);
        anim.position.add(3.0, [1.0, 0.0, 0.0], Easing::Linear);
        assert_eq!(anim.duration(), 3.0);
    }

    #[test]
    fn turntable() {
        let anim = CameraAnimation::turntable([0.0; 3], 5.0, 30.0, 2.0, 36);
        assert!((anim.duration() - 2.0).abs() < 1e-6);
        // At t=0 and t=2.0 the camera should be at approximately the same position
        let mut cam = crate::render::Camera::default();
        anim.apply(&mut cam, 0.0);
        let start = cam.position;
        anim.apply(&mut cam, 2.0);
        let end = cam.position;
        assert!((start.x - end.x).abs() < 0.1);
        assert!((start.z - end.z).abs() < 0.1);
    }

    #[test]
    fn zoom_animation() {
        let anim = CameraAnimation::zoom([0.0, 0.0, 10.0], [0.0, 0.0, 2.0], [0.0; 3], 1.0);
        let mut cam = crate::render::Camera::default();
        anim.apply(&mut cam, 0.5);
        assert!(cam.position.z > 2.0 && cam.position.z < 10.0);
    }
}
