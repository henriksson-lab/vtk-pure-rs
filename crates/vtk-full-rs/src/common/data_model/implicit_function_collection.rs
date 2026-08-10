use std::fmt;

use crate::common::core::{CollectionSimpleIterator, Object, VtkMTimeType};
use crate::common::data_model::ImplicitFunctionHandle;

/// VTK: `vtkImplicitFunctionCollection`.
#[derive(Clone)]
pub struct ImplicitFunctionCollection {
    object: Object,
    current: usize,
    functions: Vec<ImplicitFunctionHandle>,
}

impl ImplicitFunctionCollection {
    /// VTK: `vtkImplicitFunctionCollection::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkImplicitFunctionCollection"),
            current: 0,
            functions: Vec::new(),
        }
    }

    /// VTK: `vtkImplicitFunctionCollection::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Number Of Items: {}\n", self.functions.len())
    }

    /// VTK: `vtkImplicitFunctionCollection::AddItem`.
    pub fn add_item(&mut self, function: ImplicitFunctionHandle) {
        self.functions.push(function);
        self.modified();
    }

    /// VTK: `vtkImplicitFunctionCollection::GetNextItem`.
    pub fn get_next_item(&mut self) -> Option<ImplicitFunctionHandle> {
        if self.current >= self.functions.len() {
            return None;
        }
        let function = self.functions[self.current].clone();
        self.current += 1;
        Some(function)
    }

    /// VTK: `vtkImplicitFunctionCollection::GetNextImplicitFunction`.
    pub fn get_next_implicit_function(
        &self,
        cookie: &mut CollectionSimpleIterator,
    ) -> Option<ImplicitFunctionHandle> {
        if *cookie >= self.functions.len() {
            return None;
        }
        let function = self.functions[*cookie].clone();
        *cookie += 1;
        Some(function)
    }

    /// VTK: `vtkCollection::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.current = 0;
    }

    /// VTK: `vtkCollection::InitTraversal(vtkCollectionSimpleIterator&)`.
    pub fn init_traversal_cookie(&self, cookie: &mut CollectionSimpleIterator) {
        *cookie = 0;
    }

    /// VTK: `vtkCollection::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> i32 {
        self.functions.len() as i32
    }

    /// VTK: `vtkCollection::RemoveAllItems`.
    pub fn remove_all_items(&mut self) {
        if self.functions.is_empty() {
            return;
        }
        self.functions.clear();
        self.current = 0;
        self.modified();
    }

    /// VTK: `vtkCollection::IndexOfFirstOccurrence`.
    pub fn index_of_first_occurrence(&self, function: &ImplicitFunctionHandle) -> i32 {
        self.functions
            .iter()
            .position(|item| item.ptr_eq(function))
            .map_or(-1, |index| index as i32)
    }

    /// VTK: `vtkCollection::RemoveItem(vtkObject*)`.
    pub fn remove_item(&mut self, function: &ImplicitFunctionHandle) {
        let Some(index) = self.functions.iter().position(|item| item.ptr_eq(function)) else {
            return;
        };
        self.functions.remove(index);
        if index < self.current {
            self.current -= 1;
        }
        self.modified();
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ImplicitFunctionHandle> {
        self.functions.iter()
    }

    pub(crate) fn first(&self) -> Option<&ImplicitFunctionHandle> {
        self.functions.first()
    }

    pub(crate) fn len(&self) -> usize {
        self.functions.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }
}

impl Default for ImplicitFunctionCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ImplicitFunctionCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImplicitFunctionCollection")
            .field("class_name", &self.get_class_name())
            .field("current", &self.current)
            .field("function_count", &self.functions.len())
            .finish()
    }
}
