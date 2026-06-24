//! Extract skeleton curves from a ReebGraph as PolyData line cells.
//!
//! The surface skeleton connects critical points with straight lines,
//! while the volume skeleton adds midpoints along arcs using vertex_ids.

use crate::data::reeb_graph::ReebGraph;
use crate::data::DataArray;
use crate::data::{CellArray, Points, PolyData};

/// Extract the surface skeleton from a ReebGraph as PolyData line cells.
///
/// For each arc in the ReebGraph, creates a line from source node position
/// to target node position. Adds "ArcId" cell data (i32) and "ScalarValue"
/// point data (f64).
pub fn surface_skeleton(graph: &ReebGraph) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut scalar_values: Vec<f64> = Vec::new();
    let mut arc_ids: Vec<i32> = Vec::new();

    // Add all node positions as points
    for node in &graph.nodes {
        points.push([node.scalar_value, 0.0, 0.0]);
        scalar_values.push(node.scalar_value);
    }

    // Add arcs as line cells
    for (arc_idx, arc) in graph.arcs.iter().enumerate() {
        lines.push_cell(&[arc.source as i64, arc.target as i64]);
        arc_ids.push(arc_idx as i32);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;

    // Add point data: ScalarValue
    let scalar_arr = DataArray::from_vec("ScalarValue", scalar_values, 1);
    pd.point_data_mut().add_array(scalar_arr.into());

    // Add cell data: ArcId
    let arc_arr = DataArray::from_vec("ArcId", arc_ids, 1);
    pd.cell_data_mut().add_array(arc_arr.into());

    pd
}

/// Extract the volume skeleton from a ReebGraph as PolyData line cells.
///
/// Like the surface skeleton, but adds midpoints along arcs using the
/// arc's `vertex_ids`. For each arc with N intermediate vertices, the line
/// cell has N+2 points (source + intermediates + target).
pub fn volume_skeleton(graph: &ReebGraph) -> PolyData {
    let mut points = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut scalar_values: Vec<f64> = Vec::new();
    let mut arc_ids: Vec<i32> = Vec::new();

    // Add all node positions as points (indices 0..num_nodes)
    for node in &graph.nodes {
        points.push([node.scalar_value, 0.0, 0.0]);
        scalar_values.push(node.scalar_value);
    }

    // Add arcs as polyline cells with midpoints
    for (arc_idx, arc) in graph.arcs.iter().enumerate() {
        let s0 = graph.node(arc.source).scalar_value;
        let s1 = graph.node(arc.target).scalar_value;

        let mut cell = Vec::with_capacity(2 + arc.vertex_ids.len());
        cell.push(arc.source as i64);

        // Add midpoints for each intermediate vertex
        for (i, _vid) in arc.vertex_ids.iter().enumerate() {
            let t = (i as f64 + 1.0) / (arc.vertex_ids.len() as f64 + 1.0);
            let s = s0 + t * (s1 - s0);
            let pt_idx = points.len();
            points.push([s, 0.0, 0.0]);
            scalar_values.push(s);
            cell.push(pt_idx as i64);
        }

        cell.push(arc.target as i64);
        lines.push_cell(&cell);
        arc_ids.push(arc_idx as i32);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.lines = lines;

    // Add point data: ScalarValue
    let scalar_arr = DataArray::from_vec("ScalarValue", scalar_values, 1);
    pd.point_data_mut().add_array(scalar_arr.into());

    // Add cell data: ArcId
    let arc_arr = DataArray::from_vec("ArcId", arc_ids, 1);
    pd.cell_data_mut().add_array(arc_arr.into());

    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::reeb_graph::NodeType;

    fn make_test_graph() -> ReebGraph {
        let mut rg = ReebGraph::new();
        let n0 = rg.add_node(0, 0.0, NodeType::Minimum);
        let n1 = rg.add_node(1, 0.5, NodeType::Saddle);
        let n2 = rg.add_node(2, 1.0, NodeType::Maximum);
        rg.add_arc(n0, n1, vec![]);
        rg.add_arc(n1, n2, vec![10, 11]);
        rg
    }

    #[test]
    fn test_surface_skeleton() {
        let rg = make_test_graph();
        let pd = surface_skeleton(&rg);

        // 3 nodes = 3 points
        assert_eq!(pd.points.len(), 3);
        // 2 arcs = 2 line cells
        assert_eq!(pd.lines.num_cells(), 2);

        // Check ScalarValue point data
        let sv = pd.point_data().get_array("ScalarValue").unwrap();
        let mut buf = [0.0f64];
        sv.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 0.0).abs() < 1e-10);
        sv.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-10);

        // Check ArcId cell data
        let aid = pd.cell_data().get_array("ArcId").unwrap();
        let mut ibuf = [0.0f64];
        aid.tuple_as_f64(0, &mut ibuf);
        assert_eq!(ibuf[0] as i32, 0);
        aid.tuple_as_f64(1, &mut ibuf);
        assert_eq!(ibuf[0] as i32, 1);
    }

    #[test]
    fn test_volume_skeleton() {
        let rg = make_test_graph();
        let pd = volume_skeleton(&rg);

        // 3 nodes + 2 midpoints from the second arc = 5 points
        assert_eq!(pd.points.len(), 5);
        // 2 arcs = 2 line cells
        assert_eq!(pd.lines.num_cells(), 2);

        // First arc (no midpoints) => line with 2 points
        let cell0: Vec<i64> = pd.lines.iter().next().unwrap().to_vec();
        assert_eq!(cell0.len(), 2);

        // Second arc (2 midpoints) => line with 4 points
        let cell1: Vec<i64> = pd.lines.iter().nth(1).unwrap().to_vec();
        assert_eq!(cell1.len(), 4);
    }
}
