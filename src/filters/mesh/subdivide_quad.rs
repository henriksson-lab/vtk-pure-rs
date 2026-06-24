use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Quad subdivision: split each quad into 4 by inserting edge and face midpoints.
///
/// Triangles are passed through unchanged. Quads get a center point
/// and 4 edge midpoints, producing 4 sub-quads.
pub fn subdivide_quads(input: &PolyData) -> PolyData {
    let mut points = input.points.clone();
    let mut out_polys = CellArray::new();
    let mut mid_cache: HashMap<(i64, i64), i64> = HashMap::new();

    for cell in input.polys.iter() {
        if cell.len() == 4 {
            let a = cell[0];
            let b = cell[1];
            let c = cell[2];
            let d = cell[3];
            let mab = get_mid(a, b, &mut points, &mut mid_cache);
            let mbc = get_mid(b, c, &mut points, &mut mid_cache);
            let mcd = get_mid(c, d, &mut points, &mut mid_cache);
            let mda = get_mid(d, a, &mut points, &mut mid_cache);
            // Center point
            let pa = points.get(a as usize);
            let pb = points.get(b as usize);
            let pc = points.get(c as usize);
            let pd_p = points.get(d as usize);
            let center_idx = points.len() as i64;
            points.push([
                (pa[0] + pb[0] + pc[0] + pd_p[0]) * 0.25,
                (pa[1] + pb[1] + pc[1] + pd_p[1]) * 0.25,
                (pa[2] + pb[2] + pc[2] + pd_p[2]) * 0.25,
            ]);

            out_polys.push_cell(&[a, mab, center_idx, mda]);
            out_polys.push_cell(&[mab, b, mbc, center_idx]);
            out_polys.push_cell(&[center_idx, mbc, c, mcd]);
            out_polys.push_cell(&[mda, center_idx, mcd, d]);
        } else if cell.len() == 3 {
            out_polys.push_cell(cell);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivide_single_quad() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]);

        let result = subdivide_quads(&pd);
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn triangle_passthrough() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_quads(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn mixed_mesh() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([0.5, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2, 3]); // quad
        pd.polys.push_cell(&[2, 3, 4]); // triangle

        let result = subdivide_quads(&pd);
        assert_eq!(result.polys.num_cells(), 5); // 4+1
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(subdivide_quads(&pd).polys.num_cells(), 0);
    }
}
