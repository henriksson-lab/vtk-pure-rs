use std::{cell::RefCell, fmt, rc::Rc};

use crate::common::{
    core::{AnyArray, IdList, Object, TimeStamp, VtkIdType, VtkMTimeType},
    data_model::DataSetApi,
};

/// Shallow-copyable dynamic handle for `vtkDataSet*` storage.
#[derive(Clone)]
pub struct ScalarTreeDataSetHandle {
    data_set: Rc<RefCell<dyn DataSetApi>>,
}

impl ScalarTreeDataSetHandle {
    pub fn new<T: DataSetApi + 'static>(data_set: T) -> Self {
        Self {
            data_set: Rc::new(RefCell::new(data_set)),
        }
    }

    pub fn from_rc<T: DataSetApi + 'static>(data_set: Rc<RefCell<T>>) -> Self {
        Self { data_set }
    }

    pub fn as_ptr(&self) -> *const RefCell<dyn DataSetApi> {
        Rc::as_ptr(&self.data_set)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data_set, &other.data_set)
    }

    pub fn get_class_name(&self) -> &'static str {
        self.data_set.borrow().get_class_name()
    }

    pub fn get_m_time(&self) -> VtkMTimeType {
        self.data_set.borrow().data_set().get_m_time()
    }

    pub fn get_number_of_cells(&self) -> VtkIdType {
        self.data_set.borrow().get_number_of_cells()
    }

    pub fn get_cell_type(&self, cell_id: VtkIdType) -> i32 {
        self.data_set.borrow().get_cell_type(cell_id)
    }

    pub fn get_cell_points(&self, cell_id: VtkIdType) -> IdList {
        let mut point_ids = IdList::new();
        self.data_set
            .borrow()
            .get_cell_points(cell_id, &mut point_ids);
        point_ids
    }

    pub fn get_cell_handle(&self, cell_id: VtkIdType) -> ScalarTreeCellHandle {
        ScalarTreeCellHandle {
            cell_id,
            cell_type: self.get_cell_type(cell_id),
            point_ids: self.get_cell_points(cell_id),
        }
    }
}

impl fmt::Debug for ScalarTreeDataSetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarTreeDataSetHandle")
            .field("class_name", &self.get_class_name())
            .finish_non_exhaustive()
    }
}

/// Shallow-copyable dynamic handle for `vtkDataArray*` storage.
#[derive(Clone)]
pub struct ScalarTreeScalarsHandle {
    scalars: Rc<RefCell<AnyArray>>,
}

impl ScalarTreeScalarsHandle {
    pub fn new(scalars: AnyArray) -> Self {
        Self {
            scalars: Rc::new(RefCell::new(scalars)),
        }
    }

    pub fn from_rc(scalars: Rc<RefCell<AnyArray>>) -> Self {
        Self { scalars }
    }

    pub fn as_ptr(&self) -> *const RefCell<AnyArray> {
        Rc::as_ptr(&self.scalars)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.scalars, &other.scalars)
    }

    pub fn get_m_time(&self) -> VtkMTimeType {
        self.scalars.borrow().get_m_time()
    }

    pub fn get_range(&self) -> Option<[f64; 2]> {
        self.scalars.borrow().get_range()
    }

    pub fn get_tuple1(&self, tuple_idx: VtkIdType) -> Option<f64> {
        let tuple_idx = usize::try_from(tuple_idx).ok()?;
        self.scalars
            .borrow()
            .numeric_tuple_as_f64_checked(tuple_idx)
            .ok()?
            .first()
            .copied()
    }

    pub fn copy_tuples(&self, point_ids: &IdList, output: &mut AnyArray) -> bool {
        let scalars = self.scalars.borrow();
        output.set_number_of_components(scalars.get_number_of_components());
        output.set_number_of_tuples(point_ids.get_number_of_ids());
        for (tuple_idx, point_id) in point_ids.iter().enumerate() {
            let Ok(point_id) = usize::try_from(point_id) else {
                return false;
            };
            let Ok(tuple) = scalars.numeric_tuple_as_f64_checked(point_id) else {
                return false;
            };
            if output
                .insert_numeric_tuple_from_f64_checked(tuple_idx, &tuple)
                .is_err()
            {
                return false;
            }
        }
        true
    }
}

impl fmt::Debug for ScalarTreeScalarsHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarTreeScalarsHandle")
            .field("data_type", &self.scalars.borrow().get_data_type())
            .finish_non_exhaustive()
    }
}

/// Rust cell view returned by concrete `vtkScalarTree` traversal.
#[derive(Debug, Clone)]
pub struct ScalarTreeCellHandle {
    cell_id: VtkIdType,
    cell_type: i32,
    point_ids: IdList,
}

impl ScalarTreeCellHandle {
    /// VTK: candidate cell id associated with `vtkScalarTree::GetNextCell`.
    pub fn get_cell_id(&self) -> VtkIdType {
        self.cell_id
    }

    /// VTK: `vtkCell::GetCellType` for the returned candidate cell.
    pub fn get_cell_type(&self) -> i32 {
        self.cell_type
    }

    /// VTK: `vtkCell::GetPointIds`.
    pub fn get_point_ids(&self) -> &IdList {
        &self.point_ids
    }
}

/// VTK pure virtual API for `vtkScalarTree`.
pub trait ScalarTreeApi {
    fn scalar_tree(&self) -> &ScalarTree;
    fn scalar_tree_mut(&mut self) -> &mut ScalarTree;

    /// VTK: `vtkScalarTree::BuildTree`.
    fn build_tree(&mut self);

    /// VTK: `vtkScalarTree::Initialize`.
    fn initialize(&mut self);

    /// VTK: `vtkScalarTree::InitTraversal`.
    fn init_traversal(&mut self, scalar_value: f64);

    /// VTK: `vtkScalarTree::GetNextCell`.
    fn get_next_cell(
        &mut self,
        cell_id: &mut VtkIdType,
        pt_ids: &mut Option<IdList>,
        cell_scalars: &mut AnyArray,
    ) -> Option<ScalarTreeCellHandle>;

    /// VTK: `vtkScalarTree::GetNumberOfCellBatches`.
    fn get_number_of_cell_batches(&mut self, scalar_value: f64) -> VtkIdType;

    /// VTK: `vtkScalarTree::GetCellBatch`.
    fn get_cell_batch(&mut self, batch_num: VtkIdType) -> &[VtkIdType];
}

/// VTK: `vtkScalarTree`.
#[derive(Debug)]
pub struct ScalarTree {
    object: Object,
    data_set: Option<ScalarTreeDataSetHandle>,
    scalars: Option<ScalarTreeScalarsHandle>,
    scalar_value: f64,
    build_time: TimeStamp,
}

impl ScalarTree {
    /// VTK: `vtkScalarTree::vtkScalarTree`.
    pub fn new() -> Self {
        Self::with_class_name("vtkScalarTree")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            data_set: None,
            scalars: None,
            scalar_value: 0.0,
            build_time: TimeStamp::new(),
        }
    }

    /// VTK: `vtkScalarTree::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        output.push_str("\nDataSet: ");
        match &self.data_set {
            Some(data_set) => output.push_str(&format!(
                "{}({:p})",
                data_set.get_class_name(),
                data_set.as_ptr()
            )),
            None => output.push_str("(none)"),
        }
        output.push_str("\nScalars: ");
        match &self.scalars {
            Some(scalars) => output.push_str(&format!("{:p}", scalars.as_ptr())),
            None => output.push_str("(none)"),
        }
        output.push_str("\nBuild Time: ");
        output.push_str(&self.build_time.get_m_time().to_string());
        output
    }

    /// VTK: `vtkScalarTree::ShallowCopy`.
    pub fn shallow_copy(&mut self, stree: &Self) {
        self.set_data_set(stree.get_data_set());
        self.set_scalars(stree.get_scalars());
    }

    /// VTK: `vtkScalarTree::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: Option<ScalarTreeDataSetHandle>) {
        if option_data_set_ptr_eq(&self.data_set, &data_set) {
            return;
        }
        self.data_set = data_set;
        self.modified();
    }

    /// VTK: `vtkScalarTree::GetDataSet`.
    pub fn get_data_set(&self) -> Option<ScalarTreeDataSetHandle> {
        self.data_set.clone()
    }

    /// VTK: `vtkScalarTree::SetScalars`.
    pub fn set_scalars(&mut self, scalars: Option<ScalarTreeScalarsHandle>) {
        if option_scalars_ptr_eq(&self.scalars, &scalars) {
            return;
        }
        self.scalars = scalars;
        self.modified();
    }

    /// VTK: `vtkScalarTree::GetScalars`.
    pub fn get_scalars(&self) -> Option<ScalarTreeScalarsHandle> {
        self.scalars.clone()
    }

    /// VTK: `vtkScalarTree::GetScalarValue`.
    pub fn get_scalar_value(&self) -> f64 {
        self.scalar_value
    }

    #[allow(dead_code)]
    pub(crate) fn set_scalar_value(&mut self, scalar_value: f64) {
        self.scalar_value = scalar_value;
    }

    #[allow(dead_code)]
    pub(crate) fn build_time_mut(&mut self) -> &mut TimeStamp {
        &mut self.build_time
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        let mut m_time = self.object.get_m_time().max(self.build_time.get_m_time());
        if let Some(data_set) = &self.data_set {
            m_time = m_time.max(data_set.get_m_time());
        }
        if let Some(scalars) = &self.scalars {
            m_time = m_time.max(scalars.get_m_time());
        }
        m_time
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkScalarTree::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkScalarTree" || Object::is_type_of(name)
    }

    /// VTK: `vtkScalarTree::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkScalarTree::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkScalarTree" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkScalarTree::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.object.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.object.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.object.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.object.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.object.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for ScalarTree {
    fn default() -> Self {
        Self::new()
    }
}

fn option_data_set_ptr_eq(
    left: &Option<ScalarTreeDataSetHandle>,
    right: &Option<ScalarTreeDataSetHandle>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.ptr_eq(right),
        (None, None) => true,
        _ => false,
    }
}

fn option_scalars_ptr_eq(
    left: &Option<ScalarTreeScalarsHandle>,
    right: &Option<ScalarTreeScalarsHandle>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.ptr_eq(right),
        (None, None) => true,
        _ => false,
    }
}
