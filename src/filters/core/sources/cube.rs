use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a cube (axis-aligned box).
pub struct CubeParams {
    pub center: [f64; 3],
    pub x_length: f64,
    pub y_length: f64,
    pub z_length: f64,
}

impl Default for CubeParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            x_length: 1.0,
            y_length: 1.0,
            z_length: 1.0,
        }
    }
}

/// Generate a cube as PolyData with 8 vertices, 6 quad faces, and normals.
pub fn cube(params: &CubeParams) -> PolyData {
    let [cx, cy, cz] = params.center;
    let hx = params.x_length * 0.5;
    let hy = params.y_length * 0.5;
    let hz = params.z_length * 0.5;

    // 24 vertices (4 per face, for correct normals)
    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    let faces: [([f64; 3], [[f64; 3]; 4]); 6] = [
        // +X face
        (
            [1.0, 0.0, 0.0],
            [
                [cx + hx, cy - hy, cz - hz],
                [cx + hx, cy + hy, cz - hz],
                [cx + hx, cy + hy, cz + hz],
                [cx + hx, cy - hy, cz + hz],
            ],
        ),
        // -X face
        (
            [-1.0, 0.0, 0.0],
            [
                [cx - hx, cy - hy, cz + hz],
                [cx - hx, cy + hy, cz + hz],
                [cx - hx, cy + hy, cz - hz],
                [cx - hx, cy - hy, cz - hz],
            ],
        ),
        // +Y face
        (
            [0.0, 1.0, 0.0],
            [
                [cx - hx, cy + hy, cz - hz],
                [cx - hx, cy + hy, cz + hz],
                [cx + hx, cy + hy, cz + hz],
                [cx + hx, cy + hy, cz - hz],
            ],
        ),
        // -Y face
        (
            [0.0, -1.0, 0.0],
            [
                [cx - hx, cy - hy, cz + hz],
                [cx - hx, cy - hy, cz - hz],
                [cx + hx, cy - hy, cz - hz],
                [cx + hx, cy - hy, cz + hz],
            ],
        ),
        // +Z face
        (
            [0.0, 0.0, 1.0],
            [
                [cx - hx, cy - hy, cz + hz],
                [cx + hx, cy - hy, cz + hz],
                [cx + hx, cy + hy, cz + hz],
                [cx - hx, cy + hy, cz + hz],
            ],
        ),
        // -Z face
        (
            [0.0, 0.0, -1.0],
            [
                [cx - hx, cy + hy, cz - hz],
                [cx + hx, cy + hy, cz - hz],
                [cx + hx, cy - hy, cz - hz],
                [cx - hx, cy - hy, cz - hz],
            ],
        ),
    ];

    for (normal, verts) in &faces {
        let base = points.len() as i64;
        for v in verts {
            points.push(*v);
            normals.push_tuple(normal);
        }
        polys.push_cell(&[base, base + 1, base + 2, base + 3]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.point_data_mut().add_array(normals.into());
    pd.point_data_mut().set_active_normals("Normals");
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cube() {
        let pd = cube(&CubeParams::default());
        assert_eq!(pd.points.len(), 24); // 4 vertices per face * 6 faces
        assert_eq!(pd.polys.num_cells(), 6);
    }
}
