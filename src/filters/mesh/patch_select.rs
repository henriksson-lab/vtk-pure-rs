use crate::data::{CellArray, Points, PolyData};
use std::collections::{HashMap, HashSet, VecDeque};

/// Select all faces connected to a seed face within a geodesic radius.
///
/// Uses BFS on face adjacency graph, stopping when face centroid
/// distance from seed exceeds `radius`.
pub fn select_patch(input: &PolyData, seed_cell: usize, radius: f64) -> PolyData {
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();
    let n_cells = cells.len();
    if seed_cell >= n_cells {
        return PolyData::new();
    }

    // Face centroids
    let centroids: Vec<[f64; 3]> = cells
        .iter()
        .map(|c| {
            if c.is_empty() {
                return [0.0; 3];
            }
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &id in c {
                let p = input.points.get(id as usize);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let n = c.len() as f64;
            [cx / n, cy / n, cz / n]
        })
        .collect();

    // Edge adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_cells];
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            adj[faces[0]].push(faces[1]);
            adj[faces[1]].push(faces[0]);
        }
    }

    // BFS from seed
    let r2 = radius * radius;
    let seed_c = centroids[seed_cell];
    let mut selected: HashSet<usize> = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(seed_cell);
    selected.insert(seed_cell);

    while let Some(fi) = queue.pop_front() {
        for &ni in &adj[fi] {
            if !selected.contains(&ni) {
                let c = centroids[ni];
                let d2 = (c[0] - seed_c[0]).powi(2)
                    + (c[1] - seed_c[1]).powi(2)
                    + (c[2] - seed_c[2]).powi(2);
                if d2 <= r2 {
                    selected.insert(ni);
                    queue.push_back(ni);
                }
            }
        }
    }

    // Build output
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for &fi in &selected {
        let mapped: Vec<i64> = cells[fi]
            .iter()
            .map(|&id| {
                *pt_map.entry(id).or_insert_with(|| {
                    let idx = out_pts.len() as i64;
                    out_pts.push(input.points.get(id as usize));
                    idx
                })
            })
            .collect();
        out_polys.push_cell(&mapped);
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
    fn select_from_center() {
        let mut pd = PolyData::new();
        for j in 0..4 {
            for i in 0..4 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..3 {
            for i in 0..3 {
                let a = (j * 4 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 5]);
                pd.polys.push_cell(&[a, a + 5, a + 4]);
            }
        }

        let result = select_patch(&pd, 8, 0.8); // small radius
        assert!(result.polys.num_cells() > 0);
        assert!(
            result.polys.num_cells() < 18,
            "got {}",
            result.polys.num_cells()
        );
    }

    #[test]
    fn large_radius_all() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = select_patch(&pd, 0, 100.0);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn invalid_seed() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let result = select_patch(&pd, 999, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = select_patch(&pd, 0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
