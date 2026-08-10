use crate::common::{
    core::{AnyArray, DoubleArray, IdList, VtkIdType},
    execution_model::{
        ScalarTree, ScalarTreeApi, ScalarTreeCellHandle, ScalarTreeDataSetHandle,
        ScalarTreeScalarsHandle,
    },
};

#[derive(Clone, Copy, Debug)]
struct ScalarRange {
    min: f64,
    max: f64,
}

impl Default for ScalarRange {
    fn default() -> Self {
        Self {
            min: f64::MAX,
            max: -f64::MAX,
        }
    }
}

/// VTK: `vtkSimpleScalarTree`.
#[derive(Debug)]
pub struct SimpleScalarTree {
    scalar_tree: ScalarTree,
    max_level: i32,
    level: i32,
    branching_factor: i32,
    tree: Vec<ScalarRange>,
    tree_size: VtkIdType,
    leaf_offset: VtkIdType,
    num_cells: VtkIdType,
    tree_index: VtkIdType,
    child_number: i32,
    cell_id: VtkIdType,
    candidate_cells: Vec<VtkIdType>,
    empty_batch: Vec<VtkIdType>,
}

impl SimpleScalarTree {
    /// VTK: `vtkSimpleScalarTree::New`.
    pub fn new() -> Self {
        Self {
            scalar_tree: ScalarTree::with_class_name("vtkSimpleScalarTree"),
            max_level: 20,
            level: 0,
            branching_factor: 3,
            tree: Vec::new(),
            tree_size: 0,
            leaf_offset: 0,
            num_cells: 0,
            tree_index: 0,
            child_number: 0,
            cell_id: 0,
            candidate_cells: Vec::new(),
            empty_batch: Vec::new(),
        }
    }

    /// VTK: `vtkSimpleScalarTree::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.scalar_tree.print_self();
        output.push_str("\nLevel: ");
        output.push_str(&self.get_level().to_string());
        output.push_str("\nMax Level: ");
        output.push_str(&self.get_max_level().to_string());
        output.push_str("\nBranching Factor: ");
        output.push_str(&self.get_branching_factor().to_string());
        output
    }

    /// VTK: `vtkSimpleScalarTree::ShallowCopy`.
    pub fn shallow_copy(&mut self, stree: &Self) {
        self.set_max_level(stree.get_max_level());
        self.set_branching_factor(stree.get_branching_factor());
        self.scalar_tree.shallow_copy(stree.scalar_tree());
    }

    /// VTK: `vtkSimpleScalarTree::SetBranchingFactor`.
    pub fn set_branching_factor(&mut self, branching_factor: i32) {
        let branching_factor = branching_factor.max(2);
        if self.branching_factor != branching_factor {
            self.branching_factor = branching_factor;
            self.modified();
        }
    }

    /// VTK: `vtkSimpleScalarTree::GetBranchingFactor`.
    pub fn get_branching_factor(&self) -> i32 {
        self.branching_factor
    }

    /// VTK: `vtkSimpleScalarTree::GetLevel`.
    pub fn get_level(&self) -> i32 {
        self.level
    }

    /// VTK: `vtkSimpleScalarTree::SetMaxLevel`.
    pub fn set_max_level(&mut self, max_level: i32) {
        let max_level = max_level.max(1);
        if self.max_level != max_level {
            self.max_level = max_level;
            self.modified();
        }
    }

    /// VTK: `vtkSimpleScalarTree::GetMaxLevel`.
    pub fn get_max_level(&self) -> i32 {
        self.max_level
    }

    /// VTK: `vtkScalarTree::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: Option<ScalarTreeDataSetHandle>) {
        self.scalar_tree.set_data_set(data_set);
    }

    /// VTK: `vtkScalarTree::GetDataSet`.
    pub fn get_data_set(&self) -> Option<ScalarTreeDataSetHandle> {
        self.scalar_tree.get_data_set()
    }

    /// VTK: `vtkScalarTree::SetScalars`.
    pub fn set_scalars(&mut self, scalars: Option<ScalarTreeScalarsHandle>) {
        self.scalar_tree.set_scalars(scalars);
    }

    /// VTK: `vtkScalarTree::GetScalars`.
    pub fn get_scalars(&self) -> Option<ScalarTreeScalarsHandle> {
        self.scalar_tree.get_scalars()
    }

    /// VTK: `vtkSimpleScalarTree::BuildTree`.
    pub fn build_tree(&mut self) {
        let Some(data_set) = self.get_data_set() else {
            self.num_cells = 0;
            return;
        };
        self.num_cells = data_set.get_number_of_cells();
        if self.num_cells < 1 {
            return;
        }

        let Some(scalars) = self.get_scalars() else {
            return;
        };

        self.initialize();

        let branching_factor = VtkIdType::from(self.branching_factor);
        let mut num_leafs = div_ceil(self.num_cells, branching_factor);
        let mut prod = 1;
        let mut num_nodes = 1;
        self.level = 0;
        while prod < num_leafs && self.level <= self.max_level {
            prod *= branching_factor;
            num_nodes += prod;
            self.level += 1;
        }

        self.leaf_offset = num_nodes - prod;
        self.tree_size = num_nodes - (prod - num_leafs);
        self.tree = vec![ScalarRange::default(); vtk_id_to_usize(self.tree_size)];

        let mut cell_id = 0;
        for node in 0..num_leafs {
            let tree_index = vtk_id_to_usize(self.leaf_offset + node);
            for _ in 0..self.branching_factor {
                if cell_id >= self.num_cells {
                    break;
                }
                if let Some(range) = scalar_range_for_cell(&data_set, &scalars, cell_id) {
                    self.tree[tree_index].min = self.tree[tree_index].min.min(range.min);
                    self.tree[tree_index].max = self.tree[tree_index].max.max(range.max);
                }
                cell_id += 1;
            }
        }

        let mut offset = self.leaf_offset;
        for _ in (1..=self.level).rev() {
            let parent_offset = offset - prod / branching_factor;
            prod /= branching_factor;
            let num_parent_leafs = div_ceil(num_leafs, branching_factor);
            let mut leaf = 0;
            for node in 0..num_parent_leafs {
                let parent_index = vtk_id_to_usize(parent_offset + node);
                for _ in 0..self.branching_factor {
                    if leaf >= num_leafs {
                        break;
                    }
                    let child = self.tree[vtk_id_to_usize(offset + leaf)];
                    self.tree[parent_index].min = self.tree[parent_index].min.min(child.min);
                    self.tree[parent_index].max = self.tree[parent_index].max.max(child.max);
                    leaf += 1;
                }
            }
            num_leafs = num_parent_leafs;
            offset = parent_offset;
        }

        self.scalar_tree.build_time_mut().modified();
    }

    /// VTK: `vtkSimpleScalarTree::Initialize`.
    pub fn initialize(&mut self) {
        self.tree.clear();
        self.tree_size = 0;
        self.leaf_offset = 0;
    }

    /// VTK: `vtkSimpleScalarTree::InitTraversal`.
    pub fn init_traversal(&mut self, scalar_value: f64) {
        self.build_tree();
        self.scalar_tree.set_scalar_value(scalar_value);
        self.tree_index = self.tree_size;

        if self.tree.is_empty()
            || self.tree[0].min > scalar_value
            || self.tree[0].max < scalar_value
        {
            return;
        }

        self.find_start_leaf(0, 0);
    }

    /// VTK: `vtkSimpleScalarTree::GetNextCell`.
    pub fn get_next_cell(
        &mut self,
        cell_id: &mut VtkIdType,
        pt_ids: &mut Option<IdList>,
        cell_scalars: &mut AnyArray,
    ) -> Option<ScalarTreeCellHandle> {
        let data_set = self.get_data_set()?;
        let scalars = self.get_scalars()?;
        let mut min = f64::MAX;
        let mut max = -f64::MAX;

        while self.tree_index < self.tree_size {
            while self.child_number < self.branching_factor && self.cell_id < self.num_cells {
                let current_cell_id = self.cell_id;
                let point_ids = data_set.get_cell_points(current_cell_id);
                if scalars.copy_tuples(&point_ids, cell_scalars) {
                    for i in 0..point_ids.get_number_of_ids() {
                        if let Some(s) = scalars.get_tuple1(point_ids.get_id(i)) {
                            min = min.min(s);
                            max = max.max(s);
                        }
                    }
                }
                if self.scalar_tree.get_scalar_value() >= min
                    && self.scalar_tree.get_scalar_value() <= max
                {
                    *cell_id = current_cell_id;
                    *pt_ids = Some(point_ids);
                    self.child_number += 1;
                    self.cell_id += 1;
                    return Some(data_set.get_cell_handle(current_cell_id));
                }
                self.child_number += 1;
                self.cell_id += 1;
            }

            self.find_next_leaf(self.tree_index, self.level);
        }

        None
    }

    /// VTK: `vtkSimpleScalarTree::GetNumberOfCellBatches`.
    pub fn get_number_of_cell_batches(&mut self, scalar_value: f64) -> VtkIdType {
        self.build_tree();
        self.scalar_tree.set_scalar_value(scalar_value);
        self.tree_index = self.tree_size;

        if self.tree.is_empty()
            || self.tree[0].min > scalar_value
            || self.tree[0].max < scalar_value
        {
            return 0;
        }

        self.candidate_cells.clear();
        if self.num_cells < 1 {
            return 0;
        }

        while self.tree_index < self.tree_size {
            while self.child_number < self.branching_factor && self.cell_id < self.num_cells {
                self.candidate_cells.push(self.cell_id);
                self.child_number += 1;
                self.cell_id += 1;
            }
            self.find_next_leaf(self.tree_index, self.level);
        }

        if self.candidate_cells.is_empty() {
            0
        } else {
            div_ceil(
                VtkIdType::try_from(self.candidate_cells.len()).expect("candidate count fits"),
                VtkIdType::from(self.branching_factor),
            )
        }
    }

    /// VTK: `vtkSimpleScalarTree::GetCellBatch`.
    pub fn get_cell_batch(&mut self, batch_num: VtkIdType) -> &[VtkIdType] {
        let pos = batch_num * VtkIdType::from(self.branching_factor);
        if self.num_cells < 1 || self.candidate_cells.is_empty() || pos > self.num_cells {
            return &self.empty_batch;
        }

        let pos = vtk_id_to_usize(pos);
        if pos >= self.candidate_cells.len() {
            return &self.empty_batch;
        }
        let remaining = self.candidate_cells.len() - pos;
        let count = remaining.min(self.branching_factor as usize);
        &self.candidate_cells[pos..pos + count]
    }

    pub(crate) fn scalar_tree(&self) -> &ScalarTree {
        &self.scalar_tree
    }

    pub(crate) fn scalar_tree_mut(&mut self) -> &mut ScalarTree {
        &mut self.scalar_tree
    }

    fn find_start_leaf(&mut self, mut index: VtkIdType, mut level: i32) -> bool {
        if level < self.level {
            let child_index = VtkIdType::from(self.branching_factor) * index + 1;
            level += 1;
            for i in 0..self.branching_factor {
                index = child_index + VtkIdType::from(i);
                if index >= self.tree_size {
                    self.tree_index = self.tree_size;
                    return false;
                }
                if self.find_start_leaf(index, level) {
                    return true;
                }
            }
            false
        } else {
            let tree = self.tree[vtk_id_to_usize(index)];
            if tree.min > self.scalar_tree.get_scalar_value()
                || tree.max < self.scalar_tree.get_scalar_value()
            {
                false
            } else {
                self.child_number = 0;
                self.tree_index = index;
                self.cell_id = (index - self.leaf_offset) * VtkIdType::from(self.branching_factor);
                true
            }
        }
    }

    fn find_next_leaf(&mut self, child_index: VtkIdType, child_level: i32) -> bool {
        let my_index = (child_index - 1) / VtkIdType::from(self.branching_factor);
        let my_level = child_level - 1;
        let first_child_index = my_index * VtkIdType::from(self.branching_factor) + 1;
        let mut child_num = child_index - first_child_index;

        child_num += 1;
        while child_num < VtkIdType::from(self.branching_factor) {
            let index = first_child_index + child_num;
            if index >= self.tree_size {
                self.tree_index = self.tree_size;
                return false;
            }
            if self.find_start_leaf(index, child_level) {
                return true;
            }
            child_num += 1;
        }

        if my_level <= 0 {
            self.tree_index = self.tree_size;
            false
        } else {
            self.find_next_leaf(my_index, my_level)
        }
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.scalar_tree.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.scalar_tree.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.scalar_tree.get_class_name()
    }

    /// VTK: `vtkSimpleScalarTree::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkSimpleScalarTree" || ScalarTree::is_type_of(name)
    }

    /// VTK: `vtkSimpleScalarTree::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }
}

impl ScalarTreeApi for SimpleScalarTree {
    fn scalar_tree(&self) -> &ScalarTree {
        self.scalar_tree()
    }

    fn scalar_tree_mut(&mut self) -> &mut ScalarTree {
        self.scalar_tree_mut()
    }

    fn build_tree(&mut self) {
        Self::build_tree(self);
    }

    fn initialize(&mut self) {
        Self::initialize(self);
    }

    fn init_traversal(&mut self, scalar_value: f64) {
        Self::init_traversal(self, scalar_value);
    }

    fn get_next_cell(
        &mut self,
        cell_id: &mut VtkIdType,
        pt_ids: &mut Option<IdList>,
        cell_scalars: &mut AnyArray,
    ) -> Option<ScalarTreeCellHandle> {
        Self::get_next_cell(self, cell_id, pt_ids, cell_scalars)
    }

    fn get_number_of_cell_batches(&mut self, scalar_value: f64) -> VtkIdType {
        Self::get_number_of_cell_batches(self, scalar_value)
    }

    fn get_cell_batch(&mut self, batch_num: VtkIdType) -> &[VtkIdType] {
        Self::get_cell_batch(self, batch_num)
    }
}

impl Default for SimpleScalarTree {
    fn default() -> Self {
        Self::new()
    }
}

fn scalar_range_for_cell(
    data_set: &ScalarTreeDataSetHandle,
    scalars: &ScalarTreeScalarsHandle,
    cell_id: VtkIdType,
) -> Option<ScalarRange> {
    let point_ids = data_set.get_cell_points(cell_id);
    let mut range = ScalarRange::default();
    for point_id in point_ids.iter() {
        let value = scalars.get_tuple1(point_id)?;
        range.min = range.min.min(value);
        range.max = range.max.max(value);
    }
    Some(range)
}

fn div_ceil(numerator: VtkIdType, denominator: VtkIdType) -> VtkIdType {
    if numerator <= 0 {
        0
    } else {
        ((numerator - 1) / denominator) + 1
    }
}

fn vtk_id_to_usize(value: VtkIdType) -> usize {
    usize::try_from(value).expect("vtkIdType must be non-negative and fit usize")
}

#[allow(dead_code)]
fn new_cell_scalars_array() -> AnyArray {
    let mut array = DoubleArray::new();
    array.set_number_of_components(1);
    AnyArray::Double(array)
}
