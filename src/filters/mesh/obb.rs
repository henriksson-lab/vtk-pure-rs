use crate::data::PolyData;

use crate::filters::mesh::pca_axes::{compute_pca, PcaResult};

/// Result of oriented bounding box computation.
#[derive(Debug, Clone)]
pub struct ObbResult {
    /// Center of the OBB.
    pub center: [f64; 3],
    /// Principal axes (unit vectors) of the OBB.
    pub axes: [[f64; 3]; 3],
    /// Half-extents along each axis.
    pub half_extents: [f64; 3],
}

impl ObbResult {
    /// Convert the OBB to a PolyData box (8 vertices, 6 quad faces).
    pub fn to_poly_data(&self) -> PolyData {
        let mut pd = PolyData::new();

        // Generate 8 corners: all combinations of +/- half_extents along each axis
        let signs: [[f64; 3]; 8] = [
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];

        for s in &signs {
            let mut pt = [0.0f64; 3];
            for k in 0..3 {
                pt[k] = self.center[k]
                    + s[0] * self.half_extents[0] * self.axes[0][k]
                    + s[1] * self.half_extents[1] * self.axes[1][k]
                    + s[2] * self.half_extents[2] * self.axes[2][k];
            }
            pd.points.push(pt);
        }

        // 6 quad faces (outward-facing)
        let faces: [[i64; 4]; 6] = [
            [0, 3, 2, 1], // -Z
            [4, 5, 6, 7], // +Z
            [0, 1, 5, 4], // -Y
            [2, 3, 7, 6], // +Y
            [0, 4, 7, 3], // -X
            [1, 2, 6, 5], // +X
        ];

        for face in &faces {
            pd.polys.push_cell(face);
        }

        pd
    }
}

/// Compute the oriented bounding box of a PolyData using PCA axes.
///
/// The OBB axes are determined by PCA on the point positions, and the
/// half-extents are the max projection distances along each axis from the center.
pub fn oriented_bounding_box(input: &PolyData) -> ObbResult {
    let pca: PcaResult = compute_pca(input);
    let n: usize = input.points.len();

    if n == 0 {
        return ObbResult {
            center: [0.0, 0.0, 0.0],
            axes: pca.axes,
            half_extents: [0.0, 0.0, 0.0],
        };
    }

    // Project all points onto PCA axes and find min/max
    let mut mins = [f64::MAX; 3];
    let mut maxs = [f64::MIN; 3];

    for i in 0..n {
        let p = input.points.get(i);
        for axis_idx in 0..3 {
            let proj: f64 = (p[0] - pca.centroid[0]) * pca.axes[axis_idx][0]
                + (p[1] - pca.centroid[1]) * pca.axes[axis_idx][1]
                + (p[2] - pca.centroid[2]) * pca.axes[axis_idx][2];
            if proj < mins[axis_idx] {
                mins[axis_idx] = proj;
            }
            if proj > maxs[axis_idx] {
                maxs[axis_idx] = proj;
            }
        }
    }

    // Center the OBB at the midpoint of the projected ranges
    let mut center = [0.0f64; 3];
    let mut half_extents = [0.0f64; 3];
    for axis_idx in 0..3 {
        let mid: f64 = (mins[axis_idx] + maxs[axis_idx]) * 0.5;
        half_extents[axis_idx] = (maxs[axis_idx] - mins[axis_idx]) * 0.5;
        for k in 0..3 {
            center[k] += mid * pca.axes[axis_idx][k];
        }
    }
    // Shift center relative to the centroid
    for k in 0..3 {
        center[k] += pca.centroid[k];
    }

    ObbResult {
        center,
        axes: pca.axes,
        half_extents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_aligned_box_points() {
        // Points forming a box [0,2] x [0,1] x [0,0.5]
        let mut pd = PolyData::new();
        for &x in &[0.0, 2.0] {
            for &y in &[0.0, 1.0] {
                for &z in &[0.0, 0.5] {
                    pd.points.push([x, y, z]);
                }
            }
        }
        let result = oriented_bounding_box(&pd);
        // Half-extents should be roughly [1.0, 0.5, 0.25] in some order
        let mut he = result.half_extents;
        he.sort_by(|a, b| b.partial_cmp(a).unwrap());
        assert!((he[0] - 1.0).abs() < 0.01);
        assert!((he[1] - 0.5).abs() < 0.01);
        assert!((he[2] - 0.25).abs() < 0.01);
    }

    #[test]
    fn obb_to_poly_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);

        let result = oriented_bounding_box(&pd);
        let box_pd = result.to_poly_data();
        assert_eq!(box_pd.points.len(), 8);
        assert_eq!(box_pd.polys.num_cells(), 6);
    }

    #[test]
    fn empty_mesh_obb() {
        let pd = PolyData::new();
        let result = oriented_bounding_box(&pd);
        assert_eq!(result.half_extents, [0.0, 0.0, 0.0]);
    }
}
