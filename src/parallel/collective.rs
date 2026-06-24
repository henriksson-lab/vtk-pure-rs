//! Collective operations on distributed datasets (serial stubs).
//!
//! These functions work on local data. When the `mpi` feature is enabled,
//! `mpi_backend` provides actual distributed implementations.

use crate::data::{AnyDataArray, DataArray, PolyData};

/// Gather PolyData from all partitions into one (serial: just merge).
pub fn gather_poly_data(partitions: &[PolyData]) -> PolyData {
    if partitions.is_empty() {
        return PolyData::new();
    }
    if partitions.len() == 1 {
        return partitions[0].clone();
    }

    let mut result = partitions[0].clone();
    for part in &partitions[1..] {
        let offset = result.points.len() as i64;
        for i in 0..part.points.len() {
            result.points.push(part.points.get(i));
        }
        for ci in 0..part.polys.num_cells() {
            let cell = part.polys.cell(ci);
            let remapped: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
            result.polys.push_cell(&remapped);
        }
    }
    result
}

/// Reduce a scalar across all partitions (serial: just compute from local data).
#[derive(Debug, Clone, Copy)]
pub enum ReduceOp {
    Sum,
    Min,
    Max,
    Mean,
}

/// Reduce a scalar value across partitions.
pub fn reduce_scalar(values: &[f64], op: ReduceOp) -> f64 {
    match op {
        ReduceOp::Sum => values.iter().sum(),
        ReduceOp::Min => values.iter().cloned().fold(f64::MAX, f64::min),
        ReduceOp::Max => values.iter().cloned().fold(f64::MIN, f64::max),
        ReduceOp::Mean => {
            if values.is_empty() {
                0.0
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            }
        }
    }
}

/// Broadcast a PolyData from rank 0 to all (serial: identity).
pub fn broadcast_poly_data(data: &PolyData) -> PolyData {
    data.clone()
}

/// Scatter: distribute parts of a PolyData to N ranks (serial: decompose by cell index).
pub fn scatter_poly_data(data: &PolyData, num_ranks: usize) -> Vec<PolyData> {
    let nc = data.polys.num_cells();
    let per_rank = (nc + num_ranks - 1) / num_ranks;
    let mut parts = Vec::new();

    for r in 0..num_ranks {
        let start = r * per_rank;
        let end = ((r + 1) * per_rank).min(nc);
        if start >= nc {
            break;
        }

        let mut point_map = vec![i64::MAX; data.points.len()];
        let mut pts = crate::data::Points::<f64>::new();
        let mut polys = crate::data::CellArray::new();

        for ci in start..end {
            let cell = data.polys.cell(ci);
            for &vid in cell {
                let vi = vid as usize;
                if point_map[vi] == i64::MAX {
                    point_map[vi] = pts.len() as i64;
                    pts.push(data.points.get(vi));
                }
            }
            let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
            polys.push_cell(&remapped);
        }

        let mut pd = PolyData::new();
        pd.points = pts;
        pd.polys = polys;
        parts.push(pd);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_two() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let merged = gather_poly_data(&[a, b]);
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);
    }

    #[test]
    fn reduce_ops() {
        assert_eq!(reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Sum), 6.0);
        assert_eq!(reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Min), 1.0);
        assert_eq!(reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Max), 3.0);
        assert!((reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Mean) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn scatter_roundtrip() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = scatter_poly_data(&pd, 2);
        assert_eq!(parts.len(), 2);
        let merged = gather_poly_data(&parts);
        assert_eq!(merged.polys.num_cells(), 2);
    }
}
