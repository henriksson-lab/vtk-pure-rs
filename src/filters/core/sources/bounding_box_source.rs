use crate::data::{CellArray, Points, PolyData};

/// Generate a wireframe bounding box as line cells.
///
/// Useful for visualizing spatial extents. The box is axis-aligned.
pub fn bounding_box_source(bounds: [f64; 6]) -> PolyData {
    let mut bounds = bounds;
    for i in (0..6).step_by(2) {
        if bounds[i] > bounds[i + 1] {
            bounds.swap(i, i + 1);
        }
    }
    let [x0, x1, y0, y1, z0, z1] = bounds;

    let mut points = Points::new();
    points.push([x0, y0, z0]); // 0
    points.push([x1, y0, z0]); // 1
    points.push([x0, y1, z0]); // 2
    points.push([x1, y1, z0]); // 3
    points.push([x0, y0, z1]); // 4
    points.push([x1, y0, z1]); // 5
    points.push([x0, y1, z1]); // 6
    points.push([x1, y1, z1]); // 7

    let mut lines = CellArray::new();
    lines.push_cell(&[0, 1]);
    lines.push_cell(&[2, 3]);
    lines.push_cell(&[4, 5]);
    lines.push_cell(&[6, 7]);
    lines.push_cell(&[0, 2]);
    lines.push_cell(&[1, 3]);
    lines.push_cell(&[4, 6]);
    lines.push_cell(&[5, 7]);
    lines.push_cell(&[0, 4]);
    lines.push_cell(&[1, 5]);
    lines.push_cell(&[2, 6]);
    lines.push_cell(&[3, 7]);

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd
}

/// Generate a bounding box source from a PolyData's bounds.
pub fn bounding_box_from_data(input: &PolyData) -> PolyData {
    let bb = input.points.bounds();
    bounding_box_source([bb.x_min, bb.x_max, bb.y_min, bb.y_max, bb.z_min, bb.z_max])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_box() {
        let pd = bounding_box_source([0.0, 1.0, 0.0, 1.0, 0.0, 1.0]);
        assert_eq!(pd.points.len(), 8);
        assert_eq!(pd.lines.num_cells(), 12);
    }

    #[test]
    fn from_data() {
        let input = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [0.0, 3.0, 2.0]],
            vec![[0, 1, 2]],
        );
        let result = bounding_box_from_data(&input);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.lines.num_cells(), 12);
        // Check extent
        let p5 = result.points.get(5); // should be at (x_max, y_min, z_max)
        assert!((p5[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn matches_vtk_outline_order_and_normalizes_bounds() {
        let pd = bounding_box_source([1.0, -1.0, 3.0, 2.0, 5.0, 4.0]);
        assert_eq!(pd.points.get(0), [-1.0, 2.0, 4.0]);
        assert_eq!(pd.points.get(3), [1.0, 3.0, 4.0]);
        assert_eq!(pd.points.get(7), [1.0, 3.0, 5.0]);
        assert_eq!(pd.lines.cell(0), &[0, 1]);
        assert_eq!(pd.lines.cell(1), &[2, 3]);
        assert_eq!(pd.lines.cell(11), &[3, 7]);
    }
}
