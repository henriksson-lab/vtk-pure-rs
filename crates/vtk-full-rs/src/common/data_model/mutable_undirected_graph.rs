use std::sync::Arc;

use crate::common::core::{points::Points, VtkIdType};

use super::graph::{
    add_edge_point_internal, add_edge_value_internal, add_vertex_internal,
    clear_edge_points_internal, deep_clone_storage, points_mut_internal, remove_edge_void_internal,
    remove_vertex_void_internal, reorder_out_vertices_internal, resize_number_of_vertices_internal,
    set_edge_point_internal, set_edge_points_flat_internal, set_points_internal, Edge, GraphError,
    GraphStorage, UndirectedGraph,
};
use super::DataSetAttributes;

#[derive(Debug, Clone, PartialEq)]
pub struct MutableUndirectedGraph {
    storage: Arc<GraphStorage>,
}

impl MutableUndirectedGraph {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(GraphStorage::default()),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_graph(graph: UndirectedGraph) -> Self {
        Self {
            storage: graph.into_storage(),
        }
    }

    pub(super) fn as_graph(&self) -> UndirectedGraph {
        UndirectedGraph::from_storage(Arc::clone(&self.storage))
    }

    pub fn get_number_of_vertices(&self) -> VtkIdType {
        self.storage.topology.adjacency.len() as VtkIdType
    }

    pub fn get_number_of_edges(&self) -> VtkIdType {
        self.storage.topology.edges.len() as VtkIdType
    }

    pub fn initialize(&mut self) {
        self.storage = Arc::new(GraphStorage::default());
    }

    pub fn get_vertex_data(&self) -> &DataSetAttributes {
        &self.storage.vertex_data
    }

    pub(super) fn get_vertex_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut Arc::make_mut(&mut self.storage).vertex_data
    }

    pub fn get_edge_data(&self) -> &DataSetAttributes {
        &self.storage.edge_data
    }

    pub(super) fn get_edge_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut Arc::make_mut(&mut self.storage).edge_data
    }

    pub fn get_points(&mut self) -> &Points {
        points_mut_internal(Arc::make_mut(&mut self.storage))
    }

    pub(crate) fn points_ref(&self) -> Option<&Points> {
        self.storage.points.as_ref()
    }

    pub(super) fn get_points_mut(&mut self) -> &mut Points {
        points_mut_internal(Arc::make_mut(&mut self.storage))
    }

    pub fn set_points(&mut self, points: Points) {
        set_points_internal(Arc::make_mut(&mut self.storage), points);
    }

    pub fn set_edge_points(&mut self, edge_id: VtkIdType, values: Vec<f64>) {
        set_edge_points_flat_internal(Arc::make_mut(&mut self.storage), edge_id, values);
    }

    #[cfg(test)]
    pub(crate) fn set_edge_points_from_triples(
        &mut self,
        edge_id: VtkIdType,
        points: Vec<[f64; 3]>,
    ) -> Result<(), GraphError> {
        super::graph::set_edge_points_from_triples_internal(
            Arc::make_mut(&mut self.storage),
            edge_id,
            points,
        )
    }

    pub fn add_edge_point(&mut self, edge_id: VtkIdType, point: [f64; 3]) {
        add_edge_point_internal(Arc::make_mut(&mut self.storage), edge_id, point);
    }

    pub fn set_edge_point(
        &mut self,
        edge_id: VtkIdType,
        point_index: VtkIdType,
        point: [f64; 3],
    ) -> bool {
        set_edge_point_internal(
            Arc::make_mut(&mut self.storage),
            edge_id,
            point_index,
            point,
        )
    }

    pub fn clear_edge_points(&mut self, edge_id: VtkIdType) {
        clear_edge_points_internal(Arc::make_mut(&mut self.storage), edge_id);
    }

    pub fn shallow_copy_edge_points(&mut self, other: &Self) {
        Arc::make_mut(&mut self.storage).edge_points = Arc::clone(&other.storage.edge_points);
    }

    pub fn deep_copy_edge_points(&mut self, other: &Self) {
        Arc::make_mut(&mut self.storage).edge_points =
            Arc::new((*other.storage.edge_points).clone());
    }

    #[cfg(test)]
    pub(crate) fn shares_edge_points_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage.edge_points, &other.storage.edge_points)
    }

    pub fn set_number_of_vertices(&mut self, count: VtkIdType) -> VtkIdType {
        if count < 0 {
            return -1;
        }
        resize_number_of_vertices_internal(Arc::make_mut(&mut self.storage), count as usize)
            as VtkIdType
    }

    pub fn add_vertex(&mut self) -> VtkIdType {
        add_vertex_internal(Arc::make_mut(&mut self.storage)) as VtkIdType
    }

    pub fn add_edge(&mut self, source: VtkIdType, target: VtkIdType) -> Edge {
        add_edge_value_internal(Arc::make_mut(&mut self.storage), source, target, false)
    }

    #[cfg(test)]
    pub(crate) fn add_graph_edge(&mut self, edge: Edge) -> Result<Edge, GraphError> {
        self.checked_add_edge(edge.source as usize, edge.target as usize)
    }

    #[cfg(test)]
    pub(crate) fn checked_add_edge(
        &mut self,
        source: usize,
        target: usize,
    ) -> Result<Edge, GraphError> {
        super::graph::add_edge_internal(Arc::make_mut(&mut self.storage), source, target, false)
    }

    #[cfg(test)]
    pub(crate) fn lazy_add_vertex(&mut self) {
        self.add_vertex();
    }

    #[cfg(test)]
    pub(crate) fn lazy_add_edge(&mut self, source: usize, target: usize) -> Result<(), GraphError> {
        self.checked_add_edge(source, target).map(|_| ())
    }

    pub fn remove_vertex(&mut self, vertex: VtkIdType) {
        remove_vertex_void_internal(Arc::make_mut(&mut self.storage), vertex, false);
    }

    pub fn remove_edge(&mut self, edge: VtkIdType) {
        remove_edge_void_internal(Arc::make_mut(&mut self.storage), edge, false);
    }

    pub fn reorder_out_vertices(
        &mut self,
        vertex: usize,
        vertices: &[usize],
    ) -> Result<(), GraphError> {
        reorder_out_vertices_internal(Arc::make_mut(&mut self.storage), vertex, vertices)
    }

    pub fn shallow_copy(&mut self, other: &Self) {
        self.storage = Arc::clone(&other.storage);
    }

    pub fn deep_copy(&mut self, other: &Self) {
        self.storage = Arc::new(deep_clone_storage(&other.storage, false));
    }

    pub fn copy_structure(&mut self, other: &Self) {
        self.storage = Arc::new(GraphStorage {
            topology: Arc::clone(&other.storage.topology),
            modified_time: self.storage.modified_time.saturating_add(1),
            vertex_data: self.storage.vertex_data.shallow_clone(),
            edge_data: self.storage.edge_data.shallow_clone(),
            points: other.storage.points.clone(),
            edge_points: self.storage.edge_points.clone(),
        });
    }

    pub fn squeeze(&mut self) {
        let mut graph = self.as_graph();
        graph.squeeze();
        self.storage = graph.into_storage();
    }

    pub fn print_self(&self) -> String {
        self.as_graph().print_self()
    }

    #[cfg(test)]
    pub(crate) fn shares_structure_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage.topology, &other.storage.topology)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::data_model::{FieldDataArray, Variant};

    #[test]
    fn undirected_edges_are_visible_from_both_endpoints() {
        let mut graph = MutableUndirectedGraph::new();
        graph.set_number_of_vertices(2);

        graph.add_edge(0, 1);
        let readonly = graph.as_graph();

        assert_eq!(readonly.get_out_degree(0), 1);
        assert_eq!(readonly.get_out_degree(1), 1);
        assert_eq!(readonly.get_in_degree(1), 1);
        assert_eq!(readonly.get_adjacent_vertices(1), vec![0]);
    }

    #[test]
    fn undirected_self_loop_is_stored_once_in_adjacency() {
        let mut graph = MutableUndirectedGraph::new();
        graph.add_vertex();
        graph.add_edge(0, 0);

        let readonly = graph.as_graph();
        assert_eq!(readonly.get_number_of_edges(), 1);
        assert_eq!(readonly.get_out_degree(0), 1);
        assert_eq!(readonly.get_in_degree(0), 1);
        assert_eq!(readonly.get_degree(0), 1);
    }

    #[test]
    fn lazy_mutators_follow_void_vtk_api_and_graph_edge_returns_handle() {
        let mut graph = MutableUndirectedGraph::new();
        graph.lazy_add_vertex();
        graph.lazy_add_vertex();
        graph.lazy_add_edge(0, 1).unwrap();
        let copied = graph
            .add_graph_edge(Edge {
                source: 1,
                target: 0,
                id: -1,
            })
            .unwrap();

        assert_eq!(graph.get_number_of_vertices(), 2);
        assert_eq!(copied.id, 1);
        assert_eq!(graph.as_graph().get_out_degree(0), 2);
    }

    #[test]
    fn vertex_removal_swaps_last_vertex_into_removed_id() {
        let mut graph = MutableUndirectedGraph::new();
        graph.set_number_of_vertices(3);
        graph.add_edge(0, 2);
        graph.add_edge(1, 2);

        graph.remove_vertex(1);
        let readonly = graph.as_graph();

        assert_eq!(readonly.get_number_of_vertices(), 2);
        assert_eq!(readonly.get_number_of_edges(), 1);
        assert_eq!(readonly.edge(0).unwrap().source, 0);
        assert_eq!(readonly.edge(0).unwrap().target, 1);
        assert_eq!(readonly.get_adjacent_vertices(1), vec![0]);
    }

    #[test]
    fn edge_data_swaps_with_last_removed_edge() {
        let mut graph = MutableUndirectedGraph::new();
        graph.set_number_of_vertices(3);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph
            .get_edge_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("weights", 1, vec![11, 22]));

        graph.remove_edge(0);

        let weights = graph
            .get_edge_data()
            .get_field_data_array("weights")
            .unwrap();
        assert_eq!(weights.values_as_variants(), &[Variant::I64(22)]);
    }

    #[test]
    fn edge_points_swap_with_last_removed_edge() {
        let mut graph = MutableUndirectedGraph::new();
        graph.set_number_of_vertices(3);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.set_edge_points(0, vec![0.0, 0.0, 0.0]);
        graph.set_edge_points(1, vec![1.0, 0.0, 0.0, 2.0, 0.0, 0.0]);

        graph.remove_edge(0);
        let readonly = graph.as_graph();

        assert_eq!(readonly.get_number_of_edge_points(0), 2);
        assert_eq!(readonly.get_edge_points(0), &[1.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
    }
}
