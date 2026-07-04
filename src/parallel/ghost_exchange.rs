//! Ghost cell/point exchange between partitions.
//!
//! Identifies shared boundary points/cells between partitions and
//! builds ghost layers for halo exchange.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, PolyData};
use crate::parallel::decomposition::Partition;

const VTK_GHOST_ARRAY_NAME: &str = "vtkGhostType";
const DUPLICATEPOINT: u8 = 1;

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

    for part in partitions {
        let mut shared = Vec::new();

        for (local_idx, &global_id) in part.global_point_ids.iter().enumerate() {
            // Check if this global point exists in any other partition
            for other_part in partitions {
                if other_part.rank == part.rank {
                    continue;
                }
                if let Some(other_local) = other_part
                    .global_point_ids
                    .iter()
                    .position(|&g| g == global_id)
                {
                    shared.push((local_idx, other_part.rank, other_local));
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
    let mut ghost_sources = Vec::new();

    for &(_, remote_rank, remote_local) in &layer.shared_points {
        if let Some(neighbor) = neighbors.iter().find(|p| p.rank == remote_rank) {
            if remote_local < neighbor.data.points.len() {
                result.points.push(neighbor.data.points.get(remote_local));
                ghost_type.push(DUPLICATEPOINT);
                ghost_sources.push((neighbor, remote_local));
            }
        }
    }

    append_ghost_point_data(partition, &ghost_sources, result.point_data_mut());
    result
        .point_data_mut()
        .add_array(AnyDataArray::U8(DataArray::from_vec(
            VTK_GHOST_ARRAY_NAME,
            ghost_type,
            1,
        )));

    result
}

fn append_ghost_point_data(
    partition: &Partition,
    ghost_sources: &[(&Partition, usize)],
    target: &mut DataSetAttributes,
) {
    for source_array in partition.data.point_data().iter() {
        if source_array.name() == VTK_GHOST_ARRAY_NAME
            || source_array.num_tuples() != partition.data.points.len()
        {
            continue;
        }
        let mut compatible = true;
        let mut remote_arrays = Vec::with_capacity(ghost_sources.len());
        for &(neighbor, _) in ghost_sources {
            let Some(remote_array) = neighbor.data.point_data().get_array(source_array.name())
            else {
                compatible = false;
                break;
            };
            if remote_array.scalar_type() != source_array.scalar_type()
                || remote_array.num_components() != source_array.num_components()
                || remote_array.num_tuples() != neighbor.data.points.len()
            {
                compatible = false;
                break;
            }
            remote_arrays.push(remote_array);
        }
        if compatible {
            if let Some(array) = append_tuples(source_array, ghost_sources, &remote_arrays) {
                let name = array.name().to_string();
                target.add_array(array);
                if partition.data.point_data().scalars().map(|a| a.name()) == Some(name.as_str()) {
                    target.set_active_scalars(&name);
                }
                if partition.data.point_data().vectors().map(|a| a.name()) == Some(name.as_str()) {
                    target.set_active_vectors(&name);
                }
                if partition.data.point_data().normals().map(|a| a.name()) == Some(name.as_str()) {
                    target.set_active_normals(&name);
                }
            }
        }
    }
}

fn append_tuples(
    source: &AnyDataArray,
    ghost_sources: &[(&Partition, usize)],
    remote_arrays: &[&AnyDataArray],
) -> Option<AnyDataArray> {
    macro_rules! append_variant {
        ($source:expr, $variant:ident) => {{
            let nc = $source.num_components();
            let mut data = $source.as_slice().to_vec();
            for ((_, tuple_id), remote_array) in ghost_sources.iter().zip(remote_arrays.iter()) {
                let AnyDataArray::$variant(remote_array) = *remote_array else {
                    return None;
                };
                data.extend_from_slice(remote_array.tuple(*tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $source.name(),
                data,
                nc,
            )))
        }};
    }
    match source {
        AnyDataArray::F32(a) => append_variant!(a, F32),
        AnyDataArray::F64(a) => append_variant!(a, F64),
        AnyDataArray::I8(a) => append_variant!(a, I8),
        AnyDataArray::I16(a) => append_variant!(a, I16),
        AnyDataArray::I32(a) => append_variant!(a, I32),
        AnyDataArray::I64(a) => append_variant!(a, I64),
        AnyDataArray::U8(a) => append_variant!(a, U8),
        AnyDataArray::U16(a) => append_variant!(a, U16),
        AnyDataArray::U32(a) => append_variant!(a, U32),
        AnyDataArray::U64(a) => append_variant!(a, U64),
    }
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
