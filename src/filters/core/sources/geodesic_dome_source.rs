//! Geodesic dome by icosphere subdivision.
use crate::data::{CellArray, Points, PolyData};
pub fn geodesic_dome(radius: f64, subdivisions: usize) -> PolyData {
    let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
    let a = radius / (1.0 + phi * phi).sqrt();
    let b = a * phi;
    let mut pts: Vec<[f64; 3]> = vec![
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
    let mut tris: Vec<[usize; 3]> = vec![
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
    for _ in 0..subdivisions {
        let mut new_tris = Vec::new();
        let mut em: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        for t in &tris {
            let m01 = mid(&mut pts, &mut em, t[0], t[1], radius);
            let m12 = mid(&mut pts, &mut em, t[1], t[2], radius);
            let m20 = mid(&mut pts, &mut em, t[2], t[0], radius);
            new_tris.push([t[0], m01, m20]);
            new_tris.push([t[1], m12, m01]);
            new_tris.push([t[2], m20, m12]);
            new_tris.push([m01, m12, m20]);
        }
        tris = new_tris;
    }

    let dome_tris: Vec<[usize; 3]> = tris
        .into_iter()
        .filter(|t| pts[t[0]][2] >= -1e-12 && pts[t[1]][2] >= -1e-12 && pts[t[2]][2] >= -1e-12)
        .collect();
    let mut point_map = vec![usize::MAX; pts.len()];
    let mut mesh_pts = Points::<f64>::new();
    for t in &dome_tris {
        for &idx in t {
            if point_map[idx] == usize::MAX {
                point_map[idx] = mesh_pts.len();
                mesh_pts.push(pts[idx]);
            }
        }
    }
    let mut polys = CellArray::new();
    for t in &dome_tris {
        polys.push_cell(&[
            point_map[t[0]] as i64,
            point_map[t[1]] as i64,
            point_map[t[2]] as i64,
        ]);
    }
    let mut r = PolyData::new();
    r.points = mesh_pts;
    r.polys = polys;
    r
}
fn mid(
    pts: &mut Vec<[f64; 3]>,
    cache: &mut std::collections::HashMap<(usize, usize), usize>,
    a: usize,
    b: usize,
    r: f64,
) -> usize {
    let key = (a.min(b), a.max(b));
    *cache.entry(key).or_insert_with(|| {
        let m = [
            (pts[a][0] + pts[b][0]) / 2.0,
            (pts[a][1] + pts[b][1]) / 2.0,
            (pts[a][2] + pts[b][2]) / 2.0,
        ];
        let l = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt().max(1e-15);
        let i = pts.len();
        pts.push([m[0] / l * r, m[1] / l * r, m[2] / l * r]);
        i
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_0() {
        let d = geodesic_dome(1.0, 0);
        assert!(d.points.len() < 12);
        assert!(d.polys.num_cells() < 20);
        for i in 0..d.points.len() {
            assert!(d.points.get(i)[2] >= -1e-12);
        }
    }
    #[test]
    fn test_2() {
        let d = geodesic_dome(1.0, 2);
        assert!(d.polys.num_cells() < 320);
        // All points on upper sphere
        for i in 0..d.points.len() {
            let p = d.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-10);
            assert!(p[2] >= -1e-12);
        }
    }
}
