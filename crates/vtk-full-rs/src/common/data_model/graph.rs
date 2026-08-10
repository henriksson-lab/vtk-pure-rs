use std::{collections::HashSet, marker::PhantomData, mem, ops::Range, sync::Arc};

#[cfg(test)]
use crate::common::core::IdTypeArray;
use crate::common::core::{points::Points, VtkIdType};

use super::{BoundingBox, DataSetAttributes, FieldData, Variant, EDGE, VERTEX};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Edge {
    pub source: VtkIdType,
    pub target: VtkIdType,
    pub id: VtkIdType,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutEdge {
    pub target: VtkIdType,
    pub id: VtkIdType,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InEdge {
    pub source: VtkIdType,
    pub id: VtkIdType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(crate) enum GraphDirection {
    Directed,
    Undirected,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Directed;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Undirected;

pub(crate) trait Direction: Clone + std::fmt::Debug + Default + PartialEq + Eq {
    const DIRECTED: bool;
    #[cfg(test)]
    const DIRECTION: GraphDirection;
}

impl Direction for Directed {
    const DIRECTED: bool = true;
    #[cfg(test)]
    const DIRECTION: GraphDirection = GraphDirection::Directed;
}

impl Direction for Undirected {
    const DIRECTED: bool = false;
    #[cfg(test)]
    const DIRECTION: GraphDirection = GraphDirection::Undirected;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HalfEdge {
    pub vertex: usize,
    pub edge_id: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct VertexAdjacency {
    pub out_edges: Vec<HalfEdge>,
    pub in_edges: Vec<HalfEdge>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct GraphTopology {
    pub adjacency: Vec<VertexAdjacency>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct GraphEdgePoints {
    edges: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GraphStorage {
    pub topology: Arc<GraphTopology>,
    pub modified_time: u64,
    pub vertex_data: DataSetAttributes,
    pub edge_data: DataSetAttributes,
    pub points: Option<Points>,
    pub edge_points: Arc<GraphEdgePoints>,
}

impl Default for GraphStorage {
    fn default() -> Self {
        Self {
            topology: Arc::new(GraphTopology::default()),
            modified_time: 0,
            vertex_data: DataSetAttributes::new(),
            edge_data: DataSetAttributes::new(),
            points: None,
            edge_points: Arc::new(GraphEdgePoints::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    #[error("vertex {vertex} out of range for graph with {number_of_vertices} vertices")]
    VertexOutOfRange {
        vertex: usize,
        number_of_vertices: usize,
    },
    #[error("edge {edge} out of range for graph with {number_of_edges} edges")]
    EdgeOutOfRange { edge: usize, number_of_edges: usize },
    #[error("reorder list does not match current outgoing adjacency for vertex {vertex}")]
    InvalidReorder { vertex: usize },
    #[error("property array count {properties} does not match attribute array count {arrays}")]
    PropertyArrayCountMismatch { properties: usize, arrays: usize },
    #[error("graph has no active vertex pedigree ID array")]
    MissingPedigreeIds,
    #[error("tree root is not set")]
    MissingTreeRoot,
    #[error("tree cannot add a second parent for vertex {vertex}")]
    DuplicateTreeParent { vertex: usize },
}

#[derive(Debug, PartialEq)]
pub(crate) struct GraphImpl<D: Direction> {
    storage: Arc<GraphStorage>,
    direction: PhantomData<D>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirectedGraph {
    inner: GraphImpl<Directed>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UndirectedGraph {
    inner: GraphImpl<Undirected>,
}

macro_rules! impl_graph_wrapper {
    ($name:ident, $direction:ty) => {
        impl $name {
            pub fn new() -> Self {
                Self {
                    inner: GraphImpl::new(),
                }
            }

            pub(crate) fn from_storage(storage: Arc<GraphStorage>) -> Self {
                Self {
                    inner: GraphImpl::<$direction>::from_storage(storage),
                }
            }

            pub(crate) fn into_storage(self) -> Arc<GraphStorage> {
                self.inner.storage
            }

            pub(super) fn storage(&self) -> &Arc<GraphStorage> {
                &self.inner.storage
            }

            pub(super) fn storage_mut(&mut self) -> &mut Arc<GraphStorage> {
                &mut self.inner.storage
            }

            #[cfg(test)]
            pub(crate) fn direction(&self) -> GraphDirection {
                self.inner.direction()
            }

            #[cfg(test)]
            pub(crate) fn is_directed(&self) -> bool {
                self.inner.is_directed()
            }

            pub fn get_number_of_vertices(&self) -> VtkIdType {
                self.inner.get_number_of_vertices()
            }

            pub fn get_number_of_edges(&self) -> VtkIdType {
                self.inner.get_number_of_edges()
            }

            pub(super) fn vertices(&self) -> Range<usize> {
                self.inner.vertices()
            }

            #[cfg(test)]
            pub(crate) fn edges(&self) -> &[Edge] {
                self.inner.edges()
            }

            #[cfg(test)]
            fn edge_list(&self) -> Vec<[usize; 2]> {
                self.inner.edge_list()
            }

            #[cfg(test)]
            pub(crate) fn get_edge_list(&self) -> IdTypeArray {
                self.inner.get_edge_list()
            }

            #[cfg(test)]
            pub(crate) fn build_edge_list(&self) -> IdTypeArray {
                self.inner.build_edge_list()
            }

            pub fn get_source_vertex(&self, edge_id: VtkIdType) -> VtkIdType {
                self.inner.get_source_vertex(edge_id)
            }

            pub fn get_target_vertex(&self, edge_id: VtkIdType) -> VtkIdType {
                self.inner.get_target_vertex(edge_id)
            }

            #[cfg(test)]
            pub(crate) fn edge(&self, edge_id: usize) -> Result<Edge, GraphError> {
                self.inner.edge(edge_id)
            }

            pub fn get_edge_id(&self, a: VtkIdType, b: VtkIdType) -> VtkIdType {
                self.inner.get_edge_id(a, b)
            }

            pub fn get_out_edges(&self, vertex: VtkIdType) -> Vec<OutEdge> {
                self.inner.get_out_edges(vertex)
            }

            pub fn get_in_edges(&self, vertex: VtkIdType) -> Vec<InEdge> {
                self.inner.get_in_edges(vertex)
            }

            pub fn get_out_edge(&self, vertex: VtkIdType, adjacency_index: VtkIdType) -> OutEdge {
                self.inner.get_out_edge(vertex, adjacency_index)
            }

            pub fn get_in_edge(&self, vertex: VtkIdType, adjacency_index: VtkIdType) -> InEdge {
                self.inner.get_in_edge(vertex, adjacency_index)
            }

            #[cfg(test)]
            pub(crate) fn get_out_vertex(
                &self,
                vertex: usize,
                adjacency_index: usize,
            ) -> Result<Option<usize>, GraphError> {
                self.inner.get_out_vertex(vertex, adjacency_index)
            }

            #[cfg(test)]
            pub(crate) fn get_in_vertex(
                &self,
                vertex: usize,
                adjacency_index: usize,
            ) -> Result<Option<usize>, GraphError> {
                self.inner.get_in_vertex(vertex, adjacency_index)
            }

            #[cfg(test)]
            pub(crate) fn get_opposite_vertex(
                &self,
                edge_id: VtkIdType,
                vertex: usize,
            ) -> Result<Option<usize>, GraphError> {
                self.inner.get_opposite_vertex(edge_id, vertex)
            }

            pub fn get_out_degree(&self, vertex: VtkIdType) -> VtkIdType {
                self.inner.get_out_degree(vertex)
            }

            pub fn get_in_degree(&self, vertex: VtkIdType) -> VtkIdType {
                self.inner.get_in_degree(vertex)
            }

            pub fn get_degree(&self, vertex: VtkIdType) -> VtkIdType {
                self.inner.get_degree(vertex)
            }

            pub fn get_adjacent_vertices(&self, vertex: VtkIdType) -> Vec<VtkIdType> {
                self.inner.get_adjacent_vertices(vertex)
            }

            pub fn find_vertex(&self, pedigree_id: &Variant) -> VtkIdType {
                self.inner.find_vertex(pedigree_id)
            }

            pub fn get_vertex_data(&self) -> &DataSetAttributes {
                self.inner.get_vertex_data()
            }

            pub fn get_edge_data(&self) -> &DataSetAttributes {
                self.inner.get_edge_data()
            }

            pub fn get_points(&mut self) -> &Points {
                self.inner.get_points()
            }

            pub fn get_point(&self, vertex: VtkIdType) -> [f64; 3] {
                self.inner.get_point(vertex)
            }

            pub fn get_bounds(&self) -> [f64; 6] {
                self.inner.get_bounds()
            }

            pub fn get_edge_points(&self, edge_id: VtkIdType) -> &[f64] {
                self.inner.get_edge_points(edge_id)
            }

            #[cfg(test)]
            pub(crate) fn edge_points_as_triples(
                &self,
                edge_id: VtkIdType,
            ) -> Result<Vec<[f64; 3]>, GraphError> {
                self.inner.edge_points_as_triples(edge_id)
            }

            #[cfg(test)]
            pub(crate) fn edge_points_flat(&self, edge_id: VtkIdType) -> &[f64] {
                self.inner.edge_points_flat(edge_id)
            }

            pub fn get_edge_point(
                &self,
                edge_id: VtkIdType,
                point_index: VtkIdType,
            ) -> Option<[f64; 3]> {
                self.inner.get_edge_point(edge_id, point_index)
            }

            pub fn get_number_of_edge_points(&self, edge_id: VtkIdType) -> VtkIdType {
                self.inner.get_number_of_edge_points(edge_id)
            }

            pub fn get_induced_edges(
                &self,
                vertices: &[usize],
            ) -> Result<Vec<VtkIdType>, GraphError> {
                self.inner.get_induced_edges(vertices)
            }

            pub fn get_attributes_as_field_data(&self, attribute_type: i32) -> Option<&FieldData> {
                self.inner.get_attributes_as_field_data(attribute_type)
            }

            pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
                self.inner.get_number_of_elements(attribute_type)
            }

            pub fn get_actual_memory_size(&self) -> usize {
                self.inner.get_actual_memory_size()
            }

            pub fn get_m_time(&self) -> u64 {
                self.inner.get_m_time()
            }

            pub fn shallow_copy(&mut self, other: &Self) {
                self.inner.shallow_copy(&other.inner);
            }

            pub fn deep_copy(&mut self, other: &Self) {
                self.inner.deep_copy(&other.inner);
            }

            #[cfg(test)]
            pub(crate) fn copy_internal_from(&mut self, other: &Self, deep: bool) {
                self.inner.copy_internal_from(&other.inner, deep);
            }

            pub fn copy_structure(&mut self, other: &Self) {
                self.inner.copy_structure(&other.inner);
            }

            pub fn shallow_copy_edge_points(&mut self, other: &Self) {
                self.inner.shallow_copy_edge_points(&other.inner);
            }

            pub fn deep_copy_edge_points(&mut self, other: &Self) {
                self.inner.deep_copy_edge_points(&other.inner);
            }

            #[cfg(test)]
            pub(crate) fn shares_edge_points_with(&self, other: &Self) -> bool {
                self.inner.shares_edge_points_with(&other.inner)
            }

            #[cfg(test)]
            pub(crate) fn force_ownership(&mut self) {
                self.inner.force_ownership();
            }

            pub fn squeeze(&mut self) {
                self.inner.squeeze();
            }

            pub fn dump(&self) -> String {
                self.inner.dump()
            }

            pub fn print_self(&self) -> String {
                self.inner.print_self()
            }

            #[cfg(test)]
            pub(crate) fn shares_structure_with(&self, other: &Self) -> bool {
                self.inner.shares_structure_with(&other.inner)
            }

            pub fn is_same_structure(&self, other: &Self) -> bool {
                self.inner.is_same_structure(&other.inner)
            }

            pub fn initialize(&mut self) {
                self.inner.initialize();
            }
        }
    };
}

impl_graph_wrapper!(DirectedGraph, Directed);
impl_graph_wrapper!(UndirectedGraph, Undirected);

impl DirectedGraph {
    #[cfg(test)]
    pub(crate) fn to_undirected_graph(&self) -> UndirectedGraph {
        self.inner.to_undirected_graph()
    }
}

impl UndirectedGraph {
    #[cfg(test)]
    pub(crate) fn to_directed_graph(&self) -> DirectedGraph {
        self.inner.to_directed_graph()
    }
}

impl<D: Direction> Clone for GraphImpl<D> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            direction: PhantomData,
        }
    }
}

impl<D: Direction> Default for GraphImpl<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphEdgePoints {
    fn resize_edges(&mut self, number_of_edges: usize) {
        self.edges.resize_with(number_of_edges, Vec::new);
    }

    fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    fn edge_flat(&self, edge_id: usize) -> &[f64] {
        self.edges.get(edge_id).map_or(&[], Vec::as_slice)
    }

    #[cfg(test)]
    fn edge_points(&self, edge_id: usize) -> Vec<[f64; 3]> {
        self.edge_flat(edge_id)
            .chunks_exact(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect()
    }

    fn number_of_edge_points(&self, edge_id: usize) -> usize {
        self.edge_flat(edge_id).len() / 3
    }

    fn edge_point(&self, edge_id: usize, point_index: usize) -> Option<[f64; 3]> {
        let start = point_index.checked_mul(3)?;
        let values = self.edge_flat(edge_id).get(start..start + 3)?;
        Some([values[0], values[1], values[2]])
    }

    fn set_edge_points_flat(&mut self, edge_id: usize, values: Vec<f64>) {
        self.edges[edge_id] = values;
    }

    #[cfg(test)]
    fn set_edge_points(&mut self, edge_id: usize, points: Vec<[f64; 3]>) {
        let mut values = Vec::with_capacity(points.len() * 3);
        for point in points {
            values.extend(point);
        }
        self.set_edge_points_flat(edge_id, values);
    }

    fn add_edge_point(&mut self, edge_id: usize, point: [f64; 3]) {
        self.edges[edge_id].extend(point);
    }

    fn set_edge_point(&mut self, edge_id: usize, point_index: usize, point: [f64; 3]) -> bool {
        let start = point_index * 3;
        let values = &mut self.edges[edge_id];
        if start + 3 > values.len() {
            return false;
        }
        values[start..start + 3].copy_from_slice(&point);
        true
    }

    fn clear_edge_points(&mut self, edge_id: usize) {
        self.edges[edge_id].clear();
    }

    fn swap_remove(&mut self, edge_id: usize) {
        self.edges.swap_remove(edge_id);
    }

    fn squeeze(&mut self) {
        self.edges.shrink_to_fit();
        for edge in &mut self.edges {
            edge.shrink_to_fit();
        }
    }

    fn actual_memory_size_bytes(&self) -> usize {
        self.edges
            .iter()
            .map(|points| points.capacity() * mem::size_of::<f64>())
            .sum()
    }
}

impl<D: Direction> GraphImpl<D> {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(GraphStorage::default()),
            direction: PhantomData,
        }
    }

    pub(crate) fn from_storage(storage: Arc<GraphStorage>) -> Self {
        Self {
            storage,
            direction: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn direction(&self) -> GraphDirection {
        D::DIRECTION
    }

    #[cfg(test)]
    pub(crate) fn is_directed(&self) -> bool {
        D::DIRECTED
    }

    fn number_of_vertices(&self) -> usize {
        self.storage.topology.adjacency.len()
    }

    pub fn get_number_of_vertices(&self) -> VtkIdType {
        self.number_of_vertices() as VtkIdType
    }

    fn number_of_edges(&self) -> usize {
        self.storage.topology.edges.len()
    }

    pub fn get_number_of_edges(&self) -> VtkIdType {
        self.number_of_edges() as VtkIdType
    }

    fn vertices(&self) -> Range<usize> {
        0..self.number_of_vertices()
    }

    #[cfg(test)]
    pub(crate) fn edges(&self) -> &[Edge] {
        &self.storage.topology.edges
    }

    #[cfg(test)]
    fn edge_list(&self) -> Vec<[usize; 2]> {
        self.storage
            .topology
            .edges
            .iter()
            .map(|edge| [edge.source as usize, edge.target as usize])
            .collect()
    }

    /// VTK: private/protected `vtkGraph::GetEdgeList` / `vtkGraph::BuildEdgeList`.
    #[cfg(test)]
    pub(crate) fn get_edge_list(&self) -> IdTypeArray {
        let mut values = Vec::with_capacity(self.number_of_edges() * 2);
        for edge in &self.storage.topology.edges {
            values.push(edge.source as i64);
            values.push(edge.target as i64);
        }
        IdTypeArray::from_vec("EdgeList", values, 2)
    }

    /// VTK: `vtkGraph::BuildEdgeList`.
    #[cfg(test)]
    pub(crate) fn build_edge_list(&self) -> IdTypeArray {
        self.get_edge_list()
    }

    /// VTK: `vtkGraph::GetSourceVertex`.
    pub fn get_source_vertex(&self, edge_id: VtkIdType) -> VtkIdType {
        self.edge_id_to_index(edge_id)
            .and_then(|edge_id| self.storage.topology.edges.get(edge_id))
            .map_or(-1, |edge| edge.source)
    }

    /// VTK: `vtkGraph::GetTargetVertex`.
    pub fn get_target_vertex(&self, edge_id: VtkIdType) -> VtkIdType {
        self.edge_id_to_index(edge_id)
            .and_then(|edge_id| self.storage.topology.edges.get(edge_id))
            .map_or(-1, |edge| edge.target)
    }

    #[cfg(test)]
    pub(crate) fn edge(&self, edge_id: usize) -> Result<Edge, GraphError> {
        self.storage
            .topology
            .edges
            .get(edge_id)
            .copied()
            .ok_or(GraphError::EdgeOutOfRange {
                edge: edge_id,
                number_of_edges: self.number_of_edges(),
            })
    }

    pub fn get_edge_id(&self, a: VtkIdType, b: VtkIdType) -> VtkIdType {
        let (Some(a), Some(b)) = (self.vertex_id_to_index(a), self.vertex_id_to_index(b)) else {
            return -1;
        };
        let Ok(adjacency) = self.vertex_adjacency(a) else {
            return -1;
        };
        for half_edge in &adjacency.in_edges {
            if half_edge.vertex == b {
                return half_edge.edge_id as VtkIdType;
            }
        }
        for half_edge in &adjacency.out_edges {
            if half_edge.vertex == b {
                return half_edge.edge_id as VtkIdType;
            }
        }
        -1
    }

    pub fn get_out_edges(&self, vertex: VtkIdType) -> Vec<OutEdge> {
        let Some(vertex) = self.vertex_id_to_index(vertex) else {
            return Vec::new();
        };
        self.storage.topology.adjacency[vertex]
            .out_edges
            .iter()
            .map(|half_edge| OutEdge {
                target: half_edge.vertex as VtkIdType,
                id: half_edge.edge_id as VtkIdType,
            })
            .collect()
    }

    pub fn get_in_edges(&self, vertex: VtkIdType) -> Vec<InEdge> {
        let Some(vertex) = self.vertex_id_to_index(vertex) else {
            return Vec::new();
        };
        let adjacency = &self.storage.topology.adjacency[vertex];
        let half_edges = if D::DIRECTED {
            &adjacency.in_edges
        } else {
            &adjacency.out_edges
        };
        half_edges
            .iter()
            .map(|half_edge| InEdge {
                source: half_edge.vertex as VtkIdType,
                id: half_edge.edge_id as VtkIdType,
            })
            .collect()
    }

    pub fn get_out_edge(&self, vertex: VtkIdType, adjacency_index: VtkIdType) -> OutEdge {
        let Some(vertex) = self.vertex_id_to_index(vertex) else {
            return OutEdge::default();
        };
        let Some(adjacency_index) = (adjacency_index >= 0).then_some(adjacency_index as usize)
        else {
            return OutEdge::default();
        };
        self.storage.topology.adjacency[vertex]
            .out_edges
            .get(adjacency_index)
            .map(|half_edge| OutEdge {
                target: half_edge.vertex as VtkIdType,
                id: half_edge.edge_id as VtkIdType,
            })
            .unwrap_or_default()
    }

    pub fn get_in_edge(&self, vertex: VtkIdType, adjacency_index: VtkIdType) -> InEdge {
        let Some(vertex) = self.vertex_id_to_index(vertex) else {
            return InEdge::default();
        };
        let Some(adjacency_index) = (adjacency_index >= 0).then_some(adjacency_index as usize)
        else {
            return InEdge::default();
        };
        let adjacency = &self.storage.topology.adjacency[vertex];
        let half_edges = if D::DIRECTED {
            &adjacency.in_edges
        } else {
            &adjacency.out_edges
        };
        half_edges
            .get(adjacency_index)
            .map(|half_edge| InEdge {
                source: half_edge.vertex as VtkIdType,
                id: half_edge.edge_id as VtkIdType,
            })
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub fn get_out_vertex(
        &self,
        vertex: usize,
        adjacency_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        Ok(self
            .vertex_adjacency(vertex)?
            .out_edges
            .get(adjacency_index)
            .map(|half_edge| half_edge.vertex))
    }

    #[cfg(test)]
    pub fn get_in_vertex(
        &self,
        vertex: usize,
        adjacency_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        let adjacency = self.vertex_adjacency(vertex)?;
        let half_edges = if D::DIRECTED {
            &adjacency.in_edges
        } else {
            &adjacency.out_edges
        };
        Ok(half_edges
            .get(adjacency_index)
            .map(|half_edge| half_edge.vertex))
    }

    #[cfg(test)]
    pub fn get_opposite_vertex(
        &self,
        edge_id: VtkIdType,
        vertex: usize,
    ) -> Result<Option<usize>, GraphError> {
        let edge = self.edge(self.validate_edge_id(edge_id)?)?;
        self.vertex_adjacency(vertex)?;
        if edge.source == vertex as VtkIdType {
            Ok(Some(edge.target as usize))
        } else if edge.target == vertex as VtkIdType {
            Ok(Some(edge.source as usize))
        } else {
            Ok(None)
        }
    }

    pub fn get_out_degree(&self, vertex: VtkIdType) -> VtkIdType {
        self.vertex_id_to_index(vertex)
            .map(|index| self.storage.topology.adjacency[index].out_edges.len() as VtkIdType)
            .unwrap_or(0)
    }

    pub fn get_in_degree(&self, vertex: VtkIdType) -> VtkIdType {
        let Some(index) = self.vertex_id_to_index(vertex) else {
            return 0;
        };
        let adjacency = &self.storage.topology.adjacency[index];
        if D::DIRECTED {
            adjacency.in_edges.len() as VtkIdType
        } else {
            adjacency.out_edges.len() as VtkIdType
        }
    }

    pub fn get_degree(&self, vertex: VtkIdType) -> VtkIdType {
        let Some(index) = self.vertex_id_to_index(vertex) else {
            return 0;
        };
        let adjacency = &self.storage.topology.adjacency[index];
        if D::DIRECTED {
            (adjacency.out_edges.len() + adjacency.in_edges.len()) as VtkIdType
        } else {
            adjacency.out_edges.len() as VtkIdType
        }
    }

    pub fn get_adjacent_vertices(&self, vertex: VtkIdType) -> Vec<VtkIdType> {
        let Some(vertex) = self.vertex_id_to_index(vertex) else {
            return Vec::new();
        };
        self.storage.topology.adjacency[vertex]
            .out_edges
            .iter()
            .map(|half_edge| half_edge.vertex as VtkIdType)
            .collect()
    }

    pub fn find_vertex(&self, pedigree_id: &Variant) -> VtkIdType {
        let Some(pedigrees) = self.get_vertex_data().get_field_data_pedigree_ids() else {
            return -1;
        };
        let components = pedigrees.get_number_of_components();
        pedigrees
            .values_as_variants()
            .iter()
            .position(|value| value == pedigree_id)
            .map(|value_index| value_index / components)
            .filter(|&vertex| vertex < self.number_of_vertices())
            .map_or(-1, |vertex| vertex as VtkIdType)
    }

    pub fn get_vertex_data(&self) -> &DataSetAttributes {
        &self.storage.vertex_data
    }

    pub fn get_edge_data(&self) -> &DataSetAttributes {
        &self.storage.edge_data
    }

    pub fn get_points(&mut self) -> &Points {
        points_mut_internal(Arc::make_mut(&mut self.storage))
    }

    pub fn get_point(&self, vertex: VtkIdType) -> [f64; 3] {
        self.storage
            .points
            .as_ref()
            .map_or([0.0; 3], |points| points.get_point(vertex))
    }

    pub fn get_bounds(&self) -> [f64; 6] {
        self.storage
            .points
            .as_ref()
            .map(Points::get_bounds)
            .unwrap_or_else(|| BoundingBox::empty().get_bounds())
    }

    /// VTK: `vtkGraph::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &[f64] {
        let Some(edge_id) = self.edge_id_to_index(edge_id) else {
            return &[];
        };
        self.storage.edge_points.edge_flat(edge_id)
    }

    #[cfg(test)]
    pub(crate) fn edge_points_as_triples(
        &self,
        edge_id: VtkIdType,
    ) -> Result<Vec<[f64; 3]>, GraphError> {
        let edge_id = self.validate_edge_id(edge_id)?;
        Ok(self.storage.edge_points.edge_points(edge_id))
    }

    #[cfg(test)]
    pub(crate) fn edge_points_flat(&self, edge_id: VtkIdType) -> &[f64] {
        self.get_edge_points(edge_id)
    }

    pub fn get_edge_point(&self, edge_id: VtkIdType, point_index: VtkIdType) -> Option<[f64; 3]> {
        let edge_id = self.edge_id_to_index(edge_id)?;
        if point_index < 0 {
            return None;
        }
        self.storage
            .edge_points
            .edge_point(edge_id, point_index as usize)
    }

    pub fn get_number_of_edge_points(&self, edge_id: VtkIdType) -> VtkIdType {
        self.edge_id_to_index(edge_id)
            .map(|edge_id| self.storage.edge_points.number_of_edge_points(edge_id) as VtkIdType)
            .unwrap_or(0)
    }

    pub fn get_induced_edges(&self, vertices: &[usize]) -> Result<Vec<VtkIdType>, GraphError> {
        let mut vertex_set = HashSet::with_capacity(vertices.len());
        for &vertex in vertices {
            self.vertex_adjacency(vertex)?;
            vertex_set.insert(vertex);
        }
        Ok(self
            .storage
            .topology
            .edges
            .iter()
            .filter(|edge| {
                vertex_set.contains(&(edge.source as usize))
                    && vertex_set.contains(&(edge.target as usize))
            })
            .map(|edge| edge.id)
            .collect())
    }

    pub fn get_attributes_as_field_data(&self, attribute_type: i32) -> Option<&FieldData> {
        match attribute_type {
            VERTEX => Some(self.get_vertex_data().field_data()),
            EDGE => Some(self.get_edge_data().field_data()),
            _ => None,
        }
    }

    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            VERTEX => self.get_number_of_vertices(),
            EDGE => self.get_number_of_edges(),
            _ => 0,
        }
    }

    pub fn get_actual_memory_size(&self) -> usize {
        let topology_bytes = self.storage.topology.adjacency.capacity()
            * mem::size_of::<VertexAdjacency>()
            + self.storage.topology.edges.capacity() * mem::size_of::<Edge>();
        let edge_point_bytes = self.storage.edge_points.actual_memory_size_bytes();
        let attribute_kib = self.get_vertex_data().get_actual_memory_size()
            + self.get_edge_data().get_actual_memory_size();
        let point_kib = self
            .storage
            .points
            .as_ref()
            .map_or(0, Points::get_actual_memory_size);
        (topology_bytes + edge_point_bytes).div_ceil(1024) + attribute_kib + point_kib
    }

    pub fn get_m_time(&self) -> u64 {
        let mut time = self.storage.modified_time;
        time = time.max(self.get_vertex_data().get_m_time());
        time = time.max(self.get_edge_data().get_m_time());
        if let Some(points) = self.storage.points.as_ref() {
            time = time.max(points.get_m_time());
        }
        time
    }

    pub fn shallow_copy(&mut self, other: &Self) {
        self.storage = Arc::clone(&other.storage);
    }

    pub fn deep_copy(&mut self, other: &Self) {
        self.storage = Arc::new(deep_clone_storage(&other.storage, D::DIRECTED));
    }

    #[cfg(test)]
    pub(crate) fn copy_internal_from(&mut self, other: &Self, deep: bool) {
        if deep {
            self.deep_copy(other);
        } else {
            self.shallow_copy(other);
        }
    }

    pub fn copy_structure(&mut self, other: &Self) {
        self.storage = Arc::new(GraphStorage {
            topology: Arc::clone(&other.storage.topology),
            modified_time: self.storage.modified_time.saturating_add(1),
            vertex_data: self.storage.vertex_data.shallow_clone(),
            edge_data: self.storage.edge_data.shallow_clone(),
            points: other.storage.points.clone(),
            edge_points: Arc::clone(&self.storage.edge_points),
        });
    }

    pub fn shallow_copy_edge_points(&mut self, other: &Self) {
        let storage = Arc::make_mut(&mut self.storage);
        storage.edge_points = Arc::clone(&other.storage.edge_points);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn deep_copy_edge_points(&mut self, other: &Self) {
        let storage = Arc::make_mut(&mut self.storage);
        storage.edge_points = Arc::new((*other.storage.edge_points).clone());
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    #[cfg(test)]
    pub(crate) fn shares_edge_points_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage.edge_points, &other.storage.edge_points)
    }

    #[cfg(test)]
    pub(crate) fn force_ownership(&mut self) {
        let storage = Arc::make_mut(&mut self.storage);
        if Arc::strong_count(&storage.topology) > 1 {
            storage.topology = Arc::new((*storage.topology).clone());
        }
        if Arc::strong_count(&storage.edge_points) > 1 {
            storage.edge_points = Arc::new((*storage.edge_points).clone());
        }
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn squeeze(&mut self) {
        let storage = Arc::make_mut(&mut self.storage);
        let topology = Arc::make_mut(&mut storage.topology);
        topology.adjacency.shrink_to_fit();
        topology.edges.shrink_to_fit();
        storage.vertex_data.squeeze();
        storage.edge_data.squeeze();
        if let Some(points) = &mut storage.points {
            points.squeeze();
        }
        Arc::make_mut(&mut storage.edge_points).squeeze();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub(crate) fn dump(&self) -> String {
        use std::fmt::Write as _;

        let mut output = String::new();
        output.push_str("vertex adjacency:\n");
        for (vertex, adjacency) in self.storage.topology.adjacency.iter().enumerate() {
            let _ = write!(output, "{vertex} (out): ");
            for edge in &adjacency.out_edges {
                let _ = write!(output, "[{},{}]", edge.edge_id, edge.vertex);
            }
            output.push_str(" (in): ");
            for edge in &adjacency.in_edges {
                let _ = write!(output, "[{},{}]", edge.edge_id, edge.vertex);
            }
            output.push('\n');
        }
        output.push_str("edge list:\n");
        for edge in &self.storage.topology.edges {
            let _ = writeln!(output, "{}: ({},{})", edge.id, edge.source, edge.target);
        }
        output
    }

    pub fn print_self(&self) -> String {
        format!(
            "Graph\nVertexData: {} arrays\nEdgeData: {} arrays\nDistributedHelper: (none)\n",
            self.get_vertex_data().get_number_of_arrays(),
            self.get_edge_data().get_number_of_arrays()
        )
    }

    #[cfg(test)]
    pub(crate) fn shares_structure_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage.topology, &other.storage.topology)
    }

    pub(crate) fn is_same_structure(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage.topology, &other.storage.topology)
    }

    pub fn initialize(&mut self) {
        self.storage = Arc::new(GraphStorage::default());
    }

    #[cfg(test)]
    pub(crate) fn to_directed_graph(&self) -> DirectedGraph {
        if D::DIRECTED {
            DirectedGraph::from_storage(Arc::clone(&self.storage))
        } else {
            DirectedGraph::from_storage(Arc::new(convert_storage_direction(&self.storage, true)))
        }
    }

    #[cfg(test)]
    pub(crate) fn to_undirected_graph(&self) -> UndirectedGraph {
        if D::DIRECTED {
            UndirectedGraph::from_storage(Arc::new(convert_storage_direction(&self.storage, false)))
        } else {
            UndirectedGraph::from_storage(Arc::clone(&self.storage))
        }
    }

    fn vertex_adjacency(&self, vertex: usize) -> Result<&VertexAdjacency, GraphError> {
        self.storage
            .topology
            .adjacency
            .get(vertex)
            .ok_or(GraphError::VertexOutOfRange {
                vertex,
                number_of_vertices: self.number_of_vertices(),
            })
    }

    fn vertex_id_to_index(&self, vertex: VtkIdType) -> Option<usize> {
        (vertex >= 0)
            .then_some(vertex as usize)
            .filter(|&vertex| vertex < self.number_of_vertices())
    }

    fn edge_id_to_index(&self, edge: VtkIdType) -> Option<usize> {
        (edge >= 0)
            .then_some(edge as usize)
            .filter(|&edge| edge < self.number_of_edges())
    }

    #[cfg(test)]
    fn validate_edge_id(&self, edge: VtkIdType) -> Result<usize, GraphError> {
        let edge_id = edge.max(0) as usize;
        self.edge_id_to_index(edge)
            .ok_or(GraphError::EdgeOutOfRange {
                edge: edge_id,
                number_of_edges: self.number_of_edges(),
            })
    }
}

pub(crate) fn add_vertex_internal(storage: &mut GraphStorage) -> usize {
    let topology = Arc::make_mut(&mut storage.topology);
    let vertex = topology.adjacency.len();
    topology.adjacency.push(VertexAdjacency::default());
    let number_of_vertices = topology.adjacency.len();
    storage
        .vertex_data
        .set_number_of_tuples(number_of_vertices as VtkIdType);
    if let Some(points) = &mut storage.points {
        points.set_number_of_points(number_of_vertices as VtkIdType);
    }
    storage.modified_time = storage.modified_time.saturating_add(1);
    vertex
}

pub(crate) fn resize_number_of_vertices_internal(
    storage: &mut GraphStorage,
    count: usize,
) -> usize {
    let topology = Arc::make_mut(&mut storage.topology);
    let previous = topology.adjacency.len();
    topology
        .adjacency
        .resize_with(count, VertexAdjacency::default);
    storage.vertex_data.set_number_of_tuples(count as VtkIdType);
    if let Some(points) = &mut storage.points {
        points.set_number_of_points(count as VtkIdType);
    }
    storage.modified_time = storage.modified_time.saturating_add(1);
    previous
}

pub(crate) fn add_edge_internal(
    storage: &mut GraphStorage,
    source: usize,
    target: usize,
    directed: bool,
) -> Result<Edge, GraphError> {
    validate_vertex(storage, source)?;
    validate_vertex(storage, target)?;

    let topology = Arc::make_mut(&mut storage.topology);
    let edge_id = topology.edges.len();
    let edge = Edge {
        source: source as VtkIdType,
        target: target as VtkIdType,
        id: edge_id as VtkIdType,
    };
    topology.edges.push(edge);
    let number_of_edges = topology.edges.len();
    storage
        .edge_data
        .set_number_of_tuples(number_of_edges as VtkIdType);
    Arc::make_mut(&mut storage.edge_points).resize_edges(number_of_edges);
    topology.adjacency[source].out_edges.push(HalfEdge {
        vertex: target,
        edge_id,
    });
    if directed {
        topology.adjacency[target].in_edges.push(HalfEdge {
            vertex: source,
            edge_id,
        });
    } else if source != target {
        topology.adjacency[target].out_edges.push(HalfEdge {
            vertex: source,
            edge_id,
        });
    }
    storage.modified_time = storage.modified_time.saturating_add(1);
    Ok(edge)
}

pub(crate) fn add_edge_value_internal(
    storage: &mut GraphStorage,
    source: VtkIdType,
    target: VtkIdType,
    directed: bool,
) -> Edge {
    if source < 0 || target < 0 {
        return Edge::default();
    }
    add_edge_internal(storage, source as usize, target as usize, directed).unwrap_or_default()
}

pub(crate) fn validate_vertex(storage: &GraphStorage, vertex: usize) -> Result<(), GraphError> {
    if vertex < storage.topology.adjacency.len() {
        Ok(())
    } else {
        Err(GraphError::VertexOutOfRange {
            vertex,
            number_of_vertices: storage.topology.adjacency.len(),
        })
    }
}

pub(crate) fn deep_clone_storage(source: &GraphStorage, directed: bool) -> GraphStorage {
    let mut vertex_data = DataSetAttributes::new();
    vertex_data.deep_copy(&source.vertex_data);
    let mut edge_data = DataSetAttributes::new();
    edge_data.deep_copy(&source.edge_data);
    let points = source.points.as_ref().map(|points| {
        let mut copy = Points::new();
        copy.deep_copy(points);
        copy
    });
    let mut storage = GraphStorage {
        topology: Arc::new(GraphTopology {
            adjacency: source.topology.adjacency.clone(),
            edges: source.topology.edges.clone(),
        }),
        modified_time: source.modified_time,
        vertex_data,
        edge_data,
        points,
        edge_points: Arc::new((*source.edge_points).clone()),
    };
    rebuild_adjacency(Arc::make_mut(&mut storage.topology), directed);
    storage
}

#[cfg(test)]
pub(crate) fn convert_storage_direction(source: &GraphStorage, directed: bool) -> GraphStorage {
    let mut storage = GraphStorage {
        topology: Arc::new(GraphTopology {
            adjacency: vec![VertexAdjacency::default(); source.topology.adjacency.len()],
            edges: source.topology.edges.clone(),
        }),
        modified_time: source.modified_time,
        vertex_data: source.vertex_data.shallow_clone(),
        edge_data: source.edge_data.shallow_clone(),
        points: source.points.clone(),
        edge_points: Arc::clone(&source.edge_points),
    };
    rebuild_adjacency(Arc::make_mut(&mut storage.topology), directed);
    storage
}

pub(crate) fn points_mut_internal(storage: &mut GraphStorage) -> &mut Points {
    let number_of_vertices = storage.topology.adjacency.len();
    let points = storage.points.get_or_insert_with(Points::new);
    if points.get_number_of_points() != number_of_vertices as VtkIdType {
        points.set_number_of_points(number_of_vertices as VtkIdType);
        for vertex in 0..number_of_vertices {
            points.set_point(vertex as VtkIdType, [0.0; 3]);
        }
    }
    points
}

pub(crate) fn set_points_internal(storage: &mut GraphStorage, mut points: Points) {
    points.set_number_of_points(storage.topology.adjacency.len() as VtkIdType);
    storage.points = Some(points);
    storage.modified_time = storage.modified_time.saturating_add(1);
}

#[cfg(test)]
pub(crate) fn set_edge_points_from_triples_internal(
    storage: &mut GraphStorage,
    edge_id: VtkIdType,
    points: Vec<[f64; 3]>,
) -> Result<(), GraphError> {
    let edge_id = validate_edge_id_value(storage, edge_id)?;
    let edge_points = Arc::make_mut(&mut storage.edge_points);
    edge_points.resize_edges(storage.topology.edges.len());
    edge_points.set_edge_points(edge_id, points);
    storage.modified_time = storage.modified_time.saturating_add(1);
    Ok(())
}

pub(crate) fn set_edge_points_flat_internal(
    storage: &mut GraphStorage,
    edge_id: VtkIdType,
    values: Vec<f64>,
) {
    let Some(edge_id) = validate_edge_id_for_void(storage, edge_id) else {
        return;
    };
    if values.len() % 3 != 0 {
        return;
    }
    let edge_points = Arc::make_mut(&mut storage.edge_points);
    edge_points.resize_edges(storage.topology.edges.len());
    edge_points.set_edge_points_flat(edge_id, values);
    storage.modified_time = storage.modified_time.saturating_add(1);
}

pub(crate) fn add_edge_point_internal(
    storage: &mut GraphStorage,
    edge_id: VtkIdType,
    point: [f64; 3],
) {
    let Some(edge_id) = validate_edge_id_for_void(storage, edge_id) else {
        return;
    };
    let edge_points = Arc::make_mut(&mut storage.edge_points);
    edge_points.resize_edges(storage.topology.edges.len());
    edge_points.add_edge_point(edge_id, point);
    storage.modified_time = storage.modified_time.saturating_add(1);
}

pub(crate) fn set_edge_point_internal(
    storage: &mut GraphStorage,
    edge_id: VtkIdType,
    point_index: VtkIdType,
    point: [f64; 3],
) -> bool {
    let Some(edge_id) = validate_edge_id_for_void(storage, edge_id) else {
        return false;
    };
    if point_index < 0 {
        return false;
    }
    let edge_points = Arc::make_mut(&mut storage.edge_points);
    edge_points.resize_edges(storage.topology.edges.len());
    if !edge_points.set_edge_point(edge_id, point_index as usize, point) {
        return false;
    }
    storage.modified_time = storage.modified_time.saturating_add(1);
    true
}

pub(crate) fn clear_edge_points_internal(storage: &mut GraphStorage, edge_id: VtkIdType) {
    let Some(edge_id) = validate_edge_id_for_void(storage, edge_id) else {
        return;
    };
    let edge_points = Arc::make_mut(&mut storage.edge_points);
    edge_points.resize_edges(storage.topology.edges.len());
    edge_points.clear_edge_points(edge_id);
    storage.modified_time = storage.modified_time.saturating_add(1);
}

pub(crate) fn reorder_out_vertices_internal(
    storage: &mut GraphStorage,
    vertex: usize,
    vertices: &[usize],
) -> Result<(), GraphError> {
    validate_vertex(storage, vertex)?;
    let topology = Arc::make_mut(&mut storage.topology);
    let current = topology.adjacency[vertex].out_edges.clone();
    let mut used = vec![false; current.len()];
    let mut reordered = Vec::with_capacity(current.len());

    for &target in vertices {
        let Some((index, half_edge)) = current
            .iter()
            .enumerate()
            .find(|(index, edge)| !used[*index] && edge.vertex == target)
        else {
            return Err(GraphError::InvalidReorder { vertex });
        };
        used[index] = true;
        reordered.push(*half_edge);
    }

    if reordered.len() != current.len() {
        return Err(GraphError::InvalidReorder { vertex });
    }

    topology.adjacency[vertex].out_edges = reordered;
    storage.modified_time = storage.modified_time.saturating_add(1);
    Ok(())
}

#[cfg(test)]
fn validate_edge_id_value(storage: &GraphStorage, edge_id: VtkIdType) -> Result<usize, GraphError> {
    if edge_id < 0 {
        return Err(GraphError::EdgeOutOfRange {
            edge: 0,
            number_of_edges: storage.topology.edges.len(),
        });
    }
    let edge = edge_id as usize;
    validate_edge(storage, edge)?;
    Ok(edge)
}

fn validate_edge_id_for_void(storage: &GraphStorage, edge_id: VtkIdType) -> Option<usize> {
    (edge_id >= 0)
        .then_some(edge_id as usize)
        .filter(|&edge| edge < storage.topology.edges.len())
}

#[cfg(test)]
fn validate_edge(storage: &GraphStorage, edge_id: usize) -> Result<(), GraphError> {
    if edge_id < storage.topology.edges.len() {
        Ok(())
    } else {
        Err(GraphError::EdgeOutOfRange {
            edge: edge_id,
            number_of_edges: storage.topology.edges.len(),
        })
    }
}

pub(crate) fn rebuild_adjacency(topology: &mut GraphTopology, directed: bool) {
    for adjacency in &mut topology.adjacency {
        adjacency.out_edges.clear();
        adjacency.in_edges.clear();
    }

    for (new_id, edge) in topology.edges.iter_mut().enumerate() {
        edge.id = new_id as VtkIdType;
        let source = edge.source as usize;
        let target = edge.target as usize;
        topology.adjacency[source].out_edges.push(HalfEdge {
            vertex: target,
            edge_id: new_id,
        });
        if directed {
            topology.adjacency[target].in_edges.push(HalfEdge {
                vertex: source,
                edge_id: new_id,
            });
        } else if source != target {
            topology.adjacency[target].out_edges.push(HalfEdge {
                vertex: source,
                edge_id: new_id,
            });
        }
    }
}

pub(crate) fn remove_edge_internal(
    storage: &mut GraphStorage,
    edge_id: usize,
    directed: bool,
) -> Result<Option<Edge>, GraphError> {
    if edge_id >= storage.topology.edges.len() {
        return Ok(None);
    }

    let topology = Arc::make_mut(&mut storage.topology);
    let removed = topology.edges[edge_id];
    storage.edge_data.remove_tuple_swap_with_last(edge_id);
    if !storage.edge_points.is_empty() {
        let edge_points = Arc::make_mut(&mut storage.edge_points);
        edge_points.resize_edges(topology.edges.len());
        edge_points.swap_remove(edge_id);
    }
    topology.edges.swap_remove(edge_id);
    if edge_id < topology.edges.len() {
        topology.edges[edge_id].id = edge_id as VtkIdType;
    }
    rebuild_adjacency(topology, directed);
    storage.modified_time = storage.modified_time.saturating_add(1);
    Ok(Some(removed))
}

pub(crate) fn remove_vertex_internal(
    storage: &mut GraphStorage,
    vertex: usize,
    directed: bool,
) -> Result<Option<usize>, GraphError> {
    if vertex >= storage.topology.adjacency.len() {
        return Ok(None);
    }

    let mut edge_ids: Vec<_> = storage.topology.adjacency[vertex]
        .out_edges
        .iter()
        .chain(storage.topology.adjacency[vertex].in_edges.iter())
        .map(|edge| edge.edge_id)
        .collect();
    edge_ids.sort_unstable();
    edge_ids.dedup();
    for edge_id in edge_ids.into_iter().rev() {
        remove_edge_internal(storage, edge_id, directed)?;
    }

    let topology = Arc::make_mut(&mut storage.topology);
    let last_vertex = topology.adjacency.len() - 1;
    storage.vertex_data.remove_tuple_swap_with_last(vertex);
    if let Some(points) = &mut storage.points {
        if vertex != last_vertex {
            let point = points.get_point(last_vertex as VtkIdType);
            points.set_point(vertex as VtkIdType, point);
        }
        points.set_number_of_points(last_vertex as VtkIdType);
    }
    topology.adjacency.swap_remove(vertex);
    if vertex != last_vertex {
        for edge in &mut topology.edges {
            if edge.source == last_vertex as VtkIdType {
                edge.source = vertex as VtkIdType;
            }
            if edge.target == last_vertex as VtkIdType {
                edge.target = vertex as VtkIdType;
            }
        }
        rebuild_adjacency(topology, directed);
    }
    storage.modified_time = storage.modified_time.saturating_add(1);
    Ok(Some(last_vertex))
}

pub(crate) fn remove_edge_void_internal(
    storage: &mut GraphStorage,
    edge_id: VtkIdType,
    directed: bool,
) {
    if edge_id < 0 {
        return;
    }
    let _ = remove_edge_internal(storage, edge_id as usize, directed);
}

pub(crate) fn remove_vertex_void_internal(
    storage: &mut GraphStorage,
    vertex: VtkIdType,
    directed: bool,
) {
    if vertex < 0 {
        return;
    }
    let _ = remove_vertex_internal(storage, vertex as usize, directed);
}
