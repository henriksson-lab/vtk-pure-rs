//! Soccer ball (truncated icosahedron) geometry.
use crate::data::{CellArray, Points, PolyData};
pub fn soccer_ball(radius: f64) -> PolyData {
    // Start with icosahedron and truncate
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let a = radius / (1.0 + phi * phi).sqrt();
    let b = a * phi;
    let ico_v = [
        [-a, b, 0.0],
        [a, b, 0.0],
        [-a, -b, 0.0],
        [a, -b, 0.0],
        [0.0, -a, b],
        [0.0, a, b],
        [0.0, -a, -b],
        [0.0, a, -b],
        [b, 0.0, -a],
        [b, 0.0, a],
        [-b, 0.0, -a],
        [-b, 0.0, a],
    ];
    let ico_f: [[usize; 3]; 20] = [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    // Truncation: for each edge, place a point at 1/3 from each end
    let mut edge_pts: std::collections::HashMap<(usize, usize), [usize; 2]> =
        std::collections::HashMap::new();
    let mut pts = Points::<f64>::new();
    for &[a, b, c] in &ico_f {
        let edges = [(a, b), (b, c), (c, a)];
        for &(u, v) in &edges {
            let key = (u.min(v), u.max(v));
            edge_pts.entry(key).or_insert_with(|| {
                let pu = ico_v[u];
                let pv = ico_v[v];
                let p1 = [
                    pu[0] + (pv[0] - pu[0]) / 3.0,
                    pu[1] + (pv[1] - pu[1]) / 3.0,
                    pu[2] + (pv[2] - pu[2]) / 3.0,
                ];
                let p2 = [
                    pu[0] + 2.0 * (pv[0] - pu[0]) / 3.0,
                    pu[1] + 2.0 * (pv[1] - pu[1]) / 3.0,
                    pu[2] + 2.0 * (pv[2] - pu[2]) / 3.0,
                ];
                // Project onto sphere
                let r1 = (p1[0] * p1[0] + p1[1] * p1[1] + p1[2] * p1[2]).sqrt();
                let r2 = (p2[0] * p2[0] + p2[1] * p2[1] + p2[2] * p2[2]).sqrt();
                let i1 = pts.len();
                pts.push([
                    p1[0] / r1 * radius,
                    p1[1] / r1 * radius,
                    p1[2] / r1 * radius,
                ]);
                let i2 = pts.len();
                pts.push([
                    p2[0] / r2 * radius,
                    p2[1] / r2 * radius,
                    p2[2] / r2 * radius,
                ]);
                [i1, i2]
            });
        }
    }
    // Build hexagonal faces (from original triangles) and pentagonal faces (from original vertices)
    let mut polys = CellArray::new();
    for &[a, b, c] in &ico_f {
        let get_edge_pt = |u: usize, v: usize| -> usize {
            let key = (u.min(v), u.max(v));
            let ep = edge_pts[&key];
            if u < v {
                ep[0]
            } else {
                ep[1]
            }
        };
        let ab0 = get_edge_pt(a, b);
        let ab1 = get_edge_pt(b, a);
        let bc0 = get_edge_pt(b, c);
        let bc1 = get_edge_pt(c, b);
        let ca0 = get_edge_pt(c, a);
        let ca1 = get_edge_pt(a, c);
        polys.push_cell(&[
            ab0 as i64, ab1 as i64, bc0 as i64, bc1 as i64, ca0 as i64, ca1 as i64,
        ]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.polys = polys;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let s = soccer_ball(1.0);
        assert!(s.points.len() >= 30);
        assert_eq!(s.polys.num_cells(), 20);
    }
}
