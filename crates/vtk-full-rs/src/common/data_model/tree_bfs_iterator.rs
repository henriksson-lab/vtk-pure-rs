use std::collections::VecDeque;

use crate::common::core::{VtkIdType, VtkMTimeType};

use super::{Tree, TreeIterator, TreeIteratorApi};

const WHITE: i32 = 0;
const GRAY: i32 = 1;
const BLACK: i32 = 2;

/// VTK: `vtkTreeBFSIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeBFSIterator {
    tree_iterator: TreeIterator,
    queue: VecDeque<VtkIdType>,
    color: Vec<i32>,
}

impl TreeBFSIterator {
    /// VTK: `vtkTreeBFSIterator::New`.
    pub fn new() -> Self {
        Self {
            tree_iterator: TreeIterator::with_class_name("vtkTreeBFSIterator"),
            queue: VecDeque::new(),
            color: Vec::new(),
        }
    }

    /// VTK: `vtkTreeBFSIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.tree_iterator.print_self()
    }

    /// VTK: `vtkTreeIterator::SetTree`.
    pub fn set_tree(&mut self, tree: &Tree) {
        <Self as TreeIteratorApi>::set_tree(self, tree);
    }

    /// VTK: `vtkTreeIterator::GetTree`.
    pub fn get_tree(&self) -> Option<&Tree> {
        <Self as TreeIteratorApi>::get_tree(self)
    }

    /// VTK: `vtkTreeIterator::SetStartVertex`.
    pub fn set_start_vertex(&mut self, vertex: VtkIdType) {
        <Self as TreeIteratorApi>::set_start_vertex(self, vertex);
    }

    /// VTK: `vtkTreeIterator::GetStartVertex`.
    pub fn get_start_vertex(&self) -> VtkIdType {
        <Self as TreeIteratorApi>::get_start_vertex(self)
    }

    /// VTK: `vtkTreeIterator::Next`.
    pub fn next(&mut self) -> VtkIdType {
        <Self as TreeIteratorApi>::next(self)
    }

    /// VTK: `vtkTreeIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        <Self as TreeIteratorApi>::has_next(self)
    }

    /// VTK: `vtkTreeIterator::Restart`.
    pub fn restart(&mut self) {
        <Self as TreeIteratorApi>::restart(self);
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.tree_iterator.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.tree_iterator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.tree_iterator.get_m_time()
    }
}

impl TreeIteratorApi for TreeBFSIterator {
    fn tree_iterator(&self) -> &TreeIterator {
        &self.tree_iterator
    }

    fn tree_iterator_mut(&mut self) -> &mut TreeIterator {
        &mut self.tree_iterator
    }

    /// VTK: `vtkTreeBFSIterator::Initialize`.
    fn initialize(&mut self) {
        let Some(tree) = self.tree_iterator.tree() else {
            return;
        };

        let number_of_vertices = tree.get_number_of_vertices();
        self.color.clear();
        self.color.resize(number_of_vertices as usize, WHITE);
        if self.tree_iterator.start_vertex() < 0 {
            self.tree_iterator
                .set_start_vertex_internal(tree.get_root());
        }
        self.queue.clear();

        if number_of_vertices > 0 {
            let next_id = self.next_internal();
            self.tree_iterator.set_next_id(next_id);
        } else {
            self.tree_iterator.set_next_id(-1);
        }
    }

    /// VTK: `vtkTreeBFSIterator::NextInternal`.
    fn next_internal(&mut self) -> VtkIdType {
        let start_vertex = self.tree_iterator.start_vertex();
        if start_vertex < 0 || start_vertex as usize >= self.color.len() {
            return -1;
        }

        if self.color[start_vertex as usize] == WHITE {
            self.color[start_vertex as usize] = GRAY;
            self.queue.push_back(start_vertex);
        }

        while let Some(current_id) = self.queue.pop_front() {
            let Some(tree) = self.tree_iterator.tree() else {
                return -1;
            };
            for child_num in 0..tree.get_number_of_children(current_id) {
                let child_id = tree.get_child(current_id, child_num);
                if child_id >= 0 && self.color[child_id as usize] == WHITE {
                    self.color[child_id as usize] = GRAY;
                    self.queue.push_back(child_id);
                }
            }

            self.color[current_id as usize] = BLACK;
            return current_id;
        }
        -1
    }
}

impl Default for TreeBFSIterator {
    fn default() -> Self {
        Self::new()
    }
}
