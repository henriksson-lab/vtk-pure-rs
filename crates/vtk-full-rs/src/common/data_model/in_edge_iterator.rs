use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::{DirectedGraph, GraphEdge, InEdge, UndirectedGraph};

/// Rust handle used where VTK stores a `vtkGraph*`.
#[derive(Debug, Clone, PartialEq)]
pub enum InEdgeIteratorGraphHandle {
    Directed(DirectedGraph),
    Undirected(UndirectedGraph),
}

impl InEdgeIteratorGraphHandle {
    fn get_in_edges(&self, vertex: VtkIdType) -> Vec<InEdge> {
        match self {
            Self::Directed(graph) => graph.get_in_edges(vertex),
            Self::Undirected(graph) => graph.get_in_edges(vertex),
        }
    }

    fn print_self(&self) -> String {
        match self {
            Self::Directed(graph) => graph.print_self(),
            Self::Undirected(graph) => graph.print_self(),
        }
    }
}

impl From<&DirectedGraph> for InEdgeIteratorGraphHandle {
    fn from(graph: &DirectedGraph) -> Self {
        Self::Directed(graph.clone())
    }
}

impl From<&UndirectedGraph> for InEdgeIteratorGraphHandle {
    fn from(graph: &UndirectedGraph) -> Self {
        Self::Undirected(graph.clone())
    }
}

/// VTK: `vtkInEdgeIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct InEdgeIterator {
    object: Object,
    graph: Option<InEdgeIteratorGraphHandle>,
    edges: Vec<InEdge>,
    current: usize,
    vertex: VtkIdType,
    graph_edge: Option<GraphEdge>,
}

impl InEdgeIterator {
    /// VTK: `vtkInEdgeIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkInEdgeIterator"),
            graph: None,
            edges: Vec::new(),
            current: 0,
            vertex: 0,
            graph_edge: None,
        }
    }

    /// VTK: `vtkInEdgeIterator::Initialize`.
    pub fn initialize<G>(&mut self, graph: G, vertex: VtkIdType)
    where
        G: Into<InEdgeIteratorGraphHandle>,
    {
        self.set_graph(Some(graph.into()));
        self.vertex = vertex;
        self.edges = self
            .graph
            .as_ref()
            .map(|graph| graph.get_in_edges(vertex))
            .unwrap_or_default();
        self.current = 0;
    }

    /// VTK: `vtkInEdgeIterator::GetGraph`.
    pub fn get_graph(&self) -> Option<&InEdgeIteratorGraphHandle> {
        self.graph.as_ref()
    }

    /// VTK: `vtkInEdgeIterator::GetVertex`.
    pub fn get_vertex(&self) -> VtkIdType {
        self.vertex
    }

    /// VTK: `vtkInEdgeIterator::Next`.
    pub fn next(&mut self) -> InEdge {
        assert!(self.has_next());
        let edge = self.edges[self.current];
        self.current += 1;
        edge
    }

    /// VTK: `vtkInEdgeIterator::NextGraphEdge`.
    pub fn next_graph_edge(&mut self) -> &GraphEdge {
        let edge = self.next();
        let graph_edge = self.graph_edge.get_or_insert_with(GraphEdge::new);
        graph_edge.set_source(edge.source);
        graph_edge.set_target(self.vertex);
        graph_edge.set_id(edge.id);
        graph_edge
    }

    /// VTK: `vtkInEdgeIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        self.current != self.edges.len()
    }

    /// VTK protected macro implementation: `vtkInEdgeIterator::SetGraph`.
    pub(crate) fn set_graph(&mut self, graph: Option<InEdgeIteratorGraphHandle>) {
        self.graph = graph;
        self.modified();
    }

    /// VTK: `vtkInEdgeIterator::PrintSelf`.
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

impl Default for InEdgeIterator {
    fn default() -> Self {
        Self::new()
    }
}
