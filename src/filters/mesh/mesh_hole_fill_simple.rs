//! Simple hole filling by connecting boundary loops.
use crate::data::PolyData;
pub fn fill_holes_fan(mesh: &PolyData) -> PolyData {
    let npts = mesh.points.len();
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let Some(a) = valid_point_id(cell[i], npts) else {
                continue;
            };
            let Some(b) = valid_point_id(cell[(i + 1) % nc], npts) else {
                continue;
            };
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
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
            if !valid_triangle(&tri, npts) {
                continue;
            }
            count_edge(tri[0], tri[1], npts, &mut ec);
            count_edge(tri[1], tri[2], npts, &mut ec);
            count_edge(tri[2], tri[0], npts, &mut ec);
        }
    }
    let boundary: Vec<(usize, usize)> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    if boundary.is_empty() {
        return mesh.clone();
    }
    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &boundary {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let mut result = mesh.clone();
    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &start in adj.keys() {
        if visited.contains(&start) {
            continue;
        }
        if adj.get(&start).map_or(0, Vec::len) != 2 {
            visited.insert(start);
            continue;
        }
        let mut loop_v = vec![start];
        let mut prev = usize::MAX;
        let mut cur = start;
        visited.insert(start);
        loop {
            let Some(next) = adj
                .get(&cur)
                .and_then(|nbs| nbs.iter().copied().find(|&n| n != prev))
            else {
                loop_v.clear();
                break;
            };
            if next == start {
                break;
            }
            if visited.contains(&next) || adj.get(&next).map_or(0, Vec::len) != 2 {
                loop_v.clear();
                break;
            }
            visited.insert(next);
            loop_v.push(next);
            prev = cur;
            cur = next;
        }
        if loop_v.len() >= 3 {
            for i in 1..loop_v.len() - 1 {
                result
                    .polys
                    .push_cell(&[loop_v[0] as i64, loop_v[i] as i64, loop_v[i + 1] as i64]);
            }
        }
    }
    result
}

fn valid_point_id(point_id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(point_id)
        .ok()
        .filter(|&point_id| point_id < n_points)
}

fn valid_triangle(tri: &[i64], n_points: usize) -> bool {
    tri.len() == 3
        && tri[0] != tri[1]
        && tri[1] != tri[2]
        && tri[2] != tri[0]
        && tri
            .iter()
            .all(|&point_id| valid_point_id(point_id, n_points).is_some())
}

fn count_edge(
    a: i64,
    b: i64,
    n_points: usize,
    ec: &mut std::collections::HashMap<(usize, usize), usize>,
) {
    let Some(a) = valid_point_id(a, n_points) else {
        return;
    };
    let Some(b) = valid_point_id(b, n_points) else {
        return;
    };
    *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_fill() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = fill_holes_fan(&m);
        assert!(r.polys.num_cells() >= 1);
    }
}
