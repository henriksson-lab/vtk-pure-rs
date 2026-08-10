use crate::data::{DataSet, Points, PolyData};

/// Twist a mesh around an axis by a given angle per unit length.
///
/// Points are rotated around the specified axis proportionally to their
/// position along that axis. The rate is in radians per unit.
///
/// Single implementation lives in [`crate::filters::mesh::mesh_twist_deform`];
/// it rotates about the world origin using the standard right-handed rotation
/// matrix for each axis.
pub use crate::filters::mesh::mesh_twist_deform::twist;

/// Bend a mesh around an axis. Points farther from the axis are
/// displaced along a circular arc. `curvature` controls bend amount.
pub fn bend(input: &PolyData, axis: usize, curvature: f64) -> PolyData {
    let n = input.points.len();
    if n == 0 || curvature.abs() < 1e-15 {
        return input.clone();
    }

    let bb = input.bounds();
    let center = bb.center();

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let h = p[axis.min(2)] - center[axis.min(2)];
        let angle = h * curvature;
        let r = 1.0 / curvature.abs();

        let mut out = p;
        match axis.min(2) {
            0 => {
                out[1] += r * (1.0 - angle.cos());
                out[0] = center[0] + r * angle.sin();
            }
            1 => {
                out[2] += r * (1.0 - angle.cos());
                out[1] = center[1] + r * angle.sin();
            }
            _ => {
                out[0] += r * (1.0 - angle.cos());
                out[2] = center[2] + r * angle.sin();
            }
        }
        points.push(out);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twist_z() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 1.0]);
        pd.points.push([1.0, 0.0, -1.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = twist(&pd, 2, std::f64::consts::PI);
        assert_eq!(result.points.len(), 3);
        // Point at z=0 shouldn't rotate
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn zero_twist_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = twist(&pd, 2, 0.0);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn bend_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);

        let result = bend(&pd, 1, 0.5);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let _ = twist(&pd, 0, 1.0);
        let _ = bend(&pd, 0, 1.0);
    }
}
