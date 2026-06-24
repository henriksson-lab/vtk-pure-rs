use crate::render::ColorMap;

/// Orientation of the scalar bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarBarOrientation {
    Vertical,
    Horizontal,
}

/// A scalar bar (color legend) widget configuration.
///
/// Describes how to render a color legend that maps scalar values to colors.
/// Position and size are in normalized device coordinates [0, 1] where
/// (0, 0) is bottom-left and (1, 1) is top-right.
#[derive(Debug, Clone)]
pub struct ScalarBar {
    /// Title displayed above the scalar bar.
    pub title: String,
    /// Color map to display.
    pub color_map: ColorMap,
    /// Scalar range [min, max].
    pub range: [f64; 2],
    /// Number of labeled tick marks.
    pub num_labels: usize,
    /// Orientation.
    pub orientation: ScalarBarOrientation,
    /// Position of bottom-left corner in NDC [0, 1].
    pub position: [f32; 2],
    /// Size [width, height] in NDC [0, 1].
    pub size: [f32; 2],
    /// Number of color bands to subdivide the bar into.
    pub num_bands: usize,
    /// Text color for labels. Default: white.
    pub text_color: [f32; 3],
    /// Background color (with alpha). Default: semi-transparent black.
    pub background_color: [f32; 4],
}

impl ScalarBar {
    /// Create a scalar bar with default placement (right side, vertical).
    pub fn new(title: &str, color_map: ColorMap, range: [f64; 2]) -> Self {
        Self {
            title: title.to_string(),
            color_map,
            range,
            num_labels: 5,
            orientation: ScalarBarOrientation::Vertical,
            position: [0.85, 0.1],
            size: [0.08, 0.8],
            num_bands: 256,
            text_color: [1.0, 1.0, 1.0],
            background_color: [0.0, 0.0, 0.0, 0.5],
        }
    }

    /// Generate the geometry for the color bar as a list of colored quads.
    ///
    /// Returns a list of `(x, y, w, h, [r, g, b, a])` in NDC [0,1] space.
    pub fn color_band_quads(&self) -> Vec<([f32; 2], [f32; 2], [f32; 4])> {
        let mut quads = Vec::with_capacity(self.num_bands);
        let n = self.num_bands;

        for i in 0..n {
            let t = i as f64 / n as f64;
            let t_next = (i + 1) as f64 / n as f64;
            let t_mid = (t + t_next) / 2.0;
            let color = self.color_map.map(t_mid);

            let (pos, size) = match self.orientation {
                ScalarBarOrientation::Vertical => {
                    let y = self.position[1] + self.size[1] * t as f32;
                    let h = self.size[1] / n as f32;
                    ([self.position[0], y], [self.size[0], h])
                }
                ScalarBarOrientation::Horizontal => {
                    let x = self.position[0] + self.size[0] * t as f32;
                    let w = self.size[0] / n as f32;
                    ([x, self.position[1]], [w, self.size[1]])
                }
            };

            quads.push((pos, size, [color[0], color[1], color[2], 1.0]));
        }

        quads
    }

    /// Generate label positions and text for the scalar bar.
    ///
    /// Returns `(ndc_x, ndc_y, label_text)` for each tick mark.
    pub fn label_info(&self) -> Vec<([f32; 2], String)> {
        let mut labels = Vec::with_capacity(self.num_labels);
        let n = self.num_labels;
        if n < 2 {
            return labels;
        }

        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let value = self.range[0] + (self.range[1] - self.range[0]) * t as f64;

            let pos = match self.orientation {
                ScalarBarOrientation::Vertical => {
                    let y = self.position[1] + self.size[1] * t;
                    [self.position[0] + self.size[0] + 0.005, y]
                }
                ScalarBarOrientation::Horizontal => {
                    let x = self.position[0] + self.size[0] * t;
                    [x, self.position[1] - 0.03]
                }
            };

            let text = format_label(value);
            labels.push((pos, text));
        }

        labels
    }

    /// Generate vertex positions, RGBA colors, and triangle indices for
    /// rendering a smooth gradient color bar as quads.
    ///
    /// Each band interpolates between two adjacent color map values,
    /// producing a smooth gradient. The bar is rendered as a vertical strip
    /// from `(x, y)` with the given `width` and `height`.
    ///
    /// Returns `(positions, colors, indices)` where positions are `[f32; 2]`,
    /// colors are `[f32; 4]` RGBA, and indices form triangles.
    pub fn to_gradient_quads(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        num_bands: usize,
    ) -> (Vec<[f32; 2]>, Vec<[f32; 4]>, Vec<u32>) {
        let num_bands = num_bands.max(1);
        let num_verts = (num_bands + 1) * 2; // 2 vertices per horizontal row
        let mut positions = Vec::with_capacity(num_verts);
        let mut colors = Vec::with_capacity(num_verts);
        let mut indices = Vec::with_capacity(num_bands * 6);

        // Generate vertices: for each row (0..=num_bands), left and right vertices
        for i in 0..=num_bands {
            let t = i as f64 / num_bands as f64;
            let row_y = y + height * t as f32;
            let rgb = self.color_map.map(t);

            // Left vertex
            positions.push([x, row_y]);
            colors.push([rgb[0], rgb[1], rgb[2], 1.0]);

            // Right vertex
            positions.push([x + width, row_y]);
            colors.push([rgb[0], rgb[1], rgb[2], 1.0]);
        }

        // Generate indices: two triangles per band
        for i in 0..num_bands {
            let base = (i * 2) as u32;
            // Triangle 1: bottom-left, bottom-right, top-left
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            // Triangle 2: bottom-right, top-right, top-left
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }

        (positions, colors, indices)
    }

    /// Generate the title position in NDC.
    pub fn title_position(&self) -> [f32; 2] {
        match self.orientation {
            ScalarBarOrientation::Vertical => {
                [self.position[0], self.position[1] + self.size[1] + 0.02]
            }
            ScalarBarOrientation::Horizontal => [
                self.position[0] + self.size[0] / 2.0,
                self.position[1] + self.size[1] + 0.02,
            ],
        }
    }
}

fn format_label(value: f64) -> String {
    let abs = value.abs();
    if abs == 0.0 {
        "0".to_string()
    } else if abs >= 1000.0 || abs < 0.01 {
        format!("{:.2e}", value)
    } else if abs >= 1.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.3}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_bar_default() {
        let sb = ScalarBar::new("Temperature", ColorMap::jet(), [0.0, 100.0]);
        assert_eq!(sb.num_labels, 5);
        assert_eq!(sb.title, "Temperature");
        assert_eq!(sb.range, [0.0, 100.0]);
    }

    #[test]
    fn color_band_quads() {
        let sb = ScalarBar::new("T", ColorMap::jet(), [0.0, 1.0]);
        let quads = sb.color_band_quads();
        assert_eq!(quads.len(), 256);
        // First quad should be at bottom
        assert!(quads[0].0[1] < quads[255].0[1]);
    }

    #[test]
    fn label_info_count() {
        let sb = ScalarBar::new("T", ColorMap::jet(), [0.0, 100.0]);
        let labels = sb.label_info();
        assert_eq!(labels.len(), 5);
        assert_eq!(labels[0].1, "0");
        assert_eq!(labels[4].1, "100.0");
    }

    #[test]
    fn label_format_scientific() {
        assert_eq!(format_label(0.001), "1.00e-3");
        assert_eq!(format_label(50000.0), "5.00e4");
    }

    #[test]
    fn label_format_normal() {
        assert_eq!(format_label(25.0), "25.0");
        assert_eq!(format_label(0.5), "0.500");
    }

    #[test]
    fn gradient_quads() {
        let sb = ScalarBar::new("T", ColorMap::jet(), [0.0, 1.0]);
        let (positions, colors, indices) = sb.to_gradient_quads(0.0, 0.0, 0.1, 1.0, 4);

        // 4 bands => 5 rows * 2 verts = 10 vertices
        assert_eq!(positions.len(), 10);
        assert_eq!(colors.len(), 10);
        // 4 bands * 6 indices = 24
        assert_eq!(indices.len(), 24);

        // Bottom-left vertex at (0, 0)
        assert!((positions[0][0]).abs() < 1e-6);
        assert!((positions[0][1]).abs() < 1e-6);

        // Top-left vertex at (0, 1.0)
        assert!((positions[8][1] - 1.0).abs() < 1e-6);

        // All colors should have alpha=1
        for c in &colors {
            assert!((c[3] - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn horizontal_orientation() {
        let mut sb = ScalarBar::new("T", ColorMap::jet(), [0.0, 1.0]);
        sb.orientation = ScalarBarOrientation::Horizontal;
        sb.num_bands = 10;
        let quads = sb.color_band_quads();
        assert_eq!(quads.len(), 10);
        // Horizontal: x increases across quads
        assert!(quads[0].0[0] < quads[9].0[0]);
    }
}
