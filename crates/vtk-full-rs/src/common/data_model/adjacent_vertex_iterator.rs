use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::{DirectedGraph, OutEdge, UndirectedGraph};

/// Rust handle used where VTK stores a `vtkGraph*`.
#[derive(Debug, Clone, PartialEq)]
pub enum AdjacentVertexIteratorGraphHandle {
    Directed(DirectedGraph),
    Undirected(UndirectedGraph),
}

impl AdjacentVertexIteratorGraphHandle {
    fn get_out_edges(&self, vertex: VtkIdType) -> Vec<OutEdge> {
        match self {
            Self::Directed(graph) => graph.get_out_edges(vertex),
            Self::Undirected(graph) => graph.get_out_edges(vertex),
        }
    }

    fn print_self(&self) -> String {
        match self {
            Self::Directed(graph) => graph.print_self(),
            Self::Undirected(graph) => graph.print_self(),
        }
    }
}

impl From<&DirectedGraph> for AdjacentVertexIteratorGraphHandle {
    fn from(graph: &DirectedGraph) -> Self {
        Self::Directed(graph.clone())
    }
}

impl From<&UndirectedGraph> for AdjacentVertexIteratorGraphHandle {
    fn from(graph: &UndirectedGraph) -> Self {
        Self::Undirected(graph.clone())
    }
}

/// VTK: `vtkAdjacentVertexIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct AdjacentVertexIterator {
    object: Object,
    graph: Option<AdjacentVertexIteratorGraphHandle>,
    edges: Vec<OutEdge>,
    current: usize,
    vertex: VtkIdType,
}

impl AdjacentVertexIterator {
    /// VTK: `vtkAdjacentVertexIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkAdjacentVertexIterator"),
            graph: None,
            edges: Vec::new(),
            current: 0,
            vertex: 0,
        }
    }

    /// VTK: `vtkAdjacentVertexIterator::Initialize`.
    pub fn initialize<G>(&mut self, graph: G, vertex: VtkIdType)
    where
        G: Into<AdjacentVertexIteratorGraphHandle>,
    {
        self.set_graph(Some(graph.into()));
        self.vertex = vertex;
        self.edges = self
            .graph
            .as_ref()
            .map(|graph| graph.get_out_edges(vertex))
            .unwrap_or_default();
        self.current = 0;
    }

    /// VTK: `vtkAdjacentVertexIterator::GetGraph`.
    pub fn get_graph(&self) -> Option<&AdjacentVertexIteratorGraphHandle> {
        self.graph.as_ref()
    }

    /// VTK: `vtkAdjacentVertexIterator::GetVertex`.
    pub fn get_vertex(&self) -> VtkIdType {
        self.vertex
    }

    /// VTK: `vtkAdjacentVertexIterator::Next`.
    pub fn next(&mut self) -> VtkIdType {
        assert!(self.has_next());
        let edge = self.edges[self.current];
        self.current += 1;
        edge.target
    }

    /// VTK: `vtkAdjacentVertexIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        self.current != self.edges.len()
    }

    /// VTK protected macro implementation: `vtkAdjacentVertexIterator::SetGraph`.
    pub(crate) fn set_graph(&mut self, graph: Option<AdjacentVertexIteratorGraphHandle>) {
        self.graph = graph;
        self.modified();
    }

    /// VTK: `vtkAdjacentVertexIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        let graph_text = self
            .graph
            .as_ref()
            .map(|graph| graph.print_self())
            .unwrap_or_default();
        format!(
            "{}\nGraph: {}\n{}Vertex: {}",
            self.object.get_object_description(),
            if self.graph.is_some() { "" } else { "(null)" },
            graph_text,
            self.vertex
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

impl Default for AdjacentVertexIterator {
    fn default() -> Self {
        Self::new()
    }
}
