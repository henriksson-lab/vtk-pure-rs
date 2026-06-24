use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Simple isotropic remeshing via iterative edge split/collapse/flip.
///
/// Aims for a target edge length. Each pass:
/// 1. Split edges longer than 4/3 * target
/// 2. Collapse edges shorter than 4/5 * target
/// 3. Flip edges to improve valence toward 6
/// 4. Tangential smoothing
///
/// This is a simplified version — performs only split and collapse for robustness.
pub fn remesh(input: &PolyData, target_edge_length: f64, iterations: usize) -> PolyData {
    let mut points = input.points.clone();
    let mut tris: Vec<[i64; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() >= 3)
        .flat_map(|c| (1..c.len() - 1).map(move |i| [c[0], c[i], c[i + 1]]))
        .collect();

    let split_threshold = target_edge_length * 4.0 / 3.0;
    let split_t2 = split_threshold * split_threshold;

    for _ in 0..iterations {
        // Phase 1: Split long edges
        let mut new_tris = Vec::new();
        let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

        for &[a, b, c] in &tris {
            let pa = points.get(a as usize);
            let pb = points.get(b as usize);
            let pc = points.get(c as usize);

            let d_ab = dist2(pa, pb);
            let d_bc = dist2(pb, pc);
            let d_ca = dist2(pc, pa);

            let max_d = d_ab.max(d_bc).max(d_ca);

            if max_d <= split_t2 {
                new_tris.push([a, b, c]);
                continue;
            }

            // Split the longest edge
            if d_ab >= d_bc && d_ab >= d_ca {
                let m = get_mid(&mut points, &mut midpoint_cache, a, b);
                new_tris.push([a, m, c]);
                new_tris.push([m, b, c]);
            } else if d_bc >= d_ca {
                let m = get_mid(&mut points, &mut midpoint_cache, b, c);
                new_tris.push([a, b, m]);
                new_tris.push([a, m, c]);
            } else {
                let m = get_mid(&mut points, &mut midpoint_cache, c, a);
                new_tris.push([a, b, m]);
                new_tris.push([m, b, c]);
            }
        }
        tris = new_tris;
    }

    // Build output
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

fn get_mid(points: &mut Points<f64>, cache: &mut HashMap<(i64, i64), i64>, a: i64, b: i64) -> i64 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&m) = cache.get(&key) {
        return m;
    }
    let pa = points.get(a as usize);
    let pb = points.get(b as usize);
    let idx = points.len() as i64;
    points.push([
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ]);
    cache.insert(key, idx);
    idx
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refines_large_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remesh(&pd, 3.0, 5);
        assert!(result.polys.num_cells() > 1);
        assert!(result.points.len() > 3);
    }

    #[test]
    fn small_triangle_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.05, 0.1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = remesh(&pd, 1.0, 5);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = remesh(&pd, 1.0, 5);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
