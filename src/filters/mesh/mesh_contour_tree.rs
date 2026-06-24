//! Contour tree from scalar field on mesh (simplified).
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn contour_tree(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if !nb[a].contains(&b) {
                    nb[a].push(b);
                }
                if !nb[b].contains(&a) {
                    nb[b].push(a);
                }
            }
        }
    }
    // Sort vertices by scalar value
    let mut sorted: Vec<usize> = (0..n).collect();
    sorted.sort_by(|&a, &b| {
        vals[a]
            .partial_cmp(&vals[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    // Build join tree: process vertices in ascending order
    let mut parent: Vec<usize> = (0..n).collect();
    let mut active = vec![false; n];
    let mut tree_edges: Vec<(usize, usize)> = Vec::new();
    for &vi in &sorted {
        active[vi] = true;
        let mut components: Vec<usize> = Vec::new();
        for &ni in &nb[vi] {
            if active[ni] {
                let root = find(&mut parent, ni);
                if !components.contains(&root) {
                    components.push(root);
                }
            }
        }
        match components.len() {
            0 => {} // new component
            1 => {
                union(&mut parent, vi, components[0]);
            }
            _ => {
                // merge: this is a join saddle
                for &c in &components {
                    let root = find(&mut parent, c);
                    if root != vi {
                        tree_edges.push((vi, root));
                        union(&mut parent, vi, root);
                    }
                }
            }
        }
    }
    // Build PolyData from tree edges
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for &(a, b) in &tree_edges {
        let ia = *pm.entry(a).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(a));
            i
        });
        let ib = *pm.entry(b).or_insert_with(|| {
            let i = pts.len();
            pts.push(mesh.points.get(b));
            i
        });
        lines.push_cell(&[ia as i64, ib as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 3.0, 2.0],
                1,
            )));
        let r = contour_tree(&m, "h");
        assert!(r.points.len() >= 1);
    }
}
