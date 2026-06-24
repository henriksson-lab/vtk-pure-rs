use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Subdivide only edges longer than a threshold.
///
/// Unlike `adaptive_subdivide`, this inserts midpoints on ALL long edges
/// simultaneously and re-triangulates affected faces. Produces a more
/// uniform result in one pass.
pub fn subdivide_long_edges(input: &PolyData, max_length: f64) -> PolyData {
    let max_l2 = max_length * max_length;
    let mut points = input.points.clone();
    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            out_polys.push_cell(cell);
            continue;
        }

        let a = cell[0];
        let b = cell[1];
        let c = cell[2];
        let pa = input.points.get(a as usize);
        let pb = input.points.get(b as usize);
        let pc = input.points.get(c as usize);

        let d_ab = dist2(pa, pb);
        let d_bc = dist2(pb, pc);
        let d_ca = dist2(pc, pa);
        let split_ab = d_ab > max_l2;
        let split_bc = d_bc > max_l2;
        let split_ca = d_ca > max_l2;
        let num_splits = (split_ab as u8) + (split_bc as u8) + (split_ca as u8);

        if num_splits == 0 {
            out_polys.push_cell(&[a, b, c]);
            continue;
        }

        let mid =
            |x: i64, y: i64, pts: &mut Points<f64>, cache: &mut HashMap<(i64, i64), i64>| -> i64 {
                let key = if x < y { (x, y) } else { (y, x) };
                *cache.entry(key).or_insert_with(|| {
                    let px = pts.get(x as usize);
                    let py = pts.get(y as usize);
                    let idx = pts.len() as i64;
                    pts.push([
                        (px[0] + py[0]) * 0.5,
                        (px[1] + py[1]) * 0.5,
                        (px[2] + py[2]) * 0.5,
                    ]);
                    idx
                })
            };

        if num_splits == 3 {
            let mab = mid(a, b, &mut points, &mut midpoint_cache);
            let mbc = mid(b, c, &mut points, &mut midpoint_cache);
            let mca = mid(c, a, &mut points, &mut midpoint_cache);
            out_polys.push_cell(&[a, mab, mca]);
            out_polys.push_cell(&[b, mbc, mab]);
            out_polys.push_cell(&[c, mca, mbc]);
            out_polys.push_cell(&[mab, mbc, mca]);
        } else if num_splits == 2 {
            if !split_ab {
                let mbc = mid(b, c, &mut points, &mut midpoint_cache);
                let mca = mid(c, a, &mut points, &mut midpoint_cache);
                out_polys.push_cell(&[a, b, mbc]);
                out_polys.push_cell(&[a, mbc, mca]);
                out_polys.push_cell(&[mca, mbc, c]);
            } else if !split_bc {
                let mab = mid(a, b, &mut points, &mut midpoint_cache);
                let mca = mid(c, a, &mut points, &mut midpoint_cache);
                out_polys.push_cell(&[b, c, mca]);
                out_polys.push_cell(&[b, mca, mab]);
                out_polys.push_cell(&[mab, mca, a]);
            } else {
                let mab = mid(a, b, &mut points, &mut midpoint_cache);
                let mbc = mid(b, c, &mut points, &mut midpoint_cache);
                out_polys.push_cell(&[c, a, mab]);
                out_polys.push_cell(&[c, mab, mbc]);
                out_polys.push_cell(&[mbc, mab, b]);
            }
        } else {
            if split_ab {
                let m = mid(a, b, &mut points, &mut midpoint_cache);
                out_polys.push_cell(&[a, m, c]);
                out_polys.push_cell(&[m, b, c]);
            } else if split_bc {
                let m = mid(b, c, &mut points, &mut midpoint_cache);
                out_polys.push_cell(&[a, b, m]);
                out_polys.push_cell(&[a, m, c]);
            } else {
                let m = mid(c, a, &mut points, &mut midpoint_cache);
                out_polys.push_cell(&[a, b, m]);
                out_polys.push_cell(&[m, b, c]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = out_polys;
    pd
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_long() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_long_edges(&pd, 5.0);
        assert!(result.polys.num_cells() > 1);
    }

    #[test]
    fn short_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.05, 0.1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_long_edges(&pd, 1.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(subdivide_long_edges(&pd, 1.0).polys.num_cells(), 0);
    }
}
