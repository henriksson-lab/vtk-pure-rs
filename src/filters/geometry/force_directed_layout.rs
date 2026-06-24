//! Force-directed graph layout for visualization.
//!
//! Positions graph vertices using spring-electric force simulation
//! (Fruchterman-Reingold style).

use crate::data::{AnyDataArray, CellArray, DataArray, Graph, Points, PolyData};

/// Compute a 2D force-directed layout of a Graph.
///
/// Returns a PolyData with vertex positions and edge lines.
pub fn force_directed_layout(graph: &Graph, iterations: usize, area: f64) -> PolyData {
    let n = graph.num_vertices();
    if n == 0 {
        return PolyData::new();
    }

    let k = (area / n as f64).sqrt(); // ideal spring length

    // Initialize positions randomly (deterministic seed)
    let mut pos: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let angle = i as f64 * 2.399; // golden angle
            let r = (i as f64 + 1.0).sqrt() * k * 0.5;
            [r * angle.cos(), r * angle.sin()]
        })
        .collect();

    let temp_start = area.sqrt() * 0.1;

    for iter in 0..iterations {
        let temp = temp_start * (1.0 - iter as f64 / iterations as f64);
        if temp < 1e-10 {
            break;
        }

        let mut disp = vec![[0.0f64; 2]; n];

        // Repulsive forces between all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = pos[i][0] - pos[j][0];
                let dy = pos[i][1] - pos[j][1];
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = k * k / dist; // repulsive
                let fx = dx / dist * force;
                let fy = dy / dist * force;
                disp[i][0] += fx;
                disp[i][1] += fy;
                disp[j][0] -= fx;
                disp[j][1] -= fy;
            }
        }

        // Attractive forces along edges
        for ei in 0..graph.num_edges() {
            let (src, dst) = graph.edge(ei);
            let dx = pos[src][0] - pos[dst][0];
            let dy = pos[src][1] - pos[dst][1];
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let force = dist * dist / k; // attractive
            let fx = dx / dist * force;
            let fy = dy / dist * force;
            disp[src][0] -= fx;
            disp[src][1] -= fy;
            disp[dst][0] += fx;
            disp[dst][1] += fy;
        }

        // Apply displacement with temperature limiting
        for i in 0..n {
            let mag = (disp[i][0] * disp[i][0] + disp[i][1] * disp[i][1]).sqrt();
            if mag > 1e-10 {
                let scale = mag.min(temp) / mag;
                pos[i][0] += disp[i][0] * scale;
                pos[i][1] += disp[i][1] * scale;
            }
        }
    }

    // Build PolyData
    let mut points = Points::<f64>::new();
    for p in &pos {
        points.push([p[0], p[1], 0.0]);
    }

    let mut lines = CellArray::new();
    for ei in 0..graph.num_edges() {
        let (src, dst) = graph.edge(ei);
        lines.push_cell(&[src as i64, dst as i64]);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.lines = lines;

    // Add vertex degree as point data
    let mut degree = vec![0.0f64; n];
    for ei in 0..graph.num_edges() {
        let (src, dst) = graph.edge(ei);
        degree[src] += 1.0;
        degree[dst] += 1.0;
    }
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Degree", degree, 1)));

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangle_graph() {
        let mut g = Graph::new_undirected();
        g.add_vertex();
        g.add_vertex();
        g.add_vertex();
        g.add_edge(0, 1);
        g.add_edge(1, 2);
        g.add_edge(2, 0);

        let layout = force_directed_layout(&g, 50, 100.0);
        assert_eq!(layout.points.len(), 3);
        assert_eq!(layout.lines.num_cells(), 3);
        assert!(layout.point_data().get_array("Degree").is_some());
    }

    #[test]
    fn path_graph() {
        let mut g = Graph::new_undirected();
        for _ in 0..5 {
            g.add_vertex();
        }
        for i in 0..4 {
            g.add_edge(i, i + 1);
        }

        let layout = force_directed_layout(&g, 100, 100.0);
        assert_eq!(layout.points.len(), 5);
        assert_eq!(layout.lines.num_cells(), 4);
    }

    #[test]
    fn empty_graph() {
        let g = Graph::new_undirected();
        let layout = force_directed_layout(&g, 50, 100.0);
        assert_eq!(layout.points.len(), 0);
    }

    #[test]
    fn single_vertex() {
        let mut g = Graph::new_undirected();
        g.add_vertex();
        let layout = force_directed_layout(&g, 10, 100.0);
        assert_eq!(layout.points.len(), 1);
    }
}
