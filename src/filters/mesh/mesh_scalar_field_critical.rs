//! Find critical points of a scalar field on mesh (minima, maxima, saddles).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub struct CriticalPoint {
    pub vertex: usize,
    pub value: f64,
    pub kind: CriticalKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CriticalKind {
    Minimum,
    Maximum,
    Saddle,
    Regular,
}
pub fn find_critical_points(mesh: &PolyData, array_name: &str) -> Vec<CriticalPoint> {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 && a.num_tuples() == mesh.points.len() => a,
        _ => return vec![],
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut link_edges: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
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
            if nc > 2 {
                let center = cell[i] as usize;
                let prev = cell[(i + nc - 1) % nc] as usize;
                let next = cell[(i + 1) % nc] as usize;
                if center < n && prev < n && next < n && prev != next {
                    link_edges[center].push((prev, next));
                }
            }
        }
    }
    let mut result = Vec::new();
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let vi = vals[i];
        let lower_components = link_components(&nb[i], &link_edges[i], |j| vals[j] < vi);
        let upper_components = link_components(&nb[i], &link_edges[i], |j| vals[j] > vi);
        let kind = if lower_components == 0 && upper_components > 0 {
            CriticalKind::Minimum
        } else if upper_components == 0 && lower_components > 0 {
            CriticalKind::Maximum
        } else if lower_components > 1 || upper_components > 1 {
            CriticalKind::Saddle
        } else {
            CriticalKind::Regular
        };
        match kind {
            CriticalKind::Regular => {}
            _ => result.push(CriticalPoint {
                vertex: i,
                value: vi,
                kind,
            }),
        }
    }
    result
}

fn link_components<F>(neighbors: &[usize], edges: &[(usize, usize)], keep: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let mut selected = Vec::new();
    for &neighbor in neighbors {
        if keep(neighbor) {
            selected.push(neighbor);
        }
    }
    let mut visited = vec![false; selected.len()];
    let mut components = 0;
    for start in 0..selected.len() {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut stack = vec![selected[start]];
        visited[start] = true;
        while let Some(vertex) = stack.pop() {
            for &(a, b) in edges {
                let other = if a == vertex && keep(b) {
                    b
                } else if b == vertex && keep(a) {
                    a
                } else {
                    continue;
                };
                if let Some(pos) = selected.iter().position(|&v| v == other) {
                    if !visited[pos] {
                        visited[pos] = true;
                        stack.push(other);
                    }
                }
            }
        }
    }
    components
}
pub fn attach_critical_type(mesh: &PolyData, array_name: &str) -> PolyData {
    let crits = find_critical_points(mesh, array_name);
    let n = mesh.points.len();
    let mut data = vec![0.0f64; n]; // 0=regular
    for c in &crits {
        match c.kind {
            CriticalKind::Minimum => data[c.vertex] = -1.0,
            CriticalKind::Maximum => data[c.vertex] = 1.0,
            CriticalKind::Saddle => data[c.vertex] = 0.5,
            _ => {}
        }
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CriticalType",
            data,
            1,
        )));
    r
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
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 3], [1, 2, 3], [0, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 0.0, 0.0, 1.0],
                1,
            )));
        let crits = find_critical_points(&m, "h");
        assert!(crits
            .iter()
            .any(|c| matches!(c.kind, CriticalKind::Maximum)));
    }
    #[test]
    fn test_attach() {
        let mut m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 0.5],
                1,
            )));
        let r = attach_critical_type(&m, "s");
        assert!(r.point_data().get_array("CriticalType").is_some());
    }
}
