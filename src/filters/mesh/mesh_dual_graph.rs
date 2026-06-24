//! Construct dual graph: each face becomes a vertex, connected faces share edges.
use crate::data::{CellArray, Points, PolyData};

pub fn dual_graph(mesh: &PolyData) -> PolyData {
    let n = mesh.points.len();
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if nc == 0 {
        return PolyData::new();
    }
    // Compute face centroids
    let mut pts = Points::<f64>::new();
    for cell in &cells {
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        let nv = cell.len() as f64;
        for &v in cell {
            let vi = v as usize;
            if vi < n {
                let p = mesh.points.get(vi);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
        }
        pts.push([cx / nv, cy / nv, cz / nv]);
    }
    // Build edge-to-face adjacency
    let mut edge_faces: std::collections::HashMap<(i64, i64), Vec<usize>> =
        std::collections::HashMap::new();
    for (fi, cell) in cells.iter().enumerate() {
        let ncv = cell.len();
        for i in 0..ncv {
            let a = cell[i];
            let b = cell[(i + 1) % ncv];
            let e = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(e).or_default().push(fi);
        }
    }
    let mut lines = CellArray::new();
    let mut seen = std::collections::HashSet::new();
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            let (a, b) = (faces[0].min(faces[1]), faces[0].max(faces[1]));
            if seen.insert((a, b)) {
                lines.push_cell(&[a as i64, b as i64]);
            }
        }
    }
    let mut m = PolyData::new();
    m.points = pts;
    m.lines = lines;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dual() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let r = dual_graph(&mesh);
        assert_eq!(r.points.len(), 2);
        assert_eq!(r.lines.num_cells(), 1);
    }
}
