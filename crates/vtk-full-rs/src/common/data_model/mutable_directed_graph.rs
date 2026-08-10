use std::sync::Arc;

use crate::common::core::{points::Points, VtkIdType};

use super::graph::{
    add_edge_point_internal, add_edge_value_internal, add_vertex_internal,
    clear_edge_points_internal, deep_clone_storage, points_mut_internal, remove_edge_void_internal,
    remove_vertex_void_internal, reorder_out_vertices_internal, resize_number_of_vertices_internal,
    set_edge_point_internal, set_edge_points_flat_internal, set_points_internal, DirectedGraph,
    Edge, GraphError, GraphStorage,
};
use super::DataSetAttributes;

#[derive(Debug, Clone, PartialEq)]
pub struct MutableDirectedGraph {
    storage: Arc<GraphStorage>,
}

impl MutableDirectedGraph {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(GraphStorage::default()),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_graph(graph: DirectedGraph) -> Self {
        Self {
            storage: graph.into_storage(),
        }
    }

    pub(super) fn as_graph(&self) -> DirectedGraph {
        DirectedGraph::from_storage(Arc::clone(&self.storage))
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

    #[cfg(test)]
    pub(crate) fn get_vertex_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut Arc::make_mut(&mut self.storage).vertex_data
    }

    pub fn get_edge_data(&self) -> &DataSetAttributes {
        &self.storage.edge_data
    }

    #[cfg(test)]
    pub(crate) fn get_edge_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut Arc::make_mut(&mut self.storage).edge_data
    }

    pub fn get_points(&mut self) -> &Points {
        points_mut_internal(Arc::make_mut(&mut self.storage))
    }

    pub(crate) fn points_ref(&self) -> Option<&Points> {
        self.storage.points.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn get_points_mut(&mut self) -> &mut Points {
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
        add_edge_value_internal(Arc::make_mut(&mut self.storage), source, target, true)
    }

    #[cfg(test)]
    pub(crate) fn add_graph_edge(&mut self, edge: Edge) -> Result<Edge, GraphError> {
        self.checked_add_edge(edge.source as usize, edge.target as usize)
    }

    pub fn add_child(&mut self, parent: VtkIdType) -> VtkIdType {
        let child = self.add_vertex();
        self.add_edge(parent, child);
        child
    }

    #[cfg(test)]
    pub(crate) fn checked_add_edge(
        &mut self,
        source: usize,
        target: usize,
    ) -> Result<Edge, GraphError> {
        super::graph::add_edge_internal(Arc::make_mut(&mut self.storage), source, target, true)
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
        remove_vertex_void_internal(Arc::make_mut(&mut self.storage), vertex, true);
    }

    pub fn remove_edge(&mut self, edge: VtkIdType) {
        remove_edge_void_internal(Arc::make_mut(&mut self.storage), edge, true);
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
        self.storage = Arc::new(deep_clone_storage(&other.storage, true));
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
    fn directed_edges_populate_in_and_out_adjacency() {
        let mut graph = MutableDirectedGraph::new();
        assert_eq!(graph.set_number_of_vertices(3), 0);

        let edge = graph.add_edge(0, 2);
        let readonly = graph.as_graph();

        assert_eq!(edge.id, 0);
        assert_eq!(readonly.get_number_of_vertices(), 3);
        assert_eq!(readonly.get_number_of_edges(), 1);
        assert_eq!(readonly.get_out_degree(0), 1);
        assert_eq!(readonly.get_in_degree(2), 1);
        assert_eq!(readonly.get_adjacent_vertices(0), vec![2]);
    }

    #[test]
    fn mutation_detaches_shallow_copy() {
        let mut original = MutableDirectedGraph::new();
        original.set_number_of_vertices(1);

        let mut copy = MutableDirectedGraph::new();
        copy.shallow_copy(&original);
        assert!(copy.shares_structure_with(&original));

        copy.add_vertex();

        assert!(!copy.shares_structure_with(&original));
        assert_eq!(original.get_number_of_vertices(), 1);
        assert_eq!(copy.get_number_of_vertices(), 2);
    }

    #[test]
    fn copy_structure_shares_topology_until_next_structural_mutation() {
        let mut original = MutableDirectedGraph::new();
        original.set_number_of_vertices(2);
        original.add_edge(0, 1);
        original.get_points_mut().set_point(1, [1.0, 2.0, 3.0]);

        let mut copy = MutableDirectedGraph::new();
        copy.add_vertex();
        copy.get_vertex_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![99]));
        copy.copy_structure(&original);

        assert!(copy.shares_structure_with(&original));
        assert_eq!(copy.get_number_of_vertices(), 2);
        assert_eq!(copy.as_graph().get_point(1), [1.0, 2.0, 3.0]);
        assert_eq!(
            copy.get_vertex_data()
                .get_field_data_array("ids")
                .unwrap()
                .values_as_variants(),
            &[Variant::I64(99)]
        );

        copy.add_vertex();

        assert!(!copy.shares_structure_with(&original));
        assert_eq!(original.get_number_of_vertices(), 2);
        assert_eq!(copy.get_number_of_vertices(), 3);
    }

    #[test]
    fn lazy_mutators_follow_void_vtk_api_and_graph_edge_returns_handle() {
        let mut graph = MutableDirectedGraph::new();
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
        assert_eq!(graph.as_graph().edge(0).unwrap().source, 0);
        assert_eq!(graph.as_graph().edge(0).unwrap().target, 1);
        assert_eq!(copied.id, 1);
        assert_eq!(copied.source, 1);
        assert_eq!(copied.target, 0);
    }

    #[test]
    fn removal_swaps_last_edge_into_removed_id() {
        let mut graph = MutableDirectedGraph::new();
        graph.set_number_of_vertices(3);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);

        graph.remove_edge(0);
        let readonly = graph.as_graph();

        assert_eq!(readonly.get_number_of_edges(), 1);
        assert_eq!(readonly.edge(0).unwrap().source, 1);
        assert_eq!(readonly.edge(0).unwrap().target, 2);
        assert_eq!(readonly.edge(0).unwrap().id, 0);
    }

    #[test]
    fn vertex_and_edge_data_follow_structural_edits() {
        let mut graph = MutableDirectedGraph::new();
        graph.set_number_of_vertices(2);
        graph
            .get_vertex_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![10, 20]));
        graph.add_vertex();
        graph.add_edge(0, 1);
        graph.add_edge(2, 1);
        graph
            .get_edge_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("weights", 1, vec![5, 7]));

        graph.remove_vertex(1);

        let vertices = graph.get_vertex_data().get_field_data_array("ids").unwrap();
        let edges = graph
            .get_edge_data()
            .get_field_data_array("weights")
            .unwrap();
        assert_eq!(
            vertices.values_as_variants(),
            &[Variant::I64(10), Variant::I64(0)]
        );
        assert_eq!(edges.get_number_of_tuples(), 0);
    }

    #[test]
    fn structural_mutation_detaches_nested_vertex_data_storage() {
        let mut original = MutableDirectedGraph::new();
        original.set_number_of_vertices(1);
        original
            .get_vertex_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![10]));

        let mut copy = MutableDirectedGraph::new();
        copy.shallow_copy(&original);

        copy.add_vertex();

        let original_ids = original
            .get_vertex_data()
            .get_field_data_array("ids")
            .unwrap();
        let copy_ids = copy.get_vertex_data().get_field_data_array("ids").unwrap();
        assert_eq!(original_ids.values_as_variants(), &[Variant::I64(10)]);
        assert_eq!(
            copy_ids.values_as_variants(),
            &[Variant::I64(10), Variant::I64(0)]
        );
        assert!(!original_ids.shares_values_with(copy_ids));
    }

    #[test]
    fn points_materialize_and_resize_with_vertices() {
        let mut graph = MutableDirectedGraph::new();
        graph.set_number_of_vertices(2);

        graph.get_points_mut().set_point(1, [1.0, 2.0, 3.0]);
        graph.add_vertex();
        let readonly = graph.as_graph();

        assert_eq!(readonly.get_point(0), [0.0; 3]);
        assert_eq!(readonly.get_point(1), [1.0, 2.0, 3.0]);
        assert_eq!(readonly.get_point(2), [0.0; 3]);
    }

    #[test]
    fn edge_point_get_and_set_match_indexed_vtk_api() {
        let mut graph = MutableDirectedGraph::new();
        graph.set_number_of_vertices(2);
        let edge = graph.add_edge(0, 1);
        graph.set_edge_points(edge.id, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);

        assert!(graph.set_edge_point(edge.id, 1, [2.0, 3.0, 4.0]));
        assert!(!graph.set_edge_point(edge.id, 2, [9.0, 9.0, 9.0]));

        let readonly = graph.as_graph();
        assert_eq!(readonly.get_edge_point(edge.id, 0), Some([0.0, 0.0, 0.0]));
        assert_eq!(readonly.get_edge_point(edge.id, 1), Some([2.0, 3.0, 4.0]));
        assert_eq!(readonly.get_edge_point(edge.id, 2), None);
    }

    #[test]
    fn reorder_out_vertices_reorders_adjacency_only() {
        let mut graph = MutableDirectedGraph::new();
        graph.set_number_of_vertices(4);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(0, 3);

        graph.reorder_out_vertices(0, &[3, 1, 2]).unwrap();

        assert_eq!(graph.as_graph().get_adjacent_vertices(0), vec![3, 1, 2]);
        assert_eq!(graph.as_graph().edges()[0].target, 1);
    }

    #[test]
    fn initialize_clears_structure_attributes_and_points() {
        let mut graph = MutableDirectedGraph::new();
        graph.set_number_of_vertices(1);
        graph
            .get_vertex_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![10]));
        graph.get_points_mut().set_point(0, [1.0, 2.0, 3.0]);

        graph.initialize();

        assert_eq!(graph.get_number_of_vertices(), 0);
        assert_eq!(graph.get_vertex_data().get_number_of_arrays(), 0);
        assert!(graph.points_ref().is_none());
    }
}
