//! Compute Willmore energy (integral of mean curvature squared).
use crate::data::PolyData;
pub fn willmore_energy(mesh: &PolyData) -> f64 {
    let n = mesh.points.len();
    if n == 0 {
        return 0.0;
    }
    let nb = build_neighbors(mesh, n);
    let nm = calc_nm(mesh);
    let mut total = 0.0;
    for i in 0..n {
        if nb[i].is_empty() {
            continue;
        }
        let p = mesh.points.get(i);
        let ni = nm[i];
        let k = nb[i].len() as f64;
        let mut lap = [0.0, 0.0, 0.0];
        for &j in &nb[i] {
            let q = mesh.points.get(j);
            lap[0] += q[0] - p[0];
            lap[1] += q[1] - p[1];
            lap[2] += q[2] - p[2];
        }
        let hn = (lap[0] * ni[0] + lap[1] * ni[1] + lap[2] * ni[2]) / k;
        total += hn * hn;
    }
    total
}

fn build_neighbors(mesh: &PolyData, n: usize) -> Vec<Vec<usize>> {
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        if nc < 2 {
            continue;
        }
        for i in 0..nc {
            add_neighbor_edge(cell[i], cell[(i + 1) % nc], n, &mut nb);
        }
    }
    for cell in mesh.lines.iter() {
        for edge in cell.windows(2) {
            add_neighbor_edge(edge[0], edge[1], n, &mut nb);
        }
    }
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            add_neighbor_edge(tri[0], tri[1], n, &mut nb);
            add_neighbor_edge(tri[1], tri[2], n, &mut nb);
            add_neighbor_edge(tri[2], tri[0], n, &mut nb);
        }
    }
    nb
}

fn add_neighbor_edge(a: i64, b: i64, n: usize, nb: &mut [Vec<usize>]) {
    let (Some(a), Some(b)) = (valid_point_index(a, n), valid_point_index(b, n)) else {
        return;
    };
    if a == b {
        return;
    }
    if !nb[a].contains(&b) {
        nb[a].push(b);
    }
    if !nb[b].contains(&a) {
        nb[b].push(a);
    }
}

pub fn normalized_willmore(mesh: &PolyData) -> f64 {
    let w = willmore_energy(mesh);
    let n = mesh.points.len();
    if n > 0 {
        w / n as f64
    } else {
        0.0
    }
}
pub fn is_willmore_surface(mesh: &PolyData, tolerance: f64) -> bool {
    // A Willmore surface has W ≈ 4π (for a sphere)
    let w = willmore_energy(mesh);
    (w - 4.0 * std::f64::consts::PI).abs() < tolerance
}
fn calc_nm(mesh: &PolyData) -> Vec<[f64; 3]> {
    let n = mesh.points.len();
    let mut nm = vec![[0.0f64; 3]; n];
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        add_face_normal(mesh, cell, &mut nm);
    }
    for strip in mesh.strips.iter() {
        for (i, tri) in strip.windows(3).enumerate() {
            let tri = if i % 2 == 0 {
                [tri[0], tri[1], tri[2]]
            } else {
                [tri[1], tri[0], tri[2]]
            };
            add_face_normal(mesh, &tri, &mut nm);
        }
    }
    for v in &mut nm {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l > 1e-15 {
            v[0] /= l;
            v[1] /= l;
            v[2] /= l;
        }
    }
    nm
}

fn add_face_normal(mesh: &PolyData, cell: &[i64], nm: &mut [[f64; 3]]) {
    let n = mesh.points.len();
    let (Some(ia), Some(ib), Some(ic)) = (
        valid_point_index(cell[0], n),
        valid_point_index(cell[1], n),
        valid_point_index(cell[2], n),
    ) else {
        return;
    };
    let a = mesh.points.get(ia);
    let b = mesh.points.get(ib);
    let c = mesh.points.get(ic);
    let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let fn_ = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    for &v in cell {
        if let Some(vi) = valid_point_index(v, n) {
            nm[vi][0] += fn_[0];
            nm[vi][1] += fn_[1];
            nm[vi][2] += fn_[2];
        }
    }
}

fn valid_point_index(id: i64, n: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_energy() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.3, 0.5],
            ],
            vec![[0, 1, 3], [1, 2, 3], [2, 0, 3]],
        );
        let w = willmore_energy(&m);
        assert!(w > 0.0);
    }
    #[test]
    fn test_normalized() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let nw = normalized_willmore(&m);
        assert!(nw >= 0.0);
    }
}
