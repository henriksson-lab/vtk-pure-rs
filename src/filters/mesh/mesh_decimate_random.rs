//! Random face removal decimation.
use crate::data::{CellArray, Points, PolyData};
pub fn decimate_random(mesh: &PolyData, keep_ratio: f64, seed: u64) -> PolyData {
    let nc = mesh.polys.num_cells();
    let keep = (nc as f64 * keep_ratio.clamp(0.0, 1.0)) as usize;
    let mut indices: Vec<usize> = (0..nc).collect();
    let mut rng = seed;
    for i in (1..indices.len()).rev() {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }
    let kept: std::collections::HashSet<usize> = indices[..keep].iter().copied().collect();
    let cells: Vec<Vec<i64>> = mesh
        .polys
        .iter()
        .enumerate()
        .filter(|(i, _)| kept.contains(i))
        .map(|(_, c)| c.to_vec())
        .collect();
    let mut used = vec![false; mesh.points.len()];
    for c in &cells {
        for &v in c {
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
    for c in &cells {
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
            ],
            vec![[0, 1, 2], [1, 4, 3], [1, 3, 2]],
        );
        let r = decimate_random(&m, 0.5, 42);
        assert!(r.polys.num_cells() <= 3);
    }
}
