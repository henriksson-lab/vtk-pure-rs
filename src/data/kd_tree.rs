/// A k-d tree for fast nearest-neighbor queries on 3D point sets.
///
/// Build once, query many times. Uses median-of-three partitioning.
#[derive(Debug, Clone)]
pub struct KdTree {
    nodes: Vec<KdNode>,
    points: Vec<[f64; 3]>,
    indices: Vec<usize>,
}

#[derive(Debug, Clone)]
struct KdNode {
    /// Index into the `indices` array.
    point_idx: usize,
    /// Split axis (0=x, 1=y, 2=z).
    axis: u8,
    left: Option<usize>,
    right: Option<usize>,
}

impl KdTree {
    /// Build a k-d tree from a slice of 3D points.
    pub fn build(points: &[[f64; 3]]) -> Self {
        let pts: Vec<[f64; 3]> = points.to_vec();
        let n = pts.len();
        let mut indices: Vec<usize> = (0..n).collect();
        let mut nodes = Vec::with_capacity(n);

        if n > 0 {
            build_subtree(&pts, &mut indices, 0, n, 0, &mut nodes);
        }

        Self {
            nodes,
            points: pts,
            indices,
        }
    }

    /// Build from a `Points` object.
    pub fn from_points(points: &crate::data::Points<f64>) -> Self {
        let pts: Vec<[f64; 3]> = (0..points.len()).map(|i| points.get(i)).collect();
        Self::build(&pts)
    }

    /// Find the nearest point to `query`. Returns (original_index, distance²).
    pub fn nearest(&self, query: [f64; 3]) -> Option<(usize, f64)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut best_idx = 0;
        let mut best_dist2 = f64::INFINITY;
        self.search_nearest(0, &query, &mut best_idx, &mut best_dist2);
        Some((self.indices[best_idx], best_dist2))
    }

    /// Find all points within `radius` of `query`. Returns vec of (original_index, distance²).
    pub fn find_within_radius(&self, query: [f64; 3], radius: f64) -> Vec<(usize, f64)> {
        let mut results = Vec::new();
        if !self.nodes.is_empty() {
            let r2 = radius * radius;
            self.search_radius(0, &query, r2, &mut results);
        }
        results
    }

    /// Find k nearest neighbors. Returns vec of (original_index, distance²) sorted by distance.
    pub fn k_nearest(&self, query: [f64; 3], k: usize) -> Vec<(usize, f64)> {
        if self.nodes.is_empty() || k == 0 {
            return Vec::new();
        }
        let mut heap: Vec<(usize, f64)> = Vec::with_capacity(k + 1);
        self.search_knn(0, &query, k, &mut heap);
        heap.sort_by(|a, b| a.1.total_cmp(&b.1));
        heap
    }

    fn search_nearest(
        &self,
        node_idx: usize,
        query: &[f64; 3],
        best_idx: &mut usize,
        best_dist2: &mut f64,
    ) {
        let node = &self.nodes[node_idx];
        let pt = self.points[self.indices[node.point_idx]];
        let d2 = dist2(*query, pt);
        if d2 <= *best_dist2 {
            *best_dist2 = d2;
            *best_idx = node.point_idx;
        }

        let axis = node.axis as usize;
        let diff = query[axis] - pt[axis];
        let (first, second) = if diff < 0.0 {
            (node.left, node.right)
        } else {
            (node.right, node.left)
        };

        if let Some(child) = first {
            self.search_nearest(child, query, best_idx, best_dist2);
        }
        if diff * diff < *best_dist2 {
            if let Some(child) = second {
                self.search_nearest(child, query, best_idx, best_dist2);
            }
        }
    }

    fn search_radius(
        &self,
        node_idx: usize,
        query: &[f64; 3],
        r2: f64,
        results: &mut Vec<(usize, f64)>,
    ) {
        let node = &self.nodes[node_idx];
        let pt = self.points[self.indices[node.point_idx]];
        let d2 = dist2(*query, pt);
        if d2 <= r2 {
            results.push((self.indices[node.point_idx], d2));
        }

        let axis = node.axis as usize;
        let diff = query[axis] - pt[axis];

        if let Some(left) = node.left {
            if diff - r2.sqrt() <= 0.0 || diff * diff <= r2 {
                self.search_radius(left, query, r2, results);
            }
        }
        if let Some(right) = node.right {
            if diff + r2.sqrt() >= 0.0 || diff * diff <= r2 {
                self.search_radius(right, query, r2, results);
            }
        }
    }

    fn search_knn(
        &self,
        node_idx: usize,
        query: &[f64; 3],
        k: usize,
        heap: &mut Vec<(usize, f64)>,
    ) {
        let node = &self.nodes[node_idx];
        let pt = self.points[self.indices[node.point_idx]];
        let d2 = dist2(*query, pt);

        if heap.len() < k {
            heap.push((self.indices[node.point_idx], d2));
            heap.sort_by(|a, b| b.1.total_cmp(&a.1)); // max-heap order
        } else if d2 < heap[0].1 {
            heap[0] = (self.indices[node.point_idx], d2);
            heap.sort_by(|a, b| b.1.total_cmp(&a.1));
        }

        let axis = node.axis as usize;
        let diff = query[axis] - pt[axis];
        let (first, second) = if diff < 0.0 {
            (node.left, node.right)
        } else {
            (node.right, node.left)
        };

        if let Some(child) = first {
            self.search_knn(child, query, k, heap);
        }
        let worst = if heap.len() < k { f64::MAX } else { heap[0].1 };
        if diff * diff < worst {
            if let Some(child) = second {
                self.search_knn(child, query, k, heap);
            }
        }
    }
}

fn build_subtree(
    points: &[[f64; 3]],
    indices: &mut [usize],
    start: usize,
    end: usize,
    depth: usize,
    nodes: &mut Vec<KdNode>,
) -> usize {
    if start >= end {
        return usize::MAX;
    }

    let axis = (depth % 3) as u8;
    let mid = (start + end) / 2;

    // Partial sort to find median
    indices[start..end].select_nth_unstable_by(mid - start, |&a, &b| {
        points[a][axis as usize].total_cmp(&points[b][axis as usize])
    });

    let node_idx = nodes.len();
    nodes.push(KdNode {
        point_idx: mid,
        axis,
        left: None,
        right: None,
    });

    let left = if mid > start {
        Some(build_subtree(points, indices, start, mid, depth + 1, nodes))
    } else {
        None
    };

    let right = if mid + 1 < end {
        Some(build_subtree(
            points,
            indices,
            mid + 1,
            end,
            depth + 1,
            nodes,
        ))
    } else {
        None
    };

    nodes[node_idx].left = left;
    nodes[node_idx].right = right;
    node_idx
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1]) + (a[2] - b[2]) * (a[2] - b[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_neighbor() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ];
        let tree = KdTree::build(&points);
        let (idx, d2) = tree.nearest([0.6, 0.6, 0.0]).unwrap();
        assert_eq!(idx, 4); // closest to (0.5, 0.5, 0)
        assert!(d2 < 0.1);
    }

    #[test]
    fn k_nearest() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];
        let tree = KdTree::build(&points);
        let results = tree.k_nearest([0.5, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        // Two nearest: index 0 and index 1
        let indices: Vec<usize> = results.iter().map(|r| r.0).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn radius_search() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let tree = KdTree::build(&points);
        let results = tree.find_within_radius([0.0, 0.0, 0.0], 2.0);
        assert_eq!(results.len(), 2); // points 0 and 1
        assert_eq!(tree.find_within_radius([0.0, 0.0, 0.0], -2.0).len(), 2);
    }

    #[test]
    fn empty_tree() {
        let tree = KdTree::build(&[]);
        assert!(tree.nearest([0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn single_point() {
        let tree = KdTree::build(&[[1.0, 2.0, 3.0]]);
        let (idx, _) = tree.nearest([0.0, 0.0, 0.0]).unwrap();
        assert_eq!(idx, 0);
    }

    #[test]
    fn nan_coordinate_does_not_panic_builder_or_knn_sort() {
        let tree = KdTree::build(&[[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        let _ = tree.k_nearest([0.0, 0.0, 0.0], 2);
        assert!(tree.nearest([f64::NAN, 0.0, 0.0]).is_some());
    }

    #[test]
    fn nearest_allows_infinite_squared_distance_from_overflow() {
        let tree = KdTree::build(&[[f64::MAX, 0.0, 0.0]]);
        assert_eq!(tree.nearest([0.0, 0.0, 0.0]), Some((0, f64::INFINITY)));
    }
}
