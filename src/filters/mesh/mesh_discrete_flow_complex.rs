//! Discrete flow complex (ascending/descending manifolds from scalar field).
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn ascending_manifold(mesh: &PolyData, array_name: &str) -> PolyData {
    flow_manifold(mesh, array_name, true)
}
pub fn descending_manifold(mesh: &PolyData, array_name: &str) -> PolyData {
    flow_manifold(mesh, array_name, false)
}
fn flow_manifold(mesh: &PolyData, array_name: &str, ascending: bool) -> PolyData {
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
    // Each vertex flows to steepest ascent/descent neighbor
    let mut flow_to = vec![0usize; n];
    for i in 0..n {
        let mut best = i;
        for &j in &nb[i] {
            if ascending {
                if vals[j] > vals[best] {
                    best = j;
                }
            } else {
                if vals[j] < vals[best] {
                    best = j;
                }
            }
        }
        flow_to[i] = best;
    }
    // Follow flow to fixed point
    let mut root = vec![0usize; n];
    for i in 0..n {
        let mut cur = i;
        let mut steps = 0;
        while flow_to[cur] != cur && steps < n {
            cur = flow_to[cur];
            steps += 1;
        }
        root[i] = cur;
    }
    // Label by root
    let mut label_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut next = 0;
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            *label_map.entry(root[i]).or_insert_with(|| {
                let l = next;
                next += 1;
                l
            }) as f64
        })
        .collect();
    let name = if ascending {
        "AscManifold"
    } else {
        "DescManifold"
    };
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(name, labels, 1)));
    r.point_data_mut().set_active_scalars(name);
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_asc() {
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
        let r = ascending_manifold(&m, "h");
        assert!(r.point_data().get_array("AscManifold").is_some());
    }
    #[test]
    fn test_desc() {
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
        let r = descending_manifold(&m, "h");
        assert!(r.point_data().get_array("DescManifold").is_some());
    }
}
