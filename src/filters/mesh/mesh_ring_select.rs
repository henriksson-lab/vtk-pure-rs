//! Select faces within N edge-rings of a seed face.
use crate::data::{CellArray, Points, PolyData};
pub fn select_face_ring(mesh: &PolyData, seed_face: usize, rings: usize) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    if seed_face >= nc {
        return PolyData::new();
    }
    let mut ef: std::collections::HashMap<(usize, usize), Vec<usize>> =
        std::collections::HashMap::new();
    for (ci, c) in cells.iter().enumerate() {
        let n = c.len();
        for i in 0..n {
            let a = c[i] as usize;
            let b = c[(i + 1) % n] as usize;
            ef.entry((a.min(b), a.max(b))).or_default().push(ci);
        }
    }
    let mut fadj: Vec<Vec<usize>> = vec![Vec::new(); nc];
    for (_, faces) in &ef {
        for i in 0..faces.len() {
            for j in i + 1..faces.len() {
                fadj[faces[i]].push(faces[j]);
                fadj[faces[j]].push(faces[i]);
            }
        }
    }
    let mut selected = vec![false; nc];
    selected[seed_face] = true;
    let mut frontier = vec![seed_face];
    for _ in 0..rings {
        let mut next = Vec::new();
        for &fi in &frontier {
            for &ni in &fadj[fi] {
                if !selected[ni] {
                    selected[ni] = true;
                    next.push(ni);
                }
            }
        }
        frontier = next;
    }
    let mut used = vec![false; mesh.points.len()];
    let mut kept = Vec::new();
    for (ci, c) in cells.iter().enumerate() {
        if selected[ci] {
            for &v in c {
                used[v as usize] = true;
            }
            kept.push(c.clone());
        }
    }
    let mut pm = vec![0usize; mesh.points.len()];
    let mut pts = Points::<f64>::new();
    for i in 0..mesh.points.len() {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for c in &kept {
        polys.push_cell(&c.iter().map(|&v| pm[v as usize] as i64).collect::<Vec<_>>());
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
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2], [2, 3, 5]],
        );
        let r = select_face_ring(&m, 0, 1);
        assert!(r.polys.num_cells() >= 2 && r.polys.num_cells() <= 4);
    }
}
