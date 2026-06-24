//! Watershed segmentation on mesh scalar field.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn watershed(mesh: &PolyData, array_name: &str) -> PolyData {
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
    // Each vertex flows to its lowest neighbor
    let mut flow_to = vec![0usize; n];
    for i in 0..n {
        let mut lowest = i;
        for &j in &nb[i] {
            if vals[j] < vals[lowest] {
                lowest = j;
            }
        }
        flow_to[i] = lowest;
    }
    // Follow flow to find basins (fixed points)
    let mut basin = vec![0usize; n];
    for i in 0..n {
        let mut cur = i;
        let mut visited = Vec::new();
        while flow_to[cur] != cur && !visited.contains(&cur) {
            visited.push(cur);
            cur = flow_to[cur];
        }
        let root = cur;
        for &v in &visited {
            basin[v] = root;
        }
        basin[i] = root;
    }
    // Relabel sequentially
    let mut label_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut next = 0;
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            *label_map.entry(basin[i]).or_insert_with(|| {
                let l = next;
                next += 1;
                l
            }) as f64
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Watershed",
            labels,
            1,
        )));
    r.point_data_mut().set_active_scalars("Watershed");
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
                [2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 1.0, 2.0, 0.5],
                1,
            )));
        let r = watershed(&m, "h");
        assert!(r.point_data().get_array("Watershed").is_some());
    }
}
