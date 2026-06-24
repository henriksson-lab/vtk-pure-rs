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
    for cell in mesh.polys.iter() {
        if cell.len() < 2 {
            continue;
        }
        let first = cell[0] as usize;
        for i in 1..cell.len() {
            union(&mut parent, first, cell[i] as usize);
        }
    }
    let mut comp_verts: std::collections::HashMap<usize, std::collections::HashSet<usize>> =
        std::collections::HashMap::new();
    let mut comp_faces: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut comp_area: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let root = find(&mut parent, cell[0] as usize);
        *comp_faces.entry(root).or_insert(0) += 1;
        for &v in cell {
            comp_verts
                .entry(root)
                .or_default()
                .insert(find(&mut parent, v as usize));
        }
        if cell.len() >= 3 {
            let a = mesh.points.get(cell[0] as usize);
            for i in 1..cell.len() - 1 {
                let b = mesh.points.get(cell[i] as usize);
                let c = mesh.points.get(cell[i + 1] as usize);
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
    let mut result: Vec<ComponentInfo> = comp_verts
        .keys()
        .enumerate()
        .map(|(id, &root)| ComponentInfo {
            component_id: id,
            num_vertices: comp_verts[&root].len(),
            num_faces: *comp_faces.get(&root).unwrap_or(&0),
            area: *comp_area.get(&root).unwrap_or(&0.0),
        })
        .collect();
    result.sort_by(|a, b| b.num_faces.cmp(&a.num_faces));
    result
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
