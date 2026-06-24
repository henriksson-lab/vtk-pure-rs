use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Collapse short edges in a triangle mesh.
///
/// Iteratively collapses edges shorter than `min_length` by merging
/// the two endpoints to their midpoint. Removes resulting degenerate
/// triangles. Runs up to `max_passes` iterations.
pub fn collapse_edges(input: &PolyData, min_length: f64, max_passes: usize) -> PolyData {
    let mut points = input.points.clone();
    let mut tris: Vec<[i64; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .flat_map(|c| (1..c.len() - 1).map(move |i| [c[0], c[i], c[i + 1]]))
        .collect();

    let min_len2 = min_length * min_length;

    for _ in 0..max_passes {
        let mut merge_map: HashMap<i64, i64> = HashMap::new();
        let mut any_collapsed = false;

        // Find short edges and mark for collapse
        for tri in &tris {
            for k in 0..3 {
                let a = resolve(&merge_map, tri[k]);
                let b = resolve(&merge_map, tri[(k + 1) % 3]);
                if a == b {
                    continue;
                }

                let pa = points.get(a as usize);
                let pb = points.get(b as usize);
                let d2 =
                    (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);

                if d2 < min_len2 {
                    // Merge b into a (or a into b, pick lower index)
                    let (keep, remove) = if a < b { (a, b) } else { (b, a) };
                    let pk = points.get(keep as usize);
                    let pr = points.get(remove as usize);
                    // Move kept point to midpoint
                    let mid = [
                        (pk[0] + pr[0]) * 0.5,
                        (pk[1] + pr[1]) * 0.5,
                        (pk[2] + pr[2]) * 0.5,
                    ];
                    // We can't easily modify points in-place with the Points API,
                    // so we'll add a new point and map both to it
                    let new_id = points.len() as i64;
                    points.push(mid);
                    merge_map.insert(keep, new_id);
                    merge_map.insert(remove, new_id);
                    any_collapsed = true;
                }
            }
        }

        if !any_collapsed {
            break;
        }

        // Remap and remove degenerates
        tris = tris
            .iter()
            .filter_map(|tri| {
                let a = resolve(&merge_map, tri[0]);
                let b = resolve(&merge_map, tri[1]);
                let c = resolve(&merge_map, tri[2]);
                if a == b || b == c || a == c {
                    None
                } else {
                    Some([a, b, c])
                }
            })
            .collect();
    }

    // Compact: only keep used points
    let mut used: HashMap<i64, i64> = HashMap::new();
    let mut out_points = Points::<f64>::new();

    let mut out_polys = CellArray::new();
    for tri in &tris {
        let mapped: Vec<i64> = tri
            .iter()
            .map(|&id| {
                *used.entry(id).or_insert_with(|| {
                    let idx = out_points.len() as i64;
                    out_points.push(points.get(id as usize));
                    idx
                })
            })
            .collect();
        out_polys.push_cell(&mapped);
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.polys = out_polys;
    pd
}

fn resolve(map: &HashMap<i64, i64>, mut id: i64) -> i64 {
    let mut seen = 0;
    while let Some(&target) = map.get(&id) {
        if target == id {
            break;
        }
        id = target;
        seen += 1;
        if seen > 100 {
            break;
        } // safety
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_short_edge() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.0, 0.0]); // very close to 0
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = collapse_edges(&pd, 0.01, 5);
        // Edge 0-1 collapsed -> degenerate triangle removed
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn no_collapse_long_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = collapse_edges(&pd, 0.01, 5);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = collapse_edges(&pd, 0.01, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn reduces_points() {
        let mut pd = PolyData::new();
        // Two triangles sharing a very short edge
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.001, 0.001, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 1, 3]);

        let result = collapse_edges(&pd, 0.01, 5);
        assert!(result.points.len() <= 3);
    }
}
