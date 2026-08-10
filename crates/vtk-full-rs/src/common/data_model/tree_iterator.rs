use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::Tree;

/// VTK: `vtkTreeIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeIterator {
    object: Object,
    tree: Option<Tree>,
    start_vertex: VtkIdType,
    next_id: VtkIdType,
}

impl TreeIterator {
    /// VTK: `vtkTreeIterator::vtkTreeIterator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            tree: None,
            start_vertex: -1,
            next_id: -1,
        }
    }

    pub(crate) fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    pub(crate) fn set_tree_internal(&mut self, tree: Tree) {
        self.tree = Some(tree);
    }

    pub(crate) fn start_vertex(&self) -> VtkIdType {
        self.start_vertex
    }

    pub(crate) fn set_start_vertex_internal(&mut self, vertex: VtkIdType) {
        self.start_vertex = vertex;
    }

    pub(crate) fn next_id(&self) -> VtkIdType {
        self.next_id
    }

    pub(crate) fn set_next_id(&mut self, next_id: VtkIdType) {
        self.next_id = next_id;
    }

    /// VTK: `vtkTreeIterator::GetTree`.
    pub fn get_tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    /// VTK: `vtkTreeIterator::GetStartVertex`.
    pub fn get_start_vertex(&self) -> VtkIdType {
        self.start_vertex
    }

    /// VTK: `vtkTreeIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        self.next_id != -1
    }

    /// VTK: `vtkTreeIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nTree: {}\nStartVertex: {}\nNextId: {}",
            self.object.get_object_description(),
            if self.tree.is_some() { "set" } else { "(null)" },
            self.start_vertex,
            self.next_id
        )
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
}

impl Default for TreeIterator {
    fn default() -> Self {
        Self::with_class_name("vtkTreeIterator")
    }
}

pub trait TreeIteratorApi {
    fn tree_iterator(&self) -> &TreeIterator;
    fn tree_iterator_mut(&mut self) -> &mut TreeIterator;
    fn initialize(&mut self);
    fn next_internal(&mut self) -> VtkIdType;

    /// VTK: `vtkTreeIterator::SetTree`.
    fn set_tree(&mut self, tree: &Tree) {
        self.tree_iterator_mut().set_tree_internal(tree.clone());
        self.tree_iterator_mut().set_start_vertex_internal(-1);
        self.initialize();
    }

    /// VTK: `vtkTreeIterator::GetTree`.
    fn get_tree(&self) -> Option<&Tree> {
        self.tree_iterator().get_tree()
    }

    /// VTK: `vtkTreeIterator::SetStartVertex`.
    fn set_start_vertex(&mut self, vertex: VtkIdType) {
        if self.tree_iterator().get_start_vertex() != vertex {
            self.tree_iterator_mut().set_start_vertex_internal(vertex);
            self.initialize();
            self.tree_iterator_mut().modified();
        }
    }

    /// VTK: `vtkTreeIterator::GetStartVertex`.
    fn get_start_vertex(&self) -> VtkIdType {
        self.tree_iterator().get_start_vertex()
    }

    /// VTK: `vtkTreeIterator::Next`.
    fn next(&mut self) -> VtkIdType {
        let last = self.tree_iterator().next_id();
        if last != -1 {
            let next = self.next_internal();
            self.tree_iterator_mut().set_next_id(next);
        }
        last
    }

    /// VTK: `vtkTreeIterator::HasNext`.
    fn has_next(&self) -> bool {
        self.tree_iterator().has_next()
    }

    /// VTK: `vtkTreeIterator::Restart`.
    fn restart(&mut self) {
        self.initialize();
    }
}
