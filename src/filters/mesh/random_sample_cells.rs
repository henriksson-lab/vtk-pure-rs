use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Randomly select a fraction of cells from a mesh.
///
/// Keeps each cell with probability `fraction`. Deterministic via `seed`.
pub fn random_sample_cells(input: &PolyData, fraction: f64, seed: u64) -> PolyData {
    let frac = fraction.clamp(0.0, 1.0);
    let mut rng = seed;

    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for cell in input.polys.iter() {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = (rng >> 33) as f64 / (1u64 << 31) as f64;
        if r < frac {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

/// Select every Nth cell (systematic sampling).
pub fn every_nth_cell(input: &PolyData, n: usize) -> PolyData {
    let n = n.max(1);
    let mut pt_map: HashMap<i64, i64> = HashMap::new();
    let mut out_pts = Points::<f64>::new();
    let mut out_polys = CellArray::new();

    for (ci, cell) in input.polys.iter().enumerate() {
        if ci % n == 0 {
            let mapped: Vec<i64> = cell
                .iter()
                .map(|&id| {
                    *pt_map.entry(id).or_insert_with(|| {
                        let idx = out_pts.len() as i64;
                        out_pts.push(input.points.get(id as usize));
                        idx
                    })
                })
                .collect();
            out_polys.push_cell(&mapped);
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.polys = out_polys;
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_fraction() {
        let mut pd = PolyData::new();
        for j in 0..5 {
            for i in 0..5 {
                pd.points.push([i as f64, j as f64, 0.0]);
            }
        }
        for j in 0..4 {
            for i in 0..4 {
                let a = (j * 5 + i) as i64;
                pd.polys.push_cell(&[a, a + 1, a + 6]);
                pd.polys.push_cell(&[a, a + 6, a + 5]);
            }
        }

        let result = random_sample_cells(&pd, 0.5, 42);
        assert!(result.polys.num_cells() > 5);
        assert!(result.polys.num_cells() < 28);
    }

    #[test]
    fn every_2nd() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.polys.push_cell(&[0, 1, 3]);
        pd.polys.push_cell(&[1, 2, 3]);

        let result = every_nth_cell(&pd, 2);
        assert_eq!(result.polys.num_cells(), 2); // cells 0 and 2
    }

    #[test]
    fn fraction_zero() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = random_sample_cells(&pd, 0.0, 0);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        assert_eq!(random_sample_cells(&pd, 0.5, 0).polys.num_cells(), 0);
        assert_eq!(every_nth_cell(&pd, 2).polys.num_cells(), 0);
    }
}
