//! Mesh comparison and difference visualization.
//!
//! Computes per-vertex distance maps, overlap analysis, and
//! difference metrics between two meshes.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Comparison statistics between two meshes.
#[derive(Debug, Clone)]
pub struct MeshComparisonStats {
    pub mean_distance: f64,
    pub max_distance: f64,
    pub rms_distance: f64,
    pub hausdorff_distance: f64,
    pub point_count_a: usize,
    pub point_count_b: usize,
    pub cell_count_a: usize,
    pub cell_count_b: usize,
}

impl std::fmt::Display for MeshComparisonStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MeshComparison: mean={:.6}, max={:.6}, RMS={:.6}, Hausdorff={:.6}, pts={}/{}",
            self.mean_distance,
            self.max_distance,
            self.rms_distance,
            self.hausdorff_distance,
            self.point_count_a,
            self.point_count_b
        )
    }
}

/// Compare two meshes and compute distance statistics.
///
/// For each point in mesh A, finds the closest point in mesh B.
/// Returns mesh A with a "Distance" point data array and statistics.
pub fn compare_meshes(mesh_a: &PolyData, mesh_b: &PolyData) -> (PolyData, MeshComparisonStats) {
    let na = mesh_a.points.len();
    let nb = mesh_b.points.len();

    if na == 0 || nb == 0 {
        return (
            mesh_a.clone(),
            MeshComparisonStats {
                mean_distance: 0.0,
                max_distance: 0.0,
                rms_distance: 0.0,
                hausdorff_distance: 0.0,
                point_count_a: na,
                point_count_b: nb,
                cell_count_a: mesh_a.polys.num_cells(),
                cell_count_b: mesh_b.polys.num_cells(),
            },
        );
    }

    let pts_b: Vec<[f64; 3]> = (0..nb).map(|i| mesh_b.points.get(i)).collect();

    let mut distances = Vec::with_capacity(na);
    let mut sum = 0.0;
    let mut sum2 = 0.0;
    let mut max_d = 0.0f64;

    for i in 0..na {
        let pa = mesh_a.points.get(i);
        let mut min_d2 = f64::MAX;
        for pb in &pts_b {
            let d2 = (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);
            min_d2 = min_d2.min(d2);
        }
        let d = min_d2.sqrt();
        distances.push(d);
        sum += d;
        sum2 += d * d;
        max_d = max_d.max(d);
    }

    // Compute reverse Hausdorff (B→A)
    let pts_a: Vec<[f64; 3]> = (0..na).map(|i| mesh_a.points.get(i)).collect();
    let mut max_d_reverse = 0.0f64;
    for pb in &pts_b {
        let mut min_d2 = f64::MAX;
        for pa in &pts_a {
            let d2 = (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);
            min_d2 = min_d2.min(d2);
        }
        max_d_reverse = max_d_reverse.max(min_d2.sqrt());
    }

    let hausdorff = max_d.max(max_d_reverse);
    let mean = sum / na as f64;
    let rms = (sum2 / na as f64).sqrt();

    let mut result = mesh_a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Distance", distances, 1,
        )));

    let stats = MeshComparisonStats {
        mean_distance: mean,
        max_distance: max_d,
        rms_distance: rms,
        hausdorff_distance: hausdorff,
        point_count_a: na,
        point_count_b: nb,
        cell_count_a: mesh_a.polys.num_cells(),
        cell_count_b: mesh_b.polys.num_cells(),
    };

    (result, stats)
}

/// Compute overlap ratio between two meshes' bounding boxes.
pub fn bounding_box_overlap_ratio(a: &PolyData, b: &PolyData) -> f64 {
    if a.points.len() == 0 || b.points.len() == 0 {
        return 0.0;
    }

    let bb = |mesh: &PolyData| -> ([f64; 3], [f64; 3]) {
        let mut min = mesh.points.get(0);
        let mut max = min;
        for i in 1..mesh.points.len() {
            let p = mesh.points.get(i);
            for j in 0..3 {
                min[j] = min[j].min(p[j]);
                max[j] = max[j].max(p[j]);
            }
        }
        (min, max)
    };

    let (min_a, max_a) = bb(a);
    let (min_b, max_b) = bb(b);

    let mut overlap_measure = 1.0;
    let mut measure_a = 1.0;
    let mut measure_b = 1.0;
    let mut active_dims = 0;

    for i in 0..3 {
        let o_min = min_a[i].max(min_b[i]);
        let o_max = max_a[i].min(max_b[i]);
        let extent_a = max_a[i] - min_a[i];
        let extent_b = max_b[i] - min_b[i];
        let union_min = min_a[i].min(min_b[i]);
        let union_max = max_a[i].max(max_b[i]);
        let union_extent = union_max - union_min;

        if union_extent <= 1e-15 {
            continue;
        }
        if o_min > o_max || (o_max - o_min) <= 1e-15 {
            return 0.0;
        }
        overlap_measure *= o_max - o_min;
        measure_a *= extent_a.max(1e-15);
        measure_b *= extent_b.max(1e-15);
        active_dims += 1;
    }

    if active_dims == 0 {
        return 1.0;
    }

    let union_measure = measure_a + measure_b - overlap_measure;
    if union_measure > 1e-15 {
        overlap_measure / union_measure
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_meshes() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let (_, stats) = compare_meshes(&mesh, &mesh);
        assert!((stats.mean_distance).abs() < 1e-10);
        assert!((stats.hausdorff_distance).abs() < 1e-10);
    }

    #[test]
    fn shifted_meshes() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let b = PolyData::from_points(vec![[0.5, 0.0, 0.0], [1.5, 0.0, 0.0]]);
        let (result, stats) = compare_meshes(&a, &b);
        assert!(stats.mean_distance > 0.0);
        assert!(result.point_data().get_array("Distance").is_some());
    }

    #[test]
    fn overlap_ratio() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        let b = PolyData::from_points(vec![[0.5, 0.5, 0.5], [1.5, 1.5, 1.5]]);
        let ratio = bounding_box_overlap_ratio(&a, &b);
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn no_overlap() {
        let a = PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        let b = PolyData::from_points(vec![[5.0, 5.0, 5.0], [6.0, 6.0, 6.0]]);
        assert_eq!(bounding_box_overlap_ratio(&a, &b), 0.0);
    }

    #[test]
    fn display() {
        let mesh = PolyData::from_points(vec![[0.0; 3]]);
        let (_, stats) = compare_meshes(&mesh, &mesh);
        let s = format!("{stats}");
        assert!(s.contains("MeshComparison"));
    }
}
