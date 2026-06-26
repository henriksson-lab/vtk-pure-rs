use crate::data::{CellArray, Points, PolyData};

/// Result of symmetry plane detection.
#[derive(Debug, Clone)]
pub struct SymmetryPlaneResult {
    /// A point on the symmetry plane (the centroid of the mesh).
    pub point: [f64; 3],
    /// The normal direction of the symmetry plane.
    pub normal: [f64; 3],
    /// A score from 0.0 to 1.0 indicating how symmetric the mesh is
    /// about the detected plane (1.0 = perfectly symmetric).
    pub symmetry_score: f64,
}

/// Detect the approximate symmetry plane of a mesh using PCA.
///
/// Computes the centroid and covariance matrix of the point cloud,
/// then uses eigenvalue decomposition to find the principal axes.
/// The symmetry plane is perpendicular to the eigenvector with the
/// smallest eigenvalue, passing through the centroid.
///
/// The symmetry score measures how close to symmetric the point cloud is
/// with respect to the detected plane (based on the distance distribution
/// of reflected points to their nearest neighbors).
pub fn find_symmetry_plane(input: &PolyData) -> SymmetryPlaneResult {
    let n: usize = input.points.len();
    if n == 0 {
        return SymmetryPlaneResult {
            point: [0.0, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            symmetry_score: 0.0,
        };
    }

    // Compute centroid
    let mut cx: f64 = 0.0;
    let mut cy: f64 = 0.0;
    let mut cz: f64 = 0.0;
    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let nf: f64 = n as f64;
    cx /= nf;
    cy /= nf;
    cz /= nf;

    // Compute covariance matrix (symmetric 3x3)
    let mut cov: [[f64; 3]; 3] = [[0.0; 3]; 3];
    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        let dx: f64 = p[0] - cx;
        let dy: f64 = p[1] - cy;
        let dz: f64 = p[2] - cz;
        cov[0][0] += dx * dx;
        cov[0][1] += dx * dy;
        cov[0][2] += dx * dz;
        cov[1][1] += dy * dy;
        cov[1][2] += dy * dz;
        cov[2][2] += dz * dz;
    }
    cov[1][0] = cov[0][1];
    cov[2][0] = cov[0][2];
    cov[2][1] = cov[1][2];
    for row in &mut cov {
        for val in row.iter_mut() {
            *val /= nf;
        }
    }

    // Find eigenvectors using Jacobi iteration for 3x3 symmetric matrix
    let (_eigenvalues, eigenvectors) = jacobi_3x3(cov);

    // The symmetry plane normal is the eigenvector with the smallest eigenvalue.
    // For a symmetric object, the direction of least variance is the symmetry axis.
    // We test all three axes and pick the one that gives best symmetry.
    let mut best_idx: usize = 0;
    let mut best_score: f64 = -1.0;
    let centroid: [f64; 3] = [cx, cy, cz];

    for axis in 0..3 {
        let normal: [f64; 3] = eigenvectors[axis];
        let score: f64 = compute_symmetry_score(input, centroid, normal);
        if score > best_score {
            best_score = score;
            best_idx = axis;
        }
    }

    SymmetryPlaneResult {
        point: centroid,
        normal: eigenvectors[best_idx],
        symmetry_score: best_score,
    }
}

/// Reflect a PolyData mesh about an arbitrary plane defined by a point and normal.
///
/// The reflected polygons have reversed winding order to maintain consistent normals.
pub fn reflect_about_plane(input: &PolyData, point: [f64; 3], normal: [f64; 3]) -> PolyData {
    let n: usize = input.points.len();

    // Normalize the normal vector
    let len: f64 = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-15 {
        return input.clone();
    }
    let nx: f64 = normal[0] / len;
    let ny: f64 = normal[1] / len;
    let nz: f64 = normal[2] / len;

    let mut out_points: Points<f64> = Points::new();
    let mut out_polys: CellArray = CellArray::new();
    let mut out_lines: CellArray = CellArray::new();
    let mut out_verts: CellArray = CellArray::new();

    for i in 0..n {
        let p: [f64; 3] = input.points.get(i);
        // Reflect: p' = p - 2 * dot(p - point, n) * n
        let dx: f64 = p[0] - point[0];
        let dy: f64 = p[1] - point[1];
        let dz: f64 = p[2] - point[2];
        let dot: f64 = dx * nx + dy * ny + dz * nz;
        out_points.push([
            p[0] - 2.0 * dot * nx,
            p[1] - 2.0 * dot * ny,
            p[2] - 2.0 * dot * nz,
        ]);
    }

    // Reverse winding order for polys
    for cell in input.polys.iter() {
        let reversed: Vec<i64> = cell.iter().rev().copied().collect();
        out_polys.push_cell(&reversed);
    }

    for cell in input.lines.iter() {
        out_lines.push_cell(cell);
    }

    for cell in input.verts.iter() {
        out_verts.push_cell(cell);
    }

    let mut out = input.clone();
    out.points = out_points;
    out.polys = out_polys;
    out.lines = out_lines;
    out.verts = out_verts;
    out
}

/// Compute how symmetric a point set is about a plane (point + normal).
/// Returns a score between 0.0 and 1.0.
fn compute_symmetry_score(pd: &PolyData, plane_point: [f64; 3], normal: [f64; 3]) -> f64 {
    let n: usize = pd.points.len();
    if n == 0 {
        return 0.0;
    }

    let len: f64 = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
    if len < 1e-15 {
        return 0.0;
    }
    let nx: f64 = normal[0] / len;
    let ny: f64 = normal[1] / len;
    let nz: f64 = normal[2] / len;

    // Compute bounding box diagonal for normalization
    let mut min_pt: [f64; 3] = pd.points.get(0);
    let mut max_pt: [f64; 3] = pd.points.get(0);
    for i in 1..n {
        let p: [f64; 3] = pd.points.get(i);
        for d in 0..3 {
            if p[d] < min_pt[d] {
                min_pt[d] = p[d];
            }
            if p[d] > max_pt[d] {
                max_pt[d] = p[d];
            }
        }
    }
    let diag: f64 = ((max_pt[0] - min_pt[0]).powi(2)
        + (max_pt[1] - min_pt[1]).powi(2)
        + (max_pt[2] - min_pt[2]).powi(2))
    .sqrt();
    if diag < 1e-15 {
        return 1.0;
    }

    // For each point, reflect it and find the nearest original point
    let mut total_dist: f64 = 0.0;
    for i in 0..n {
        let p: [f64; 3] = pd.points.get(i);
        let dx: f64 = p[0] - plane_point[0];
        let dy: f64 = p[1] - plane_point[1];
        let dz: f64 = p[2] - plane_point[2];
        let dot: f64 = dx * nx + dy * ny + dz * nz;
        let rp: [f64; 3] = [
            p[0] - 2.0 * dot * nx,
            p[1] - 2.0 * dot * ny,
            p[2] - 2.0 * dot * nz,
        ];

        // Find nearest point in original set
        let mut min_d2: f64 = f64::MAX;
        for j in 0..n {
            let q: [f64; 3] = pd.points.get(j);
            let d2: f64 = (rp[0] - q[0]).powi(2) + (rp[1] - q[1]).powi(2) + (rp[2] - q[2]).powi(2);
            if d2 < min_d2 {
                min_d2 = d2;
            }
        }
        total_dist += min_d2.sqrt();
    }

    let avg_dist: f64 = total_dist / n as f64;
    let normalized: f64 = avg_dist / diag;
    (1.0 - normalized.min(1.0)).max(0.0)
}

/// Jacobi eigenvalue iteration for a 3x3 symmetric matrix.
/// Returns (eigenvalues, eigenvectors) sorted by eigenvalue ascending.
fn jacobi_3x3(mat: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let mut a: [[f64; 3]; 3] = mat;
    // V starts as identity
    let mut v: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..100 {
        // Find the largest off-diagonal element
        let mut max_val: f64 = 0.0;
        let mut p: usize = 0;
        let mut q: usize = 1;
        for i in 0..3 {
            for j in (i + 1)..3 {
                if a[i][j].abs() > max_val {
                    max_val = a[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }
        if max_val < 1e-15 {
            break;
        }

        // Compute rotation angle
        let theta: f64 = if (a[p][p] - a[q][q]).abs() < 1e-15 {
            std::f64::consts::FRAC_PI_4
        } else {
            0.5 * ((2.0 * a[p][q]) / (a[p][p] - a[q][q])).atan()
        };

        let c: f64 = theta.cos();
        let s: f64 = theta.sin();

        // Apply Givens rotation
        let mut new_a: [[f64; 3]; 3] = a;
        new_a[p][p] = c * c * a[p][p] + 2.0 * s * c * a[p][q] + s * s * a[q][q];
        new_a[q][q] = s * s * a[p][p] - 2.0 * s * c * a[p][q] + c * c * a[q][q];
        new_a[p][q] = 0.0;
        new_a[q][p] = 0.0;

        for r in 0..3 {
            if r != p && r != q {
                let val_rp: f64 = c * a[r][p] + s * a[r][q];
                let val_rq: f64 = -s * a[r][p] + c * a[r][q];
                new_a[r][p] = val_rp;
                new_a[p][r] = val_rp;
                new_a[r][q] = val_rq;
                new_a[q][r] = val_rq;
            }
        }
        a = new_a;

        // Update eigenvectors
        for r in 0..3 {
            let vp: f64 = v[r][p];
            let vq: f64 = v[r][q];
            v[r][p] = c * vp + s * vq;
            v[r][q] = -s * vp + c * vq;
        }
    }

    let eigenvalues: [f64; 3] = [a[0][0], a[1][1], a[2][2]];
    // Eigenvectors are columns of v
    let eigenvectors: [[f64; 3]; 3] = [
        [v[0][0], v[1][0], v[2][0]],
        [v[0][1], v[1][1], v[2][1]],
        [v[0][2], v[1][2], v[2][2]],
    ];

    // Sort by eigenvalue ascending
    let mut indices: [usize; 3] = [0, 1, 2];
    indices.sort_by(|&a_idx, &b_idx| {
        eigenvalues[a_idx]
            .partial_cmp(&eigenvalues[b_idx])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sorted_evals: [f64; 3] = [
        eigenvalues[indices[0]],
        eigenvalues[indices[1]],
        eigenvalues[indices[2]],
    ];
    let sorted_evecs: [[f64; 3]; 3] = [
        eigenvectors[indices[0]],
        eigenvectors[indices[1]],
        eigenvectors[indices[2]],
    ];

    (sorted_evals, sorted_evecs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_symmetric_mesh() -> PolyData {
        // Mesh symmetric about the YZ plane (X=0)
        let mut pd = PolyData::new();
        pd.points.push([-1.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, -1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.points.push([0.0, 0.0, -1.0]);
        pd.polys.push_cell(&[0, 2, 4]);
        pd.polys.push_cell(&[1, 2, 4]);
        pd.polys.push_cell(&[0, 3, 5]);
        pd.polys.push_cell(&[1, 3, 5]);
        pd
    }

    #[test]
    fn detect_symmetry_plane_of_symmetric_mesh() {
        let mesh = make_symmetric_mesh();
        let result = find_symmetry_plane(&mesh);
        // Centroid should be at origin
        assert!(result.point[0].abs() < 1e-10);
        assert!(result.point[1].abs() < 1e-10);
        assert!(result.point[2].abs() < 1e-10);
        // Score should be high for a symmetric mesh
        assert!(result.symmetry_score > 0.5);
    }

    #[test]
    fn reflect_about_yz_plane() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        pd.points.push([4.0, 5.0, 6.0]);
        pd.polys.push_cell(&[0, 1, 0]); // degenerate, just for testing

        let reflected = reflect_about_plane(&pd, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let p0: [f64; 3] = reflected.points.get(0);
        let p1: [f64; 3] = reflected.points.get(1);

        assert!((p0[0] - (-1.0)).abs() < 1e-10);
        assert!((p0[1] - 2.0).abs() < 1e-10);
        assert!((p0[2] - 3.0).abs() < 1e-10);
        assert!((p1[0] - (-4.0)).abs() < 1e-10);
        assert!((p1[1] - 5.0).abs() < 1e-10);
        assert!((p1[2] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn reflect_preserves_point_count() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let reflected = reflect_about_plane(&pd, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(reflected.points.len(), 3);
        assert_eq!(reflected.polys.num_cells(), 1);
    }
}
