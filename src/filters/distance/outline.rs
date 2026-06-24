use crate::data::{CellArray, DataSet, Points, PolyData};

/// Generate a wireframe bounding box outline from any PolyData.
///
/// Creates 12 line segments representing the edges of the axis-aligned
/// bounding box of the input geometry. Useful for visualization overlays.
pub fn outline(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    let (xmin, xmax) = (bb.x_min, bb.x_max);
    let (ymin, ymax) = (bb.y_min, bb.y_max);
    let (zmin, zmax) = (bb.z_min, bb.z_max);

    let mut points = Points::<f64>::new();
    // 8 corners of the bounding box
    points.push([xmin, ymin, zmin]); // 0
    points.push([xmax, ymin, zmin]); // 1
    points.push([xmax, ymax, zmin]); // 2
    points.push([xmin, ymax, zmin]); // 3
    points.push([xmin, ymin, zmax]); // 4
    points.push([xmax, ymin, zmax]); // 5
    points.push([xmax, ymax, zmax]); // 6
    points.push([xmin, ymax, zmax]); // 7

    let mut lines = CellArray::new();
    // 12 edges
    let edges: [[i64; 2]; 12] = [
        [0, 1],
        [1, 2],
        [2, 3],
        [3, 0], // bottom face
        [4, 5],
        [5, 6],
        [6, 7],
        [7, 4], // top face
        [0, 4],
        [1, 5],
        [2, 6],
        [3, 7], // vertical edges
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
