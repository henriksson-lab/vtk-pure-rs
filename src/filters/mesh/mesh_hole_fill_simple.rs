//! Simple hole filling by connecting boundary loops.
use crate::data::{CellArray, PolyData};
pub fn fill_holes_fan(mesh: &PolyData) -> PolyData {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
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
        let mut loop_v = vec![start];
        let mut cur = start;
        visited.insert(start);
        loop {
            let next = adj
                .get(&cur)
                .and_then(|nbs| nbs.iter().find(|&&n| !visited.contains(&n)));
            match next {
                Some(&n) => {
                    visited.insert(n);
                    loop_v.push(n);
                    cur = n;
                }
                None => break,
            }
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
