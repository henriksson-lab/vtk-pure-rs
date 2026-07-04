use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extrude a 2D PolyData along a direction vector to create a 3D solid.
///
/// Each polygon is duplicated at an offset, and side quads are generated
/// connecting the original and extruded edges. Optionally caps the ends.
pub fn extrude(input: &PolyData, direction: [f64; 3], capping: bool) -> PolyData {
    let n = input.points.len();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();
    let mut out_lines = CellArray::new();
    let mut out_strips = CellArray::new();

    // Copy original points
    for i in 0..n {
        out_points.push(input.points.get(i));
    }
    // Extruded points
    for i in 0..n {
        let p = input.points.get(i);
        out_points.push([
            p[0] + direction[0],
            p[1] + direction[1],
            p[2] + direction[2],
        ]);
    }

    let offset = n as i64;

    // VTK creates lines from input vertices.
    for cell in input.verts.iter() {
        for &pt_id in cell {
            out_lines.push_cell(&[pt_id, pt_id + offset]);
        }
    }

    // VTK creates strips from input line segments.
    for cell in input.lines.iter() {
        for i in 0..cell.len().saturating_sub(1) {
            let a = cell[i];
            let b = cell[i + 1];
            out_strips.push_cell(&[a, b, a + offset, b + offset]);
        }
    }

    if capping {
        for cell in input.polys.iter() {
            out_polys.push_cell(cell);
        }
        for cell in input.polys.iter() {
            let extruded: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            out_polys.push_cell(&extruded);
        }
    }

    for (a, b) in boundary_edges(input) {
        out_strips.push_cell(&[a, b, a + offset, b + offset]);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.polys = out_polys;
    pd.strips = out_strips;
    pd
}

fn boundary_edges(input: &PolyData) -> Vec<(i64, i64)> {
    let mut edge_counts: HashMap<(i64, i64), usize> = HashMap::new();
    let mut ordered_edges = Vec::new();

    for cell in input.polys.iter().chain(input.strips.iter()) {
        if cell.len() < 2 {
            continue;
        }
        for i in 0..cell.len() {
            let a = cell[i];
            let b = cell[(i + 1) % cell.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_counts
                .entry(key)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            ordered_edges.push((a, b, key));
        }
    }

    ordered_edges
        .into_iter()
        .filter_map(|(a, b, key)| (edge_counts[&key] == 1).then_some((a, b)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrude_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extrude(&pd, [0.0, 0.0, 1.0], true);
        assert_eq!(result.points.len(), 6); // 3 original + 3 extruded
                                            // 3 side quads + 1 bottom cap + 1 top cap = 5
        assert_eq!(result.polys.num_cells() + result.strips.num_cells(), 5);
    }

    #[test]
    fn extrude_no_cap() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extrude(&pd, [0.0, 0.0, 2.0], false);
        assert_eq!(result.strips.num_cells(), 3); // just side strips
    }

    #[test]
    fn extrude_line() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1]);

        let result = extrude(&pd, [0.0, 0.0, 1.0], false);
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.strips.num_cells(), 1); // one side strip
    }

    #[test]
    fn extrude_direction() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extrude(&pd, [0.0, 0.0, 5.0], false);
        let p = result.points.get(3); // extruded point 0
        assert!((p[2] - 5.0).abs() < 1e-10);
    }
}
