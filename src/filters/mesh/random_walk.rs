use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Simulate random walks on the mesh graph from source vertices.
///
/// For each vertex, counts how many random walks (from various sources)
/// visit it. Adds "WalkVisitCount" scalar. Approximates stationary distribution.
pub fn random_walk_distribution(
    input: &PolyData,
    sources: &[usize],
    walks_per_source: usize,
    walk_length: usize,
) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    let mut visit_count = vec![0.0f64; n];
    let mut rng = 42u64;

    for &src in sources {
        if src >= n {
            continue;
        }
        for _ in 0..walks_per_source {
            let mut cur = src;
            for _ in 0..walk_length {
                visit_count[cur] += 1.0;
                if adj[cur].is_empty() {
                    break;
                }
                rng = rng
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let idx = (rng >> 33) as usize % adj[cur].len();
                cur = adj[cur][idx];
            }
            visit_count[cur] += 1.0;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "WalkVisitCount",
            visit_count,
            1,
        )));
    pd
}

/// Generate a single random walk as a polyline.
pub fn random_walk_path(input: &PolyData, start: usize, length: usize, seed: u64) -> PolyData {
    let n = input.points.len();
    if start >= n {
        return PolyData::new();
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % cell.len()] as usize;
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    let mut rng = seed;
    let mut path = vec![start];
    let mut cur = start;
    for _ in 0..length {
        if adj[cur].is_empty() {
            break;
        }
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (rng >> 33) as usize % adj[cur].len();
        cur = adj[cur][idx];
        path.push(cur);
    }

    let mut out_pts = Points::<f64>::new();
    let ids: Vec<i64> = path
        .iter()
        .map(|&p| {
            let i = out_pts.len() as i64;
            out_pts.push(input.points.get(p));
            i
        })
        .collect();
    let mut out_lines = CellArray::new();
    out_lines.push_cell(&ids);

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.lines = out_lines;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_visits() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let result = random_walk_distribution(&pd, &[0], 10, 5);
        let arr = result.point_data().get_array("WalkVisitCount").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0); // source visited
    }

    #[test]
    fn walk_path() {
        let mut pd = PolyData::new();
        for i in 0..5 {
            pd.points.push([i as f64, 0.0, 0.0]);
        }
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[2, 3, 4]);

        let path = random_walk_path(&pd, 0, 10, 42);
        assert!(path.lines.num_cells() == 1);
        assert!(path.points.len() > 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(random_walk_distribution(&pd, &[0], 5, 5).points.len(), 0);
    }
}
