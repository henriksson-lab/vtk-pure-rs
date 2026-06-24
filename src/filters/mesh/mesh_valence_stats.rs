//! Compute vertex valence (degree) and store as scalar data.
use crate::data::{AnyDataArray, DataArray, PolyData};

pub fn valence_stats(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let mut valence = vec![0u32; n];
    let mut seen: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                if seen[a].insert(b) {
                    valence[a] += 1;
                }
                if seen[b].insert(a) {
                    valence[b] += 1;
                }
            }
        }
    }
    let data: Vec<f64> = valence.iter().map(|&v| v as f64).collect();
    let mut result = mesh.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Valence", data, 1)));
    result.point_data_mut().set_active_scalars("Valence");
    result
}

pub fn valence_histogram(mesh: &PolyData) -> Vec<(u32, usize)> {
    let n = mesh.points.len();
    let mut valence = vec![0u32; n];
    let mut seen: Vec<std::collections::HashSet<usize>> = vec![std::collections::HashSet::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if a < n && b < n {
                seen[a].insert(b);
                seen[b].insert(a);
            }
        }
    }
    for i in 0..n {
        valence[i] = seen[i].len() as u32;
    }
    let mut hist = std::collections::BTreeMap::new();
    for &v in &valence {
        *hist.entry(v).or_insert(0usize) += 1;
    }
    hist.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_valence() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = valence_stats(&mesh);
        let arr = r.point_data().get_array("Valence").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert_eq!(b[0], 2.0); // each vertex has 2 neighbors in single triangle
    }
}
