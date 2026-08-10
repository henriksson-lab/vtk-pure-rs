use crate::data::{CellArray, Points, PolyData};

/// Extract specific cells by index from a PolyData.
///
/// Returns a new PolyData containing only the selected polygon cells,
/// with compacted points (only referenced points are kept).
pub fn extract_cells(input: &PolyData, cell_indices: &[usize]) -> PolyData {
    let n_cells = input.polys.num_cells();
    let n_points = input.points.len();
    let pts = input.points.as_flat_slice();
    let mut point_map = vec![-1i64; n_points];
    let mut out_pts = Vec::<f64>::new();
    let mut out_offsets = Vec::<i64>::with_capacity(cell_indices.len() + 1);
    let mut out_conn = Vec::<i64>::with_capacity(cell_indices.len() * 3);
    out_offsets.push(0);

    for &ci in cell_indices {
        if ci >= n_cells {
            continue;
        }
        let cell = input.polys.cell(ci);
        let conn_start = out_conn.len();
        let mut valid_cell = true;
        for &id in cell {
            let Ok(src_id) = usize::try_from(id) else {
                valid_cell = false;
                break;
            };
            if src_id >= n_points {
                valid_cell = false;
                break;
            }

            let mut dst_id = point_map[src_id];
            if dst_id < 0 {
                dst_id = (out_pts.len() / 3) as i64;
                point_map[src_id] = dst_id;
                let b = src_id * 3;
                out_pts.extend_from_slice(&pts[b..b + 3]);
            }
            out_conn.push(dst_id);
        }
        if valid_cell {
            out_offsets.push(out_conn.len() as i64);
        } else {
            out_conn.truncate(conn_start);
        }
    }

    let mut pd = PolyData::new();
    pd.points = Points::from_flat_vec(out_pts);
    pd.polys = CellArray::from_raw(out_offsets, out_conn);
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
