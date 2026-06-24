use crate::render::{ColorMap, TransferFunction};

/// An editable transfer function with interactive control points.
///
/// Allows adding, removing, and moving control points for both
/// color and opacity mapping. Can be converted to a `TransferFunction`.
#[derive(Debug, Clone)]
pub struct TransferFunctionEditor {
    /// Color control points: (position, [r, g, b]).
    color_points: Vec<(f64, [f32; 3])>,
    /// Opacity control points: (position, opacity).
    opacity_points: Vec<(f64, f64)>,
}

impl TransferFunctionEditor {
    /// Create from a color map with linear opacity.
    pub fn from_color_map(cm: &ColorMap) -> Self {
        // Sample the color map at regular intervals
        let n = 5;
        let color_points: Vec<(f64, [f32; 3])> = (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                (t, cm.map(t))
            })
            .collect();

        Self {
            color_points,
            opacity_points: vec![(0.0, 0.0), (1.0, 1.0)],
        }
    }

    /// Create with custom color and opacity points.
    pub fn new(color_points: Vec<(f64, [f32; 3])>, opacity_points: Vec<(f64, f64)>) -> Self {
        let mut editor = Self {
            color_points,
            opacity_points,
        };
        editor.sort();
        editor
    }

    /// Add a color control point.
    pub fn add_color_point(&mut self, position: f64, color: [f32; 3]) {
        self.color_points.push((position, color));
        self.color_points
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    /// Add an opacity control point.
    pub fn add_opacity_point(&mut self, position: f64, opacity: f64) {
        self.opacity_points.push((position, opacity));
        self.opacity_points
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    /// Remove a color control point by index.
    pub fn remove_color_point(&mut self, index: usize) {
        if index < self.color_points.len() {
            self.color_points.remove(index);
        }
    }

    /// Remove an opacity control point by index.
    pub fn remove_opacity_point(&mut self, index: usize) {
        if index < self.opacity_points.len() {
            self.opacity_points.remove(index);
        }
    }

    /// Move a color control point to a new position.
    pub fn move_color_point(&mut self, index: usize, new_position: f64) {
        if index < self.color_points.len() {
            self.color_points[index].0 = new_position.clamp(0.0, 1.0);
            self.color_points
                .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        }
    }

    /// Number of color control points.
    pub fn num_color_points(&self) -> usize {
        self.color_points.len()
    }

    /// Number of opacity control points.
    pub fn num_opacity_points(&self) -> usize {
        self.opacity_points.len()
    }

    /// Convert to a TransferFunction for rendering.
    pub fn to_transfer_function(&self) -> TransferFunction {
        let cm = ColorMap::new(self.color_points.clone());
        let mut tf = TransferFunction::linear(cm);
        tf.set_opacity_points(self.opacity_points.clone());
        tf
    }

    fn sort(&mut self) {
        self.color_points
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.opacity_points
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_color_map() {
        let editor = TransferFunctionEditor::from_color_map(&ColorMap::jet());
        assert!(editor.num_color_points() >= 2);
        assert_eq!(editor.num_opacity_points(), 2);
    }

    #[test]
    fn add_remove_points() {
        let mut editor = TransferFunctionEditor::new(
            vec![(0.0, [0.0; 3]), (1.0, [1.0; 3])],
            vec![(0.0, 0.0), (1.0, 1.0)],
        );
        editor.add_color_point(0.5, [0.5; 3]);
        assert_eq!(editor.num_color_points(), 3);
        editor.remove_color_point(1);
        assert_eq!(editor.num_color_points(), 2);
    }

    #[test]
    fn to_transfer_function() {
        let editor = TransferFunctionEditor::from_color_map(&ColorMap::viridis());
        let tf = editor.to_transfer_function();
        let rgba = tf.sample(0.5);
        assert!(rgba[0] >= 0.0 && rgba[0] <= 1.0);
    }
}
