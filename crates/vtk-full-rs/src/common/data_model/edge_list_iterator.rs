use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::{DirectedGraph, Edge, GraphEdge, UndirectedGraph};

/// Rust handle used where VTK stores a `vtkGraph*`.
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeListIteratorGraphHandle {
    Directed(DirectedGraph),
    Undirected(UndirectedGraph),
}

impl EdgeListIteratorGraphHandle {
    fn is_directed(&self) -> bool {
        matches!(self, Self::Directed(_))
    }

    fn edges(&self) -> Vec<Edge> {
        match self {
            Self::Directed(graph) => collect_directed_edges(graph),
            Self::Undirected(graph) => collect_undirected_edges(graph),
        }
    }

    fn print_self(&self) -> String {
        match self {
            Self::Directed(graph) => graph.print_self(),
            Self::Undirected(graph) => graph.print_self(),
        }
    }
}

impl From<&DirectedGraph> for EdgeListIteratorGraphHandle {
    fn from(graph: &DirectedGraph) -> Self {
        Self::Directed(graph.clone())
    }
}

impl From<&UndirectedGraph> for EdgeListIteratorGraphHandle {
    fn from(graph: &UndirectedGraph) -> Self {
        Self::Undirected(graph.clone())
    }
}

/// VTK: `vtkEdgeListIterator`.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeListIterator {
    object: Object,
    graph: Option<EdgeListIteratorGraphHandle>,
    edges: Vec<Edge>,
    current: usize,
    vertex: VtkIdType,
    directed: bool,
    graph_edge: Option<GraphEdge>,
}

impl EdgeListIterator {
    /// VTK: `vtkEdgeListIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkEdgeListIterator"),
            graph: None,
            edges: Vec::new(),
            current: 0,
            vertex: 0,
            directed: false,
            graph_edge: None,
        }
    }

    /// VTK: `vtkEdgeListIterator::GetGraph`.
    pub fn get_graph(&self) -> Option<&EdgeListIteratorGraphHandle> {
        self.graph.as_ref()
    }

    /// VTK: `vtkEdgeListIterator::SetGraph`.
    pub fn set_graph<G>(&mut self, graph: G)
    where
        G: Into<EdgeListIteratorGraphHandle>,
    {
        self.set_graph_handle(Some(graph.into()));
    }

    fn set_graph_handle(&mut self, graph: Option<EdgeListIteratorGraphHandle>) {
        self.graph = graph;
        self.current = 0;
        self.vertex = 0;
        self.directed = self
            .graph
            .as_ref()
            .map(EdgeListIteratorGraphHandle::is_directed)
            .unwrap_or(false);
        self.edges = self
            .graph
            .as_ref()
            .map(EdgeListIteratorGraphHandle::edges)
            .unwrap_or_default();
        self.modified();
    }

    /// VTK: `vtkEdgeListIterator::Next`.
    pub fn next(&mut self) -> Edge {
        assert!(self.has_next());
        let edge = self.edges[self.current];
        self.vertex = edge.source;
        self.increment();
        edge
    }

    /// VTK: `vtkEdgeListIterator::NextGraphEdge`.
    pub fn next_graph_edge(&mut self) -> &GraphEdge {
        let edge = self.next();
        let graph_edge = self.graph_edge.get_or_insert_with(GraphEdge::new);
        graph_edge.set_source(edge.source);
        graph_edge.set_target(edge.target);
        graph_edge.set_id(edge.id);
        graph_edge
    }

    /// VTK: `vtkEdgeListIterator::HasNext`.
    pub fn has_next(&self) -> bool {
        self.current < self.edges.len()
    }

    /// VTK protected helper: `vtkEdgeListIterator::Increment`.
    pub(crate) fn increment(&mut self) {
        if self.current < self.edges.len() {
            self.current += 1;
        }
        if let Some(edge) = self.edges.get(self.current) {
            self.vertex = edge.source;
        }
    }

    /// VTK: `vtkEdgeListIterator::PrintSelf`.
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

impl Default for EdgeListIterator {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_directed_edges(graph: &DirectedGraph) -> Vec<Edge> {
    let mut edges = Vec::new();
    for source in 0..graph.get_number_of_vertices() {
        edges.extend(graph.get_out_edges(source).into_iter().map(|edge| Edge {
            source,
            target: edge.target,
            id: edge.id,
        }));
    }
    edges
}

fn collect_undirected_edges(graph: &UndirectedGraph) -> Vec<Edge> {
    let mut edges = Vec::new();
    for source in 0..graph.get_number_of_vertices() {
        edges.extend(
            graph
                .get_out_edges(source)
                .into_iter()
                .filter(move |edge| source <= edge.target)
                .map(|edge| Edge {
                    source,
                    target: edge.target,
                    id: edge.id,
                }),
        );
    }
    edges
}
