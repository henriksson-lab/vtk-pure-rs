//! Compute face centroids as a point cloud.
use crate::data::{CellArray, Points, PolyData};
pub fn face_centroids(mesh: &PolyData) -> PolyData {
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for cell in mesh.polys.iter() {
        if cell.is_empty() {
            continue;
        }
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &v in cell {
            let p = mesh.points.get(v as usize);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let nf = cell.len() as f64;
        let idx = pts.len();
        pts.push([cx / nf, cy / nf, cz / nf]);
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let m = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = face_centroids(&m);
        assert_eq!(r.points.len(), 1);
        let p = r.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!((p[1] - 1.0).abs() < 1e-10);
    }
}
