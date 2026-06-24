/// A 3D orientation cube widget that shows face labels (+X, -X, etc.)
/// for the three visible faces based on the current view direction.
///
/// Similar to the orientation cube found in many CAD applications.
#[derive(Debug, Clone)]
pub struct AxesCube {
    /// Labels for each face: [+X, -X, +Y, -Y, +Z, -Z].
    pub labels: [String; 6],
    /// Colors for each face: [+X, -X, +Y, -Y, +Z, -Z] as RGBA.
    pub colors: [[f32; 4]; 6],
    /// Size of the cube widget in screen pixels.
    pub size: f32,
}

impl Default for AxesCube {
    fn default() -> Self {
        Self::new()
    }
}

impl AxesCube {
    /// Create a new axes cube with default labels and colors.
    pub fn new() -> Self {
        Self {
            labels: [
                "+X".to_string(),
                "-X".to_string(),
                "+Y".to_string(),
                "-Y".to_string(),
                "+Z".to_string(),
                "-Z".to_string(),
            ],
            colors: [
                [0.8, 0.2, 0.2, 1.0], // +X red
                [0.5, 0.1, 0.1, 1.0], // -X dark red
                [0.2, 0.8, 0.2, 1.0], // +Y green
                [0.1, 0.5, 0.1, 1.0], // -Y dark green
                [0.2, 0.2, 0.8, 1.0], // +Z blue
                [0.1, 0.1, 0.5, 1.0], // -Z dark blue
            ],
            size: 80.0,
        }
    }

    /// Set the cube widget size.
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Compute the visible faces based on the view matrix and return their
    /// 2D screen positions, colors, and labels.
    ///
    /// The view matrix is a 4x4 column-major matrix (row-of-columns layout
    /// `[[f64;4];4]` where `view_matrix[row][col]`).
    ///
    /// Returns up to 3 face entries: `(screen_position, rgba_color, label)`.
    /// The faces are those whose normals point toward the camera.
    pub fn face_quads(&self, view_matrix: &[[f64; 4]; 4]) -> Vec<([f32; 2], [f32; 4], &str)> {
        // Face normals in world space: +X, -X, +Y, -Y, +Z, -Z
        let normals: [[f64; 3]; 6] = [
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];

        // Extract the camera forward direction from the view matrix.
        // In a standard view matrix, the third row gives the -forward direction.
        let forward = [view_matrix[2][0], view_matrix[2][1], view_matrix[2][2]];

        let half = (self.size / 2.0) as f64;
        let mut result = Vec::new();

        for (i, normal) in normals.iter().enumerate() {
            // Face is visible if its normal points toward the camera (dot with forward > 0)
            // The view matrix third row is the direction *into* the screen, so
            // a positive dot means the face normal aligns with the camera look-at.
            let dot = normal[0] * forward[0] + normal[1] * forward[1] + normal[2] * forward[2];
            if dot > 0.0 {
                // Project face center through the view matrix to get a 2D position.
                // Face center is at 0.5 * normal.
                let cx = 0.5 * normal[0];
                let cy = 0.5 * normal[1];
                let cz = 0.5 * normal[2];

                let sx = view_matrix[0][0] * cx + view_matrix[0][1] * cy + view_matrix[0][2] * cz;
                let sy = view_matrix[1][0] * cx + view_matrix[1][1] * cy + view_matrix[1][2] * cz;

                let screen_x = (sx * half) as f32;
                let screen_y = (sy * half) as f32;

                result.push((
                    [screen_x, screen_y],
                    self.colors[i],
                    self.labels[i].as_str(),
                ));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_axes_cube() {
        let cube = AxesCube::new();
        assert_eq!(cube.labels[0], "+X");
        assert_eq!(cube.labels[5], "-Z");
        assert_eq!(cube.size, 80.0);

        let cube2 = AxesCube::default();
        assert_eq!(cube2.size, 80.0);

        let cube3 = cube2.with_size(120.0);
        assert_eq!(cube3.size, 120.0);
    }

    #[test]
    fn face_visibility() {
        let cube = AxesCube::new();

        // Identity-like view matrix: camera looking along -Z,
        // so +Z face (normal [0,0,1]) should NOT be visible (dot with forward [0,0,-1] < 0),
        // and -Z face (normal [0,0,-1]) SHOULD be visible.
        // For a standard OpenGL-like view looking down -Z:
        // Row 0: [1, 0, 0, 0]  (right)
        // Row 1: [0, 1, 0, 0]  (up)
        // Row 2: [0, 0, 1, 0]  (forward = +Z, but this means looking along -Z in world)
        // Row 3: [0, 0, 0, 1]
        let view: [[f64; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let faces = cube.face_quads(&view);
        // With forward = [0,0,1], faces with normals that have positive dot:
        // +Z: dot = 1.0 (visible)
        // Others with zero or negative dots are not visible.
        // Only +Z should be visible.
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].2, "+Z");

        // Camera looking along +X direction (forward = [1,0,0])
        let view2: [[f64; 4]; 4] = [
            [0.0, 0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let faces2 = cube.face_quads(&view2);
        assert_eq!(faces2.len(), 1);
        assert_eq!(faces2[0].2, "+X");
    }
}
