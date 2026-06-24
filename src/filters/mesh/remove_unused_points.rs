use crate::data::{CellArray, Points, PolyData};

/// Remove points that are not referenced by any cell (polys, lines, verts, strips)
/// and reindex the remaining points and cell connectivity.
pub fn remove_unused_points(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    let mut used = vec![false; n];

    // Mark all referenced points
    mark_used(&input.polys, &mut used);
    mark_used(&input.lines, &mut used);
    mark_used(&input.verts, &mut used);
    mark_used(&input.strips, &mut used);

    // Build old-to-new index mapping; -1 means unused
    let mut old_to_new: Vec<i64> = vec![-1; n];
    let mut new_idx: i64 = 0;
    let mut out_points = Points::<f64>::new();

    for i in 0..n {
        if used[i] {
            old_to_new[i] = new_idx;
            out_points.push(input.points.get(i));
            new_idx += 1;
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = remap_cell_array(&input.polys, &old_to_new);
    pd.lines = remap_cell_array(&input.lines, &old_to_new);
    pd.verts = remap_cell_array(&input.verts, &old_to_new);
    pd.strips = remap_cell_array(&input.strips, &old_to_new);
    pd
}

fn mark_used(cells: &CellArray, used: &mut [bool]) {
    for cell in cells.iter() {
        for &id in cell {
            let idx: usize = id as usize;
            if idx < used.len() {
                used[idx] = true;
            }
        }
    }
}

fn remap_cell_array(src: &CellArray, old_to_new: &[i64]) -> CellArray {
    let mut dst = CellArray::new();
    for cell in src.iter() {
        let remapped: Vec<i64> = cell.iter().map(|&id| old_to_new[id as usize]).collect();
        dst.push_cell(&remapped);
    }
    dst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_unreferenced_points() {
        // 4 points but triangle only uses 0,1,2 — point 3 is unused
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([99.0, 99.0, 99.0]); // unused
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn keeps_all_when_all_used() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn reindexes_correctly() {
        // Points 0,1 unused; triangle uses points 2,3,4
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // unused
        pd.points.push([0.0, 0.0, 0.0]); // unused
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 1.0]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
        // First point in result should be old point 2 => (1,0,0)
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10);
        // Cell should now reference 0,1,2
        let cells: Vec<Vec<i64>> = result.polys.iter().map(|c| c.to_vec()).collect();
        assert_eq!(cells[0], vec![0, 1, 2]);
    }
}
