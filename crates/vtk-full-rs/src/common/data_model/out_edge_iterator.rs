use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::{DirectedGraph, GraphEdge, OutEdge, UndirectedGraph};

/// Rust handle used where VTK stores a `vtkGraph*`.
#[derive(Debug, Clone, PartialEq)]
pub enum OutEdgeIteratorGraphHandle {
    Directed(DirectedGraph),
    Undirected(UndirectedGraph),
}

impl OutEdgeIteratorGraphHandle {
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

impl From<&DirectedGraph> for OutEdgeIteratorGraphHandle {
    fn from(graph: &DirectedGraph) -> Self {
        Self::Directed(graph.clone())
    }
}

impl From<&UndirectedGraph> for OutEdgeIteratorGraphHandle {
    fn from(graph: &UndirectedGraph) -> Self {
        Self::Undirected(graph.clone())
    }
}

/// VTK: `vtkOutEdgeIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct OutEdgeIterator {
    object: Object,
    graph: Option<OutEdgeIteratorGraphHandle>,
    edges: Vec<OutEdge>,
    current: usize,
    vertex: VtkIdType,
    graph_edge: Option<GraphEdge>,
}

impl OutEdgeIterator {
    /// VTK: `vtkOutEdgeIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkOutEdgeIterator"),
            graph: None,
            edges: Vec::new(),
            current: 0,
            vertex: 0,
            graph_edge: None,
        }
    }

    /// VTK: `vtkOutEdgeIterator::Initialize`.
    pub fn initialize<G>(&mut self, graph: G, vertex: VtkIdType)
    where
        G: Into<OutEdgeIteratorGraphHandle>,
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

    /// VTK: `vtkOutEdgeIterator::GetGraph`.
    pub fn get_graph(&self) -> Option<&OutEdgeIteratorGraphHandle> {
        self.graph.as_ref()
    }

    /// VTK: `vtkOutEdgeIterator::GetVertex`.
    pub fn get_vertex(&self) -> VtkIdType {
        self.vertex
    }

    /// VTK: `vtkOutEdgeIterator::Next`.
    pub fn next(&mut self) -> OutEdge {
        assert!(self.has_next());
        let edge = self.edges[self.current];
        self.current += 1;
        edge
    }

    /// VTK: `vtkOutEdgeIterator::NextGraphEdge`.
    pub fn next_graph_edge(&mut self) -> &GraphEdge {
        let edge = self.next();
        let graph_edge = self.graph_edge.get_or_insert_with(GraphEdge::new);
        graph_edge.set_source(self.vertex);
        graph_edge.set_target(edge.target);
        graph_edge.set_id(edge.id);
        graph_edge
    }

    /// VTK: `vtkOutEdgeIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        self.current != self.edges.len()
    }

    /// VTK protected macro implementation: `vtkOutEdgeIterator::SetGraph`.
    pub(crate) fn set_graph(&mut self, graph: Option<OutEdgeIteratorGraphHandle>) {
        self.graph = graph;
        self.modified();
    }

    /// VTK: `vtkOutEdgeIterator::PrintSelf`.
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

impl Default for OutEdgeIterator {
    fn default() -> Self {
        Self::new()
    }
}
