//! Generate persistence diagram as PolyData (birth-death scatter plot).
use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
pub fn persistence_diagram_polydata(mesh: &PolyData, array_name: &str) -> PolyData {
    let arr = match mesh.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return PolyData::new(),
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
    let mut sorted: Vec<usize> = (0..n).collect();
    sorted.sort_by(|&a, &b| {
        vals[a]
            .partial_cmp(&vals[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut parent: Vec<usize> = (0..n).collect();
    let mut active = vec![false; n];
    let mut pairs: Vec<(f64, f64)> = Vec::new();
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
    // Build scatter plot PolyData
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    let mut persistence = Vec::new();
    for &(birth, death) in &pairs {
        let idx = pts.len();
        pts.push([birth, death, 0.0]);
        verts.push_cell(&[idx as i64]);
        persistence.push(death - birth);
    }
    // Add diagonal line
    let mn = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut lines = CellArray::new();
    let db = pts.len();
    pts.push([mn, mn, 0.0]);
    pts.push([mx, mx, 0.0]);
    lines.push_cell(&[db as i64, (db + 1) as i64]);
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r.lines = lines;
    r.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Persistence",
            persistence,
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
    fn test() {
        let mut m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.5, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 4], [1, 4, 2]],
        );
        m.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "h",
                vec![0.0, 2.0, 3.0, 0.5, 2.5],
                1,
            )));
        let r = persistence_diagram_polydata(&m, "h");
        assert!(r.points.len() >= 1);
        assert!(r.lines.num_cells() >= 1);
    }
}
