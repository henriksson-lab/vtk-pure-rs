//! Approximate persistent homology (Betti numbers) from scalar filtration.
use crate::data::PolyData;
pub struct PersistenceInfo {
    pub num_components_at: Vec<(f64, usize)>,
    pub betti_0_final: usize,
}
pub fn filtration_components(
    mesh: &PolyData,
    array_name: &str,
    num_levels: usize,
) -> PersistenceInfo {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => {
            return PersistenceInfo {
                num_components_at: vec![],
                betti_0_final: 0,
            }
        }
    };
    let n = mesh.points.len();
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = (mx - mn).max(1e-15);
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
    let nl = num_levels.max(2);
    let mut result = Vec::new();
    for li in 0..=nl {
        let thresh = mn + range * li as f64 / nl as f64;
        let active: Vec<bool> = vals.iter().map(|&v| v <= thresh).collect();
        // Count components among active vertices
        let mut parent: Vec<usize> = (0..n).collect();
        for i in 0..n {
            if !active[i] {
                continue;
            }
            for &j in &nb[i] {
                if active[j] {
                    union(&mut parent, i, j);
                }
            }
        }
        let roots: std::collections::HashSet<usize> = (0..n)
            .filter(|&i| active[i])
            .map(|i| find(&mut parent, i))
            .collect();
        result.push((thresh, roots.len()));
    }
    let betti = result.last().map(|&(_, c)| c).unwrap_or(0);
    PersistenceInfo {
        num_components_at: result,
        betti_0_final: betti,
    }
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
        m.point_data_mut().add_array(crate::data::AnyDataArray::F64(
            crate::data::DataArray::from_vec("h", vec![0.0, 1.0, 2.0, 3.0], 1),
        ));
        let pi = filtration_components(&m, "h", 10);
        assert!(pi.num_components_at.len() > 5);
        assert_eq!(pi.betti_0_final, 1);
    }
}
