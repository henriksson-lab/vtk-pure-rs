use std::ffi::c_void;

use crate::common::core::{Object, ObjectBaseApi, VtkMTimeType};

/// VTK: `vtkCompositeDataSet*`.
pub type CompositeDataSetHandle = *mut c_void;
/// VTK: `vtkDataObject*`.
pub type CompositeIteratorDataObjectHandle = *mut c_void;
/// VTK: `vtkInformation*`.
pub type CompositeIteratorInformationHandle = *mut c_void;

/// VTK: `vtkCompositeDataIterator`.
///
/// This stores the abstract iterator base state. Concrete composite-data
/// iterators implement `CompositeDataIteratorApi`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeDataIterator {
    object: Object,
    skip_empty_nodes: bool,
    reverse: i32,
    data_set: CompositeDataSetHandle,
}

impl CompositeDataIterator {
    /// VTK: `vtkCompositeDataIterator::vtkCompositeDataIterator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            skip_empty_nodes: true,
            reverse: 0,
            data_set: std::ptr::null_mut(),
        }
    }

    /// VTK: `vtkCompositeDataIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}Reverse: {}\nSkipEmptyNodes: {}\n",
            self.object.print_self(),
            if self.reverse != 0 { "On" } else { "Off" },
            if self.skip_empty_nodes { "On" } else { "Off" }
        )
    }

    /// VTK: `vtkCompositeDataIterator::GetDataSet`.
    pub fn get_data_set(&self) -> CompositeDataSetHandle {
        self.data_set
    }

    pub(crate) fn set_data_set_pointer(&mut self, data_set: CompositeDataSetHandle) {
        if self.data_set != data_set {
            self.data_set = data_set;
            self.modified();
        }
    }

    /// VTK: `vtkCompositeDataIterator::GetSkipEmptyNodes`.
    pub fn get_skip_empty_nodes(&self) -> bool {
        self.skip_empty_nodes
    }

    /// VTK: `vtkCompositeDataIterator::SetSkipEmptyNodes`.
    pub fn set_skip_empty_nodes(&mut self, skip_empty_nodes: bool) {
        if self.skip_empty_nodes != skip_empty_nodes {
            self.skip_empty_nodes = skip_empty_nodes;
            self.modified();
        }
    }

    /// VTK: `vtkCompositeDataIterator::SkipEmptyNodesOn`.
    pub fn skip_empty_nodes_on(&mut self) {
        self.set_skip_empty_nodes(true);
    }

    /// VTK: `vtkCompositeDataIterator::SkipEmptyNodesOff`.
    pub fn skip_empty_nodes_off(&mut self) {
        self.set_skip_empty_nodes(false);
    }

    /// VTK: `vtkCompositeDataIterator::GetReverse`.
    pub fn get_reverse(&self) -> i32 {
        self.reverse
    }

    /// VTK: protected `vtkCompositeDataIterator::SetReverse`.
    pub(crate) fn set_reverse(&mut self, reverse: i32) {
        if self.reverse != reverse {
            self.reverse = reverse;
            self.modified();
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCompositeDataIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCompositeDataIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkCompositeDataIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCompositeDataIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkCompositeDataIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkCompositeDataIterator::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

impl Default for CompositeDataIterator {
    fn default() -> Self {
        Self::with_class_name("vtkCompositeDataIterator")
    }
}

/// VTK virtual API for `vtkCompositeDataIterator`.
pub trait CompositeDataIteratorApi {
    /// Access to the translated abstract base state.
    fn composite_data_iterator(&self) -> &CompositeDataIterator;

    /// Mutable access to the translated abstract base state.
    fn composite_data_iterator_mut(&mut self) -> &mut CompositeDataIterator;

    /// VTK: `vtkCompositeDataIterator::SetDataSet`.
    fn set_data_set(&mut self, data_set: CompositeDataSetHandle) {
        self.composite_data_iterator_mut()
            .set_data_set_pointer(data_set);
        if !data_set.is_null() {
            self.go_to_first_item();
        }
    }

    /// VTK: `vtkCompositeDataIterator::GetDataSet`.
    fn get_data_set(&self) -> CompositeDataSetHandle {
        self.composite_data_iterator().get_data_set()
    }

    /// VTK: `vtkCompositeDataIterator::InitTraversal`.
    fn init_traversal(&mut self) {
        self.composite_data_iterator_mut().set_reverse(0);
        self.go_to_first_item();
    }

    /// VTK: `vtkCompositeDataIterator::InitReverseTraversal`.
    fn init_reverse_traversal(&mut self) {
        self.composite_data_iterator_mut().set_reverse(1);
        self.go_to_first_item();
    }

    /// VTK: `vtkCompositeDataIterator::GoToFirstItem`.
    fn go_to_first_item(&mut self);

    /// VTK: `vtkCompositeDataIterator::GoToNextItem`.
    fn go_to_next_item(&mut self);

    /// VTK: `vtkCompositeDataIterator::IsDoneWithTraversal`.
    fn is_done_with_traversal(&self) -> i32;

    /// VTK: `vtkCompositeDataIterator::GetCurrentDataObject`.
    fn get_current_data_object(&mut self) -> CompositeIteratorDataObjectHandle;

    /// VTK: `vtkCompositeDataIterator::GetCurrentMetaData`.
    fn get_current_meta_data(&mut self) -> CompositeIteratorInformationHandle;

    /// VTK: `vtkCompositeDataIterator::HasCurrentMetaData`.
    fn has_current_meta_data(&self) -> bool;

    /// VTK: `vtkCompositeDataIterator::GetCurrentFlatIndex`.
    fn get_current_flat_index(&self) -> u32;

    /// VTK: `vtkCompositeDataIterator::GetSkipEmptyNodes`.
    fn get_skip_empty_nodes(&self) -> bool {
        self.composite_data_iterator().get_skip_empty_nodes()
    }

    /// VTK: `vtkCompositeDataIterator::SetSkipEmptyNodes`.
    fn set_skip_empty_nodes(&mut self, skip_empty_nodes: bool) {
        self.composite_data_iterator_mut()
            .set_skip_empty_nodes(skip_empty_nodes);
    }

    /// VTK: `vtkCompositeDataIterator::SkipEmptyNodesOn`.
    fn skip_empty_nodes_on(&mut self) {
        self.composite_data_iterator_mut().skip_empty_nodes_on();
    }

    /// VTK: `vtkCompositeDataIterator::SkipEmptyNodesOff`.
    fn skip_empty_nodes_off(&mut self) {
        self.composite_data_iterator_mut().skip_empty_nodes_off();
    }

    /// VTK: `vtkCompositeDataIterator::GetReverse`.
    fn get_reverse(&self) -> i32 {
        self.composite_data_iterator().get_reverse()
    }

    /// VTK: `vtkCompositeDataIterator::PrintSelf`.
    fn print_self(&self) -> String {
        self.composite_data_iterator().print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &'static str {
        self.composite_data_iterator().get_class_name()
    }

    /// VTK: `vtkCompositeDataIterator::IsA`.
    fn is_a(&self, name: &str) -> bool {
        self.composite_data_iterator().is_a(name)
    }

    /// VTK: `vtkCompositeDataIterator::GetNumberOfGenerationsFromBase`.
    fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        self.composite_data_iterator()
            .get_number_of_generations_from_base(name)
    }

    /// VTK: `vtkObject::Modified`.
    fn modified(&mut self) {
        self.composite_data_iterator_mut().modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    fn get_m_time(&self) -> VtkMTimeType {
        self.composite_data_iterator().get_m_time()
    }
}
