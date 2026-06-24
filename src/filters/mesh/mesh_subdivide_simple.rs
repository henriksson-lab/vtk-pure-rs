//! Simple midpoint subdivision for triangle meshes.

use crate::data::{CellArray, Points, PolyData};

/// Subdivide all triangles by inserting midpoints on each edge (Loop-like topology).
pub fn subdivide_midpoint_all(mesh: &PolyData) -> PolyData {
    subdivide_n(mesh, 1)
}

/// Subdivide N times.
pub fn subdivide_n(mesh: &PolyData, n: usize) -> PolyData {
    let mut current = mesh.clone();
    for _ in 0..n {
        current = subdivide_once(&current);
    }
    current
}

fn subdivide_once(mesh: &PolyData) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut new_polys = CellArray::new();
    let mut edge_mids: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();

    for cell in mesh.polys.iter() {
        if cell.len() == 3 {
            let v = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            let m01 = get_mid(&mut pts, &mut edge_mids, v[0], v[1]);
            let m12 = get_mid(&mut pts, &mut edge_mids, v[1], v[2]);
            let m20 = get_mid(&mut pts, &mut edge_mids, v[2], v[0]);
            new_polys.push_cell(&[v[0] as i64, m01 as i64, m20 as i64]);
            new_polys.push_cell(&[v[1] as i64, m12 as i64, m01 as i64]);
            new_polys.push_cell(&[v[2] as i64, m20 as i64, m12 as i64]);
            new_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
        } else if cell.len() == 4 {
            let v = [
                cell[0] as usize,
                cell[1] as usize,
                cell[2] as usize,
                cell[3] as usize,
            ];
            let m01 = get_mid(&mut pts, &mut edge_mids, v[0], v[1]);
            let m12 = get_mid(&mut pts, &mut edge_mids, v[1], v[2]);
            let m23 = get_mid(&mut pts, &mut edge_mids, v[2], v[3]);
            let m30 = get_mid(&mut pts, &mut edge_mids, v[3], v[0]);
            let center = pts.len();
            pts.push([
                (pts[v[0]][0] + pts[v[1]][0] + pts[v[2]][0] + pts[v[3]][0]) / 4.0,
                (pts[v[0]][1] + pts[v[1]][1] + pts[v[2]][1] + pts[v[3]][1]) / 4.0,
                (pts[v[0]][2] + pts[v[1]][2] + pts[v[2]][2] + pts[v[3]][2]) / 4.0,
            ]);
            new_polys.push_cell(&[v[0] as i64, m01 as i64, center as i64, m30 as i64]);
            new_polys.push_cell(&[m01 as i64, v[1] as i64, m12 as i64, center as i64]);
            new_polys.push_cell(&[center as i64, m12 as i64, v[2] as i64, m23 as i64]);
            new_polys.push_cell(&[m30 as i64, center as i64, m23 as i64, v[3] as i64]);
        } else {
            new_polys.push_cell(cell);
        }
    }

    let mut new_pts = Points::<f64>::new();
    for p in &pts {
        new_pts.push(*p);
    }
    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = new_polys;
    result
}

fn get_mid(
    pts: &mut Vec<[f64; 3]>,
    cache: &mut std::collections::HashMap<(usize, usize), usize>,
    a: usize,
    b: usize,
) -> usize {
    let key = (a.min(b), a.max(b));
    *cache.entry(key).or_insert_with(|| {
        let i = pts.len();
        pts.push([
            (pts[a][0] + pts[b][0]) / 2.0,
            (pts[a][1] + pts[b][1]) / 2.0,
            (pts[a][2] + pts[b][2]) / 2.0,
        ]);
        i
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_once() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = subdivide_midpoint_all(&mesh);
        assert_eq!(r.polys.num_cells(), 4);
        assert_eq!(r.points.len(), 6); // 3 original + 3 midpoints
    }
    #[test]
    fn test_twice() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = subdivide_n(&mesh, 2);
        assert_eq!(r.polys.num_cells(), 16); // 4^2
    }
    #[test]
    fn test_quad() {
        let mut mesh = PolyData::new();
        mesh.points.push([0.0, 0.0, 0.0]);
        mesh.points.push([1.0, 0.0, 0.0]);
        mesh.points.push([1.0, 1.0, 0.0]);
        mesh.points.push([0.0, 1.0, 0.0]);
        mesh.polys.push_cell(&[0, 1, 2, 3]);
        let r = subdivide_midpoint_all(&mesh);
        assert_eq!(r.polys.num_cells(), 4);
    }
}
