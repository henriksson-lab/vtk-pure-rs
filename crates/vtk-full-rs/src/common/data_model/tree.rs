use std::sync::Arc;

use crate::common::core::{points::Points, VtkIdType};

#[cfg(test)]
use super::graph::{points_mut_internal, GraphError};
use super::{
    graph::{deep_clone_storage, reorder_out_vertices_internal, DirectedGraph, Edge},
    DataSetAttributes,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    graph: DirectedGraph,
    root: Option<usize>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            graph: DirectedGraph::new(),
            root: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn as_graph(&self) -> DirectedGraph {
        self.graph.clone()
    }

    pub fn get_number_of_vertices(&self) -> VtkIdType {
        self.graph.get_number_of_vertices()
    }

    pub fn get_number_of_edges(&self) -> VtkIdType {
        self.graph.get_number_of_edges()
    }

    pub fn get_root(&self) -> VtkIdType {
        self.root.map_or(-1, |root| root as VtkIdType)
    }

    pub fn set_root(&mut self, root: VtkIdType) {
        self.root = (root >= 0).then_some(root as usize);
    }

    pub fn get_vertex_data(&self) -> &DataSetAttributes {
        self.graph.get_vertex_data()
    }

    #[cfg(test)]
    pub(crate) fn get_vertex_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut Arc::make_mut(self.graph.storage_mut()).vertex_data
    }

    pub fn get_edge_data(&self) -> &DataSetAttributes {
        self.graph.get_edge_data()
    }

    pub fn get_points(&mut self) -> &Points {
        self.graph.get_points()
    }

    #[cfg(test)]
    pub(crate) fn get_points_mut(&mut self) -> &mut Points {
        points_mut_internal(Arc::make_mut(self.graph.storage_mut()))
    }

    #[cfg(test)]
    pub(crate) fn add_root(&mut self) -> usize {
        if let Some(root) = self.root {
            return root;
        }

        let storage = Arc::make_mut(self.graph.storage_mut());
        let root = if storage.topology.adjacency.is_empty() {
            super::graph::add_vertex_internal(storage)
        } else {
            0
        };
        self.root = Some(root);
        root
    }

    #[cfg(test)]
    pub(crate) fn add_child(&mut self, parent: usize) -> Result<usize, GraphError> {
        self.check_vertex(parent)?;
        let storage = Arc::make_mut(self.graph.storage_mut());
        let child = super::graph::add_vertex_internal(storage);
        super::graph::add_edge_internal(storage, parent, child, true)?;
        Ok(child)
    }

    pub fn get_number_of_children(&self, vertex: VtkIdType) -> VtkIdType {
        self.graph.get_out_degree(vertex)
    }

    pub fn get_parent(&self, vertex: VtkIdType) -> VtkIdType {
        let incoming = self.graph.get_in_edges(vertex);
        incoming.first().map_or(-1, |edge| edge.source)
    }

    pub fn get_parent_edge(&self, vertex: VtkIdType) -> Edge {
        let incoming = self.graph.get_in_edges(vertex);
        incoming.first().map_or_else(Edge::default, |edge| Edge {
            source: edge.source,
            target: vertex,
            id: edge.id,
        })
    }

    pub fn get_child(&self, vertex: VtkIdType, child_index: VtkIdType) -> VtkIdType {
        let Some(child_index) = (child_index >= 0).then_some(child_index as usize) else {
            return -1;
        };
        self.graph
            .get_out_edges(vertex)
            .get(child_index)
            .map_or(-1, |edge| edge.target)
    }

    pub fn get_children(&self, vertex: VtkIdType) -> Vec<VtkIdType> {
        self.graph
            .get_out_edges(vertex)
            .into_iter()
            .map(|edge| edge.target)
            .collect()
    }

    pub fn is_leaf(&self, vertex: VtkIdType) -> bool {
        self.get_number_of_children(vertex) == 0
    }

    pub fn get_level(&self, vertex: VtkIdType) -> VtkIdType {
        if vertex < 0 || vertex >= self.get_number_of_vertices() {
            return -1;
        }
        let root = self.get_root();
        if root < 0 {
            return -1;
        }
        let mut level: VtkIdType = 0;
        let mut current = vertex;
        while current != root {
            let parent = self.get_parent(current);
            if parent < 0 {
                return -1;
            }
            current = parent;
            level += 1;
        }
        level
    }

    pub fn reorder_children(&mut self, parent: VtkIdType, children: &[VtkIdType]) {
        if parent < 0 {
            return;
        }
        let children: Vec<usize> = children
            .iter()
            .copied()
            .filter_map(|child| (child >= 0).then_some(child as usize))
            .collect();
        let _ = reorder_out_vertices_internal(
            Arc::make_mut(self.graph.storage_mut()),
            parent as usize,
            &children,
        );
    }

    #[cfg(test)]
    pub(crate) fn is_structure_valid(graph: &DirectedGraph) -> bool {
        Self::validated_graph_root(graph).is_some()
    }

    pub fn checked_shallow_copy(&mut self, graph: &DirectedGraph) -> bool {
        let Some(root) = Self::validated_graph_root(graph) else {
            return false;
        };

        self.graph.shallow_copy(graph);
        self.root = root;
        true
    }

    pub fn initialize(&mut self) {
        self.graph.initialize();
        self.root = None;
    }

    pub fn shallow_copy(&mut self, other: &Self) {
        self.graph.shallow_copy(&other.graph);
        self.root = other.root;
    }

    pub fn deep_copy(&mut self, other: &Self) {
        *self.graph.storage_mut() = Arc::new(deep_clone_storage(other.graph.storage(), true));
        self.root = other.root;
    }

    #[cfg(test)]
    pub(crate) fn shares_structure_with(&self, other: &Self) -> bool {
        self.graph.shares_structure_with(&other.graph)
    }

    pub fn print_self(&self) -> String {
        format!("{}Root: {}\n", self.graph.print_self(), self.get_root())
    }

    #[cfg(test)]
    fn check_vertex(&self, vertex: usize) -> Result<(), GraphError> {
        if vertex < self.get_number_of_vertices() as usize {
            Ok(())
        } else {
            Err(GraphError::VertexOutOfRange {
                vertex,
                number_of_vertices: self.get_number_of_vertices() as usize,
            })
        }
    }

    fn validated_graph_root(graph: &DirectedGraph) -> Option<Option<usize>> {
        let number_of_vertices = graph.get_number_of_vertices();
        if number_of_vertices == 0 {
            return Some(None);
        }
        if graph.get_number_of_edges() != number_of_vertices - 1 {
            return None;
        }

        let mut root = None;
        for vertex in graph.vertices() {
            let in_degree = graph.get_in_degree(vertex as VtkIdType);
            if in_degree > 1 {
                return None;
            }
            if in_degree == 0 && root.replace(vertex).is_some() {
                return None;
            }
        }
        let root = root?;

        let mut visited = vec![false; number_of_vertices as usize];
        let mut stack = vec![root];
        while let Some(vertex) = stack.pop() {
            if visited[vertex] {
                return None;
            }
            visited[vertex] = true;
            for edge in graph.get_out_edges(vertex as VtkIdType) {
                stack.push(edge.target as usize);
            }
        }
        visited
            .into_iter()
            .all(|visited| visited)
            .then_some(Some(root))
    }
}
