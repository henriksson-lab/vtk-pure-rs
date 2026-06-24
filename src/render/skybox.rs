/// Skybox / environment background configuration.
///
/// Renders a gradient or solid color background behind the 3D scene.
/// More advanced skyboxes (cubemap textures) can be added later.
#[derive(Debug, Clone)]
pub enum Skybox {
    /// Solid color background (default behavior).
    Solid([f32; 4]),
    /// Vertical gradient from bottom color to top color.
    Gradient { bottom: [f32; 3], top: [f32; 3] },
    /// Three-stop gradient (bottom, horizon, top).
    ThreeStop {
        bottom: [f32; 3],
        horizon: [f32; 3],
        top: [f32; 3],
    },
}

impl Default for Skybox {
    fn default() -> Self {
        Skybox::Solid([0.1, 0.1, 0.1, 1.0])
    }
}

impl Skybox {
    /// Dark studio environment.
    pub fn studio() -> Self {
        Skybox::Gradient {
            bottom: [0.15, 0.15, 0.18],
            top: [0.05, 0.05, 0.08],
        }
    }

    /// Outdoor sky gradient.
    pub fn sky() -> Self {
        Skybox::ThreeStop {
            bottom: [0.4, 0.35, 0.3],
            horizon: [0.7, 0.75, 0.85],
            top: [0.3, 0.5, 0.9],
        }
    }

    /// Pure white background (for publication figures).
    pub fn white() -> Self {
        Skybox::Solid([1.0, 1.0, 1.0, 1.0])
    }

    /// Pure black background.
    pub fn black() -> Self {
        Skybox::Solid([0.0, 0.0, 0.0, 1.0])
    }

    /// Sample the skybox color at a normalized y coordinate [0=bottom, 1=top].
    pub fn sample(&self, y: f32) -> [f32; 3] {
        match self {
            Skybox::Solid(c) => [c[0], c[1], c[2]],
            Skybox::Gradient { bottom, top } => {
                let t = y.clamp(0.0, 1.0);
                [
                    bottom[0] + t * (top[0] - bottom[0]),
                    bottom[1] + t * (top[1] - bottom[1]),
                    bottom[2] + t * (top[2] - bottom[2]),
                ]
            }
            Skybox::ThreeStop {
                bottom,
                horizon,
                top,
            } => {
                let t = y.clamp(0.0, 1.0);
                if t < 0.5 {
                    let s = t * 2.0;
                    [
                        bottom[0] + s * (horizon[0] - bottom[0]),
                        bottom[1] + s * (horizon[1] - bottom[1]),
                        bottom[2] + s * (horizon[2] - bottom[2]),
                    ]
                } else {
                    let s = (t - 0.5) * 2.0;
                    [
                        horizon[0] + s * (top[0] - horizon[0]),
                        horizon[1] + s * (top[1] - horizon[1]),
                        horizon[2] + s * (top[2] - horizon[2]),
                    ]
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid() {
        let sky = Skybox::Solid([0.5, 0.5, 0.5, 1.0]);
        assert_eq!(sky.sample(0.0), [0.5, 0.5, 0.5]);
        assert_eq!(sky.sample(1.0), [0.5, 0.5, 0.5]);
    }

    #[test]
    fn gradient() {
        let sky = Skybox::Gradient {
            bottom: [0.0, 0.0, 0.0],
            top: [1.0, 1.0, 1.0],
        };
        let mid = sky.sample(0.5);
        assert!((mid[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn three_stop() {
        let sky = Skybox::ThreeStop {
            bottom: [0.0, 0.0, 0.0],
            horizon: [0.5, 0.5, 0.5],
            top: [1.0, 1.0, 1.0],
        };
        assert!((sky.sample(0.0)[0]).abs() < 1e-5);
        assert!((sky.sample(0.5)[0] - 0.5).abs() < 1e-5);
        assert!((sky.sample(1.0)[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn presets() {
        let _ = Skybox::studio();
        let _ = Skybox::sky();
        let _ = Skybox::white();
        let _ = Skybox::black();
    }
}
