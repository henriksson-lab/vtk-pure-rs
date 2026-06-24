//! Compute topological persistence pairs from scalar field.
use crate::data::PolyData;
pub struct PersistencePair {
    pub birth: f64,
    pub death: f64,
    pub dimension: usize,
    pub birth_vertex: usize,
    pub death_vertex: usize,
}
pub fn persistence_pairs(mesh: &PolyData, array_name: &str) -> Vec<PersistencePair> {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return vec![],
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
    let mut sorted: Vec<usize> = (0..n).collect();
    sorted.sort_by(|&a, &b| {
        vals[a]
            .partial_cmp(&vals[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut parent: Vec<usize> = (0..n).collect();
    let mut active = vec![false; n];
    let mut pairs = Vec::new();
    for &vi in &sorted {
        active[vi] = true;
        let mut roots: Vec<usize> = Vec::new();
        for &ni in &nb[vi] {
            if active[ni] {
                let r = find(&mut parent, ni);
                if !roots.contains(&r) {
                    roots.push(r);
                }
            }
        }
        if roots.is_empty() { /* new component born */
        } else {
            let oldest = *roots
                .iter()
                .min_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            for &r in &roots {
                if r != oldest {
                    pairs.push(PersistencePair {
                        birth: vals[r],
                        death: vals[vi],
                        dimension: 0,
                        birth_vertex: r,
                        death_vertex: vi,
                    });
                    union(&mut parent, oldest, r);
                }
            }
            union(&mut parent, oldest, vi);
        }
    }
    pairs.sort_by(|a, b| {
        (b.death - b.birth)
            .partial_cmp(&(a.death - a.birth))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    pairs
}
pub fn persistence_diagram_values(mesh: &PolyData, array_name: &str) -> Vec<(f64, f64)> {
    persistence_pairs(mesh, array_name)
        .iter()
        .map(|p| (p.birth, p.death))
        .collect()
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
    use crate::data::{AnyDataArray, DataArray};
    #[test]
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4], [1, 4, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 3.0, 0.5, 2.5],
                1,
            )));
        let pairs = persistence_pairs(&m, "h");
        assert!(!pairs.is_empty());
    }
    #[test]
    fn test_diagram() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let d = persistence_diagram_values(&m, "h"); // may be empty for single component
        assert!(d.len() <= 2);
    }
}
