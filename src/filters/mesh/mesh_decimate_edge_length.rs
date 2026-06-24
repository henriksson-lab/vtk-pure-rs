//! Decimate by collapsing shortest edges.
use crate::data::{CellArray, Points, PolyData};
pub fn collapse_short_edges(mesh: &PolyData, min_length: f64) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();
    let npts = pts.len();
    let ml2 = min_length * min_length;
    let mut remap: Vec<usize> = (0..npts).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for ti in 0..tris.len() {
            let t = [
                res(tris[ti][0], &remap),
                res(tris[ti][1], &remap),
                res(tris[ti][2], &remap),
            ];
            for i in 0..3 {
                let a = t[i];
                let b = t[(i + 1) % 3];
                if a == b {
                    continue;
                }
                let d2 = (pts[a][0] - pts[b][0]).powi(2)
                    + (pts[a][1] - pts[b][1]).powi(2)
                    + (pts[a][2] - pts[b][2]).powi(2);
                if d2 < ml2 {
                    pts[a] = [
                        (pts[a][0] + pts[b][0]) / 2.0,
                        (pts[a][1] + pts[b][1]) / 2.0,
                        (pts[a][2] + pts[b][2]) / 2.0,
                    ];
                    remap[b] = a;
                    changed = true;
                    break;
                }
            }
        }
    }
    let remaining: Vec<[usize; 3]> = tris
        .iter()
        .map(|t| [res(t[0], &remap), res(t[1], &remap), res(t[2], &remap)])
        .filter(|t| t[0] != t[1] && t[1] != t[2] && t[2] != t[0])
        .collect();
    let mut used = vec![false; npts];
    for t in &remaining {
        for &v in t {
            used[v] = true;
        }
    }
    let mut pm = vec![0usize; npts];
    let mut np = Points::<f64>::new();
    for i in 0..npts {
        if used[i] {
            pm[i] = np.len();
            np.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &remaining {
        polys.push_cell(&[pm[t[0]] as i64, pm[t[1]] as i64, pm[t[2]] as i64]);
    }
    let mut r = PolyData::new();
    r.points = np;
    r.polys = polys;
    r
}
fn res(mut v: usize, remap: &[usize]) -> usize {
    while remap[v] != v {
        v = remap[v];
    }
    v
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [0.01, 0.0, 0.0],
                [0.005, 0.01, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 4]],
        );
        let r = collapse_short_edges(&m, 0.05);
        assert!(r.polys.num_cells() <= 2);
    }
}
