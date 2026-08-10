use crate::data::{CellArray, DataSet, Points, PolyData};

/// Compute the axis-aligned bounding box of a PolyData as a closed quad mesh.
///
/// Returns a PolyData with 8 vertices and 6 quad faces representing
/// the tight bounding box of the input geometry.
pub fn bounding_box_mesh(input: &PolyData) -> PolyData {
    let bb = input.bounds();
    let corners = [
        [bb.x_min, bb.y_min, bb.z_min],
        [bb.x_max, bb.y_min, bb.z_min],
        [bb.x_max, bb.y_max, bb.z_min],
        [bb.x_min, bb.y_max, bb.z_min],
        [bb.x_min, bb.y_min, bb.z_max],
        [bb.x_max, bb.y_min, bb.z_max],
        [bb.x_max, bb.y_max, bb.z_max],
        [bb.x_min, bb.y_max, bb.z_max],
    ];

    let mut points = Points::<f64>::new();
    for c in &corners {
        points.push(*c);
    }

    let mut polys = CellArray::new();
    let faces: [[i64; 4]; 6] = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 4, 7, 3],
        [1, 2, 6, 5],
    ];
    for f in &faces {
        polys.push_cell(f);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

/// Compute the oriented bounding box dimensions (extent along each principal axis).
///
/// Returns `(center, half_extents, axes)`. This is a thin adapter over the single
/// implementation in [`crate::filters::mesh::obb::oriented_bounding_box`], which
/// follows `vtkOBBTree::ComputeOBB` (PCA axes, then the min/max projection along
/// each axis); only the return shape differs.
pub fn oriented_bounding_box(input: &PolyData) -> ([f64; 3], [f64; 3], [[f64; 3]; 3]) {
    let obb = crate::filters::mesh::obb::oriented_bounding_box(input);
    (obb.center, obb.half_extents, obb.axes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 3.0]);
        pd.polys.push_cell(&[0, 1]);

        let result = bounding_box_mesh(&pd);
        assert_eq!(result.points.len(), 8);
        assert_eq!(result.polys.num_cells(), 6);
    }

    #[test]
    fn obb_extents() {
        let mut pd = PolyData::new();
        // Points along X axis
        for i in 0..10 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        let (center, extents, _) = oriented_bounding_box(&pd);
        assert!((center[0] - 4.5).abs() < 1e-10);
        // Largest extent should be along X
        let max_ext = extents[0].max(extents[1]).max(extents[2]);
        assert!((max_ext - 4.5).abs() < 1e-10);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (_, extents, _) = oriented_bounding_box(&pd);
        assert_eq!(extents, [0.0, 0.0, 0.0]);
    }
}
