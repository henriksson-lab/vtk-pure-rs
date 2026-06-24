//! Weld vertices closer than tolerance.
use crate::data::{CellArray, Points, PolyData};
pub fn weld_vertices(mesh: &PolyData, tolerance: f64) -> PolyData {
    let n = mesh.points.len();
    let t2 = tolerance * tolerance;
    let mut remap: Vec<usize> = (0..n).collect();
    for i in 0..n {
        if remap[i] != i {
            continue;
        }
        for j in i + 1..n {
            if remap[j] != j {
                continue;
            }
            let pi = mesh.points.get(i);
            let pj = mesh.points.get(j);
            if (pi[0] - pj[0]).powi(2) + (pi[1] - pj[1]).powi(2) + (pi[2] - pj[2]).powi(2) < t2 {
                remap[j] = i;
            }
        }
    }
    let mut used = vec![false; n];
    let mut kept = Vec::new();
    for cell in mesh.polys.iter() {
        let mapped: Vec<usize> = cell
            .iter()
            .map(|&v| {
                let mut r = v as usize;
                while remap[r] != r {
                    r = remap[r];
                }
                r
            })
            .collect();
        let unique: std::collections::HashSet<usize> = mapped.iter().copied().collect();
        if unique.len() >= 3 {
            for &v in &mapped {
                used[v] = true;
            }
            kept.push(mapped);
        }
    }
    let mut pm = vec![0usize; n];
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        if used[i] {
            pm[i] = pts.len();
            pts.push(mesh.points.get(i));
        }
    }
    let mut polys = CellArray::new();
    for cell in &kept {
        polys.push_cell(&cell.iter().map(|&v| pm[v] as i64).collect::<Vec<_>>());
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
                [0.001, 0.001, 0.0],
            ],
            vec![[0, 1, 2]],
        );
        let r = weld_vertices(&m, 0.01);
        assert!(r.points.len() <= 3);
    }
}
