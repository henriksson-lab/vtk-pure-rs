use crate::data::{Points, PolyData};

/// Snap all point positions to a regular grid with the given spacing.
///
/// Each coordinate is rounded to the nearest multiple of `spacing`.
/// Useful for regularizing point clouds or aligning geometry to a grid.
pub fn snap_to_grid(input: &PolyData, spacing: [f64; 3]) -> PolyData {
    let n = input.points.len();
    let sx = spacing[0].max(1e-15);
    let sy = spacing[1].max(1e-15);
    let sz = spacing[2].max(1e-15);

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        points.push([
            (p[0] / sx).round() * sx,
            (p[1] / sy).round() * sy,
            (p[2] / sz).round() * sz,
        ]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

/// Quantize point positions to N discrete levels per axis within bounds.
pub fn quantize_points(input: &PolyData, levels: usize) -> PolyData {
    use crate::data::DataSet;
    let n = input.points.len();
    if n == 0 || levels < 2 {
        return input.clone();
    }

    let bb = input.bounds();
    let dx = (bb.x_max - bb.x_min).max(1e-15);
    let dy = (bb.y_max - bb.y_min).max(1e-15);
    let dz = (bb.z_max - bb.z_min).max(1e-15);
    let l = (levels - 1) as f64;

    let mut points = Points::<f64>::new();
    for i in 0..n {
        let p = input.points.get(i);
        let qx = ((p[0] - bb.x_min) / dx * l).round() / l * dx + bb.x_min;
        let qy = ((p[1] - bb.y_min) / dy * l).round() / l * dy + bb.y_min;
        let qz = ((p[2] - bb.z_min) / dz * l).round() / l * dz + bb.z_min;
        points.push([qx, qy, qz]);
    }

    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.3, 0.7, 1.1]);

        let result = snap_to_grid(&pd, [0.5, 0.5, 0.5]);
        let p = result.points.get(0);
        assert_eq!(p[0], 0.5);
        assert_eq!(p[1], 0.5);
        assert_eq!(p[2], 1.0);
    }

    #[test]
    fn snap_preserves_topology() {
        let mut pd = PolyData::new();
        pd.points.push([0.1, 0.1, 0.0]);
        pd.points.push([0.9, 0.1, 0.0]);
        pd.points.push([0.5, 0.9, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = snap_to_grid(&pd, [1.0, 1.0, 1.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn quantize_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.33, 0.0, 0.0]);
        pd.points.push([0.66, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = quantize_points(&pd, 3); // 3 levels: 0, 0.5, 1.0
        let p1 = result.points.get(1);
        assert!((p1[0] - 0.5).abs() < 1e-10 || p1[0].abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = snap_to_grid(&pd, [1.0, 1.0, 1.0]);
        assert_eq!(result.points.len(), 0);
    }
}
