use crate::data::{CellArray, DataSet, Points, PolyData};

/// Generate a wireframe bounding box outline from any PolyData.
///
/// Creates 12 line segments representing the edges of the axis-aligned
/// bounding box of the input geometry. Useful for visualization overlays.
pub fn outline(input: &PolyData) -> PolyData {
    if input.points.len() == 0 {
        return PolyData::new();
    }

    let bb = input.bounds();
    let (xmin, xmax) = (bb.x_min, bb.x_max);
    let (ymin, ymax) = (bb.y_min, bb.y_max);
    let (zmin, zmax) = (bb.z_min, bb.z_max);

    let mut points = Points::<f64>::new();
    // 8 corners of the bounding box
    points.push([xmin, ymin, zmin]); // 0
    points.push([xmax, ymin, zmin]); // 1
    points.push([xmin, ymax, zmin]); // 2
    points.push([xmax, ymax, zmin]); // 3
    points.push([xmin, ymin, zmax]); // 4
    points.push([xmax, ymin, zmax]); // 5
    points.push([xmin, ymax, zmax]); // 6
    points.push([xmax, ymax, zmax]); // 7

    let mut lines = CellArray::new();
    let edges: [[i64; 2]; 12] = [
        [0, 1],
        [2, 3],
        [4, 5],
        [6, 7],
        [0, 2],
        [1, 3],
        [4, 6],
        [5, 7],
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7],
    ];
    for e in &edges {
        lines.push_cell(e);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_cube() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = outline(&pd);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.lines.num_cells(), 12);
    }

    #[test]
    fn outline_flat() {
        // All Z=0 -> flat bounding box
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = outline(&pd);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.lines.num_cells(), 12);
        // Z should be same for all points (flat)
        for i in 0..8 {
            assert!((result.points.get(i)[2]).abs() < 1e-10);
        }
    }
}
