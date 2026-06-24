//! Randomly subsample vertices or faces.
use crate::data::{CellArray, Points, PolyData};
pub fn random_vertex_subsample(mesh: &PolyData, ratio: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let keep = (n as f64 * ratio.clamp(0.0, 1.0)) as usize;
    let mut indices: Vec<usize> = (0..n).collect();
    let mut rng = seed;
    for i in (1..n).rev() {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }
    let mut pts = Points::<f64>::new();
    let mut verts = CellArray::new();
    for i in 0..keep {
        let idx = pts.len();
        pts.push(mesh.points.get(indices[i]));
        verts.push_cell(&[idx as i64]);
    }
    let mut r = PolyData::new();
    r.points = pts;
    r.verts = verts;
    r
}
pub fn random_face_subsample(mesh: &PolyData, ratio: f64, seed: u64) -> PolyData {
    let cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let nc = cells.len();
    let keep = (nc as f64 * ratio.clamp(0.0, 1.0)) as usize;
    let mut indices: Vec<usize> = (0..nc).collect();
    let mut rng = seed;
    for i in (1..nc).rev() {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }
    let kept_set: std::collections::HashSet<usize> = indices[..keep].iter().copied().collect();
    let mut used = vec![false; mesh.points.len()];
    let kept_cells: Vec<&Vec<i64>> = cells
        .iter()
        .enumerate()
        .filter(|(i, _)| kept_set.contains(i))
        .map(|(_, c)| c)
        .collect();
    for c in &kept_cells {
        for &v in *c {
            used[v as usize] = true;
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
    for c in &kept_cells {
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
    fn test_verts() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3]],
        );
        let r = random_vertex_subsample(&m, 0.5, 42);
        assert!(r.points.len() <= 5);
    }
    #[test]
    fn test_faces() {
        let m = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 4, 3]],
        );
        let r = random_face_subsample(&m, 0.5, 42);
        assert!(r.polys.num_cells() <= 2);
    }
}
