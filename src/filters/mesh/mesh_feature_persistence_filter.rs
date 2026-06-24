//! Filter mesh features by topological persistence.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn persistence_simplify(mesh: &PolyData, array_name: &str, min_persistence: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
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
    // Find critical points and their persistence
    let mut simplified = vals.clone();
    let mut sorted: Vec<usize> = (0..n).collect();
    sorted.sort_by(|&a, &b| {
        vals[a]
            .partial_cmp(&vals[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut parent: Vec<usize> = (0..n).collect();
    let mut active = vec![false; n];
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
        if roots.len() > 1 {
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
                    let persistence = vals[vi] - vals[r];
                    if persistence < min_persistence {
                        // Cancel this pair: set all vertices in this component to the merge value
                        for j in 0..n {
                            if find(&mut parent, j) == r {
                                simplified[j] = vals[vi];
                            }
                        }
                    }
                    union(&mut parent, oldest, r);
                }
            }
            union(&mut parent, oldest, vi);
        } else if !roots.is_empty() {
            union(&mut parent, roots[0], vi);
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, simplified, 1,
        )));
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
                vec![0.0, 0.5, 3.0, 0.3],
                1,
            )));
        let r = persistence_simplify(&m, "h", 0.3);
        assert!(r.point_data().get_array("h").is_some());
    }
}
