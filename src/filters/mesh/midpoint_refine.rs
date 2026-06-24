use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Refine a mesh by inserting midpoints on all edges and splitting each triangle into 4.
///
/// Pure midpoint subdivision (no smoothing). Equivalent to one level of
/// Loop subdivision without the vertex repositioning step.
pub fn midpoint_refine(input: &PolyData) -> PolyData {
    let mut points = input.points.clone();
    let mut mid_cache: HashMap<(i64, i64), i64> = HashMap::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() == 3 {
            let a = cell[0];
            let b = cell[1];
            let c = cell[2];
            let mab = get_mid(a, b, &mut points, &mut mid_cache);
            let mbc = get_mid(b, c, &mut points, &mut mid_cache);
            let mca = get_mid(c, a, &mut points, &mut mid_cache);
            out_polys.push_cell(&[a, mab, mca]);
            out_polys.push_cell(&[mab, b, mbc]);
            out_polys.push_cell(&[mca, mbc, c]);
            out_polys.push_cell(&[mab, mbc, mca]);
        } else if cell.len() == 4 {
            let a = cell[0];
            let b = cell[1];
            let c = cell[2];
            let d = cell[3];
            let mab = get_mid(a, b, &mut points, &mut mid_cache);
            let mbc = get_mid(b, c, &mut points, &mut mid_cache);
            let mcd = get_mid(c, d, &mut points, &mut mid_cache);
            let mda = get_mid(d, a, &mut points, &mut mid_cache);
            let center = points.len() as i64;
            let pa = points.get(a as usize);
            let pc = points.get(c as usize);
            points.push([
                (pa[0] + pc[0]) * 0.5,
                (pa[1] + pc[1]) * 0.5,
                (pa[2] + pc[2]) * 0.5,
            ]);
            out_polys.push_cell(&[a, mab, center, mda]);
            out_polys.push_cell(&[mab, b, mbc, center]);
            out_polys.push_cell(&[center, mbc, c, mcd]);
            out_polys.push_cell(&[mda, center, mcd, d]);
        } else {
            out_polys.push_cell(cell);
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = out_polys;
    pd
}

fn get_mid(a: i64, b: i64, pts: &mut Points<f64>, cache: &mut HashMap<(i64, i64), i64>) -> i64 {
    let key = if a < b { (a, b) } else { (b, a) };
    *cache.entry(key).or_insert_with(|| {
        let pa = pts.get(a as usize);
        let pb = pts.get(b as usize);
        let idx = pts.len() as i64;
        pts.push([
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ]);
        idx
    })
}

/// Count how many times midpoint refinement is needed to reach a target face count.
pub fn refinement_levels_for_target(current_faces: usize, target_faces: usize) -> usize {
    let mut faces = current_faces;
    let mut levels = 0;
    while faces < target_faces {
        faces *= 4;
        levels += 1;
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refine_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = midpoint_refine(&pd);
        assert_eq!(result.polys.num_cells(), 4);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn refine_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = midpoint_refine(&pd);
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn shared_edges_reuse_midpoints() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = midpoint_refine(&pd);
        assert_eq!(result.polys.num_cells(), 8);
        // 4 original + 5 edge midpoints (shared edge 0-2 only counted once) = 9
        assert!(result.points.len() <= 9);
    }

    #[test]
    fn target_levels() {
        assert_eq!(refinement_levels_for_target(1, 100), 4); // 1->4->16->64->256
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(midpoint_refine(&pd).polys.num_cells(), 0);
    }
}
