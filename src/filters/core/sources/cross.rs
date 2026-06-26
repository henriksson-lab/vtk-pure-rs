use crate::data::{CellArray, DataArray, Points, PolyData};

/// Parameters for generating a 3D cross (plus sign) made of 3 orthogonal boxes.
pub struct CrossParams {
    pub center: [f64; 3],
    /// Length of each arm from center to tip.
    pub arm_length: f64,
    /// Width (and depth) of each arm.
    pub arm_width: f64,
}

impl Default for CrossParams {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0, 0.0],
            arm_length: 1.0,
            arm_width: 0.3,
        }
    }
}

/// Add a box defined by (x_min..x_max, y_min..y_max, z_min..z_max) to the given points/polys.
/// Each box contributes 24 points, 6 quad faces, and per-face normals.
fn add_box(
    points: &mut Points<f64>,
    normals: &mut DataArray<f64>,
    polys: &mut CellArray,
    bounds: [[f64; 2]; 3],
) {
    let [xr, yr, zr] = bounds;

    // 8 corners of the box:
    // 0: (x0,y0,z0)  1: (x1,y0,z0)  2: (x1,y1,z0)  3: (x0,y1,z0)
    // 4: (x0,y0,z1)  5: (x1,y0,z1)  6: (x1,y1,z1)  7: (x0,y1,z1)
    let corners = [
        [xr[0], yr[0], zr[0]],
        [xr[1], yr[0], zr[0]],
        [xr[1], yr[1], zr[0]],
        [xr[0], yr[1], zr[0]],
        [xr[0], yr[0], zr[1]],
        [xr[1], yr[0], zr[1]],
        [xr[1], yr[1], zr[1]],
        [xr[0], yr[1], zr[1]],
    ];

    // Face definitions: [vertex indices, outward normal]
    let faces: [([usize; 4], [f64; 3]); 6] = [
        ([0, 3, 2, 1], [0.0, 0.0, -1.0]), // -Z
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),  // +Z
        ([0, 1, 5, 4], [0.0, -1.0, 0.0]), // -Y
        ([2, 3, 7, 6], [0.0, 1.0, 0.0]),  // +Y
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]), // -X
        ([1, 2, 6, 5], [1.0, 0.0, 0.0]),  // +X
    ];

    // For proper per-face normals we duplicate vertices per face (24 verts per box)
    for (verts, n) in &faces {
        let face_base = points.len() as i64;
        for &vi in verts {
            points.push(corners[vi]);
            normals.push_tuple(n);
        }
        polys.push_cell(&[face_base, face_base + 1, face_base + 2, face_base + 3]);
    }
}

/// Generate a 3D cross/plus shape as PolyData.
///
/// The cross is formed by 3 orthogonal rectangular boxes sharing a common center,
/// one along each axis (X, Y, Z).
pub fn cross(params: &CrossParams) -> PolyData {
    let [cx, cy, cz] = params.center;
    let al = params.arm_length;
    let hw = params.arm_width / 2.0;

    let mut points = Points::new();
    let mut normals = DataArray::<f64>::new("Normals", 3);
    let mut polys = CellArray::new();

    // X-arm box: extends along X, narrow in Y and Z
    add_box(
        &mut points,
        &mut normals,
        &mut polys,
        [[cx - al, cx + al], [cy - hw, cy + hw], [cz - hw, cz + hw]],
    );

    // Y-arm box: extends along Y, narrow in X and Z
    add_box(
        &mut points,
        &mut normals,
        &mut polys,
        [[cx - hw, cx + hw], [cy - al, cy + al], [cz - hw, cz + hw]],
    );

    // Z-arm box: extends along Z, narrow in X and Y
    add_box(
        &mut points,
        &mut normals,
        &mut polys,
        [[cx - hw, cx + hw], [cy - hw, cy + hw], [cz - al, cz + al]],
    );

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
    fn default_cross() {
        let pd = cross(&CrossParams::default());
        // 3 boxes * 6 faces * 4 verts = 72 points
        assert_eq!(pd.points.len(), 72);
        // 3 boxes * 6 faces = 18 quads
        assert_eq!(pd.polys.num_cells(), 18);
        assert!(pd.point_data().normals().is_some());
    }

    #[test]
    fn custom_cross() {
        let pd = cross(&CrossParams {
            arm_length: 2.0,
            arm_width: 0.5,
            center: [1.0, 2.0, 3.0],
        });
        assert_eq!(pd.points.len(), 72);
        assert_eq!(pd.polys.num_cells(), 18);
    }
}
