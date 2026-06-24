/// Shadow mapping configuration.
///
/// Controls shadow rendering for directional lights. Shadows are computed
/// by rendering a depth map from the light's perspective, then sampling
/// it during the main render pass to determine shadow coverage.
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    /// Shadow map resolution (width = height). Default: 1024
    pub resolution: u32,
    /// Shadow bias to prevent shadow acne. Default: 0.005
    pub bias: f64,
    /// Shadow softness (0 = hard, higher = softer). Default: 1.0
    pub softness: f64,
    /// Shadow darkness (0 = no shadow, 1 = full black). Default: 0.5
    pub darkness: f64,
    /// Whether shadows are enabled.
    pub enabled: bool,
    /// Orthographic projection size for the shadow camera. Default: 10.0
    pub ortho_size: f64,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 1024,
            bias: 0.005,
            softness: 1.0,
            darkness: 0.5,
            enabled: false,
            ortho_size: 10.0,
        }
    }
}

impl ShadowConfig {
    /// Create enabled shadow config with default settings.
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Set shadow map resolution.
    pub fn with_resolution(mut self, res: u32) -> Self {
        self.resolution = res;
        self
    }

    /// Set shadow softness.
    pub fn with_softness(mut self, softness: f64) -> Self {
        self.softness = softness;
        self
    }

    /// Compute the light's view-projection matrix for a directional light.
    pub fn light_vp_matrix(&self, light_dir: [f64; 3], center: [f64; 3]) -> [[f64; 4]; 4] {
        let s = self.ortho_size;
        let ld = normalize3(light_dir);
        let light_pos = [
            center[0] - ld[0] * s * 2.0,
            center[1] - ld[1] * s * 2.0,
            center[2] - ld[2] * s * 2.0,
        ];

        // Simple orthographic view-projection
        let view = look_at(light_pos, center, [0.0, 1.0, 0.0]);
        let proj = ortho(-s, s, -s, s, 0.1, s * 4.0);
        mat4_mul(proj, view)
    }
}

fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        return [0.0, -1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn look_at(eye: [f64; 3], target: [f64; 3], up: [f64; 3]) -> [[f64; 4]; 4] {
    let f = normalize3([target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]]);
    let s = normalize3(cross3(f, up));
    let u = cross3(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot3(s, eye), -dot3(u, eye), dot3(f, eye), 1.0],
    ]
}

fn ortho(l: f64, r: f64, b: f64, t: f64, n: f64, f: f64) -> [[f64; 4]; 4] {
    [
        [2.0 / (r - l), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (t - b), 0.0, 0.0],
        [0.0, 0.0, -2.0 / (f - n), 0.0],
        [
            -(r + l) / (r - l),
            -(t + b) / (t - b),
            -(f + n) / (f - n),
            1.0,
        ],
    ]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn mat4_mul(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut r = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                r[i][j] += a[k][j] * b[i][k];
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let sc = ShadowConfig::default();
        assert!(!sc.enabled);
        assert_eq!(sc.resolution, 1024);
    }

    #[test]
    fn enabled_config() {
        let sc = ShadowConfig::new().with_resolution(2048).with_softness(2.0);
        assert!(sc.enabled);
        assert_eq!(sc.resolution, 2048);
        assert_eq!(sc.softness, 2.0);
    }

    #[test]
    fn light_vp_matrix() {
        let sc = ShadowConfig::new();
        let vp = sc.light_vp_matrix([0.0, -1.0, 0.0], [0.0, 0.0, 0.0]);
        // Should produce a valid 4x4 matrix
        assert!(vp[3][3].abs() > 0.0);
    }

    #[test]
    fn normalize() {
        let n = normalize3([3.0, 4.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-10);
    }
}
