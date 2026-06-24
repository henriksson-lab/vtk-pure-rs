use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Project a mesh onto one of the coordinate planes (drop one axis).
///
/// `drop_axis`: 0=drop X (project to YZ), 1=drop Y (project to XZ), 2=drop Z (project to XY).
pub fn project_to_axis_plane(input: &PolyData, drop_axis: usize) -> PolyData {
    let n = input.points.len();
    let axis = drop_axis.min(2);
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let mut out = p;
        out[axis] = 0.0;
        points.push(out);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Project mesh onto a sphere of given radius centered at origin.
pub fn project_to_sphere(input: &PolyData, radius: f64) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        if len > 1e-15 {
            points.push([
                p[0] / len * radius,
                p[1] / len * radius,
                p[2] / len * radius,
            ]);
        } else {
            points.push([0.0, 0.0, radius]);
        }
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Project mesh onto a cylinder of given radius around the Y axis.
pub fn project_to_cylinder(input: &PolyData, radius: f64) -> PolyData {
    let n = input.points.len();
    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let rho = (p[0] * p[0] + p[2] * p[2]).sqrt();
        if rho > 1e-15 {
            points.push([p[0] / rho * radius, p[1], p[2] / rho * radius]);
        } else {
            points.push([radius, p[1], 0.0]);
        }
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_to_xy() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = project_to_axis_plane(&pd, 2);
        assert_eq!(result.points.get(0)[2], 0.0);
        assert_eq!(result.points.get(0)[0], 1.0);
    }

    #[test]
    fn project_sphere() {
        let mut pd = PolyData::new();
        pd.points.push([3.0, 4.0, 0.0]); // distance 5
        let result = project_to_sphere(&pd, 1.0);
        let p = result.points.get(0);
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn project_cylinder() {
        let mut pd = PolyData::new();
        pd.points.push([3.0, 5.0, 4.0]); // rho=5 in XZ
        let result = project_to_cylinder(&pd, 2.0);
        let p = result.points.get(0);
        let rho = (p[0] * p[0] + p[2] * p[2]).sqrt();
        assert!((rho - 2.0).abs() < 1e-10);
        assert_eq!(p[1], 5.0); // Y preserved
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(project_to_sphere(&pd, 1.0).points.len(), 0);
    }
}
