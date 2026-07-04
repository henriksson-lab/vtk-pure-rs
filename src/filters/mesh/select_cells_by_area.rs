use crate::data::{CellArray, Points, PolyData};

/// Select polygon cells whose area falls within [min_area, max_area].
///
/// Returns a new PolyData containing only the selected cells with
/// compacted points (unused points are removed).
pub fn select_cells_by_area(input: &PolyData, min_area: f64, max_area: f64) -> PolyData {
    // First pass: compute area for each cell and decide which to keep.
    let mut keep_cells: Vec<Vec<i64>> = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 || !valid_cell(input, cell) {
            continue;
        }

        let area: f64 = polygon_area(input, cell);
        if area >= min_area && area <= max_area {
            keep_cells.push(cell.to_vec());
        }
    }

    // Collect used point indices and build a mapping.
    let mut used = vec![false; input.points.len()];
    for cell in &keep_cells {
        for &idx in cell {
            used[idx as usize] = true;
        }
    }

    let mut old_to_new: Vec<i64> = vec![-1; input.points.len()];
    let mut new_points: Points<f64> = Points::new();
    let mut next_id: i64 = 0;
    for (i, &is_used) in used.iter().enumerate() {
        if is_used {
            old_to_new[i] = next_id;
            next_id += 1;
            new_points.push(input.points.get(i));
        }
    }

    // Build new cell array with remapped indices.
    let mut new_polys = CellArray::new();
    for cell in &keep_cells {
        let remapped: Vec<i64> = cell.iter().map(|&idx| old_to_new[idx as usize]).collect();
        new_polys.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = new_points;
    pd.polys = new_polys;
    pd
}

fn valid_cell(input: &PolyData, cell: &[i64]) -> bool {
    cell.iter()
        .all(|&idx| idx >= 0 && (idx as usize) < input.points.len())
}

fn polygon_area(input: &PolyData, cell: &[i64]) -> f64 {
    let p0 = input.points.get(cell[0] as usize);
    let mut total: f64 = 0.0;

    for i in 1..cell.len() - 1 {
        let p1 = input.points.get(cell[i] as usize);
        let p2 = input.points.get(cell[i + 1] as usize);
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        total += 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_large_triangle() {
        // Two triangles: one with area 0.5, one with area 2.0
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [0.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = select_cells_by_area(&pd, 1.0, 10.0);
        assert_eq!(result.polys.num_cells(), 1);
        // Only the larger triangle should remain
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn select_all() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = select_cells_by_area(&pd, 0.0, 100.0);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn select_none() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = select_cells_by_area(&pd, 10.0, 20.0);
        assert_eq!(result.polys.num_cells(), 0);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn skips_invalid_cell_ids() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.polys.push_cell(&[0, 1, 99]);
        pd.polys.push_cell(&[0, -1, 2]);

        let result = select_cells_by_area(&pd, 0.0, 10.0);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }
}
