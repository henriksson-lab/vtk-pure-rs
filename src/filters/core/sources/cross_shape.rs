//! Cross/plus-sign shaped geometry.

use crate::data::{CellArray, Points, PolyData};

/// Create a 3D cross (plus sign) shape.
pub fn cross_shape(arm_length: f64, arm_width: f64, thickness: f64) -> PolyData {
    let al = arm_length;
    let aw = arm_width / 2.0;
    let h = thickness / 2.0;
    // 12 vertices for the cross profile (top face), duplicated for bottom
    let profile = [
        [-aw, -al],
        [aw, -al],
        [aw, -aw],
        [al, -aw],
        [al, aw],
        [aw, aw],
        [aw, al],
        [-aw, al],
        [-aw, aw],
        [-al, aw],
        [-al, -aw],
        [-aw, -aw],
    ];
    let np = profile.len();
    let mut pts = Points::<f64>::new();
    for p in &profile {
        pts.push([p[0], p[1], -h]);
    }
    for p in &profile {
        pts.push([p[0], p[1], h]);
    }

    let mut polys = CellArray::new();
    // Bottom face (fan)
    for i in 1..np - 1 {
        polys.push_cell(&[0, (i + 1) as i64, i as i64]);
    }
    // Top face
    for i in 1..np - 1 {
        polys.push_cell(&[np as i64, (np + i) as i64, (np + i + 1) as i64]);
    }
    // Side faces
    for i in 0..np {
        let j = (i + 1) % np;
        polys.push_cell(&[i as i64, j as i64, (np + j) as i64, (np + i) as i64]);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cross() {
        let c = cross_shape(2.0, 0.5, 0.3);
        assert_eq!(c.points.len(), 24);
        assert!(c.polys.num_cells() > 10);
    }
}
