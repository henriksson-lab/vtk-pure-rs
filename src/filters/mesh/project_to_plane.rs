use crate::data::{Points, PolyData};

/// Project all mesh points onto a plane defined by a point and normal.
///
/// For each point P, the projected point is:
///   P' = P - dot(P - plane_point, normal) * normal
///
/// The normal is normalized internally. Topology (cells) is preserved.
pub fn project_to_plane(input: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let len: f64 = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-20 {
        return input.clone();
    }

    let n: [f64; 3] = [normal[0] / len, normal[1] / len, normal[2] / len];

    let mut out_points = Points::<f64>::new();

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        let diff: [f64; 3] = [p[0] - point[0], p[1] - point[1], p[2] - point[2]];
        let dist: f64 = diff[0] * n[0] + diff[1] * n[1] + diff[2] * n[2];
        let projected: [f64; 3] = [p[0] - dist * n[0], p[1] - dist * n[1], p[2] - dist * n[2]];
        out_points.push(projected);
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{CellArray, Points};

    fn make_triangle(pts: [[f64; 3]; 3]) -> PolyData {
        let mut points = Points::<f64>::new();
        for p in &pts {
            points.push(*p);
        }
        let mut polys = CellArray::new();
        polys.push_cell(&[0, 1, 2]);

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd
    }

    #[test]
    fn test_project_onto_xy_plane() {
        let pd = make_triangle([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);

        let result = project_to_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);

        for i in 0..3 {
            let p = result.points.get(i);
            assert!(
                (p[2] - 0.0).abs() < 1e-10,
                "z should be 0 after projecting onto XY plane"
            );
        }
        // x and y should be preserved
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10);
        assert!((p0[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_project_onto_offset_plane() {
        let pd = make_triangle([[0.0, 0.0, 5.0], [1.0, 0.0, 5.0], [0.0, 1.0, 5.0]]);

        // Plane at z=3 with normal (0,0,1)
        let result = project_to_plane(&pd, [0.0, 0.0, 3.0], [0.0, 0.0, 1.0]);

        for i in 0..3 {
            let p = result.points.get(i);
            assert!((p[2] - 3.0).abs() < 1e-10, "z should be 3 after projection");
        }
    }

    #[test]
    fn test_points_already_on_plane_unchanged() {
        let pd = make_triangle([[1.0, 2.0, 0.0], [3.0, 4.0, 0.0], [5.0, 6.0, 0.0]]);

        let result = project_to_plane(&pd, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);

        for i in 0..3 {
            let orig = pd.points.get(i);
            let proj = result.points.get(i);
            for k in 0..3 {
                assert!(
                    (orig[k] - proj[k]).abs() < 1e-10,
                    "points on the plane should not move"
                );
            }
        }
    }
}
