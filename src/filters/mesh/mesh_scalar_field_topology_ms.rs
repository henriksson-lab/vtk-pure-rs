//! Morse-Smale complex approximation on mesh scalar field.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn morse_smale_regions(mesh: &PolyData, array_name: &str) -> PolyData {
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
    // Ascending manifold: each vertex flows to its steepest ascent neighbor
    let mut asc = vec![0usize; n];
    for i in 0..n {
        let mut best = i;
        for &j in &nb[i] {
            if vals[j] > vals[best] {
                best = j;
            }
        }
        asc[i] = best;
    }
    // Follow to fixed point (maximum)
    let mut asc_root = vec![0usize; n];
    for i in 0..n {
        let mut cur = i;
        let mut steps = 0;
        while asc[cur] != cur && steps < n {
            cur = asc[cur];
            steps += 1;
        }
        asc_root[i] = cur;
    }
    // Descending manifold
    let mut desc = vec![0usize; n];
    for i in 0..n {
        let mut best = i;
        for &j in &nb[i] {
            if vals[j] < vals[best] {
                best = j;
            }
        }
        desc[i] = best;
    }
    let mut desc_root = vec![0usize; n];
    for i in 0..n {
        let mut cur = i;
        let mut steps = 0;
        while desc[cur] != cur && steps < n {
            cur = desc[cur];
            steps += 1;
        }
        desc_root[i] = cur;
    }
    // MS region = (ascending root, descending root) pair
    let mut region_map: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    let mut next = 0;
    let labels: Vec<f64> = (0..n)
        .map(|i| {
            let key = (asc_root[i], desc_root[i]);
            *region_map.entry(key).or_insert_with(|| {
                let l = next;
                next += 1;
                l
            }) as f64
        })
        .collect();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "MSRegion", labels, 1,
        )));
    r.point_data_mut().set_active_scalars("MSRegion");
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
                vec![0.0, 1.0, 3.0, 2.0],
                1,
            )));
        let r = morse_smale_regions(&m, "h");
        assert!(r.point_data().get_array("MSRegion").is_some());
    }
}
