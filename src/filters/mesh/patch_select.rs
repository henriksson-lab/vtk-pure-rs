use crate::data::{CellArray, Points, PolyData};
use std::collections::{HashMap, VecDeque};

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
    let centroids: Vec<Option<[f64; 3]>> = cells
        .iter()
        .map(|c| cell_centroid(c, &input.points))
        .collect();
    let Some(seed_c) = centroids[seed_cell] else {
        return PolyData::new();
    };

    // Edge adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, c) in cells.iter().enumerate() {
        if centroids[fi].is_none() || c.len() < 2 {
            continue;
        }
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            if a == b
                || valid_point_index(a, input.points.len()).is_none()
                || valid_point_index(b, input.points.len()).is_none()
            {
                continue;
            }
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
    let mut selected = vec![false; n_cells];
    let mut queue = VecDeque::new();
    queue.push_back(seed_cell);
    selected[seed_cell] = true;

    while let Some(fi) = queue.pop_front() {
        for &ni in &adj[fi] {
            if !selected[ni] {
                let Some(c) = centroids[ni] else {
                    continue;
                };
                let d2 = (c[0] - seed_c[0]).powi(2)
                    + (c[1] - seed_c[1]).powi(2)
                    + (c[2] - seed_c[2]).powi(2);
                if d2 <= r2 {
                    selected[ni] = true;
                    queue.push_back(ni);
                }
            }
        }
    }

    // Build output
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for fi in 0..n_cells {
        if !selected[fi] {
            continue;
        }
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

fn cell_centroid(cell: &[i64], points: &Points<f64>) -> Option<[f64; 3]> {
    if cell.is_empty() {
        return None;
    }
    let mut centroid = [0.0; 3];
    for &id in cell {
        let point_id = valid_point_index(id, points.len())?;
        let p = points.get(point_id);
        centroid[0] += p[0];
        centroid[1] += p[1];
        centroid[2] += p[2];
    }
    let n = cell.len() as f64;
    Some([centroid[0] / n, centroid[1] / n, centroid[2] / n])
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
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
    fn invalid_cell_point_ids_are_skipped() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 99]);

        let result = select_patch(&pd, 0, 100.0);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn invalid_seed_cell_returns_empty() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 99]);

        let result = select_patch(&pd, 0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = select_patch(&pd, 0, 1.0);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
