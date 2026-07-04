//! Extract largest connected component by cell count.
use crate::data::{CellArray, Points, PolyData};

pub fn extract_largest_connected(mesh: &PolyData) -> PolyData {
    crate::filters::mesh::extract_largest_component::extract_largest_component(mesh)
}

pub fn extract_smallest_connected(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return PolyData::new();
    }
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut parent: Vec<usize> = (0..n).collect();
    for c in &cells {
        if c.len() < 2 {
            continue;
        }
        let Some(first) = valid_point_index(c[0], n) else {
            continue;
        };
        for &id in &c[1..] {
            if let Some(idx) = valid_point_index(id, n) {
                union(&mut parent, first, idx);
            }
        }
    }
    let mut comp_faces: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        if c.is_empty() {
            continue;
        }
        let Some(first) = valid_point_index(c[0], n) else {
            continue;
        };
        let root = find(&mut parent, first);
        comp_faces.entry(root).or_default().push(ci);
    }
    let smallest = comp_faces.values().min_by_key(|v| v.len());
    let kept_cells: std::collections::HashSet<usize> = match smallest {
        Some(v) => v.iter().copied().collect(),
        None => return mesh.clone(),
    };
    let mut used = vec![false; n];
    let mut kept = Vec::new();
    for (ci, c) in cells.iter().enumerate() {
        if kept_cells.contains(&ci) {
            for &v in c {
                if let Some(idx) = valid_point_index(v, n) {
                    used[idx] = true;
                }
            }
            kept.push(c.clone());
        }
    }
    let mut pm = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        let remapped: Option<Vec<i64>> = c
            .iter()
            .map(|&v| valid_point_index(v, n).map(|idx| pm[idx] as i64))
            .collect();
        if let Some(remapped) = remapped {
            polys.push_cell(&remapped);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_largest() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [4, 5, 6]],
        );
        let r = extract_largest_connected(&m);
        assert_eq!(r.polys.num_cells(), 2);
    }
    #[test]
    fn test_smallest() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [4, 5, 6]],
        );
        let r = extract_smallest_connected(&m);
        assert_eq!(r.polys.num_cells(), 1);
    }
}
