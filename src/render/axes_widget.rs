/// Configuration for an orientation axes widget.
///
/// Renders XYZ axis arrows in a corner of the viewport.
/// The axes rotate with the camera to show the current viewing orientation.
/// Position and size are in normalized device coordinates [0, 1].
#[derive(Debug, Clone)]
pub struct AxesWidget {
    /// Position of the widget center in NDC [0, 1] (bottom-left origin).
    pub position: [f32; 2],
    /// Size (radius) in NDC units.
    pub size: f32,
    /// Whether to show axis labels (X, Y, Z).
    pub show_labels: bool,
    /// Whether the widget is enabled.
    pub enabled: bool,
}

impl Default for AxesWidget {
    fn default() -> Self {
        Self {
            position: [0.1, 0.1],
            size: 0.07,
            show_labels: true,
            enabled: true,
        }
    }
}

impl AxesWidget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute the 2D projected axis endpoints for the given view matrix.
    ///
    /// Returns `[(tip_x, tip_y, [r,g,b], label)]` for X, Y, Z axes.
    /// Coordinates are in NDC [0, 1].
    pub fn projected_axes(&self, view_matrix: &[[f64; 4]; 4]) -> Vec<([f32; 2], [f32; 3], char)> {
        let axes = [
            ([1.0, 0.0, 0.0f64], [1.0f32, 0.2, 0.2], 'X'),
            ([0.0, 1.0, 0.0f64], [0.2, 1.0, 0.2], 'Y'),
            ([0.0, 0.0, 1.0f64], [0.3, 0.3, 1.0], 'Z'),
        ];

        let mut result = Vec::with_capacity(3);
        for (dir, color, label) in &axes {
            // Transform direction by the rotational part of the view matrix (upper 3x3)
            let tx = view_matrix[0][0] * dir[0]
                + view_matrix[1][0] * dir[1]
                + view_matrix[2][0] * dir[2];
            let ty = view_matrix[0][1] * dir[0]
                + view_matrix[1][1] * dir[1]
                + view_matrix[2][1] * dir[2];
            // Project to 2D: x goes right, y goes up
            let tip_x = self.position[0] + tx as f32 * self.size;
            let tip_y = self.position[1] + ty as f32 * self.size;
            result.push(([tip_x, tip_y], *color, *label));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_axes_widget() {
        let w = AxesWidget::default();
        assert!(w.enabled);
        assert!(w.show_labels);
    }

    #[test]
    fn projected_axes_identity() {
        let w = AxesWidget::default();
        // Identity view matrix
        let view = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let axes = w.projected_axes(&view);
        assert_eq!(axes.len(), 3);
        // X axis tip should be to the right of center
        assert!(axes[0].0[0] > w.position[0]);
        // Y axis tip should be above center
        assert!(axes[1].0[1] > w.position[1]);
    }
}
