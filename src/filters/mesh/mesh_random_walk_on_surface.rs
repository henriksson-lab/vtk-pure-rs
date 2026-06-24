//! Random walk simulation on mesh surface.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub fn random_walk_visit_count(mesh: &PolyData, start: usize, steps: usize, seed: u64) -> PolyData {
    let n = mesh.points.len();
    if n == 0 || start >= n {
        return mesh.clone();
    }
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
    let mut visits = vec![0.0f64; n];
    let mut rng = seed;
    let mut cur = start;
    for _ in 0..steps {
        visits[cur] += 1.0;
        if nb[cur].is_empty() {
            break;
        }
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (rng >> 33) as usize % nb[cur].len();
        cur = nb[cur][idx];
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Visits", visits, 1)));
    r.point_data_mut().set_active_scalars("Visits");
    r
}
pub fn random_walk_steady_state(mesh: &PolyData, iterations: usize) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
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
    let mut prob = vec![1.0 / n as f64; n];
    for _ in 0..iterations {
        let mut new_prob = vec![0.0f64; n];
        for i in 0..n {
            if nb[i].is_empty() {
                new_prob[i] += prob[i];
                continue;
            }
            let share = prob[i] / nb[i].len() as f64;
            for &j in &nb[i] {
                new_prob[j] += share;
            }
        }
        prob = new_prob;
    }
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "SteadyState",
            prob,
            1,
        )));
    r.point_data_mut().set_active_scalars("SteadyState");
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_walk() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = random_walk_visit_count(&m, 0, 100, 42);
        let arr = r.point_data().get_array("Visits").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] > 0.0);
    }
    #[test]
    fn test_steady() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = random_walk_steady_state(&m, 50);
        assert!(r.point_data().get_array("SteadyState").is_some());
    }
}
