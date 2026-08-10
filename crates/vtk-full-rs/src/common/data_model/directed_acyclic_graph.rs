use super::{
    DataSetAttributes, DirectedGraph, FieldData, InEdge, MutableDirectedGraph, OutEdge, Variant,
    EDGE, VERTEX,
};
use crate::common::core::{points::Points, VtkIdType};

use super::data_object_types::VTK_DIRECTED_ACYCLIC_GRAPH;

const DFS_WHITE: u8 = 0;
const DFS_GRAY: u8 = 1;
const DFS_BLACK: u8 = 2;

/// VTK: `vtkDirectedAcyclicGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectedAcyclicGraph {
    directed_graph: DirectedGraph,
}

impl DirectedAcyclicGraph {
    /// VTK: `vtkDirectedAcyclicGraph::New`.
    pub fn new() -> Self {
        Self {
            directed_graph: DirectedGraph::new(),
        }
    }

    /// VTK: `vtkDirectedAcyclicGraph::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        VTK_DIRECTED_ACYCLIC_GRAPH
    }

    /// VTK: `vtkDirectedAcyclicGraph::IsStructureValid`.
    pub(crate) fn is_structure_valid(graph: &DirectedGraph) -> bool {
        is_directed_acyclic(graph)
    }

    /// VTK: `vtkGraph::CheckedShallowCopy`.
    pub fn checked_shallow_copy(&mut self, graph: &DirectedGraph) -> bool {
        if !Self::is_structure_valid(graph) {
            return false;
        }
        self.directed_graph.shallow_copy(graph);
        true
    }

    /// VTK: `vtkGraph::CheckedShallowCopy` for a mutable directed graph source.
    pub fn checked_shallow_copy_from_mutable(&mut self, graph: &MutableDirectedGraph) -> bool {
        self.checked_shallow_copy(&graph.as_graph())
    }

    /// VTK: `vtkGraph::ShallowCopy` for the same concrete graph type.
    pub fn shallow_copy(&mut self, other: &Self) {
        self.directed_graph.shallow_copy(&other.directed_graph);
    }

    /// VTK: `vtkGraph::DeepCopy` for the same concrete graph type.
    pub fn deep_copy(&mut self, other: &Self) {
        self.directed_graph.deep_copy(&other.directed_graph);
    }

    /// VTK: `vtkGraph::CopyStructure` for the same concrete graph type.
    pub fn copy_structure(&mut self, other: &Self) {
        self.directed_graph.copy_structure(&other.directed_graph);
    }

    /// VTK: `vtkGraph::GetNumberOfVertices`.
    pub fn get_number_of_vertices(&self) -> VtkIdType {
        self.directed_graph.get_number_of_vertices()
    }

    /// VTK: `vtkGraph::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> VtkIdType {
        self.directed_graph.get_number_of_edges()
    }

    /// VTK: `vtkGraph::GetSourceVertex`.
    pub fn get_source_vertex(&self, edge_id: VtkIdType) -> VtkIdType {
        self.directed_graph.get_source_vertex(edge_id)
    }

    /// VTK: `vtkGraph::GetTargetVertex`.
    pub fn get_target_vertex(&self, edge_id: VtkIdType) -> VtkIdType {
        self.directed_graph.get_target_vertex(edge_id)
    }

    /// VTK: `vtkGraph::GetEdgeId`.
    pub fn get_edge_id(&self, a: VtkIdType, b: VtkIdType) -> VtkIdType {
        self.directed_graph.get_edge_id(a, b)
    }

    /// VTK: `vtkGraph::GetOutEdges`.
    pub fn get_out_edges(&self, vertex: VtkIdType) -> Vec<OutEdge> {
        self.directed_graph.get_out_edges(vertex)
    }

    /// VTK: `vtkGraph::GetInEdges`.
    pub fn get_in_edges(&self, vertex: VtkIdType) -> Vec<InEdge> {
        self.directed_graph.get_in_edges(vertex)
    }

    /// VTK: `vtkGraph::GetOutDegree`.
    pub fn get_out_degree(&self, vertex: VtkIdType) -> VtkIdType {
        self.directed_graph.get_out_degree(vertex)
    }

    /// VTK: `vtkGraph::GetInDegree`.
    pub fn get_in_degree(&self, vertex: VtkIdType) -> VtkIdType {
        self.directed_graph.get_in_degree(vertex)
    }

    /// VTK: `vtkGraph::GetDegree`.
    pub fn get_degree(&self, vertex: VtkIdType) -> VtkIdType {
        self.directed_graph.get_degree(vertex)
    }

    /// VTK: `vtkGraph::GetAdjacentVertices`.
    pub fn get_adjacent_vertices(&self, vertex: VtkIdType) -> Vec<VtkIdType> {
        self.directed_graph.get_adjacent_vertices(vertex)
    }

    /// VTK: `vtkGraph::FindVertex`.
    pub fn find_vertex(&self, pedigree_id: &Variant) -> VtkIdType {
        self.directed_graph.find_vertex(pedigree_id)
    }

    /// VTK: `vtkGraph::GetVertexData`.
    pub fn get_vertex_data(&self) -> &DataSetAttributes {
        self.directed_graph.get_vertex_data()
    }

    /// VTK: `vtkGraph::GetEdgeData`.
    pub fn get_edge_data(&self) -> &DataSetAttributes {
        self.directed_graph.get_edge_data()
    }

    /// VTK: `vtkGraph::GetPoints`.
    pub fn get_points(&mut self) -> &Points {
        self.directed_graph.get_points()
    }

    /// VTK: `vtkGraph::GetPoint`.
    pub fn get_point(&self, vertex: VtkIdType) -> [f64; 3] {
        self.directed_graph.get_point(vertex)
    }

    /// VTK: `vtkGraph::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.directed_graph.get_bounds()
    }

    /// VTK: `vtkGraph::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &[f64] {
        self.directed_graph.get_edge_points(edge_id)
    }

    /// VTK: `vtkGraph::GetEdgePoint`.
    pub fn get_edge_point(&self, edge_id: VtkIdType, point_index: VtkIdType) -> Option<[f64; 3]> {
        self.directed_graph.get_edge_point(edge_id, point_index)
    }

    /// VTK: `vtkGraph::GetNumberOfEdgePoints`.
    pub fn get_number_of_edge_points(&self, edge_id: VtkIdType) -> VtkIdType {
        self.directed_graph.get_number_of_edge_points(edge_id)
    }

    /// VTK: `vtkGraph::GetInducedEdges`.
    pub fn get_induced_edges(
        &self,
        vertices: &[usize],
    ) -> Result<Vec<VtkIdType>, super::GraphError> {
        self.directed_graph.get_induced_edges(vertices)
    }

    /// VTK: `vtkGraph::GetAttributesAsFieldData`.
    pub fn get_attributes_as_field_data(&self, attribute_type: i32) -> Option<&FieldData> {
        match attribute_type {
            VERTEX | EDGE => self
                .directed_graph
                .get_attributes_as_field_data(attribute_type),
            _ => None,
        }
    }

    /// VTK: `vtkGraph::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        self.directed_graph.get_number_of_elements(attribute_type)
    }

    /// VTK: `vtkGraph::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.directed_graph.get_actual_memory_size()
    }

    /// VTK: `vtkGraph::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.directed_graph.get_m_time()
    }

    /// VTK: `vtkGraph::Squeeze`.
    pub fn squeeze(&mut self) {
        self.directed_graph.squeeze();
    }

    /// VTK: `vtkGraph::Dump`.
    pub fn dump(&self) -> String {
        self.directed_graph.dump()
    }

    /// VTK: `vtkDirectedAcyclicGraph::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.directed_graph.print_self()
    }

    /// VTK: `vtkGraph::IsSameStructure`.
    pub fn is_same_structure(&self, other: &Self) -> bool {
        self.directed_graph.is_same_structure(&other.directed_graph)
    }

    /// VTK: `vtkGraph::Initialize`.
    pub fn initialize(&mut self) {
        self.directed_graph.initialize();
    }
}

impl Default for DirectedAcyclicGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn is_directed_acyclic(graph: &DirectedGraph) -> bool {
    let number_of_vertices = graph.get_number_of_vertices();
    if number_of_vertices <= 0 {
        return true;
    }

    let mut color = vec![DFS_WHITE; number_of_vertices as usize];
    for vertex in 0..number_of_vertices {
        if color[vertex as usize] == DFS_WHITE && !dfs_visit(graph, vertex, &mut color) {
            return false;
        }
    }
    true
}

fn dfs_visit(graph: &DirectedGraph, vertex: VtkIdType, color: &mut [u8]) -> bool {
    let vertex_index = vertex as usize;
    color[vertex_index] = DFS_GRAY;
    for edge in graph.get_out_edges(vertex) {
        let target = edge.target as usize;
        if color[target] == DFS_WHITE {
            if !dfs_visit(graph, edge.target, color) {
                return false;
            }
        } else if color[target] == DFS_GRAY {
            return false;
        }
    }
    color[vertex_index] = DFS_BLACK;
    true
}
