//! Extract silhouette edges from a viewpoint.
use crate::data::{CellArray, Points, PolyData};
pub fn extract_silhouette(mesh: &PolyData, view_point: [f64; 3]) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, cell) in cells.iter().enumerate() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let face_facing: Vec<bool> = cells
        .iter()
        .map(|cell| {
            if cell.len() < 3 {
                return false;
            }
            let a = mesh.points.get(cell[0] as usize);
            let b = mesh.points.get(cell[1] as usize);
            let c = mesh.points.get(cell[2] as usize);
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let to_view = [
                view_point[0] - a[0],
                view_point[1] - a[1],
                view_point[2] - a[2],
            ];
            n[0] * to_view[0] + n[1] * to_view[1] + n[2] * to_view[2] > 0.0
        })
        .collect();
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut pm: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for (&(a, b), faces) in &ef {
        let is_silhouette = if faces.len() == 1 {
            true
        } else if faces.len() == 2 {
            face_facing[faces[0]] != face_facing[faces[1]]
        } else {
            false
        };
        if is_silhouette {
            let ia = *pm.entry(a).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(a));
                i
            });
            let ib = *pm.entry(b).or_insert_with(|| {
                let i = pts.len();
                pts.push(mesh.points.get(b));
                i
            });
            lines.push_cell(&[ia as i64, ib as i64]);
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
    fn test() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );
        let r = extract_silhouette(&m, [0.0, 0.0, 10.0]);
        assert!(r.lines.num_cells() >= 1);
    }
}
