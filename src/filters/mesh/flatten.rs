use crate::data::{DataSet, Points, PolyData};

/// Flatten a mesh onto the XY plane by discarding Z.
pub fn flatten_z(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([p[0], p[1], 0.0]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Project mesh onto a plane defined by normal through centroid.
pub fn project_to_plane(input: &PolyData, normal: [f64; 3]) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let nlen = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if nlen < 1e-15 {
        return input.clone();
    }
    let nn = [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen];

    // Compute centroid
    let bb = input.bounds();
    let c = bb.center();

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let d = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
        let dist = d[0] * nn[0] + d[1] * nn[1] + d[2] * nn[2];
        points.push([
            p[0] - dist * nn[0],
            p[1] - dist * nn[1],
            p[2] - dist * nn[2],
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Scale Z coordinates by a factor (useful for terrain exaggeration).
pub fn scale_z(input: &PolyData, factor: f64) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([p[0], p[1], p[2] * factor]);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        let result = flatten_z(&pd);
        assert_eq!(result.points.get(0)[2], 0.0);
        assert_eq!(result.points.get(1)[2], 0.0);
    }

    #[test]
    fn project() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 1.0]);
        pd.points.push([0.0, 0.0, -1.0]);
        let result = project_to_plane(&pd, [0.0, 0.0, 1.0]);
        // Both projected to z=0 plane through centroid
        let p0 = result.points.get(0);
        let p1 = result.points.get(1);
        assert!((p0[2] - p1[2]).abs() < 1e-10);
    }

    #[test]
    fn scale_z_test() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = scale_z(&pd, 2.0);
        assert_eq!(result.points.get(0)[2], 6.0);
    }

    #[test]
    fn scale_z_zero() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = scale_z(&pd, 0.0);
        assert_eq!(result.points.get(0)[2], 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let _ = flatten_z(&pd);
        let _ = project_to_plane(&pd, [0.0, 0.0, 1.0]);
    }
}
