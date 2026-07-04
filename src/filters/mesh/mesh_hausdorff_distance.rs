//! Compute one-sided and symmetric Hausdorff distance between two meshes.
use crate::data::PolyData;

const NONE: usize = 0;

struct KdNode {
    point_idx: usize,
    split_axis: u8,
    left: usize,
    right: usize,
}

struct KdTree {
    nodes: Vec<KdNode>,
    points: Vec<[f64; 3]>,
}

impl KdTree {
    fn build(points: Vec<[f64; 3]>) -> Self {
        let n = points.len();
        if n == 0 {
            return KdTree {
                nodes: Vec::new(),
                points,
            };
        }
        let mut indices: Vec<usize> = (0..n).collect();
        let mut nodes = Vec::with_capacity(n);
        nodes.push(KdNode {
            point_idx: 0,
            split_axis: 0,
            left: NONE,
            right: NONE,
        });
        Self::build_recursive(&points, &mut indices, 0, n, &mut nodes);
        KdTree { nodes, points }
    }

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

        let mid = lo + (hi - lo) / 2;
        indices[lo..hi].select_nth_unstable_by(mid - lo, |&a, &b| {
            points[a][axis as usize]
                .partial_cmp(&points[b][axis as usize])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let median_idx = indices[mid];

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

    fn nearest_sq(&self, query: [f64; 3]) -> f64 {
        if self.nodes.len() <= 1 {
            return f64::MAX;
        }
        let mut best = f64::MAX;
        self.search(1, query, &mut best);
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
        let (first, second) = if diff <= 0.0 {
            (node.left, node.right)
        } else {
            (node.right, node.left)
        };

        self.search(first, query, best);
        if diff2 < *best {
            self.search(second, query, best);
        }
    }
}

pub fn hausdorff_distance(mesh_a: &PolyData, mesh_b: &PolyData) -> (f64, f64, f64) {
    let points_a = extract_points(mesh_a);
    let points_b = extract_points(mesh_b);

    let tree_b = KdTree::build(points_b.clone());
    let tree_a = KdTree::build(points_a.clone());
    let relative_distance_a_to_b = directed_hausdorff_kd(&points_a, &tree_b);
    let relative_distance_b_to_a = directed_hausdorff_kd(&points_b, &tree_a);

    (
        relative_distance_a_to_b.max(relative_distance_b_to_a),
        relative_distance_a_to_b,
        relative_distance_b_to_a,
    )
}

fn extract_points(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push(mesh.points.get(i));
    }
    out
}

fn directed_hausdorff_kd(queries: &[[f64; 3]], tree: &KdTree) -> f64 {
    if queries.is_empty() || tree.nodes.len() <= 1 {
        return 0.0;
    }

    let mut max_d = 0.0f64;
    for &query in queries {
        let d = tree.nearest_sq(query).sqrt();
        if d > max_d {
            max_d = d;
        }
    }
    max_d
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hausdorff() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let (hausdorff, dab, dba) = hausdorff_distance(&a, &b);
        assert!((hausdorff - 1.0).abs() < 0.01);
        assert!((dab - 1.0).abs() < 0.01);
        assert!((dba - 1.0).abs() < 0.01);
    }
}
