//! Unified topological data analysis pipeline.
use crate::data::{AnyDataArray, DataArray, PolyData};
pub struct TDAResult {
    pub num_minima: usize,
    pub num_maxima: usize,
    pub num_saddles: usize,
    pub euler_characteristic: isize,
    pub persistence_pairs: Vec<(f64, f64)>,
    pub total_persistence: f64,
}
pub fn full_tda(mesh: &PolyData, array_name: &str) -> TDAResult {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => {
            return TDAResult {
                num_minima: 0,
                num_maxima: 0,
                num_saddles: 0,
                euler_characteristic: 0,
                persistence_pairs: vec![],
                total_persistence: 0.0,
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
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
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
                *edge_count.entry((a.min(b), a.max(b))).or_insert(0) += 1;
            }
        }
    }
    // Count critical points
    let mut mins = 0;
    let mut maxs = 0;
    let mut saddles = 0;
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let lower = nb[i].iter().filter(|&&j| vals[j] < vals[i]).count();
        let upper = nb[i].iter().filter(|&&j| vals[j] > vals[i]).count();
        if lower == 0 && upper > 0 {
            mins += 1;
        } else if upper == 0 && lower > 0 {
            maxs += 1;
        } else if lower > 0 && upper > 0 {
            let changes = nb[i]
                .windows(2)
                .filter(|w| (vals[w[0]] > vals[i]) != (vals[w[1]] > vals[i]))
                .count();
            if changes >= 4 {
                saddles += 1;
            }
        }
    }
    // Euler characteristic
    let v = n;
    let e = edge_count.len();
    let f = mesh.polys.num_cells();
    let euler = v as isize - e as isize + f as isize;
    // Persistence pairs via join tree
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
                    pairs.push((vals[r], vals[vi]));
                    union(&mut parent, oldest, r);
                }
            }
            union(&mut parent, oldest, vi);
        } else if !roots.is_empty() {
            union(&mut parent, roots[0], vi);
        }
    }
    let total_p: f64 = pairs.iter().map(|&(b, d)| (d - b).abs()).sum();
    TDAResult {
        num_minima: mins,
        num_maxima: maxs,
        num_saddles: saddles,
        euler_characteristic: euler,
        persistence_pairs: pairs,
        total_persistence: total_p,
    }
}
pub fn attach_tda_summary(mesh: &PolyData, array_name: &str) -> PolyData {
    let tda = full_tda(mesh, array_name);
    let n = mesh.points.len();
    let mut r = mesh.clone();
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Euler",
            vec![tda.euler_characteristic as f64; n],
            1,
        )));
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TotalPersistence",
            vec![tda.total_persistence; n],
            1,
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
    fn test_full() {
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
        let tda = full_tda(&m, "h");
        assert!(tda.num_minima + tda.num_maxima >= 1);
        assert_eq!(tda.euler_characteristic, 1);
    }
    #[test]
    fn test_attach() {
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
        let r = attach_tda_summary(&m, "h");
        assert!(r.point_data().get_array("Euler").is_some());
    }
}
