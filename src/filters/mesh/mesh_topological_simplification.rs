//! Topological simplification by persistence-guided cancellation.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn simplify_by_persistence(mesh: &PolyData, array_name: &str, threshold: f64) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return mesh.clone(),
    };
    let n = mesh.points.len();
    if arr.num_tuples() != n {
        return mesh.clone();
    }
    let mut buf = [0.0f64];
    let mut vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let nb = build_neighbors(mesh, n);
    // Iteratively cancel persistence pairs below threshold
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            if nb[i].is_empty() {
                continue;
            }
            let is_max = nb[i].iter().all(|&j| vals[j] <= vals[i]);
            let is_min = nb[i].iter().all(|&j| vals[j] >= vals[i]);
            if is_max {
                // Find nearest lower neighbor in scalar value.
                let nearest_lower = nb[i].iter().max_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                if let Some(&nl) = nearest_lower {
                    let persistence = vals[i] - vals[nl];
                    if persistence < threshold {
                        let new_value = vals[nl];
                        if vals[i] != new_value {
                            vals[i] = new_value;
                            changed = true;
                        }
                    }
                }
            }
            if is_min {
                let nearest_higher = nb[i].iter().min_by(|&&a, &&b| {
                    vals[a]
                        .partial_cmp(&vals[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                if let Some(&nh) = nearest_higher {
                    let persistence = vals[nh] - vals[i];
                    if persistence < threshold {
                        let new_value = vals[nh];
                        if vals[i] != new_value {
                            vals[i] = new_value;
                            changed = true;
                        }
                    }
                }
            }
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(array_name, vals, 1)));
    r
}
pub use crate::filters::mesh::scalar_field_topology::count_critical_points;
fn build_neighbors(mesh: &PolyData, number_of_points: usize) -> Vec<Vec<usize>> {
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); number_of_points];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            add_edge(cell[i], cell[(i + 1) % nc], &mut neighbors);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_edge(edge[0], edge[1], &mut neighbors);
        }
    }
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            if i % 2 == 0 {
                add_triangle_edges(tri[0], tri[1], tri[2], &mut neighbors);
            } else {
                add_triangle_edges(tri[1], tri[0], tri[2], &mut neighbors);
            }
        }
    }
    neighbors
}

fn add_triangle_edges(a: i64, b: i64, c: i64, neighbors: &mut [Vec<usize>]) {
    add_edge(a, b, neighbors);
    add_edge(b, c, neighbors);
    add_edge(c, a, neighbors);
}

fn add_edge(a: i64, b: i64, neighbors: &mut [Vec<usize>]) {
    if let Some((a, b)) = valid_edge(a, b, neighbors.len()) {
        if !neighbors[a].contains(&b) {
            neighbors[a].push(b);
        }
        if !neighbors[b].contains(&a) {
            neighbors[b].push(a);
        }
    }
}

fn valid_edge(a: i64, b: i64, number_of_points: usize) -> Option<(usize, usize)> {
    if a >= 0 && b >= 0 {
        let a = a as usize;
        let b = b as usize;
        if a < number_of_points && b < number_of_points {
            return Some((a, b));
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_simplify() {
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
                vec![0.0, 0.1, 1.0, 0.05],
                1,
            )));
        let r = simplify_by_persistence(&m, "h", 0.2);
        assert!(r.point_data().get_array("h").is_some());
    }
    #[test]
    fn test_simplify_plateau_terminates() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let r = simplify_by_persistence(&m, "h", 0.2);
        assert!(r.point_data().get_array("h").is_some());
    }
}
