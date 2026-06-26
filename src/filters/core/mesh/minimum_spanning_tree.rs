use crate::data::{CellArray, PolyData};

/// Compute the minimum spanning tree of the mesh edge graph.
///
/// Uses Kruskal's algorithm. Returns a PolyData with line cells
/// representing the MST edges.
pub fn minimum_spanning_tree(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return PolyData::new();
    }

    let mut edges: Vec<(f64, usize, usize)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for cell in input.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(input, edge[0], edge[1], &mut seen, &mut edges);
        }
    }
    for cell in input.polys.iter() {
        for i in 0..cell.len() {
            add_edge(
                input,
                cell[i],
                cell[(i + 1) % cell.len()],
                &mut seen,
                &mut edges,
            );
        }
    }
    for strip in input.strips.iter() {
        for tri in strip.windows(3) {
            add_edge(input, tri[0], tri[1], &mut seen, &mut edges);
            add_edge(input, tri[1], tri[2], &mut seen, &mut edges);
            add_edge(input, tri[2], tri[0], &mut seen, &mut edges);
        }
    }
    edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0u8; n];
    let find = |p: &mut Vec<usize>, mut x: usize| -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    };

    let mut out_lines = CellArray::new();
    for &(_, a, b) in &edges {
        let ra = find(&mut parent, a);
        let rb = find(&mut parent, b);
        if ra != rb {
            if rank[ra] < rank[rb] {
                parent[ra] = rb;
            } else if rank[ra] > rank[rb] {
                parent[rb] = ra;
            } else {
                parent[rb] = ra;
                rank[ra] += 1;
            }
            out_lines.push_cell(&[a as i64, b as i64]);
        }
    }

    let mut pd = PolyData::new();
    pd.points = input.points.clone();
    pd.lines = out_lines;
    pd
}

fn add_edge(
    input: &PolyData,
    a: i64,
    b: i64,
    seen: &mut std::collections::HashSet<(usize, usize)>,
    edges: &mut Vec<(f64, usize, usize)>,
) {
    let n = input.points.len();
    let (Ok(a), Ok(b)) = (usize::try_from(a), usize::try_from(b)) else {
        return;
    };
    if a == b || a >= n || b >= n {
        return;
    }
    let key = if a < b { (a, b) } else { (b, a) };
    if seen.insert(key) {
        let pa = input.points.get(a);
        let pb = input.points.get(b);
        let d =
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        edges.push((d, a, b));
    }
}

/// Compute the total weight (length) of the MST.
pub fn mst_weight(input: &PolyData) -> f64 {
    let mst = minimum_spanning_tree(input);
    let mut total = 0.0;
    for cell in mst.lines.iter() {
        if cell.len() >= 2 {
            let a = mst.points.get(cell[0] as usize);
            let b = mst.points.get(cell[1] as usize);
            total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mst_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let mst = minimum_spanning_tree(&pd);
        assert_eq!(mst.lines.num_cells(), 2); // N-1 edges for tree
    }

    #[test]
    fn mst_weight_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let w = mst_weight(&pd);
        assert!((w - 2.0).abs() < 1e-10); // two edges of length 1
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(minimum_spanning_tree(&pd).lines.num_cells(), 0);
    }

    #[test]
    fn line_cells_contribute_edges() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2]);

        let mst = minimum_spanning_tree(&pd);
        assert_eq!(mst.lines.num_cells(), 2);
        assert!((mst_weight(&pd) - 2.0).abs() < 1e-10);
    }
}
