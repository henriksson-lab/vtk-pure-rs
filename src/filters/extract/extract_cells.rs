use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Extract specific cells by index from a PolyData.
///
/// Returns a new PolyData containing only the selected polygon cells,
/// with compacted points (only referenced points are kept).
pub fn extract_cells(input: &PolyData, cell_indices: &[usize]) -> PolyData {
    let mut point_map: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    let n_cells = input.polys.num_cells();

    for &ci in cell_indices {
        if ci >= n_cells {
            continue;
        }
        let cell = input.polys.cell(ci);
        let remapped: Vec<i64> = cell
            .iter()
            .map(|&id| {
                *point_map.entry(id).or_insert_with(|| {
                    let idx = out_points.len() as i64;
                    out_points.push(input.points.get(id as usize));
                    idx
                })
            })
            .collect();
        out_polys.push_cell(&remapped);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

/// Extract cells where a predicate returns true.
pub fn extract_cells_by_predicate<F>(input: &PolyData, predicate: F) -> PolyData
where
    F: Fn(usize, &[i64]) -> bool,
{
    let indices: Vec<usize> = (0..input.polys.num_cells())
        .filter(|&i| predicate(i, input.polys.cell(i)))
        .collect();
    extract_cells(input, &indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_cell() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4]],
        );
        let result = extract_cells(&pd, &[1]);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3); // only 3 points used
    }

    #[test]
    fn extract_by_predicate() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        // Keep only cells whose first vertex has x > 5
        let result = extract_cells_by_predicate(&pd, |_i, cell| {
            let p = pd.points.get(cell[0] as usize);
            p[0] > 5.0
        });
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn extract_empty() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = extract_cells(&pd, &[]);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
