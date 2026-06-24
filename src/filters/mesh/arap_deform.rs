use crate::data::{Points, PolyData};
use std::collections::HashMap;

/// As-Rigid-As-Possible (ARAP) deformation (simplified).
///
/// Iteratively adjusts free vertices to minimize edge-length distortion
/// while respecting handle constraints. Approximation using local
/// rigidity via Laplacian fitting.
pub fn arap_deform(input: &PolyData, handles: &[(usize, [f64; 3])], iterations: usize) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !neighbors[a].contains(&b) {
                neighbors[a].push(b);
            }
            if !neighbors[b].contains(&a) {
                neighbors[b].push(a);
            }
        }
    }

    let fixed: HashMap<usize, [f64; 3]> = handles.iter().cloned().collect();
    let orig: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut pts = orig.clone();

    // Apply handles
    for (&i, &p) in &fixed {
        if i < n {
            pts[i] = p;
        }
    }

    for _ in 0..iterations {
        let mut new_pts = pts.clone();
        for i in 0..n {
            if fixed.contains_key(&i) {
                continue;
            }
            if neighbors[i].is_empty() {
                continue;
            }

            // For each neighbor, compute target position that preserves original edge length
            let mut tx = 0.0;
            let mut ty = 0.0;
            let mut tz = 0.0;
            for &j in &neighbors[i] {
                let orig_d = ((orig[i][0] - orig[j][0]).powi(2)
                    + (orig[i][1] - orig[j][1]).powi(2)
                    + (orig[i][2] - orig[j][2]).powi(2))
                .sqrt();
                let cur_d = ((pts[i][0] - pts[j][0]).powi(2)
                    + (pts[i][1] - pts[j][1]).powi(2)
                    + (pts[i][2] - pts[j][2]).powi(2))
                .sqrt();
                if cur_d > 1e-15 {
                    let dir = [
                        (pts[i][0] - pts[j][0]) / cur_d,
                        (pts[i][1] - pts[j][1]) / cur_d,
                        (pts[i][2] - pts[j][2]) / cur_d,
                    ];
                    tx += pts[j][0] + dir[0] * orig_d;
                    ty += pts[j][1] + dir[1] * orig_d;
                    tz += pts[j][2] + dir[2] * orig_d;
                } else {
                    tx += pts[j][0];
                    ty += pts[j][1];
                    tz += pts[j][2];
                }
            }
            let cnt = neighbors[i].len() as f64;
            new_pts[i] = [tx / cnt, ty / cnt, tz / cnt];
        }
        pts = new_pts;
        // Re-apply handles
        for (&i, &p) in &fixed {
            if i < n {
                pts[i] = p;
            }
        }
    }

    let mut points = Points::<f64>::new();
    for p in &pts {
        points.push(*p);
    }
    let mut pd = input.clone();
    pd.points = points;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_pinned() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = arap_deform(&pd, &[(0, [0.0, 5.0, 0.0])], 10);
        let p = result.points.get(0);
        assert_eq!(p[1], 5.0);
    }

    #[test]
    fn preserves_edge_lengths() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = arap_deform(&pd, &[(0, [0.0, 0.1, 0.0])], 20);
        // Edge lengths should be approximately preserved
        let d01 = ((result.points.get(0)[0] - result.points.get(1)[0]).powi(2)
            + (result.points.get(0)[1] - result.points.get(1)[1]).powi(2))
        .sqrt();
        assert!((d01 - 1.0).abs() < 0.5); // approximately preserved
    }

    #[test]
    fn no_handles_noop() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 2.0, 3.0]);
        let result = arap_deform(&pd, &[], 10);
        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(arap_deform(&pd, &[], 5).points.len(), 0);
    }
}
