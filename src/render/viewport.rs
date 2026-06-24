/// A viewport defines a rectangular region of the render target.
///
/// Coordinates are in normalized [0, 1] range where (0,0) is bottom-left.
/// Multiple viewports enable split-screen rendering.
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Left edge (0.0 to 1.0).
    pub x: f32,
    /// Bottom edge (0.0 to 1.0).
    pub y: f32,
    /// Width (0.0 to 1.0).
    pub width: f32,
    /// Height (0.0 to 1.0).
    pub height: f32,
}

impl Viewport {
    /// Full window viewport.
    pub fn full() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }

    /// Left half of the window.
    pub fn left_half() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 1.0,
        }
    }

    /// Right half of the window.
    pub fn right_half() -> Self {
        Self {
            x: 0.5,
            y: 0.0,
            width: 0.5,
            height: 1.0,
        }
    }

    /// Top half of the window.
    pub fn top_half() -> Self {
        Self {
            x: 0.0,
            y: 0.5,
            width: 1.0,
            height: 0.5,
        }
    }

    /// Bottom half of the window.
    pub fn bottom_half() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 0.5,
        }
    }

    /// Create a 2x2 grid of viewports.
    pub fn quad_grid() -> [Viewport; 4] {
        [
            Viewport {
                x: 0.0,
                y: 0.5,
                width: 0.5,
                height: 0.5,
            }, // top-left
            Viewport {
                x: 0.5,
                y: 0.5,
                width: 0.5,
                height: 0.5,
            }, // top-right
            Viewport {
                x: 0.0,
                y: 0.0,
                width: 0.5,
                height: 0.5,
            }, // bottom-left
            Viewport {
                x: 0.5,
                y: 0.0,
                width: 0.5,
                height: 0.5,
            }, // bottom-right
        ]
    }

    /// Custom viewport from normalized coordinates.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert to pixel coordinates given window size.
    pub fn to_pixels(&self, window_width: u32, window_height: u32) -> (u32, u32, u32, u32) {
        let px = (self.x * window_width as f32) as u32;
        let py = (self.y * window_height as f32) as u32;
        let pw = (self.width * window_width as f32) as u32;
        let ph = (self.height * window_height as f32) as u32;
        (px, py, pw, ph)
    }

    /// Aspect ratio of this viewport.
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// Check if a normalized screen point is inside this viewport.
    pub fn contains(&self, nx: f32, ny: f32) -> bool {
        nx >= self.x && nx <= self.x + self.width && ny >= self.y && ny <= self.y + self.height
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_viewport() {
        let vp = Viewport::full();
        assert_eq!(vp.to_pixels(800, 600), (0, 0, 800, 600));
        assert!(vp.contains(0.5, 0.5));
    }

    #[test]
    fn half_viewports() {
        let left = Viewport::left_half();
        let right = Viewport::right_half();
        assert!(left.contains(0.25, 0.5));
        assert!(!left.contains(0.75, 0.5));
        assert!(right.contains(0.75, 0.5));
    }

    #[test]
    fn quad_grid() {
        let vps = Viewport::quad_grid();
        assert_eq!(vps.len(), 4);
        // No overlap
        for i in 0..4 {
            let center_x = vps[i].x + vps[i].width / 2.0;
            let center_y = vps[i].y + vps[i].height / 2.0;
            for j in 0..4 {
                if i != j {
                    assert!(
                        !vps[j].contains(center_x, center_y),
                        "viewport {i} center should not be in viewport {j}"
                    );
                }
            }
        }
    }

    #[test]
    fn aspect_ratio() {
        let vp = Viewport::new(0.0, 0.0, 0.5, 1.0);
        assert!((vp.aspect_ratio() - 0.5).abs() < 1e-10);
    }
}
