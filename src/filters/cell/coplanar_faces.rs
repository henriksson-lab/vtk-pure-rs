use crate::data::PolyData;

#[path = "../core/mesh/coplanar_merge.rs"]
mod coplanar_merge;

/// Merge coplanar adjacent faces into larger polygons.
///
/// Triangles that share an edge and have normals within `angle_tolerance`
/// degrees are merged into a single polygon. Reduces face count on
/// piecewise-flat surfaces.
pub fn merge_coplanar(input: &PolyData, angle_tolerance: f64) -> PolyData {
    coplanar_merge::merge_coplanar_faces(input, angle_tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coplanar_triangles_grouped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // coplanar
        pd.polys.push_cell(&[0, 2, 3]); // coplanar

        let result = merge_coplanar(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0).len(), 4);
    }

    #[test]
    fn non_coplanar_separate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.5, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]); // XY plane
        pd.polys.push_cell(&[0, 1, 3]); // XZ plane

        let result = merge_coplanar(&pd, 1.0); // 1 degree tolerance
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = merge_coplanar(&pd, 5.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
