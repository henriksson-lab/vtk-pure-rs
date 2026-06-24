use crate::data::PolyData;

// ---------------------------------------------------------------------------
// Internal 3D kd-tree for nearest-neighbor queries
// ---------------------------------------------------------------------------

const NONE: usize = 0; // sentinel – node index 0 is never used as a child

struct KdNode {
    point_idx: usize,
    split_axis: u8,
    left: usize, // index into KdTree::nodes, NONE = no child
    right: usize,
}

struct KdTree {
    nodes: Vec<KdNode>,
    points: Vec<[f64; 3]>,
}

impl KdTree {
    /// Build a kd-tree from a slice of 3-D points.
    fn build(points: Vec<[f64; 3]>) -> Self {
        let n = points.len();
        if n == 0 {
            return KdTree {
                nodes: Vec::new(),
                points,
            };
        }
        let mut indices: Vec<usize> = (0..n).collect();
        // Pre-allocate; exact size equals n but we push as we go.
        let mut nodes = Vec::with_capacity(n);
        // Push a dummy node at index 0 so NONE (0) is never a real node.
        nodes.push(KdNode {
            point_idx: 0,
            split_axis: 0,
            left: NONE,
            right: NONE,
        });
        Self::build_recursive(&points, &mut indices, 0, n, &mut nodes);
        KdTree { nodes, points }
    }

    /// Recursively partition `indices[lo..hi]` and append nodes.
    /// Returns the index of the created node in `nodes`.
    fn build_recursive(
        points: &[[f64; 3]],
        indices: &mut [usize],
        lo: usize,
        hi: usize,
        nodes: &mut Vec<KdNode>,
    ) -> usize {
        if lo >= hi {
            return NONE;
        }
        // Choose split axis = axis with largest spread in this subset
        let axis = {
            let mut best_axis = 0u8;
            let mut best_spread = -1.0f64;
            for ax in 0..3u8 {
                let mut mn = f64::MAX;
                let mut mx = f64::MIN;
                for &idx in &indices[lo..hi] {
                    let v = points[idx][ax as usize];
                    if v < mn {
                        mn = v;
                    }
                    if v > mx {
                        mx = v;
                    }
                }
                let spread = mx - mn;
                if spread > best_spread {
                    best_spread = spread;
                    best_axis = ax;
                }
            }
            best_axis
        };

        // Partition around median
        let mid = lo + (hi - lo) / 2;
        // Use select_nth_unstable to get the median in O(n)
        indices[lo..hi].select_nth_unstable_by(mid - lo, |&a, &b| {
            points[a][axis as usize]
                .partial_cmp(&points[b][axis as usize])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let median_idx = indices[mid];

        // Reserve a slot for this node
        let node_pos = nodes.len();
        nodes.push(KdNode {
            point_idx: median_idx,
            split_axis: axis,
            left: NONE,
            right: NONE,
        });

        let left = Self::build_recursive(points, indices, lo, mid, nodes);
        let right = Self::build_recursive(points, indices, mid + 1, hi, nodes);
        nodes[node_pos].left = left;
        nodes[node_pos].right = right;
        node_pos
    }

    /// Find the squared distance from `query` to the nearest point in the tree.
    fn nearest_sq(&self, query: [f64; 3]) -> f64 {
        if self.nodes.len() <= 1 {
            // Only the dummy node exists – no real points.
            return f64::MAX;
        }
        let mut best = f64::MAX;
        self.search(1, query, &mut best); // root is at index 1
        best
    }

    fn search(&self, node_idx: usize, query: [f64; 3], best: &mut f64) {
        if node_idx == NONE {
            return;
        }
        let node = &self.nodes[node_idx];
        let p = self.points[node.point_idx];
        let dx = query[0] - p[0];
        let dy = query[1] - p[1];
        let dz = query[2] - p[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        if d2 < *best {
            *best = d2;
        }

        let axis = node.split_axis as usize;
        let diff = query[axis] - p[axis];
        let diff2 = diff * diff;

        // Visit the side of the split that contains the query first.
        let (first, second) = if diff <= 0.0 {
            (node.left, node.right)
        } else {
            (node.right, node.left)
        };

        self.search(first, query, best);

        // Only visit the other side if the splitting plane is closer than the
        // current best distance.
        if diff2 < *best {
            self.search(second, query, best);
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the Hausdorff distance between two point sets.
///
/// The Hausdorff distance is `max(d(A→B), d(B→A))` where
/// `d(A→B) = max over a in A of (min over b in B of dist(a, b))`.
///
/// Returns `(hausdorff_distance, mean_distance_a_to_b, mean_distance_b_to_a)`.
pub fn hausdorff_distance(a: &PolyData, b: &PolyData) -> (f64, f64, f64) {
    let pts_a = extract_points(a);
    let pts_b = extract_points(b);

    let tree_b = KdTree::build(pts_b.clone());
    let tree_a = KdTree::build(pts_a.clone());

    let (max_ab, mean_ab) = directed_hausdorff_kd(&pts_a, &tree_b);
    let (max_ba, mean_ba) = directed_hausdorff_kd(&pts_b, &tree_a);
    (max_ab.max(max_ba), mean_ab, mean_ba)
}

fn extract_points(pd: &PolyData) -> Vec<[f64; 3]> {
    let n = pd.points.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(pd.points.get(i));
    }
    out
}

/// Directed Hausdorff from a set of query points to the points stored in `tree`.
fn directed_hausdorff_kd(queries: &[[f64; 3]], tree: &KdTree) -> (f64, f64) {
    let n = queries.len();
    if n == 0 || tree.nodes.len() <= 1 {
        return (0.0, 0.0);
    }

    let mut max_d = 0.0f64;
    let mut sum_d = 0.0f64;

    for &q in queries {
        let d = tree.nearest_sq(q).sqrt();
        if d > max_d {
            max_d = d;
        }
        sum_d += d;
    }

    (max_d, sum_d / n as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_sets() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        let (h, _, _) = hausdorff_distance(&pd, &pd);
        assert!(h < 1e-10);
    }

    #[test]
    fn known_distance() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([3.0, 4.0, 0.0]);

        let (h, _, _) = hausdorff_distance(&a, &b);
        assert!((h - 5.0).abs() < 1e-10);
    }

    #[test]
    fn asymmetric_sets() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);

        let mut b = PolyData::new();
        b.points.push([0.0, 0.0, 0.0]);
        b.points.push([1.0, 0.0, 0.0]);
        b.points.push([5.0, 0.0, 0.0]); // extra far point

        let (h, mean_ab, mean_ba) = hausdorff_distance(&a, &b);
        // Hausdorff should be driven by b→a: point (5,0,0) is 4 from nearest in a
        assert!((h - 4.0).abs() < 1e-10);
        assert!(mean_ab < mean_ba); // a→b is closer on average
    }
}
