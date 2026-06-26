//! Mesh symmetry detection: find reflection planes and rotational symmetry.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// A detected symmetry plane.
#[derive(Debug, Clone)]
pub struct SymmetryPlane {
    pub normal: [f64; 3],
    pub point: [f64; 3],
    pub score: f64, // 0=no symmetry, 1=perfect
}

/// Detect the best reflection symmetry plane for a mesh.
///
/// Tests candidate planes through the centroid aligned with coordinate and
/// diagonal axes.
pub fn detect_symmetry_plane(mesh: &PolyData) -> Option<SymmetryPlane> {
    let n = mesh.points.len();
    if n < 3 {
        return None;
    }

    // Compute centroid
    let mut c = [0.0; 3];
    for i in 0..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            c[j] += p[j];
        }
    }
    for j in 0..3 {
        c[j] /= n as f64;
    }

    // Test 3 axis-aligned planes + 3 diagonal planes
    let candidates: Vec<[f64; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
    ];

    let mut best: Option<SymmetryPlane> = None;
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    for normal in &candidates {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let nn = [normal[0] / len, normal[1] / len, normal[2] / len];

        let score = symmetry_score(&pts, c, nn);
        if best.is_none() || score > best.as_ref().unwrap().score {
            best = Some(SymmetryPlane {
                normal: nn,
                point: c,
                score,
            });
        }
    }

    best
}

/// Compute symmetry score for a given plane.
///
/// Score is 1.0 - (mean distance to reflected closest point / mesh diameter).
pub fn symmetry_score(pts: &[[f64; 3]], plane_point: [f64; 3], plane_normal: [f64; 3]) -> f64 {
    let n = pts.len();
    if n < 2 {
        return 0.0;
    }
    let len = (plane_normal[0] * plane_normal[0]
        + plane_normal[1] * plane_normal[1]
        + plane_normal[2] * plane_normal[2])
        .sqrt();
    if len < 1e-15 {
        return 0.0;
    }
    let plane_normal = [
        plane_normal[0] / len,
        plane_normal[1] / len,
        plane_normal[2] / len,
    ];

    // Reflect all points across the plane
    let reflected: Vec<[f64; 3]> = pts
        .iter()
        .map(|p| {
            let d = (p[0] - plane_point[0]) * plane_normal[0]
                + (p[1] - plane_point[1]) * plane_normal[1]
                + (p[2] - plane_point[2]) * plane_normal[2];
            [
                p[0] - 2.0 * d * plane_normal[0],
                p[1] - 2.0 * d * plane_normal[1],
                p[2] - 2.0 * d * plane_normal[2],
            ]
        })
        .collect();

    // Compute mean closest-point distance between reflected and original
    let mut total_dist = 0.0;
    for rp in &reflected {
        let mut min_d = f64::MAX;
        for op in pts {
            let d = (rp[0] - op[0]).powi(2) + (rp[1] - op[1]).powi(2) + (rp[2] - op[2]).powi(2);
            min_d = min_d.min(d);
        }
        total_dist += min_d.sqrt();
    }
    let mean_dist = total_dist / n as f64;

    // Compute mesh diameter for normalization from the bounding box diagonal.
    let mut min_pt = pts[0];
    let mut max_pt = pts[0];
    for p in &pts[1..] {
        for j in 0..3 {
            min_pt[j] = min_pt[j].min(p[j]);
            max_pt[j] = max_pt[j].max(p[j]);
        }
    }
    let diameter = ((max_pt[0] - min_pt[0]).powi(2)
        + (max_pt[1] - min_pt[1]).powi(2)
        + (max_pt[2] - min_pt[2]).powi(2))
    .sqrt()
    .max(1e-15);

    (1.0 - mean_dist / diameter).max(0.0)
}

/// Add a "SymmetryDistance" point data array showing distance to the
/// nearest reflected point across a symmetry plane.
pub fn visualize_symmetry(
    mesh: &PolyData,
    plane_point: [f64; 3],
    plane_normal: [f64; 3],
) -> PolyData {
    let n = mesh.points.len();
    let len = (plane_normal[0] * plane_normal[0]
        + plane_normal[1] * plane_normal[1]
        + plane_normal[2] * plane_normal[2])
        .sqrt();
    if len < 1e-15 {
        return mesh.clone();
    }
    let plane_normal = [
        plane_normal[0] / len,
        plane_normal[1] / len,
        plane_normal[2] / len,
    ];
    let pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();

    let reflected: Vec<[f64; 3]> = pts
        .iter()
        .map(|p| {
            let d = (p[0] - plane_point[0]) * plane_normal[0]
                + (p[1] - plane_point[1]) * plane_normal[1]
                + (p[2] - plane_point[2]) * plane_normal[2];
            [
                p[0] - 2.0 * d * plane_normal[0],
                p[1] - 2.0 * d * plane_normal[1],
                p[2] - 2.0 * d * plane_normal[2],
            ]
        })
        .collect();

    let mut sym_dist = Vec::with_capacity(n);
    for rp in &reflected {
        let mut min_d = f64::MAX;
        for op in &pts {
            let d = (rp[0] - op[0]).powi(2) + (rp[1] - op[1]).powi(2) + (rp[2] - op[2]).powi(2);
            min_d = min_d.min(d);
        }
        sym_dist.push(min_d.sqrt());
    }

    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SymmetryDistance",
            sym_dist,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symmetric_box() {
        let mesh = PolyData::from_points(vec![
            [-1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        let plane = detect_symmetry_plane(&mesh).unwrap();
        assert!(plane.score > 0.8, "score={}", plane.score);
    }

    #[test]
    fn asymmetric() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [0.0, 0.1, 0.0]]);
        let plane = detect_symmetry_plane(&mesh);
        assert!(plane.is_some());
        // Score should be lower for asymmetric shape
    }

    #[test]
    fn visualize() {
        let mesh = PolyData::from_points(vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let result = visualize_symmetry(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let arr = result.point_data().get_array("SymmetryDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 0.01); // perfectly symmetric
    }
}
