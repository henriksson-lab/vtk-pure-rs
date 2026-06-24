use glam::{DMat4, DVec3};

/// Virtual camera for 3D viewing.
///
/// # Examples
///
/// ```
/// use crate::render::Camera;
///
/// let mut camera = Camera::new();
/// camera.look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0]);
/// assert!(camera.distance() > 4.9);
///
/// camera.orbit(45.0, 0.0);
/// // Distance is preserved after orbit
/// assert!((camera.distance() - 5.0).abs() < 0.01);
/// ```
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: DVec3,
    pub focal_point: DVec3,
    pub view_up: DVec3,
    pub fov: f64,
    pub near_clip: f64,
    pub far_clip: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: DVec3::new(0.0, 0.0, 1.0),
            focal_point: DVec3::ZERO,
            view_up: DVec3::Y,
            fov: 30.0,
            near_clip: 0.01,
            far_clip: 1000.0,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    /// View direction (normalized).
    pub fn direction(&self) -> DVec3 {
        (self.focal_point - self.position).normalize()
    }

    /// View matrix (world → camera).
    pub fn view_matrix(&self) -> DMat4 {
        DMat4::look_at_rh(self.position, self.focal_point, self.view_up)
    }

    /// Perspective projection matrix.
    pub fn projection_matrix(&self, aspect_ratio: f64) -> DMat4 {
        let fov_rad = self.fov.to_radians();
        DMat4::perspective_rh(fov_rad, aspect_ratio, self.near_clip, self.far_clip)
    }

    /// Position camera to view the given bounding box.
    pub fn reset_to_bounds(&mut self, center: [f64; 3], diagonal: f64) {
        let c = DVec3::from_array(center);
        let distance = diagonal / (2.0 * (self.fov.to_radians() / 2.0).tan());
        self.focal_point = c;
        self.position = c + DVec3::new(0.0, 0.0, distance);
        self.near_clip = distance * 0.01;
        self.far_clip = distance * 100.0;
    }

    /// Orbit around the focal point by azimuth (degrees) and elevation (degrees).
    pub fn orbit(&mut self, azimuth_deg: f64, elevation_deg: f64) {
        let offset = self.position - self.focal_point;
        let distance = offset.length();

        // Current spherical angles
        let theta = offset.z.atan2(offset.x) + azimuth_deg.to_radians();
        let phi = (offset.y / distance).asin() + elevation_deg.to_radians();
        let phi = phi.clamp(
            -std::f64::consts::FRAC_PI_2 + 0.01,
            std::f64::consts::FRAC_PI_2 - 0.01,
        );

        self.position = self.focal_point
            + DVec3::new(
                distance * phi.cos() * theta.cos(),
                distance * phi.sin(),
                distance * phi.cos() * theta.sin(),
            );
    }

    /// Unproject a screen coordinate to a world-space ray.
    ///
    /// `screen_x` and `screen_y` are pixel coordinates (origin top-left).
    /// Returns `(ray_origin, ray_direction)` in world space.
    pub fn unproject(
        &self,
        screen_x: f64,
        screen_y: f64,
        width: u32,
        height: u32,
    ) -> (DVec3, DVec3) {
        let aspect = width as f64 / height as f64;
        let view = self.view_matrix();
        let proj = self.projection_matrix(aspect);
        let vp = proj * view;
        let inv_vp = vp.inverse();

        // Convert pixel coords to NDC [-1, 1]
        let ndc_x = (2.0 * screen_x / width as f64) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / height as f64);

        // Unproject near and far points
        let near_ndc = glam::DVec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_ndc = glam::DVec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let near_world = inv_vp * near_ndc;
        let far_world = inv_vp * far_ndc;

        let near = DVec3::new(
            near_world.x / near_world.w,
            near_world.y / near_world.w,
            near_world.z / near_world.w,
        );
        let far = DVec3::new(
            far_world.x / far_world.w,
            far_world.y / far_world.w,
            far_world.z / far_world.w,
        );

        let direction = (far - near).normalize();
        (near, direction)
    }

    /// Dolly (move along view direction).
    pub fn dolly(&mut self, factor: f64) {
        let dir = self.direction();
        let distance = (self.focal_point - self.position).length();
        let delta = dir * distance * (1.0 - 1.0 / factor);
        self.position += delta;
    }

    /// Pan the camera (translate perpendicular to view direction).
    ///
    /// `dx` and `dy` are in screen-relative units scaled by distance.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let dir = self.direction();
        let distance = (self.focal_point - self.position).length();

        // Compute right and up vectors
        let right = dir.cross(self.view_up).normalize();
        let up = right.cross(dir).normalize();

        let offset = right * (-dx * distance * 0.002) + up * (dy * distance * 0.002);
        self.position += offset;
        self.focal_point += offset;
    }

    /// Position the camera to look at a target from a given position.
    pub fn look_at(&mut self, position: [f64; 3], target: [f64; 3]) {
        self.position = DVec3::from_array(position);
        self.focal_point = DVec3::from_array(target);
    }

    /// Set the camera to a standard view: front (+Z looking at origin).
    pub fn view_front(&mut self) {
        let dist = (self.focal_point - self.position).length();
        self.position = self.focal_point + DVec3::new(0.0, 0.0, dist);
        self.view_up = DVec3::Y;
    }

    /// Set the camera to a standard view: top (+Y looking down).
    pub fn view_top(&mut self) {
        let dist = (self.focal_point - self.position).length();
        self.position = self.focal_point + DVec3::new(0.0, dist, 0.0);
        self.view_up = DVec3::new(0.0, 0.0, -1.0);
    }

    /// Set the camera to a standard view: right (+X looking at origin).
    pub fn view_right(&mut self) {
        let dist = (self.focal_point - self.position).length();
        self.position = self.focal_point + DVec3::new(dist, 0.0, 0.0);
        self.view_up = DVec3::Y;
    }

    /// Set the camera to an isometric view.
    pub fn view_isometric(&mut self) {
        let dist = (self.focal_point - self.position).length();
        let d = dist / 3.0f64.sqrt();
        self.position = self.focal_point + DVec3::new(d, d, d);
        self.view_up = DVec3::Y;
    }

    /// Distance from camera to focal point.
    pub fn distance(&self) -> f64 {
        (self.focal_point - self.position).length()
    }

    /// Right vector (perpendicular to view direction and up).
    pub fn right(&self) -> DVec3 {
        self.direction().cross(self.view_up).normalize()
    }

    /// Actual up vector (perpendicular to view direction and right).
    pub fn up(&self) -> DVec3 {
        self.right().cross(self.direction()).normalize()
    }
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Camera: pos=[{:.1},{:.1},{:.1}], focal=[{:.1},{:.1},{:.1}], fov={:.0}°",
            self.position.x,
            self.position.y,
            self.position.z,
            self.focal_point.x,
            self.focal_point.y,
            self.focal_point.z,
            self.fov
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_camera() {
        let c = Camera::default();
        assert_eq!(c.position, DVec3::new(0.0, 0.0, 1.0));
        assert_eq!(c.focal_point, DVec3::ZERO);
        assert_eq!(c.fov, 30.0);
    }

    #[test]
    fn direction() {
        let c = Camera::default();
        let d = c.direction();
        assert!((d.z - (-1.0)).abs() < 1e-10); // looking along -Z
    }

    #[test]
    fn look_at() {
        let mut c = Camera::default();
        c.look_at([5.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(c.position.x, 5.0);
        assert_eq!(c.focal_point, DVec3::ZERO);
    }

    #[test]
    fn reset_to_bounds() {
        let mut c = Camera::default();
        c.reset_to_bounds([0.0, 0.0, 0.0], 10.0);
        assert!(c.distance() > 5.0); // should be far enough to see the box
        assert_eq!(c.focal_point, DVec3::ZERO);
    }

    #[test]
    fn orbit_preserves_distance() {
        let mut c = Camera::default();
        c.reset_to_bounds([0.0, 0.0, 0.0], 2.0);
        let d1 = c.distance();
        c.orbit(45.0, 30.0);
        let d2 = c.distance();
        assert!((d1 - d2).abs() < 1e-6, "orbit should preserve distance");
    }

    #[test]
    fn dolly() {
        let mut c = Camera::default();
        let d1 = c.distance();
        c.dolly(2.0);
        let d2 = c.distance();
        assert!(d2 < d1, "dolly in should decrease distance");
    }

    #[test]
    fn pan_preserves_distance() {
        let mut c = Camera::default();
        c.reset_to_bounds([0.0, 0.0, 0.0], 2.0);
        let d1 = c.distance();
        c.pan(10.0, 5.0);
        let d2 = c.distance();
        assert!((d1 - d2).abs() < 1e-6, "pan should preserve distance");
    }

    #[test]
    fn view_matrix_invertible() {
        let c = Camera::default();
        let v = c.view_matrix();
        let inv = v.inverse();
        let identity = v * inv;
        assert!((identity.x_axis.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn standard_views() {
        let mut c = Camera::default();
        c.reset_to_bounds([0.0, 0.0, 0.0], 2.0);

        c.view_front();
        assert!(c.position.z > 0.0);

        c.view_top();
        assert!(c.position.y > 0.0);

        c.view_right();
        assert!(c.position.x > 0.0);

        c.view_isometric();
        assert!(c.position.x > 0.0);
        assert!(c.position.y > 0.0);
        assert!(c.position.z > 0.0);
    }

    #[test]
    fn right_up_orthogonal() {
        let c = Camera::default();
        let r = c.right();
        let u = c.up();
        let d = c.direction();
        assert!(r.dot(u).abs() < 1e-10, "right and up should be orthogonal");
        assert!(
            r.dot(d).abs() < 1e-10,
            "right and direction should be orthogonal"
        );
    }

    #[test]
    fn unproject_center() {
        let c = Camera::default();
        let (_, dir) = c.unproject(400.0, 300.0, 800, 600);
        let cam_dir = c.direction();
        assert!(
            dir.dot(cam_dir) > 0.99,
            "center ray should align with camera"
        );
    }

    #[test]
    fn display() {
        let c = Camera::default();
        let s = format!("{c}");
        assert!(s.contains("Camera"));
        assert!(s.contains("fov=30"));
    }
}
