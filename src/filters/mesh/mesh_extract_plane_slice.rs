//! Extract intersection contour of mesh with a plane.
use crate::data::{CellArray, Points, PolyData};
pub fn slice_mesh_by_plane(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> PolyData {
    let nl = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2])
        .sqrt()
        .max(1e-15);
    let nn = [normal[0] / nl, normal[1] / nl, normal[2] / nl];
    let dist = |i: usize| -> f64 {
        let p = mesh.points.get(i);
        (p[0] - origin[0]) * nn[0] + (p[1] - origin[1]) * nn[1] + (p[2] - origin[2]) * nn[2]
    };
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    for strip in mesh.strips.iter() {
        for tri in strip.windows(3) {
            cells.push(vec![tri[0], tri[1], tri[2]]);
        }
    }
    for cell in &cells {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        let mut edge_pts: Vec<[f64; 3]> = Vec::new();
        for i in 0..nc {
            let (Some(a), Some(b)) = (
                valid_point_index(cell[i], mesh.points.len()),
                valid_point_index(cell[(i + 1) % nc], mesh.points.len()),
            ) else {
                continue;
            };
            let da = dist(a);
            let db = dist(b);
            if da.abs() <= 1e-12 && db.abs() <= 1e-12 {
                push_unique_point(&mut edge_pts, mesh.points.get(a));
                push_unique_point(&mut edge_pts, mesh.points.get(b));
            } else if da.abs() <= 1e-12 {
                push_unique_point(&mut edge_pts, mesh.points.get(a));
            } else if db.abs() <= 1e-12 {
                push_unique_point(&mut edge_pts, mesh.points.get(b));
            } else if da * db < 0.0 {
                let t = da / (da - db);
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                push_unique_point(
                    &mut edge_pts,
                    [
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ],
                );
            }
        }
        if edge_pts.len() == 2 {
            let i0 = pts.len();
            pts.push(edge_pts[0]);
            pts.push(edge_pts[1]);
            lines.push_cell(&[i0 as i64, (i0 + 1) as i64]);
        }
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.lines = lines;
    *r.field_data_mut() = mesh.field_data().clone();
    r
}

fn valid_point_index(id: i64, n_points: usize) -> Option<usize> {
    usize::try_from(id).ok().filter(|&id| id < n_points)
}

fn push_unique_point(points: &mut Vec<[f64; 3]>, point: [f64; 3]) {
    if !points.iter().any(|p| {
        (p[0] - point[0]).abs() <= 1e-12
            && (p[1] - point[1]).abs() <= 1e-12
            && (p[2] - point[2]).abs() <= 1e-12
    }) {
        points.push(point);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_slice() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, -1.0], [2.0, 0.0, -1.0], [1.0, 2.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = slice_mesh_by_plane(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(r.lines.num_cells() >= 1);
    }
    #[test]
    fn test_no_slice() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]],
            vec![[0, 1, 2]],
        );
        let r = slice_mesh_by_plane(&m, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(r.lines.num_cells(), 0);
    }
}
