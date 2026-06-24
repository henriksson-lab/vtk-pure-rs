//! Oriented Bounding Box (OBB) tree for fast spatial queries on meshes.
//!
//! Builds a binary tree of oriented bounding boxes for ray intersection,
//! closest-point queries, and collision detection.

use crate::data::PolyData;

/// An oriented bounding box defined by a center, three axes, and half-extents.
#[derive(Debug, Clone)]
pub struct Obb {
    pub center: [f64; 3],
    pub axes: [[f64; 3]; 3],
    pub half_extents: [f64; 3],
}

impl Obb {
    /// Compute OBB from a set of points using PCA.
    pub fn from_points(points: &[[f64; 3]]) -> Self {
        let finite_points: Vec<[f64; 3]> = points
            .iter()
            .copied()
            .filter(|p| is_finite_point(*p))
            .collect();
        let points = finite_points.as_slice();
        if points.is_empty() {
            return Self {
                center: [0.0; 3],
                axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                half_extents: [0.0; 3],
            };
        }

        // Compute centroid
        let n = points.len() as f64;
        let mut center = [0.0; 3];
        for p in points {
            center[0] += p[0];
            center[1] += p[1];
            center[2] += p[2];
        }
        center[0] /= n;
        center[1] /= n;
        center[2] /= n;

        // Compute covariance matrix
        let mut cov = [[0.0f64; 3]; 3];
        for p in points {
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            for i in 0..3 {
                for j in 0..3 {
                    cov[i][j] += d[i] * d[j];
                }
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                cov[i][j] /= n;
            }
        }

        obb_from_mean_cov_points(center, cov, points)
    }

    /// Test if a point is inside the OBB.
    pub fn contains(&self, point: [f64; 3]) -> bool {
        if !is_finite_point(point) {
            return false;
        }
        let d = [
            point[0] - self.center[0],
            point[1] - self.center[1],
            point[2] - self.center[2],
        ];
        for (i, axis) in self.axes.iter().enumerate() {
            let proj = (d[0] * axis[0] + d[1] * axis[1] + d[2] * axis[2]).abs();
            if proj > self.half_extents[i] {
                return false;
            }
        }
        true
    }

    /// Volume of the OBB.
    pub fn volume(&self) -> f64 {
        8.0 * self.half_extents[0] * self.half_extents[1] * self.half_extents[2]
    }
}

/// OBB tree node.
#[derive(Debug)]
enum ObbNode {
    Leaf {
        obb: Obb,
        cell_indices: Vec<usize>,
    },
    Internal {
        obb: Obb,
        left: Box<ObbNode>,
        right: Box<ObbNode>,
    },
}

#[derive(Debug, Clone)]
struct CellGeometry {
    point_ids: Vec<usize>,
    points: Vec<[f64; 3]>,
    centroid: [f64; 3],
}

/// Oriented Bounding Box tree for spatial queries.
#[derive(Debug)]
pub struct ObbTree {
    root: Option<ObbNode>,
}

impl ObbTree {
    /// Build an OBB tree from a PolyData mesh.
    pub fn build(poly_data: &PolyData, max_leaf_size: usize) -> Self {
        let num_cells = poly_data.polys.num_cells();
        if num_cells == 0 {
            return Self { root: None };
        }

        // Compute geometry for all valid cells.
        let mut cells = Vec::with_capacity(num_cells);
        let mut indices = Vec::with_capacity(num_cells);
        for ci in 0..num_cells {
            let cell = poly_data.polys.cell(ci);
            if cell.is_empty() {
                cells.push(None);
                continue;
            }
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            let mut valid = true;
            let mut point_ids = Vec::with_capacity(cell.len());
            let mut points = Vec::with_capacity(cell.len());
            for &vid in cell {
                if vid < 0 || vid as usize >= poly_data.points.len() {
                    valid = false;
                    break;
                }
                let p = poly_data.points.get(vid as usize);
                if !is_finite_point(p) {
                    valid = false;
                    break;
                }
                cx += p[0];
                cy += p[1];
                cz += p[2];
                point_ids.push(vid as usize);
                points.push(p);
            }
            if !valid {
                cells.push(None);
                continue;
            }
            let n = cell.len() as f64;
            cells.push(Some(CellGeometry {
                point_ids,
                points,
                centroid: [cx / n, cy / n, cz / n],
            }));
            indices.push(ci);
        }

        if indices.is_empty() {
            return Self { root: None };
        }
        let root = Self::build_node(&cells, indices, max_leaf_size);
        Self { root: Some(root) }
    }

    fn build_node(
        cells: &[Option<CellGeometry>],
        indices: Vec<usize>,
        max_leaf_size: usize,
    ) -> ObbNode {
        let obb = compute_cell_obb(cells, &indices);

        if indices.len() <= max_leaf_size {
            return ObbNode::Leaf {
                obb,
                cell_indices: indices,
            };
        }

        let Some((left_indices, right_indices)) = split_cells_like_vtk(cells, &indices, &obb)
        else {
            return ObbNode::Leaf {
                obb,
                cell_indices: indices,
            };
        };

        let left = Box::new(Self::build_node(cells, left_indices, max_leaf_size));
        let right = Box::new(Self::build_node(cells, right_indices, max_leaf_size));

        ObbNode::Internal { obb, left, right }
    }

    /// Find all leaf cell indices whose OBB contains the given point.
    pub fn find_cells_containing(&self, point: [f64; 3]) -> Vec<usize> {
        let mut result = Vec::new();
        if is_finite_point(point) {
            if let Some(ref root) = self.root {
                Self::query_node(root, point, &mut result);
            }
        }
        result
    }

    fn query_node(node: &ObbNode, point: [f64; 3], result: &mut Vec<usize>) {
        match node {
            ObbNode::Leaf { obb, cell_indices } => {
                if obb.contains(point) {
                    result.extend_from_slice(cell_indices);
                }
            }
            ObbNode::Internal { obb, left, right } => {
                if obb.contains(point) {
                    Self::query_node(left, point, result);
                    Self::query_node(right, point, result);
                }
            }
        }
    }

    /// Count total leaf cells.
    pub fn num_cells(&self) -> usize {
        fn count(node: &ObbNode) -> usize {
            match node {
                ObbNode::Leaf { cell_indices, .. } => cell_indices.len(),
                ObbNode::Internal { left, right, .. } => count(left) + count(right),
            }
        }
        self.root.as_ref().map_or(0, count)
    }
}

fn compute_cell_obb(cells: &[Option<CellGeometry>], indices: &[usize]) -> Obb {
    let mut unique_points = Vec::new();
    let mut inserted = vec![
        false;
        cells
            .iter()
            .filter_map(|cell| cell.as_ref())
            .flat_map(|cell| cell.point_ids.iter().copied())
            .max()
            .map_or(0, |max_id| max_id + 1)
    ];

    let mut mean = [0.0; 3];
    let mut moment = [[0.0f64; 3]; 3];
    let mut total_mass = 0.0;

    for &cell_id in indices {
        let Some(cell) = cells[cell_id].as_ref() else {
            continue;
        };

        for (&point_id, &point) in cell.point_ids.iter().zip(cell.points.iter()) {
            if !inserted[point_id] {
                inserted[point_id] = true;
                unique_points.push(point);
            }
        }

        if cell.points.len() < 3 {
            continue;
        }

        let p = cell.points[0];
        for j in 1..cell.points.len() - 1 {
            let q = cell.points[j];
            let r = cell.points[j + 1];
            let dp0 = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let dp1 = [r[0] - p[0], r[1] - p[1], r[2] - p[2]];
            let normal = cross3(dp0, dp1);
            let tri_mass = 0.5 * norm3(normal);
            if tri_mass <= 0.0 {
                continue;
            }

            let c = [
                (p[0] + q[0] + r[0]) / 3.0,
                (p[1] + q[1] + r[1]) / 3.0,
                (p[2] + q[2] + r[2]) / 3.0,
            ];
            total_mass += tri_mass;
            for k in 0..3 {
                mean[k] += tri_mass * c[k];
            }

            for a in 0..3 {
                moment[a][a] +=
                    tri_mass * (9.0 * c[a] * c[a] + p[a] * p[a] + q[a] * q[a] + r[a] * r[a]) / 12.0;
            }
            moment[0][1] +=
                tri_mass * (9.0 * c[0] * c[1] + p[0] * p[1] + q[0] * q[1] + r[0] * r[1]) / 12.0;
            moment[0][2] +=
                tri_mass * (9.0 * c[0] * c[2] + p[0] * p[2] + q[0] * q[2] + r[0] * r[2]) / 12.0;
            moment[1][2] +=
                tri_mass * (9.0 * c[1] * c[2] + p[1] * p[2] + q[1] * q[2] + r[1] * r[2]) / 12.0;
        }
    }

    if unique_points.is_empty() || total_mass <= 0.0 {
        let points: Vec<[f64; 3]> = indices
            .iter()
            .filter_map(|&i| cells[i].as_ref())
            .flat_map(|cell| cell.points.iter().copied())
            .collect();
        return Obb::from_points(&points);
    }

    for v in &mut mean {
        *v /= total_mass;
    }
    moment[1][0] = moment[0][1];
    moment[2][0] = moment[0][2];
    moment[2][1] = moment[1][2];

    let mut cov = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            cov[i][j] = moment[i][j] / total_mass - mean[i] * mean[j];
        }
    }

    obb_from_mean_cov_points(mean, cov, &unique_points)
}

fn split_cells_like_vtk(
    cells: &[Option<CellGeometry>],
    indices: &[usize],
    obb: &Obb,
) -> Option<(Vec<usize>, Vec<usize>)> {
    let mut best_ratio = 1.0;
    let mut best_split = None;

    for split_plane in 0..3 {
        let (left, right) = split_on_plane(cells, indices, obb, split_plane);
        if left.is_empty() || right.is_empty() {
            continue;
        }

        let ratio = ((right.len() as f64 - left.len() as f64) / indices.len() as f64).abs();
        if ratio < 0.6 {
            return Some((left, right));
        }
        if ratio < best_ratio {
            best_ratio = ratio;
            best_split = Some((left, right));
        }
    }

    if best_ratio < 0.95 {
        best_split
    } else {
        None
    }
}

fn split_on_plane(
    cells: &[Option<CellGeometry>],
    indices: &[usize],
    obb: &Obb,
    split_plane: usize,
) -> (Vec<usize>, Vec<usize>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let n = obb.axes[split_plane];
    let p = obb.center;

    for &cell_id in indices {
        let Some(cell) = cells[cell_id].as_ref() else {
            continue;
        };
        let mut negative = false;
        let mut positive = false;
        for &x in &cell.points {
            let val = dot3(&n, [x[0] - p[0], x[1] - p[1], x[2] - p[2]]);
            if val < 0.0 {
                negative = true;
            } else {
                positive = true;
            }
        }

        if negative && positive {
            let c = cell.centroid;
            let val = dot3(&n, [c[0] - p[0], c[1] - p[1], c[2] - p[2]]);
            if val < 0.0 {
                left.push(cell_id);
            } else {
                right.push(cell_id);
            }
        } else if negative {
            left.push(cell_id);
        } else {
            right.push(cell_id);
        }
    }

    (left, right)
}

fn obb_from_mean_cov_points(mean: [f64; 3], cov: [[f64; 3]; 3], points: &[[f64; 3]]) -> Obb {
    let axes = eigen_axes_3x3(&cov);

    let mut mins = [f64::MAX; 3];
    let mut maxs = [f64::MIN; 3];
    for p in points {
        let d = [p[0] - mean[0], p[1] - mean[1], p[2] - mean[2]];
        for (a, (mn, mx)) in axes.iter().zip(mins.iter_mut().zip(maxs.iter_mut())) {
            let proj = d[0] * a[0] + d[1] * a[1] + d[2] * a[2];
            *mn = mn.min(proj);
            *mx = mx.max(proj);
        }
    }

    let half_extents = [
        (maxs[0] - mins[0]) / 2.0,
        (maxs[1] - mins[1]) / 2.0,
        (maxs[2] - mins[2]) / 2.0,
    ];
    let mid = [
        (maxs[0] + mins[0]) / 2.0,
        (maxs[1] + mins[1]) / 2.0,
        (maxs[2] + mins[2]) / 2.0,
    ];
    let center = [
        mean[0] + mid[0] * axes[0][0] + mid[1] * axes[1][0] + mid[2] * axes[2][0],
        mean[1] + mid[0] * axes[0][1] + mid[1] * axes[1][1] + mid[2] * axes[2][1],
        mean[2] + mid[0] * axes[0][2] + mid[1] * axes[1][2] + mid[2] * axes[2][2],
    ];

    Obb {
        center,
        axes,
        half_extents,
    }
}

/// Compute 3 orthogonal eigenvectors of a 3x3 symmetric matrix via power iteration.
fn eigen_axes_3x3(cov: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let v1 = power_iteration(cov, [1.0, 0.0, 0.0], 20);
    // Deflate
    let e1 = dot3(&mat_vec(cov, v1), v1);
    let mut cov2 = *cov;
    for i in 0..3 {
        for j in 0..3 {
            cov2[i][j] -= e1 * v1[i] * v1[j];
        }
    }
    let v2_raw = power_iteration(&cov2, [0.0, 1.0, 0.0], 20);
    // Gram-Schmidt
    let proj = dot3(&v2_raw, v1);
    let mut v2 = [
        v2_raw[0] - proj * v1[0],
        v2_raw[1] - proj * v1[1],
        v2_raw[2] - proj * v1[2],
    ];
    let len = (v2[0] * v2[0] + v2[1] * v2[1] + v2[2] * v2[2]).sqrt();
    if len > 1e-15 {
        v2[0] /= len;
        v2[1] /= len;
        v2[2] /= len;
    } else {
        v2 = [0.0, 1.0, 0.0];
    }

    let v3 = cross3(v1, v2);
    [v1, v2, v3]
}

fn power_iteration(mat: &[[f64; 3]; 3], mut v: [f64; 3], iters: usize) -> [f64; 3] {
    for _ in 0..iters {
        v = mat_vec(mat, v);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len < 1e-30 {
            return [1.0, 0.0, 0.0];
        }
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
    v
}

fn mat_vec(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn dot3(a: &[f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm3(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn is_finite_point(point: [f64; 3]) -> bool {
    point.iter().all(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obb_from_axis_aligned_points() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [2.0, 1.0, 1.0],
        ];
        let obb = Obb::from_points(&points);
        assert!(obb.contains([1.0, 0.5, 0.25]));
        assert!(!obb.contains([5.0, 5.0, 5.0]));
        assert!(obb.volume() > 0.0);
    }

    #[test]
    fn obb_tree_build_and_query() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.5],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.5],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        let tree = ObbTree::build(&pd, 1);
        assert_eq!(tree.num_cells(), 2);
    }

    #[test]
    fn obb_tree_skips_empty_and_invalid_cells() {
        let mut pd = PolyData::new();
        pd.points =
            crate::data::Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        pd.polys.push_cell(&[]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 4, 2]);

        let tree = ObbTree::build(&pd, 1);
        assert_eq!(tree.num_cells(), 1);
    }

    #[test]
    fn obb_tree_leaf_bounds_cover_cell_geometry_not_only_centroids() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let tree = ObbTree::build(&pd, 1);
        assert_eq!(tree.find_cells_containing([9.0, 0.0, 0.0]), vec![0]);
        assert!(tree.find_cells_containing([f64::NAN, 0.0, 0.0]).is_empty());
    }
}
