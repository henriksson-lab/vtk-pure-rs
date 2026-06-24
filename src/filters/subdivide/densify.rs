use crate::data::{CellArray, PolyData};

/// Add points along polygon edges to increase mesh resolution.
///
/// Each edge longer than `max_edge_length` is subdivided by inserting
/// new points. The topology is preserved — polygons gain additional
/// vertices but remain as single cells.
pub fn densify(input: &PolyData, max_edge_length: f64) -> PolyData {
    let max_len = max_edge_length.max(1e-10);
    let mut out_points = input.points.clone();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        let n = cell.len();
        if n < 2 {
            out_polys.push_cell(cell);
            continue;
        }

        let mut new_cell: Vec<i64> = Vec::new();

        for i in 0..n {
            let id_a = cell[i];
            let id_b = cell[(i + 1) % n];
            let pa = input.points.get(id_a as usize);
            let pb = input.points.get(id_b as usize);

            new_cell.push(id_a);

            let dx = pb[0] - pa[0];
            let dy = pb[1] - pa[1];
            let dz = pb[2] - pa[2];
            let edge_len = (dx * dx + dy * dy + dz * dz).sqrt();

            if edge_len > max_len {
                let n_subdivs = (edge_len / max_len).ceil() as usize;
                for s in 1..n_subdivs {
                    let t = s as f64 / n_subdivs as f64;
                    let idx = out_points.len() as i64;
                    out_points.push([pa[0] + t * dx, pa[1] + t * dy, pa[2] + t * dz]);
                    new_cell.push(idx);
                }
            }
        }

        out_polys.push_cell(&new_cell);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_subdivision_needed() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.05, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = densify(&pd, 1.0);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn subdivide_long_edges() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = densify(&pd, 3.0);
        // Original 3 points + new interpolated points on long edges
        assert!(result.points.len() > 3);
        assert_eq!(result.polys.num_cells(), 1);
        // The single polygon should have more vertices
        assert!(result.polys.cell(0).len() > 3);
    }

    #[test]
    fn preserves_short_edges() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = densify(&pd, 2.0);
        // All edges < 2.0, so no new points
        assert_eq!(result.points.len(), 3);
    }
}
