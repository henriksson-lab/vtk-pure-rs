//! Compute sizes (vertex count, face count, area) of each connected component.
use crate::data::PolyData;

pub struct ComponentInfo {
    pub component_id: usize,
    pub num_vertices: usize,
    pub num_faces: usize,
    pub area: f64,
}
pub fn component_sizes(mesh: &PolyData) -> Vec<ComponentInfo> {
    let n = mesh.points.len();
    if n == 0 {
        return vec![];
    }
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0usize; n];
    for cells in [&mesh.verts, &mesh.lines, &mesh.polys, &mesh.strips] {
        for cell in cells.iter() {
            let mut ids = cell.iter().filter_map(|&id| valid_point_id(id, n));
            let Some(first) = ids.next() else {
                continue;
            };
            for id in ids {
                union(&mut parent, &mut rank, first, id);
            }
        }
    }
    let mut comp_verts: std::collections::HashMap<usize, std::collections::HashSet<usize>> =
        std::collections::HashMap::new();
    let mut comp_faces: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut comp_area: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        comp_verts.entry(root).or_default().insert(i);
    }
    for cell in mesh.polys.iter() {
        let valid: Vec<usize> = cell
            .iter()
            .filter_map(|&id| valid_point_id(id, n))
            .collect();
        if valid.is_empty() {
            continue;
        }
        let root = find(&mut parent, valid[0]);
        *comp_faces.entry(root).or_insert(0) += 1;
        if valid.len() >= 3 {
            let a = mesh.points.get(valid[0]);
            for i in 1..valid.len() - 1 {
                let b = mesh.points.get(valid[i]);
                let c = mesh.points.get(valid[i + 1]);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                *comp_area.entry(root).or_insert(0.0) += 0.5
                    * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                        + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                        + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
                    .sqrt();
            }
        }
    }
    let mut roots: Vec<usize> = comp_verts.keys().copied().collect();
    roots.sort_unstable();
    let mut result: Vec<ComponentInfo> = roots
        .into_iter()
        .enumerate()
        .map(|(component_id, root)| ComponentInfo {
            component_id,
            num_vertices: comp_verts[&root].len(),
            num_faces: *comp_faces.get(&root).unwrap_or(&0),
            area: *comp_area.get(&root).unwrap_or(&0.0),
        })
        .collect();
    result.sort_by(|a, b| b.num_faces.cmp(&a.num_faces));
    for (component_id, info) in result.iter_mut().enumerate() {
        info.component_id = component_id;
    }
    result
}
fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        p[ra] = rb;
    } else if rank[ra] > rank[rb] {
        p[rb] = ra;
    } else {
        p[rb] = ra;
        rank[ra] += 1;
    }
}

fn valid_point_id(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&idx| idx < n)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_single() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let cs = component_sizes(&m);
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].num_faces, 1);
    }
    #[test]
    fn test_two() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [10.0, 10.0, 0.0],
                [11.0, 10.0, 0.0],
                [10.5, 11.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let cs = component_sizes(&m);
        assert_eq!(cs.len(), 2);
    }
}
