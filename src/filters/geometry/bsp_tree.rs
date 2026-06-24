//! BSP (Binary Space Partition) tree for spatial cell location.
//!
//! Used for fast point-in-cell queries in flow field interpolation.

use crate::data::PolyData;

/// A BSP tree node.
#[derive(Debug)]
enum BspNode {
    Leaf {
        cell_indices: Vec<usize>,
    },
    Interior {
        axis: usize, // 0=X, 1=Y, 2=Z
        split: f64,
        left: Box<BspNode>,
        right: Box<BspNode>,
    },
}

/// BSP tree for fast cell location.
pub struct BspTree {
    root: BspNode,
    _max_cells_per_leaf: usize,
}

impl BspTree {
    /// Build a BSP tree from a triangle mesh.
    pub fn build(mesh: &PolyData, max_cells_per_leaf: usize) -> Self {
        let n_cells = mesh.polys.num_cells();
        let indices: Vec<usize> = (0..n_cells).collect();

        // Precompute cell centroids
        let centroids: Vec<[f64; 3]> = mesh
            .polys
            .iter()
            .map(|cell| {
                let mut c = [0.0; 3];
                for &pid in cell {
                    let p = mesh.points.get(pid as usize);
                    for j in 0..3 {
                        c[j] += p[j];
                    }
                }
                let n = cell.len() as f64;
                [c[0] / n, c[1] / n, c[2] / n]
            })
            .collect();

        let root = Self::build_recursive(&indices, &centroids, max_cells_per_leaf, 0);

        Self {
            root,
            _max_cells_per_leaf: max_cells_per_leaf,
        }
    }

    /// Find cells near a query point.
    ///
    /// Returns indices of cells in the leaf node containing the query point.
    pub fn find_cells(&self, point: [f64; 3]) -> &[usize] {
        Self::find_recursive(&self.root, point)
    }

    /// Count the total number of leaf nodes.
    pub fn num_leaves(&self) -> usize {
        Self::count_leaves(&self.root)
    }

    /// Maximum depth of the tree.
    pub fn max_depth(&self) -> usize {
        Self::depth(&self.root)
    }

    fn build_recursive(
        indices: &[usize],
        centroids: &[[f64; 3]],
        max_per_leaf: usize,
        depth: usize,
    ) -> BspNode {
        if indices.len() <= max_per_leaf || depth > 30 {
            return BspNode::Leaf {
                cell_indices: indices.to_vec(),
            };
        }

        // Choose split axis (cycle through X, Y, Z)
        let axis = depth % 3;

        // Find median along axis
        let mut values: Vec<f64> = indices.iter().map(|&i| centroids[i][axis]).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let split = values[values.len() / 2];

        let mut left_idx = Vec::new();
        let mut right_idx = Vec::new();
        for &i in indices {
            if centroids[i][axis] <= split {
                left_idx.push(i);
            } else {
                right_idx.push(i);
            }
        }

        // Avoid infinite recursion if all centroids are at the same position
        if left_idx.is_empty() || right_idx.is_empty() {
            return BspNode::Leaf {
                cell_indices: indices.to_vec(),
            };
        }

        BspNode::Interior {
            axis,
            split,
            left: Box::new(Self::build_recursive(
                &left_idx,
                centroids,
                max_per_leaf,
                depth + 1,
            )),
            right: Box::new(Self::build_recursive(
                &right_idx,
                centroids,
                max_per_leaf,
                depth + 1,
            )),
        }
    }

    fn find_recursive(node: &BspNode, point: [f64; 3]) -> &[usize] {
        match node {
            BspNode::Leaf { cell_indices } => cell_indices,
            BspNode::Interior {
                axis,
                split,
                left,
                right,
            } => {
                if point[*axis] <= *split {
                    Self::find_recursive(left, point)
                } else {
                    Self::find_recursive(right, point)
                }
            }
        }
    }

    fn count_leaves(node: &BspNode) -> usize {
        match node {
            BspNode::Leaf { .. } => 1,
            BspNode::Interior { left, right, .. } => {
                Self::count_leaves(left) + Self::count_leaves(right)
            }
        }
    }

    fn depth(node: &BspNode) -> usize {
        match node {
            BspNode::Leaf { .. } => 0,
            BspNode::Interior { left, right, .. } => 1 + Self::depth(left).max(Self::depth(right)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> PolyData {
        let mut pts = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn build_and_query() {
        let mesh = make_grid();
        let tree = BspTree::build(&mesh, 4);
        let cells = tree.find_cells([2.0, 2.0, 0.0]);
        assert!(!cells.is_empty());
    }

    #[test]
    fn tree_structure() {
        let mesh = make_grid();
        let tree = BspTree::build(&mesh, 2);
        assert!(tree.num_leaves() > 1);
        assert!(tree.max_depth() > 0);
    }

    #[test]
    fn single_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let tree = BspTree::build(&mesh, 10);
        assert_eq!(tree.num_leaves(), 1);
        assert_eq!(tree.find_cells([0.3, 0.3, 0.0]).len(), 1);
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::new();
        let tree = BspTree::build(&mesh, 4);
        assert_eq!(tree.num_leaves(), 1);
        assert_eq!(tree.find_cells([0.0, 0.0, 0.0]).len(), 0);
    }
}
