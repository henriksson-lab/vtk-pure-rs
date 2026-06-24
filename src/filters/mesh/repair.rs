use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Remove duplicate triangles (same 3 vertices in any order).
pub fn remove_duplicate_cells(input: &PolyData) -> PolyData {
    let mut seen: std::collections::HashSet<[i64; 3]> = std::collections::HashSet::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let mut key = [cell[0], cell[1], cell[2]];
        key.sort();
        if seen.insert(key) {
            out_polys.push_cell(cell);
        }
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    pd
}

/// Remove zero-area (degenerate) triangles.
pub fn remove_degenerate_cells(input: &PolyData, min_area: f64) -> PolyData {
    let min_a2 = min_area * min_area * 4.0; // compare with 4*area^2 to avoid sqrt
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cx = e1[1] * e2[2] - e1[2] * e2[1];
        let cy = e1[2] * e2[0] - e1[0] * e2[2];
        let cz = e1[0] * e2[1] - e1[1] * e2[0];
        let area2_x4 = cx * cx + cy * cy + cz * cz;
        if area2_x4 >= min_a2 {
            out_polys.push_cell(cell);
        }
    }

    let mut pd = input.clone();
    pd.polys = out_polys;
    pd
}

/// Remove isolated vertices (points not referenced by any cell).
pub fn remove_unused_points(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut used = vec![false; n];

    for cell in input.polys.iter() {
        for &id in cell.iter() {
            used[id as usize] = true;
        }
    }
    for cell in input.lines.iter() {
        for &id in cell.iter() {
            used[id as usize] = true;
        }
    }
    for cell in input.verts.iter() {
        for &id in cell.iter() {
            used[id as usize] = true;
        }
    }
    for cell in input.strips.iter() {
        for &id in cell.iter() {
            used[id as usize] = true;
        }
    }

    let mut pt_map = vec![-1i64; n];
    let mut out_points = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pt_map[i] = out_points.len() as i64;
            out_points.push(input.points.get(i));
        }
    }

    let remap = |cell: &[i64]| -> Vec<i64> { cell.iter().map(|&id| pt_map[id as usize]).collect() };

    let mut out_polys = CellArray::new();
    for cell in input.polys.iter() {
        out_polys.push_cell(&remap(cell));
    }
    let mut out_lines = CellArray::new();
    for cell in input.lines.iter() {
        out_lines.push_cell(&remap(cell));
    }
    let mut out_verts = CellArray::new();
    for cell in input.verts.iter() {
        out_verts.push_cell(&remap(cell));
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd.lines = out_lines;
    pd.verts = out_verts;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_duplicates() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 2]); // duplicate
        pd.polys.push_cell(&[2, 1, 0]); // same vertices different order

        let result = remove_duplicate_cells(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn remove_degenerate() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([0.0, 0.0, 0.0]); // degenerate
        pd.points.push([0.001, 0.0, 0.0]);
        pd.points.push([0.0, 0.001, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]); // good
        pd.polys.push_cell(&[3, 4, 5]); // tiny

        let result = remove_degenerate_cells(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn remove_unused() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // 0: used
        pd.points.push([1.0, 0.0, 0.0]); // 1: used
        pd.points.push([0.0, 1.0, 0.0]); // 2: used
        pd.points.push([5.0, 5.0, 5.0]); // 3: unused
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remove_unused_points(&pd);
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(remove_duplicate_cells(&pd).polys.num_cells(), 0);
        assert_eq!(remove_unused_points(&pd).points.len(), 0);
    }
}
