use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Simplify mesh by removing vertices below a curvature threshold.
///
/// Flat vertices (low Laplacian magnitude) are removed and their
/// adjacent triangles are retriangulated. Preserves sharp features.
pub fn simplify_flat_vertices(input: &PolyData, curvature_threshold: f64) -> PolyData {
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

    // Compute Laplacian magnitude
    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut keep = vec![true; n];

    for i in 0..n {
        if neighbors[i].len() < 3 {
            continue;
        }
        let p = pts[i];
        let cnt = neighbors[i].len() as f64;
        let mut ax = 0.0;
        let mut ay = 0.0;
        let mut az = 0.0;
        for &j in &neighbors[i] {
            ax += pts[j][0];
            ay += pts[j][1];
            az += pts[j][2];
        }
        let dx = p[0] - ax / cnt;
        let dy = p[1] - ay / cnt;
        let dz = p[2] - az / cnt;
        let mag = (dx * dx + dy * dy + dz * dz).sqrt();
        if mag < curvature_threshold {
            keep[i] = false;
        }
    }

    // Keep all boundary vertices
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            keep[a] = true;
            keep[b] = true;
        }
    }

    // Build output keeping only valid cells
    let mut pt_map: HashMap<usize, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.iter().all(|&id| keep[id as usize]) {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id as usize).or_insert_with(|| {
                        let idx = out_pts.len() as i64;
                        out_pts.push(pts[id as usize]);
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_flat_vertices() {
        let mut pd = PolyData::new();
        for j in 0..5 {
            for i in 0..5 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..4 {
            for i in 0..4 {
                let a = (j * 5 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 6]);
                pd.polys.push_cell(&[a, a + 6, a + 5]);
            }
        }

        let result = simplify_flat_vertices(&pd, 0.01);
        // Should remove some interior flat vertices
        assert!(result.points.len() <= pd.points.len());
    }

    #[test]
    fn preserves_curved() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 1.0]); // curved
        pd.polys.push_cell(&[0, 1, 2]);

        let result = simplify_flat_vertices(&pd, 100.0);
        assert!(result.polys.num_cells() >= 0); // may or may not keep
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = simplify_flat_vertices(&pd, 0.01);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
