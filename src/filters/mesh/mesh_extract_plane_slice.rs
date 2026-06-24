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
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let nc = cell.len();
        let mut edge_pts: Vec<[f64; 3]> = Vec::new();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let da = dist(a);
            let db = dist(b);
            if da * db < 0.0 {
                let t = da / (da - db);
                let pa = mesh.points.get(a);
                let pb = mesh.points.get(b);
                edge_pts.push([
                    pa[0] + t * (pb[0] - pa[0]),
                    pa[1] + t * (pb[1] - pa[1]),
                    pa[2] + t * (pb[2] - pa[2]),
                ]);
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
    r
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
