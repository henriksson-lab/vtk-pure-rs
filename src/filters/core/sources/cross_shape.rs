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
    // Cap the concave cross as five convex quads: four arms plus the center.
    let cap_quads = [
        [0, 1, 2, 11],
        [2, 3, 4, 5],
        [5, 6, 7, 8],
        [8, 9, 10, 11],
        [11, 2, 5, 8],
    ];
    for q in &cap_quads {
        polys.push_cell(&[q[0] as i64, q[3] as i64, q[2] as i64, q[1] as i64]);
    }
    for q in &cap_quads {
        polys.push_cell(&[
            (np + q[0]) as i64,
            (np + q[1]) as i64,
            (np + q[2]) as i64,
            (np + q[3]) as i64,
        ]);
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
