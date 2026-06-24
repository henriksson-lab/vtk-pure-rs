//! Ghost cell/point exchange between partitions.
//!
//! Identifies shared boundary points/cells between partitions and
//! builds ghost layers for halo exchange.

use crate::data::PolyData;
use crate::parallel::decomposition::Partition;

/// Ghost layer information for a partition.
#[derive(Debug, Clone)]
pub struct GhostLayer {
    /// Points shared with other partitions: (local_idx, remote_rank, remote_idx).
    pub shared_points: Vec<(usize, usize, usize)>,
    /// Number of ghost points added.
    pub num_ghost_points: usize,
    /// Number of ghost cells added.
    pub num_ghost_cells: usize,
}

/// Compute ghost layers between partitions.
///
/// For each partition, identifies boundary points that need to be
/// exchanged with neighboring partitions (those sharing global point IDs).
pub fn compute_ghost_layers(partitions: &[Partition]) -> Vec<GhostLayer> {
    let mut layers = Vec::with_capacity(partitions.len());

    for (rank, part) in partitions.iter().enumerate() {
        let mut shared = Vec::new();

        for (local_idx, &global_id) in part.global_point_ids.iter().enumerate() {
            // Check if this global point exists in any other partition
            for (other_rank, other_part) in partitions.iter().enumerate() {
                if other_rank == rank {
                    continue;
                }
                if let Some(other_local) = other_part
                    .global_point_ids
                    .iter()
                    .position(|&g| g == global_id)
                {
                    shared.push((local_idx, other_rank, other_local));
                }
            }
        }

        layers.push(GhostLayer {
            num_ghost_points: shared.len(),
            num_ghost_cells: 0,
            shared_points: shared,
        });
    }

    layers
}

/// Add ghost points from neighboring partitions.
///
/// Returns a new PolyData with ghost points appended and a
/// "GhostType" point data array (0 = owned, 1 = ghost).
pub fn add_ghost_points(
    partition: &Partition,
    neighbors: &[Partition],
    layer: &GhostLayer,
) -> PolyData {
    let mut result = partition.data.clone();
    let owned_count = result.points.len();

    let mut ghost_type = vec![0u8; owned_count];

    for &(_, remote_rank, remote_local) in &layer.shared_points {
        if let Some(neighbor) = neighbors.iter().find(|p| p.rank == remote_rank) {
            if remote_local < neighbor.data.points.len() {
                result.points.push(neighbor.data.points.get(remote_local));
                ghost_type.push(1);
            }
        }
    }

    let ghost_f64: Vec<f64> = ghost_type.iter().map(|&v| v as f64).collect();
    result
        .point_data_mut()
        .add_array(crate::data::AnyDataArray::F64(
            crate::data::DataArray::from_vec("GhostType", ghost_f64, 1),
        ));

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parallel::decomposition::decompose_poly_data;

    #[test]
    fn ghost_detection() {
        // Two triangles sharing an edge (points 1,2)
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = decompose_poly_data(&pd, 2);
        let layers = compute_ghost_layers(&parts);
        assert_eq!(layers.len(), 2);
    }
}
