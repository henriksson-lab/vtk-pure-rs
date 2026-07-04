//! Merge connected components that are close to each other.
use crate::data::{CellArray, PolyData};
pub fn merge_close_components(mesh: &PolyData, max_gap: f64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || !max_gap.is_finite() || max_gap < 0.0 {
        return mesh.clone();
    }
    // Find connected components
    let mut parent: Vec<usize> = (0..n).collect();
    for c in mesh.polys.iter().chain(mesh.lines.iter()) {
        if c.len() < 2 {
            continue;
        }
        let Ok(first) = usize::try_from(c[0]) else {
            continue;
        };
        if first >= n {
            continue;
        }
        for i in 1..c.len() {
            let Ok(idx) = usize::try_from(c[i]) else {
                continue;
            };
            if idx < n {
                union(&mut parent, first, idx);
            }
        }
    }
    for strip in mesh.strips.iter() {
        if strip.len() < 3 {
            continue;
        }
        for i in 0..strip.len() - 2 {
            let tri = if i % 2 == 0 {
                [strip[i], strip[i + 1], strip[i + 2]]
            } else {
                [strip[i + 1], strip[i], strip[i + 2]]
            };
            union_valid_edge(&mut parent, n, tri[0], tri[1]);
            union_valid_edge(&mut parent, n, tri[1], tri[2]);
            union_valid_edge(&mut parent, n, tri[2], tri[0]);
        }
    }
    // Group vertices by component
    let mut comp: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..n {
        comp.entry(find(&mut parent, i)).or_default().push(i);
    }
    let comp_list: Vec<Vec<usize>> = comp.into_values().collect();
    if comp_list.len() <= 1 {
        return mesh.clone();
    }
    // Find centroids
    let centroids: Vec<[f64; 3]> = comp_list
        .iter()
        .map(|verts| {
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &v in verts {
                let p = mesh.points.get(v);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let nf = verts.len() as f64;
            [cx / nf, cy / nf, cz / nf]
        })
        .collect();
    // Merge components whose centroids are within max_gap
    let mut comp_parent: Vec<usize> = (0..comp_list.len()).collect();
    let mut merge_pairs = Vec::new();
    for i in 0..comp_list.len() {
        for j in i + 1..comp_list.len() {
            let d = ((centroids[i][0] - centroids[j][0]).powi(2)
                + (centroids[i][1] - centroids[j][1]).powi(2)
                + (centroids[i][2] - centroids[j][2]).powi(2))
            .sqrt();
            if d <= max_gap {
                union2(&mut comp_parent, i, j);
                merge_pairs.push((i, j));
            }
        }
    }
    // Build bridge edges between merged components
    let mut new_lines = CellArray::new();
    for (i, j) in merge_pairs {
        if find2(&mut comp_parent, i) == find2(&mut comp_parent, j) {
            // Find closest pair of vertices between components
            let mut best_d = f64::INFINITY;
            let mut best_a = 0;
            let mut best_b = 0;
            for &a in &comp_list[i] {
                let pa = mesh.points.get(a);
                for &b in &comp_list[j] {
                    let pb = mesh.points.get(b);
                    let d =
                        (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2);
                    if d < best_d {
                        best_d = d;
                        best_a = a;
                        best_b = b;
                    }
                }
            }
            new_lines.push_cell(&[best_a as i64, best_b as i64]);
        }
    }
    let mut r = mesh.clone();
    // Add bridge lines
    let added_lines = new_lines.num_cells();
    for cell in new_lines.iter() {
        r.lines.push_cell(cell);
    }
    if added_lines > 0 {
        r.cell_data_mut().clear();
    }
    r
}
fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra != rb {
        p[rb] = ra;
    }
}
fn union_valid_edge(p: &mut [usize], n: usize, a_id: i64, b_id: i64) {
    let Some(a) = valid_point_id(a_id, n) else {
        return;
    };
    let Some(b) = valid_point_id(b_id, n) else {
        return;
    };
    union(p, a, b);
}
fn find2(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union2(p: &mut [usize], a: usize, b: usize) {
    let ra = find2(p, a);
    let rb = find2(p, b);
    if ra != rb {
        p[rb] = ra;
    }
}
fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [3.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [3.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = merge_close_components(&m, 5.0);
        assert!(r.lines.num_cells() >= 1);
    }

    #[test]
    fn only_directly_close_component_pairs_are_bridged() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
                [2.0, 0.0, 0.0],
                [2.1, 0.0, 0.0],
                [2.0, 0.1, 0.0],
                [4.0, 0.0, 0.0],
                [4.1, 0.0, 0.0],
                [4.0, 0.1, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5], [6, 7, 8]],
        );
        let r = merge_close_components(&m, 2.0);
        assert_eq!(r.lines.num_cells(), 2);
    }

    #[test]
    fn negative_gap_does_not_merge_components() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let r = merge_close_components(&m, -1.0);
        assert_eq!(r.lines.num_cells(), 0);
    }
}
