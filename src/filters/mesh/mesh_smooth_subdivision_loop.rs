//! Loop subdivision (approximating smooth subdivision).
use crate::data::{CellArray, Points, PolyData};
pub fn loop_subdivide(mesh: &PolyData) -> PolyData {
    loop_subdivide_n(mesh, 1)
}
pub fn loop_subdivide_n(mesh: &PolyData, n: usize) -> PolyData {
    let mut current = mesh.clone();
    for _ in 0..n {
        current = loop_once(&current);
    }
    current
}
fn loop_once(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let nc = c.len();
        for i in 0..nc {
            let a = c[i] as usize;
            let b = c[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    // Edge points
    let mut em: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    let mut pts: Vec<[f64; 3]> = (0..n).map(|i| mesh.points.get(i)).collect();
    for (&(a, b), faces) in &ef {
        let pa = pts[a];
        let pb = pts[b];
        let mid = if faces.len() == 2 {
            // Get opposite vertices
            let opp: Vec<usize> = faces
                .iter()
                .filter_map(|&fi| {
                    cells[fi]
                        .iter()
                        .find(|&&v| v as usize != a && v as usize != b)
                        .map(|&v| v as usize)
                })
                .collect();
            if opp.len() == 2 {
                let po0 = pts[opp[0]];
                let po1 = pts[opp[1]];
                [
                    (pa[0] + pb[0]) * 3.0 / 8.0 + (po0[0] + po1[0]) / 8.0,
                    (pa[1] + pb[1]) * 3.0 / 8.0 + (po0[1] + po1[1]) / 8.0,
                    (pa[2] + pb[2]) * 3.0 / 8.0 + (po0[2] + po1[2]) / 8.0,
                ]
            } else {
                [
                    (pa[0] + pb[0]) / 2.0,
                    (pa[1] + pb[1]) / 2.0,
                    (pa[2] + pb[2]) / 2.0,
                ]
            }
        } else {
            [
                (pa[0] + pb[0]) / 2.0,
                (pa[1] + pb[1]) / 2.0,
                (pa[2] + pb[2]) / 2.0,
            ]
        };
        let idx = pts.len();
        pts.push(mid);
        em.insert((a, b), idx);
    }
    // Update original vertices
    let mut nb: Vec<Vec<usize>> = vec![Vec::new(); n];
    for cell in &cells {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            if !nb[a].contains(&b) {
                nb[a].push(b);
            }
            if !nb[b].contains(&a) {
                nb[b].push(a);
            }
        }
    }
    let mut new_pts = pts.clone();
    for i in 0..n {
        let k = nb[i].len();
        if k < 3 {
            continue;
        }
        let beta = if k == 3 {
            3.0 / 16.0
        } else {
            3.0 / (8.0 * k as f64)
        };
        let mut avg = [0.0, 0.0, 0.0];
        for &j in &nb[i] {
            avg[0] += pts[j][0];
            avg[1] += pts[j][1];
            avg[2] += pts[j][2];
        }
        new_pts[i] = [
            (1.0 - k as f64 * beta) * pts[i][0] + beta * avg[0],
            (1.0 - k as f64 * beta) * pts[i][1] + beta * avg[1],
            (1.0 - k as f64 * beta) * pts[i][2] + beta * avg[2],
        ];
    }
    // Build new triangles
    let mut new_polys = CellArray::new();
    for c in &cells {
        if c.len() != 3 {
            continue;
        }
        let v = [c[0] as usize, c[1] as usize, c[2] as usize];
        let m01 = em[&(v[0].min(v[1]), v[0].max(v[1]))];
        let m12 = em[&(v[1].min(v[2]), v[1].max(v[2]))];
        let m20 = em[&(v[2].min(v[0]), v[2].max(v[0]))];
        new_polys.push_cell(&[v[0] as i64, m01 as i64, m20 as i64]);
        new_polys.push_cell(&[v[1] as i64, m12 as i64, m01 as i64]);
        new_polys.push_cell(&[v[2] as i64, m20 as i64, m12 as i64]);
        new_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
    }
    let mut mesh_pts = Points::<f64>::new();
    for p in &new_pts {
        mesh_pts.push(*p);
    }
    let mut r = PolyData::new();
    r.points = mesh_pts;
    r.polys = new_polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_once() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = loop_subdivide(&m);
        assert_eq!(r.polys.num_cells(), 4);
    }
    #[test]
    fn test_twice() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = loop_subdivide_n(&m, 2);
        assert_eq!(r.polys.num_cells(), 16);
    }
}
