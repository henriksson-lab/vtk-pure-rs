use crate::common::core::{VtkIdType, VtkMTimeType};

use super::{Tree, TreeIterator, TreeIteratorApi};

pub const DISCOVER: i32 = 0;
pub const FINISH: i32 = 1;

const WHITE: i32 = 0;
const GRAY: i32 = 1;
const BLACK: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TreeDFSIteratorPosition {
    vertex: VtkIdType,
    index: VtkIdType,
}

impl TreeDFSIteratorPosition {
    fn new(vertex: VtkIdType, index: VtkIdType) -> Self {
        Self { vertex, index }
    }
}

/// VTK: `vtkTreeDFSIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeDFSIterator {
    tree_iterator: TreeIterator,
    mode: i32,
    cur_root: VtkIdType,
    stack: Vec<TreeDFSIteratorPosition>,
    color: Vec<i32>,
}

impl TreeDFSIterator {
    /// VTK: `vtkTreeDFSIterator::New`.
    pub fn new() -> Self {
        Self {
            tree_iterator: TreeIterator::with_class_name("vtkTreeDFSIterator"),
            mode: DISCOVER,
            cur_root: -1,
            stack: Vec::new(),
            color: Vec::new(),
        }
    }

    /// VTK: `vtkTreeDFSIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nMode: {}\nCurRoot: {}",
            self.tree_iterator.print_self(),
            self.mode,
            self.cur_root
        )
    }

    /// VTK: `vtkTreeDFSIterator::SetMode`.
    pub fn set_mode(&mut self, mode: i32) {
        if self.mode != mode {
            self.mode = mode;
            self.initialize();
            self.modified();
        }
    }

    /// VTK: `vtkTreeDFSIterator::GetMode`.
    pub fn get_mode(&self) -> i32 {
        self.mode
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

impl TreeIteratorApi for TreeDFSIterator {
    fn tree_iterator(&self) -> &TreeIterator {
        &self.tree_iterator
    }

    fn tree_iterator_mut(&mut self) -> &mut TreeIterator {
        &mut self.tree_iterator
    }

    /// VTK: `vtkTreeDFSIterator::Initialize`.
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
        self.cur_root = self.tree_iterator.start_vertex();
        self.stack.clear();

        if number_of_vertices > 0 {
            let next_id = self.next_internal();
            self.tree_iterator.set_next_id(next_id);
        } else {
            self.tree_iterator.set_next_id(-1);
        }
    }

    /// VTK: `vtkTreeDFSIterator::NextInternal`.
    fn next_internal(&mut self) -> VtkIdType {
        let start_vertex = self.tree_iterator.start_vertex();
        if start_vertex < 0 || start_vertex as usize >= self.color.len() {
            return -1;
        }

        while self.color[start_vertex as usize] != BLACK {
            while let Some(mut pos) = self.stack.pop() {
                let Some(tree) = self.tree_iterator.tree() else {
                    return -1;
                };
                let nchildren = tree.get_number_of_children(pos.vertex);
                while pos.index < nchildren {
                    let child = tree.get_child(pos.vertex, pos.index);
                    if child >= 0 && self.color[child as usize] == WHITE {
                        break;
                    }
                    pos.index += 1;
                }

                if pos.index == nchildren {
                    self.color[pos.vertex as usize] = BLACK;
                    if self.mode == FINISH {
                        return pos.vertex;
                    }
                    if pos.vertex == start_vertex {
                        return -1;
                    }
                } else {
                    self.stack.push(pos);

                    let Some(tree) = self.tree_iterator.tree() else {
                        return -1;
                    };
                    let found = tree.get_child(pos.vertex, pos.index);
                    self.color[found as usize] = GRAY;
                    self.stack.push(TreeDFSIteratorPosition::new(found, 0));
                    if self.mode == DISCOVER {
                        return found;
                    }
                }
            }

            if self.color[start_vertex as usize] != BLACK {
                let Some(tree) = self.tree_iterator.tree() else {
                    return -1;
                };
                let number_of_vertices = tree.get_number_of_vertices();
                if number_of_vertices <= 0 {
                    return -1;
                }

                loop {
                    if self.cur_root < 0 || self.cur_root as usize >= self.color.len() {
                        return -1;
                    }
                    match self.color[self.cur_root as usize] {
                        WHITE => {
                            self.stack
                                .push(TreeDFSIteratorPosition::new(self.cur_root, 0));
                            self.color[self.cur_root as usize] = GRAY;
                            if self.mode == DISCOVER {
                                return self.cur_root;
                            }
                            break;
                        }
                        GRAY => {
                            self.cur_root = (self.cur_root + 1) % number_of_vertices;
                        }
                        _ => {
                            self.cur_root = (self.cur_root + 1) % number_of_vertices;
                        }
                    }
                }
            }
        }
        -1
    }
}

impl Default for TreeDFSIterator {
    fn default() -> Self {
        Self::new()
    }
}
