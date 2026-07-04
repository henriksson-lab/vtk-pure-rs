//! Per-cell subdivision: subdivide individual cells without affecting neighbors.

use crate::data::{CellArray, Points, PolyData};

/// Subdivide only cells that match a predicate.
pub fn subdivide_cells_if(mesh: &PolyData, predicate: impl Fn(usize, &[i64]) -> bool) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut new_polys = CellArray::new();
    let mut edge_mids: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();

    for (ci, cell) in mesh.polys.iter().enumerate() {
        if cell.len() == 3 && valid_cell(cell, pts.len()) && predicate(ci, cell) {
            let v = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            let m01 = get_mid(&mut pts, &mut edge_mids, v[0], v[1]);
            let m12 = get_mid(&mut pts, &mut edge_mids, v[1], v[2]);
            let m20 = get_mid(&mut pts, &mut edge_mids, v[2], v[0]);
            new_polys.push_cell(&[v[0] as i64, m01 as i64, m20 as i64]);
            new_polys.push_cell(&[v[1] as i64, m12 as i64, m01 as i64]);
            new_polys.push_cell(&[v[2] as i64, m20 as i64, m12 as i64]);
            new_polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
        } else {
            new_polys.push_cell(cell);
        }
    }

    let mut new_pts = Points::<f64>::new();
    for p in &pts {
        new_pts.push(*p);
    }
    let mut result = mesh.clone();
    result.points = new_pts;
    result.polys = new_polys;
    result
}

/// Subdivide cells with area above threshold.
pub fn subdivide_large_cells(mesh: &PolyData, max_area: f64) -> PolyData {
    subdivide_cells_if(mesh, |_ci, cell| {
        if cell.len() < 3 || !valid_cell(cell, mesh.points.len()) {
            return false;
        }
        let a = mesh.points.get(cell[0] as usize);
        let b = mesh.points.get(cell[1] as usize);
        let c = mesh.points.get(cell[2] as usize);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let area = 0.5
            * ((e1[1] * e2[2] - e1[2] * e2[1]).powi(2)
                + (e1[2] * e2[0] - e1[0] * e2[2]).powi(2)
                + (e1[0] * e2[1] - e1[1] * e2[0]).powi(2))
            .sqrt();
        area > max_area
    })
}

/// Subdivide cells by index list.
pub fn subdivide_cells_by_index(
    mesh: &PolyData,
    indices: &std::collections::HashSet<usize>,
) -> PolyData {
    subdivide_cells_if(mesh, |ci, _| indices.contains(&ci))
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

fn valid_cell(cell: &[i64], num_points: usize) -> bool {
    cell.iter()
        .all(|&pid| pid >= 0 && (pid as usize) < num_points)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subdivide_all() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_cells_if(&mesh, |_, _| true);
        assert_eq!(result.polys.num_cells(), 4);
    }
    #[test]
    fn subdivide_large() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [5.0, 10.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.1, 0.0, 0.0],
                [0.0, 0.1, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let result = subdivide_large_cells(&mesh, 1.0);
        assert!(result.polys.num_cells() > 2); // large tri subdivided, small kept
    }
}
