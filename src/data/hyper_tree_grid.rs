use crate::data::{DataSetAttributes, FieldData};
use crate::types::BoundingBox;

/// A node in a hyper tree (octree/quadtree cell).
#[derive(Debug, Clone)]
struct HyperTreeNode {
    /// Whether this node is a leaf.
    is_leaf: bool,
    /// Index of the first child (children are stored contiguously).
    /// For a 3D tree: 8 children at indices first_child..first_child+8.
    first_child: usize,
    /// Global cell index for leaf nodes (used to look up cell data).
    global_id: usize,
}

/// A single hyper tree (recursive octree/quadtree).
#[derive(Debug, Clone)]
pub struct HyperTree {
    nodes: Vec<HyperTreeNode>,
    /// Number of children per refined node: branch_factor^dimension.
    number_of_children: usize,
}

/// A leaf cell in a [`HyperTreeGrid`] with its global cell id and spatial bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyperTreeLeaf {
    pub coarse_index: [usize; 3],
    pub node_index: usize,
    pub global_id: usize,
    pub bounds: BoundingBox,
    pub depth: usize,
}

#[allow(dead_code)]
impl HyperTree {
    fn new(number_of_children: usize) -> Self {
        Self {
            nodes: vec![HyperTreeNode {
                is_leaf: true,
                first_child: 0,
                global_id: 0,
            }],
            number_of_children,
        }
    }

    /// Subdivide a leaf node, creating `number_of_children` children.
    /// Returns the index of the first child.
    fn subdivide(&mut self, node_idx: usize, next_global_id: &mut usize) -> usize {
        let first = self.nodes.len();
        self.nodes[node_idx].is_leaf = false;
        self.nodes[node_idx].first_child = first;

        for _ in 0..self.number_of_children {
            self.nodes.push(HyperTreeNode {
                is_leaf: true,
                first_child: 0,
                global_id: *next_global_id,
            });
            *next_global_id += 1;
        }
        first
    }

    /// Number of nodes in this tree.
    fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Number of leaf nodes.
    fn num_leaves(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_leaf).count()
    }

    /// Maximum depth of the tree.
    fn max_depth(&self) -> usize {
        self.depth_recursive(0)
    }

    fn depth_recursive(&self, node_idx: usize) -> usize {
        let node = &self.nodes[node_idx];
        if node.is_leaf {
            return 0;
        }
        let mut max_d = 0;
        for c in 0..self.number_of_children {
            let child_idx = node.first_child + c;
            if child_idx < self.nodes.len() {
                max_d = max_d.max(self.depth_recursive(child_idx));
            }
        }
        1 + max_d
    }
}

/// AMR-style hierarchical grid.
///
/// Analogous to VTK's `vtkHyperTreeGrid`. Stores a coarse grid where each
/// cell can be recursively subdivided into an octree (3D) or quadtree (2D).
///
/// This allows adaptive mesh refinement (AMR) with different resolution
/// in different parts of the domain.
#[derive(Debug, Clone)]
pub struct HyperTreeGrid {
    /// Number of dimensions (2 or 3).
    dimension: usize,
    /// Point dimensions [ni, nj, nk], matching vtkHyperTreeGrid::Dimensions.
    dimensions: [usize; 3],
    /// Root-cell dimensions derived from dimensions, matching vtkHyperTreeGrid::CellDims.
    cell_dims: [usize; 3],
    /// Subdivision factor in each active dimension.
    branch_factor: usize,
    /// Number of children per refined node.
    number_of_children: usize,
    /// Origin of the grid.
    origin: [f64; 3],
    /// Grid spacing for the coarse level.
    spacing: [f64; 3],
    /// One hyper tree per coarse cell.
    trees: Vec<Option<HyperTree>>,
    /// Global cell data (indexed by global_id across all trees).
    cell_data: DataSetAttributes,
    /// Field data.
    field_data: FieldData,
    /// Next available global cell ID.
    next_global_id: usize,
}

impl HyperTreeGrid {
    /// Create a new HyperTreeGrid.
    ///
    /// `dimensions` are rectilinear point dimensions [ni, nj, nk].
    /// Root-cell dimensions are `dimensions - 1`, except singleton axes keep one cell.
    pub fn new(dimensions: [usize; 3], origin: [f64; 3], spacing: [f64; 3]) -> Self {
        let (dimension, cell_dims) = Self::dimensions_to_cell_dims(dimensions);
        let branch_factor = 2;
        let number_of_children = Self::compute_number_of_children(branch_factor, dimension);
        let n_cells = cell_dims[0] * cell_dims[1] * cell_dims[2];
        Self {
            dimension,
            dimensions,
            cell_dims,
            branch_factor,
            number_of_children,
            origin,
            spacing,
            trees: vec![None; n_cells],
            cell_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
            next_global_id: 0,
        }
    }

    /// Initialize a tree at coarse cell (i, j, k).
    /// Returns the global cell ID assigned to the root leaf.
    pub fn init_tree(&mut self, i: usize, j: usize, k: usize) -> usize {
        self.try_init_tree(i, j, k)
            .expect("coarse cell index out of bounds for HyperTreeGrid")
    }

    /// Initialize a tree at coarse cell (i, j, k), returning `None` if out of bounds.
    pub fn try_init_tree(&mut self, i: usize, j: usize, k: usize) -> Option<usize> {
        let idx = self.coarse_index(i, j, k)?;
        if let Some(tree) = &self.trees[idx] {
            return Some(tree.nodes[0].global_id);
        }
        let gid = self.next_global_id;
        self.next_global_id += 1;
        let mut tree = HyperTree::new(self.number_of_children);
        tree.nodes[0].global_id = gid;
        self.trees[idx] = Some(tree);
        Some(gid)
    }

    /// Subdivide a leaf node in the tree at coarse cell (i, j, k).
    ///
    /// `node_index` is the index within the tree's node array (0 = root).
    /// Returns the index of the first child node.
    pub fn subdivide(&mut self, i: usize, j: usize, k: usize, node_index: usize) -> Option<usize> {
        let idx = self.coarse_index(i, j, k)?;
        let tree = self.trees[idx].as_mut()?;
        let node = tree.nodes.get(node_index)?;
        if !node.is_leaf {
            return None;
        }
        Some(tree.subdivide(node_index, &mut self.next_global_id))
    }

    /// Coarse grid dimensions.
    pub fn grid_size(&self) -> [usize; 3] {
        self.cell_dims
    }

    /// Point dimensions, matching VTK's `GetDimensions`.
    pub fn dimensions(&self) -> [usize; 3] {
        self.dimensions
    }

    /// Root-cell dimensions, matching VTK's `GetCellDims`.
    pub fn cell_dims(&self) -> [usize; 3] {
        self.cell_dims
    }

    /// Set point dimensions and resize the root tree slots.
    pub fn set_dimensions(&mut self, dimensions: [usize; 3]) {
        let (dimension, cell_dims) = Self::dimensions_to_cell_dims(dimensions);
        self.dimension = dimension;
        self.dimensions = dimensions;
        self.cell_dims = cell_dims;
        self.number_of_children =
            Self::compute_number_of_children(self.branch_factor, self.dimension);
        self.trees.resize_with(self.max_number_of_trees(), || None);
        self.trees.truncate(self.max_number_of_trees());
    }

    /// Subdivision factor in the grid refinement scheme.
    pub fn branch_factor(&self) -> usize {
        self.branch_factor
    }

    /// Set the subdivision factor. VTK accepts factors 2 and 3.
    pub fn set_branch_factor(&mut self, factor: usize) -> bool {
        if !(2..=3).contains(&factor) {
            return false;
        }
        self.branch_factor = factor;
        self.number_of_children = Self::compute_number_of_children(factor, self.dimension);
        true
    }

    /// Number of children each refined node has.
    pub fn number_of_children(&self) -> usize {
        self.number_of_children
    }

    /// Number of dimensions (2 or 3).
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Number of coarse cells.
    pub fn num_coarse_cells(&self) -> usize {
        self.max_number_of_trees()
    }

    /// Maximum number of trees in the level-0 grid.
    pub fn max_number_of_trees(&self) -> usize {
        self.cell_dims[0] * self.cell_dims[1] * self.cell_dims[2]
    }

    /// Total number of nodes across all trees, matching VTK's `GetNumberOfCells`.
    pub fn num_cells(&self) -> usize {
        self.trees
            .iter()
            .filter_map(|t| t.as_ref())
            .map(|t| t.num_nodes())
            .sum()
    }

    /// Total number of leaf cells across all trees.
    pub fn num_leaves(&self) -> usize {
        self.trees
            .iter()
            .filter_map(|t| t.as_ref())
            .map(|t| t.num_leaves())
            .sum()
    }

    /// Maximum refinement depth across all trees.
    pub fn max_depth(&self) -> usize {
        self.trees
            .iter()
            .filter_map(|t| t.as_ref())
            .map(|t| t.max_depth())
            .max()
            .unwrap_or(0)
    }

    /// Number of initialized trees.
    pub fn num_trees(&self) -> usize {
        self.trees.iter().filter(|t| t.is_some()).count()
    }

    /// Return all active leaf cells with their refined spatial bounds.
    pub fn leaves(&self) -> Vec<HyperTreeLeaf> {
        let mut leaves = Vec::new();
        for k in 0..self.cell_dims[2] {
            for j in 0..self.cell_dims[1] {
                for i in 0..self.cell_dims[0] {
                    let Some(idx) = self.coarse_index(i, j, k) else {
                        continue;
                    };
                    let Some(tree) = &self.trees[idx] else {
                        continue;
                    };
                    let bounds = self.coarse_cell_bounds(i, j, k);
                    self.collect_leaves(tree, 0, [i, j, k], bounds, 0, &mut leaves);
                }
            }
        }
        leaves
    }

    /// Return active leaves from one initialized coarse tree.
    pub fn tree_leaves(&self, i: usize, j: usize, k: usize) -> Option<Vec<HyperTreeLeaf>> {
        let idx = self.coarse_index(i, j, k)?;
        let tree = self.trees[idx].as_ref()?;
        let mut leaves = Vec::new();
        let bounds = self.coarse_cell_bounds(i, j, k);
        self.collect_leaves(tree, 0, [i, j, k], bounds, 0, &mut leaves);
        Some(leaves)
    }

    /// Compute bounding box of the initialized hyper tree geometry.
    pub fn bounds(&self) -> BoundingBox {
        let mut bounds = BoundingBox::empty();
        for leaf in self.leaves() {
            bounds = bounds.union(&leaf.bounds);
        }
        bounds
    }

    /// Compute bounding box of the full level-0 grid.
    pub fn grid_bounds(&self) -> BoundingBox {
        let far = [
            self.origin[0] + self.cell_dims[0] as f64 * self.spacing[0],
            self.origin[1] + self.cell_dims[1] as f64 * self.spacing[1],
            self.origin[2] + self.cell_dims[2] as f64 * self.spacing[2],
        ];
        let mut bounds = BoundingBox::empty();
        bounds.expand(self.origin);
        bounds.expand(far);
        bounds
    }

    pub fn cell_data(&self) -> &DataSetAttributes {
        &self.cell_data
    }

    pub fn cell_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.cell_data
    }

    pub fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    pub fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }

    fn coarse_index(&self, i: usize, j: usize, k: usize) -> Option<usize> {
        if i >= self.cell_dims[0] || j >= self.cell_dims[1] || k >= self.cell_dims[2] {
            return None;
        }
        let plane = self.cell_dims[0].checked_mul(self.cell_dims[1])?;
        k.checked_mul(plane)?
            .checked_add(j.checked_mul(self.cell_dims[0])?)?
            .checked_add(i)
    }

    fn coarse_cell_bounds(&self, i: usize, j: usize, k: usize) -> BoundingBox {
        let p0 = [
            self.origin[0] + i as f64 * self.spacing[0],
            self.origin[1] + j as f64 * self.spacing[1],
            self.origin[2] + k as f64 * self.spacing[2],
        ];
        let p1 = [
            p0[0] + self.spacing[0],
            p0[1] + self.spacing[1],
            p0[2] + self.spacing[2],
        ];
        let mut bounds = BoundingBox::empty();
        bounds.expand(p0);
        bounds.expand(p1);
        bounds
    }

    fn collect_leaves(
        &self,
        tree: &HyperTree,
        node_index: usize,
        coarse_index: [usize; 3],
        bounds: BoundingBox,
        depth: usize,
        leaves: &mut Vec<HyperTreeLeaf>,
    ) {
        let Some(node) = tree.nodes.get(node_index) else {
            return;
        };
        if node.is_leaf {
            leaves.push(HyperTreeLeaf {
                coarse_index,
                node_index,
                global_id: node.global_id,
                bounds,
                depth,
            });
            return;
        }

        for child in 0..tree.number_of_children {
            let child_index = node.first_child + child;
            if child_index >= tree.nodes.len() {
                continue;
            }
            let child_bounds = self.child_bounds(bounds, child);
            self.collect_leaves(
                tree,
                child_index,
                coarse_index,
                child_bounds,
                depth + 1,
                leaves,
            );
        }
    }

    fn child_bounds(&self, bounds: BoundingBox, child: usize) -> BoundingBox {
        let factor = self.branch_factor;
        let ix = child % factor;
        let iy = (child / factor) % factor;
        let iz = (child / (factor * factor)) % factor;
        let split = |min: f64, max: f64, idx: usize| {
            let step = (max - min) / factor as f64;
            (min + idx as f64 * step, min + (idx + 1) as f64 * step)
        };
        let (x0, x1) = split(bounds.x_min, bounds.x_max, ix);
        let (y0, y1) = if self.dimension >= 2 {
            split(bounds.y_min, bounds.y_max, iy)
        } else {
            (bounds.y_min, bounds.y_max)
        };
        let (z0, z1) = if self.dimension >= 3 {
            split(bounds.z_min, bounds.z_max, iz)
        } else {
            (bounds.z_min, bounds.z_max)
        };
        let mut child_bounds = BoundingBox::empty();
        child_bounds.expand([x0, y0, z0]);
        child_bounds.expand([x1, y1, z1]);
        child_bounds
    }

    fn dimensions_to_cell_dims(dimensions: [usize; 3]) -> (usize, [usize; 3]) {
        let mut dimension = 0;
        let mut cell_dims = [1; 3];
        for axis in 0..3 {
            if dimensions[axis] > 1 {
                cell_dims[axis] = dimensions[axis] - 1;
                dimension += 1;
            }
        }
        (dimension, cell_dims)
    }

    fn compute_number_of_children(branch_factor: usize, dimension: usize) -> usize {
        (0..dimension).fold(1, |num, _| num * branch_factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_2d_grid() {
        let htg = HyperTreeGrid::new([4, 4, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(htg.dimension(), 2);
        assert_eq!(htg.dimensions(), [4, 4, 1]);
        assert_eq!(htg.cell_dims(), [3, 3, 1]);
        assert_eq!(htg.num_coarse_cells(), 9);
        assert_eq!(htg.num_cells(), 0);
    }

    #[test]
    fn basic_3d_grid() {
        let htg = HyperTreeGrid::new([2, 2, 2], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(htg.dimension(), 3);
        assert_eq!(htg.num_coarse_cells(), 1);
    }

    #[test]
    fn init_and_subdivide() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

        let gid = htg.init_tree(0, 0, 0);
        assert_eq!(gid, 0);
        assert_eq!(htg.num_cells(), 1);
        assert_eq!(htg.num_leaves(), 1);

        // Subdivide root: creates 4 children (2D quadtree)
        let first_child = htg.subdivide(0, 0, 0, 0).unwrap();
        assert_eq!(first_child, 1); // first child node index
        assert_eq!(htg.num_cells(), 5); // root plus 4 children
        assert_eq!(htg.num_leaves(), 4); // root no longer leaf, 4 active leaves

        assert_eq!(htg.max_depth(), 1);
    }

    #[test]
    fn multi_level_refinement() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        htg.init_tree(0, 0, 0);

        // Level 1
        htg.subdivide(0, 0, 0, 0).unwrap();
        assert_eq!(htg.max_depth(), 1);

        // Level 2: subdivide first child (node index 1)
        htg.subdivide(0, 0, 0, 1).unwrap();
        assert_eq!(htg.max_depth(), 2);
    }

    #[test]
    fn bounds_follow_initialized_tree_geometry() {
        let mut htg = HyperTreeGrid::new([4, 3, 2], [1.0, 2.0, 3.0], [0.5, 0.5, 0.5]);
        assert!(htg.bounds().is_empty());
        htg.init_tree(1, 0, 0);
        let bb = htg.bounds();
        assert_eq!(bb.x_min, 1.5);
        assert_eq!(bb.x_max, 2.0);
        assert_eq!(bb.y_min, 2.0);
        assert_eq!(bb.y_max, 2.5);
    }

    #[test]
    fn grid_bounds_cover_full_level_zero_grid() {
        let htg = HyperTreeGrid::new([4, 3, 2], [1.0, 2.0, 3.0], [0.5, 0.5, 0.5]);
        let bb = htg.grid_bounds();
        assert_eq!(bb.x_min, 1.0);
        assert_eq!(bb.x_max, 2.5);
        assert_eq!(bb.y_min, 2.0);
        assert_eq!(bb.y_max, 3.0);
    }

    #[test]
    fn out_of_bounds_tree_access_returns_none() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert_eq!(htg.try_init_tree(2, 0, 0), None);
        assert_eq!(htg.subdivide(2, 0, 0, 0), None);
        htg.init_tree(0, 0, 0);
        assert_eq!(htg.subdivide(0, 0, 0, 10), None);
    }

    #[test]
    fn bounds_handle_negative_spacing() {
        let mut htg = HyperTreeGrid::new([2, 1, 1], [1.0, 0.0, 0.0], [-0.5, 1.0, 1.0]);
        htg.init_tree(0, 0, 0);
        let bb = htg.bounds();
        assert_eq!(bb.x_min, 0.5);
        assert_eq!(bb.x_max, 1.0);
    }

    #[test]
    fn leaves_report_refined_bounds_and_global_ids() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [2.0, 2.0, 1.0]);
        assert_eq!(htg.init_tree(0, 0, 0), 0);
        htg.subdivide(0, 0, 0, 0).unwrap();

        let leaves = htg.leaves();
        assert_eq!(leaves.len(), 4);
        assert!(leaves.iter().all(|leaf| leaf.depth == 1));
        assert_eq!(leaves[0].global_id, 1);
        assert_eq!(leaves[0].bounds.x_min, 0.0);
        assert_eq!(leaves[0].bounds.x_max, 1.0);
        assert_eq!(leaves[0].bounds.y_min, 0.0);
        assert_eq!(leaves[0].bounds.y_max, 1.0);
    }

    #[test]
    fn init_tree_is_idempotent_for_existing_cell() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [1.0; 3]);
        assert_eq!(htg.try_init_tree(0, 0, 0), Some(0));
        assert_eq!(htg.try_init_tree(0, 0, 0), Some(0));
        assert_eq!(htg.num_cells(), 1);
    }

    #[test]
    fn ternary_branch_factor_changes_number_of_children() {
        let mut htg = HyperTreeGrid::new([2, 2, 1], [0.0, 0.0, 0.0], [3.0, 3.0, 1.0]);
        assert!(htg.set_branch_factor(3));
        assert_eq!(htg.number_of_children(), 9);
        htg.init_tree(0, 0, 0);
        htg.subdivide(0, 0, 0, 0).unwrap();
        assert_eq!(htg.num_cells(), 10);
        assert_eq!(htg.num_leaves(), 9);
        assert_eq!(htg.leaves()[1].bounds.x_min, 1.0);
        assert_eq!(htg.leaves()[1].bounds.x_max, 2.0);
    }
}
