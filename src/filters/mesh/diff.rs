use crate::data::{AnyDataArray, DataArray, PolyData};

/// Result of comparing two PolyData meshes.
#[derive(Debug, Clone)]
pub struct MeshDiff {
    /// Maximum point position difference.
    pub max_position_diff: f64,
    /// Average point position difference.
    pub avg_position_diff: f64,
    /// Number of points in mesh A.
    pub points_a: usize,
    /// Number of points in mesh B.
    pub points_b: usize,
    /// Number of cells in mesh A.
    pub cells_a: usize,
    /// Number of cells in mesh B.
    pub cells_b: usize,
    /// Whether the meshes have the same topology.
    pub same_topology: bool,
    /// Whether the meshes are identical within tolerance.
    pub identical: bool,
}

impl std::fmt::Display for MeshDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.identical {
            write!(f, "Identical")
        } else {
            write!(
                f,
                "Different: max_pos_diff={:.6}, points={}vs{}, cells={}vs{}, same_topo={}",
                self.max_position_diff,
                self.points_a,
                self.points_b,
                self.cells_a,
                self.cells_b,
                self.same_topology
            )
        }
    }
}

/// Compare two PolyData meshes and compute differences.
pub fn diff(a: &PolyData, b: &PolyData) -> MeshDiff {
    let points_a = a.points.len();
    let points_b = b.points.len();
    let cells_a = a.polys.num_cells();
    let cells_b = b.polys.num_cells();

    let same_count = points_a == points_b && cells_a == cells_b;

    let same_topology =
        same_count && { a.polys.iter().zip(b.polys.iter()).all(|(ca, cb)| ca == cb) };

    let (max_diff, avg_diff) = if points_a == points_b && points_a > 0 {
        let mut max_d = 0.0f64;
        let mut sum_d = 0.0f64;
        for i in 0..points_a {
            let pa = a.points.get(i);
            let pb = b.points.get(i);
            let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2))
                .sqrt();
            max_d = max_d.max(d);
            sum_d += d;
        }
        (max_d, sum_d / points_a as f64)
    } else {
        (f64::INFINITY, f64::INFINITY)
    };

    let identical = same_topology && max_diff < 1e-10;

    MeshDiff {
        max_position_diff: max_diff,
        avg_position_diff: avg_diff,
        points_a,
        points_b,
        cells_a,
        cells_b,
        same_topology,
        identical,
    }
}

/// Compute per-point displacement between two meshes with the same topology.
/// Returns a new PolyData (copy of A) with a "Displacement" vector array.
pub fn displacement_field(a: &PolyData, b: &PolyData) -> Option<PolyData> {
    if a.points.len() != b.points.len() {
        return None;
    }

    let mut result = a.clone();
    let n = a.points.len();
    let mut disp = Vec::with_capacity(n * 3);
    for i in 0..n {
        let pa = a.points.get(i);
        let pb = b.points.get(i);
        disp.push(pb[0] - pa[0]);
        disp.push(pb[1] - pa[1]);
        disp.push(pb[2] - pa[2]);
    }
    let arr = DataArray::from_vec("Displacement", disp, 3);
    result.point_data_mut().add_array(AnyDataArray::F64(arr));
    result.point_data_mut().set_active_vectors("Displacement");
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_meshes() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = diff(&a, &a);
        assert!(d.identical);
        assert!(d.same_topology);
        assert!(d.max_position_diff < 1e-10);
    }

    #[test]
    fn different_positions() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = diff(&a, &b);
        assert!(!d.identical);
        assert!(d.same_topology);
        assert!(d.max_position_diff > 0.5);
    }

    #[test]
    fn different_topology() {
        let a = PolyData::from_triangles(
            vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let d = diff(&a, &b);
        assert!(!d.identical);
        assert!(!d.same_topology);
    }

    #[test]
    fn displacement() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let result = displacement_field(&a, &b).unwrap();
        assert!(result.point_data().vectors().is_some());
    }

    #[test]
    fn display() {
        let a = PolyData::from_triangles(
            vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let d = diff(&a, &a);
        assert_eq!(format!("{d}"), "Identical");
    }
}
