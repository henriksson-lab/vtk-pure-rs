use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::{DirectedGraph, UndirectedGraph};

/// Rust handle used where VTK stores a `vtkGraph*`.
#[derive(Debug, Clone, PartialEq)]
pub enum VertexListIteratorGraphHandle {
    Directed(DirectedGraph),
    Undirected(UndirectedGraph),
}

impl VertexListIteratorGraphHandle {
    fn get_number_of_vertices(&self) -> VtkIdType {
        match self {
            Self::Directed(graph) => graph.get_number_of_vertices(),
            Self::Undirected(graph) => graph.get_number_of_vertices(),
        }
    }

    fn print_self(&self) -> String {
        match self {
            Self::Directed(graph) => graph.print_self(),
            Self::Undirected(graph) => graph.print_self(),
        }
    }
}

impl From<&DirectedGraph> for VertexListIteratorGraphHandle {
    fn from(graph: &DirectedGraph) -> Self {
        Self::Directed(graph.clone())
    }
}

impl From<&UndirectedGraph> for VertexListIteratorGraphHandle {
    fn from(graph: &UndirectedGraph) -> Self {
        Self::Undirected(graph.clone())
    }
}

/// VTK: `vtkVertexListIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct VertexListIterator {
    object: Object,
    graph: Option<VertexListIteratorGraphHandle>,
    current: VtkIdType,
    end: VtkIdType,
}

impl VertexListIterator {
    /// VTK: `vtkVertexListIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkVertexListIterator"),
            graph: None,
            current: 0,
            end: 0,
        }
    }

    /// VTK: `vtkVertexListIterator::SetGraph`.
    pub fn set_graph<G>(&mut self, graph: G)
    where
        G: Into<VertexListIteratorGraphHandle>,
    {
        self.set_graph_handle(Some(graph.into()));
    }

    fn set_graph_handle(&mut self, graph: Option<VertexListIteratorGraphHandle>) {
        self.graph = graph;
        self.current = 0;
        self.end = self
            .graph
            .as_ref()
            .map(VertexListIteratorGraphHandle::get_number_of_vertices)
            .unwrap_or(0);
        self.modified();
    }

    /// VTK: `vtkVertexListIterator::GetGraph`.
    pub fn get_graph(&self) -> Option<&VertexListIteratorGraphHandle> {
        self.graph.as_ref()
    }

    /// VTK: `vtkVertexListIterator::Next`.
    pub fn next(&mut self) -> VtkIdType {
        assert!(self.has_next());
        let vertex = self.current;
        self.current += 1;
        vertex
    }

    /// VTK: `vtkVertexListIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        self.current != self.end
    }

    /// VTK: `vtkVertexListIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        let graph_text = self
            .graph
            .as_ref()
            .map(|graph| graph.print_self())
            .unwrap_or_default();
        format!(
            "{}\nGraph: {}\n{}",
            self.object.get_object_description(),
            if self.graph.is_some() { "" } else { "(null)" },
            graph_text
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

impl Default for VertexListIterator {
    fn default() -> Self {
        Self::new()
    }
}
