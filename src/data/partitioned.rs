//! Partitioned dataset for distributed/decomposed data.
//!
//! Analogous to VTK's `vtkPartitionedDataSet` and `vtkPartitionedDataSetCollection`.

use crate::data::{AnyDataArray, AnyDataSet, DataArray, DataSetAttributes, PolyData};

/// A collection of PolyData partitions for distributed or decomposed data.
///
/// Each partition has a name and contains a `PolyData` mesh. Partitions can
/// be merged into a single `PolyData` for rendering or further processing.
#[derive(Debug, Clone, Default)]
pub struct PartitionedDataSet {
    pub partitions: Vec<Option<AnyDataSet>>,
    pub partition_names: Vec<Option<String>>,
}

impl PartitionedDataSet {
    /// Create an empty partitioned dataset.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a named partition.
    pub fn add_partition(&mut self, name: &str, data: PolyData) {
        self.partition_names.push(Some(name.to_string()));
        self.partitions.push(Some(data.into()));
    }

    /// Set the number of partitions, initializing new slots to null.
    pub fn set_number_of_partitions(&mut self, num_partitions: usize) {
        self.partitions.resize_with(num_partitions, || None);
        self.partition_names.resize_with(num_partitions, || None);
    }

    /// Set the data object as the given partition, growing with null slots.
    pub fn set_partition(
        &mut self,
        idx: usize,
        partition: impl Into<AnyDataSet>,
    ) -> Option<AnyDataSet> {
        if idx >= self.partitions.len() {
            self.set_number_of_partitions(idx + 1);
        }
        self.partitions[idx].replace(partition.into())
    }

    /// Clear the data object at the given partition slot.
    pub fn clear_partition(&mut self, idx: usize) -> Option<AnyDataSet> {
        self.partitions.get_mut(idx).and_then(Option::take)
    }

    /// Set or clear the partition name metadata.
    pub fn set_partition_name(&mut self, idx: usize, name: Option<impl Into<String>>) -> bool {
        if idx >= self.partition_names.len() {
            return false;
        }
        self.partition_names[idx] = name.map(Into::into);
        true
    }

    /// Remove a partition and its name together.
    pub fn remove_partition(&mut self, idx: usize) -> Option<(Option<String>, AnyDataSet)> {
        if idx >= self.partitions.len() {
            return None;
        }
        let data = self.partitions.remove(idx)?;
        let name = if idx < self.partition_names.len() {
            self.partition_names.remove(idx)
        } else {
            None
        };
        Some((name, data))
    }

    /// Remove all null partition slots, preserving metadata for surviving slots.
    pub fn remove_null_partitions(&mut self) {
        let mut next = 0;
        for cc in 0..self.partitions.len() {
            if self.partitions[cc].is_some() {
                if next < cc {
                    self.partitions[next] = self.partitions[cc].take();
                    self.partition_names[next] = self.partition_names[cc].take();
                }
                next += 1;
            }
        }
        self.set_number_of_partitions(next);
    }

    /// Get a partition by index.
    pub fn partition(&self, idx: usize) -> Option<&PolyData> {
        match self.partitions.get(idx)?.as_ref()? {
            AnyDataSet::Poly(data) => Some(data),
            _ => None,
        }
    }

    /// Get a partition by index as any dataset.
    pub fn partition_as_data_set(&self, idx: usize) -> Option<&AnyDataSet> {
        self.partitions.get(idx).and_then(Option::as_ref)
    }

    /// Get a partition name by index, if present.
    pub fn partition_name(&self, idx: usize) -> Option<&str> {
        self.partition_names
            .get(idx)
            .and_then(|name| name.as_deref())
    }

    /// Get a partition by name (returns the first match).
    pub fn partition_by_name(&self, name: &str) -> Option<&PolyData> {
        self.partition_names
            .iter()
            .position(|n| n.as_deref() == Some(name))
            .and_then(|idx| self.partitions.get(idx))
            .and_then(Option::as_ref)
            .and_then(|data| match data {
                AnyDataSet::Poly(data) => Some(data),
                _ => None,
            })
    }

    /// Number of partitions.
    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }

    /// Returns true when the public partition and name vectors are aligned.
    pub fn has_consistent_names(&self) -> bool {
        self.partition_names.len() == self.partitions.len()
    }

    /// Merge all partitions into a single PolyData.
    ///
    /// Points and cells from each partition are concatenated, with cell indices
    /// offset appropriately.
    pub fn merge(&self) -> PolyData {
        let mut result = PolyData::new();
        let mut point_offset: i64 = 0;

        for part in self
            .partitions
            .iter()
            .filter_map(|part| match part.as_ref()? {
                AnyDataSet::Poly(part) => Some(part),
                _ => None,
            })
        {
            // Append points
            for i in 0..part.points.len() {
                result.points.push(part.points.get(i));
            }

            // Append verts with offset
            for ci in 0..part.verts.num_cells() {
                let cell = part.verts.cell(ci);
                let offset_cell: Vec<i64> = cell.iter().map(|&id| id + point_offset).collect();
                result.verts.push_cell(&offset_cell);
            }

            // Append lines with offset
            for ci in 0..part.lines.num_cells() {
                let cell = part.lines.cell(ci);
                let offset_cell: Vec<i64> = cell.iter().map(|&id| id + point_offset).collect();
                result.lines.push_cell(&offset_cell);
            }

            // Append polys with offset
            for ci in 0..part.polys.num_cells() {
                let cell = part.polys.cell(ci);
                let offset_cell: Vec<i64> = cell.iter().map(|&id| id + point_offset).collect();
                result.polys.push_cell(&offset_cell);
            }

            // Append triangle strips with offset
            for ci in 0..part.strips.num_cells() {
                let cell = part.strips.cell(ci);
                let offset_cell: Vec<i64> = cell.iter().map(|&id| id + point_offset).collect();
                result.strips.push_cell(&offset_cell);
            }

            point_offset += part.points.len() as i64;
        }

        append_attributes(
            result.point_data_mut(),
            &self.poly_data_partitions(),
            |part| part.point_data(),
            |part| part.points.len(),
        );
        append_attributes(
            result.cell_data_mut(),
            &self.poly_data_partitions(),
            |part| part.cell_data(),
            |part| part.total_cells(),
        );

        result
    }

    fn poly_data_partitions(&self) -> Vec<PolyData> {
        self.partitions
            .iter()
            .filter_map(|part| match part.as_ref()? {
                AnyDataSet::Poly(part) => Some(part.clone()),
                _ => None,
            })
            .collect()
    }
}

fn append_attributes(
    target: &mut DataSetAttributes,
    partitions: &[PolyData],
    attrs: impl Fn(&PolyData) -> &DataSetAttributes,
    expected_tuples: impl Fn(&PolyData) -> usize,
) {
    let Some(first) = partitions.first() else {
        return;
    };
    let first_attrs = attrs(first);
    for first_array in first_attrs.iter() {
        if first_array.num_tuples() != expected_tuples(first) {
            continue;
        }
        let mut arrays = Vec::with_capacity(partitions.len());
        let mut compatible = true;
        for part in partitions {
            let Some(array) = attrs(part).get_array(first_array.name()) else {
                compatible = false;
                break;
            };
            if array.scalar_type() != first_array.scalar_type()
                || array.num_components() != first_array.num_components()
                || array.num_tuples() != expected_tuples(part)
            {
                compatible = false;
                break;
            }
            arrays.push(array);
        }
        if compatible {
            if let Some(array) = concat_arrays(&arrays) {
                let name = array.name().to_string();
                target.add_array(array);
                if first_attrs.scalars().map(|a| a.name()) == Some(name.as_str()) {
                    target.set_active_scalars(&name);
                }
                if first_attrs.vectors().map(|a| a.name()) == Some(name.as_str()) {
                    target.set_active_vectors(&name);
                }
                if first_attrs.normals().map(|a| a.name()) == Some(name.as_str()) {
                    target.set_active_normals(&name);
                }
            }
        }
    }
}

fn concat_arrays(arrays: &[&AnyDataArray]) -> Option<AnyDataArray> {
    let first = *arrays.first()?;
    macro_rules! concat_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(first_array) = first else {
                unreachable!();
            };
            let mut data = Vec::new();
            for array in arrays {
                let AnyDataArray::$variant(array) = *array else {
                    return None;
                };
                data.extend_from_slice(array.as_slice());
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                first_array.name(),
                data,
                first_array.num_components(),
            )))
        }};
    }
    match first {
        AnyDataArray::F32(_) => concat_variant!(F32),
        AnyDataArray::F64(_) => concat_variant!(F64),
        AnyDataArray::I8(_) => concat_variant!(I8),
        AnyDataArray::I16(_) => concat_variant!(I16),
        AnyDataArray::I32(_) => concat_variant!(I32),
        AnyDataArray::I64(_) => concat_variant!(I64),
        AnyDataArray::U8(_) => concat_variant!(U8),
        AnyDataArray::U16(_) => concat_variant!(U16),
        AnyDataArray::U32(_) => concat_variant!(U32),
        AnyDataArray::U64(_) => concat_variant!(U64),
    }
}

/// A collection of `PartitionedDataSet`s.
///
/// Analogous to VTK's `vtkPartitionedDataSetCollection`.
#[derive(Debug, Clone, Default)]
pub struct PartitionedDataSetCollection {
    pub datasets: Vec<PartitionedDataSet>,
}

impl PartitionedDataSetCollection {
    /// Create an empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a partitioned dataset to the collection.
    pub fn add(&mut self, dataset: PartitionedDataSet) {
        self.datasets.push(dataset);
    }

    /// Set the number of partitioned datasets, creating empty datasets for new slots.
    pub fn set_number_of_partitioned_data_sets(&mut self, num_datasets: usize) {
        self.datasets
            .resize_with(num_datasets, PartitionedDataSet::new);
    }

    /// Number of datasets in the collection.
    pub fn num_datasets(&self) -> usize {
        self.datasets.len()
    }

    /// Get a dataset by index.
    pub fn dataset(&self, idx: usize) -> Option<&PartitionedDataSet> {
        self.datasets.get(idx)
    }

    /// Set a partitioned dataset at an index, growing the collection if needed.
    pub fn set_dataset(
        &mut self,
        idx: usize,
        dataset: PartitionedDataSet,
    ) -> Option<PartitionedDataSet> {
        if idx >= self.datasets.len() {
            self.set_number_of_partitioned_data_sets(idx + 1);
        }
        Some(std::mem::replace(&mut self.datasets[idx], dataset))
    }

    /// Remove a partitioned dataset by index.
    pub fn remove_dataset(&mut self, idx: usize) -> Option<PartitionedDataSet> {
        (idx < self.datasets.len()).then(|| self.datasets.remove(idx))
    }

    /// Set a partition through the collection tuple-index API.
    pub fn set_partition(
        &mut self,
        idx: usize,
        partition: usize,
        object: impl Into<AnyDataSet>,
    ) -> Option<AnyDataSet> {
        if idx >= self.datasets.len() {
            self.set_number_of_partitioned_data_sets(idx + 1);
        }
        self.datasets[idx].set_partition(partition, object)
    }

    /// Get a partition through the collection tuple-index API.
    pub fn partition(&self, idx: usize, partition: usize) -> Option<&PolyData> {
        self.datasets.get(idx)?.partition(partition)
    }

    /// Number of partitions in the dataset at the given index.
    pub fn num_partitions(&self, idx: usize) -> usize {
        self.datasets
            .get(idx)
            .map(PartitionedDataSet::num_partitions)
            .unwrap_or(0)
    }

    /// Set number of partitions in the dataset at the given index, growing as needed.
    pub fn set_number_of_partitions(&mut self, idx: usize, num_partitions: usize) {
        if idx >= self.datasets.len() {
            self.set_number_of_partitioned_data_sets(idx + 1);
        }
        self.datasets[idx].set_number_of_partitions(num_partitions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle(offset: f64) -> PolyData {
        PolyData::from_triangles(
            vec![
                [offset, 0.0, 0.0],
                [offset + 1.0, 0.0, 0.0],
                [offset + 0.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn add_and_query_partitions() {
        let mut pds = PartitionedDataSet::new();
        pds.add_partition("left", triangle(0.0));
        pds.add_partition("right", triangle(5.0));

        assert_eq!(pds.num_partitions(), 2);
        assert!(pds.partition(0).is_some());
        assert!(pds.partition(2).is_none());
        assert!(pds.partition_by_name("left").is_some());
        assert!(pds.partition_by_name("missing").is_none());
    }

    #[test]
    fn merge_partitions() {
        let mut pds = PartitionedDataSet::new();
        pds.add_partition("a", triangle(0.0));
        pds.add_partition("b", triangle(5.0));

        let merged = pds.merge();
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);

        // Second triangle's indices should be offset by 3
        let cell1 = merged.polys.cell(1);
        assert_eq!(cell1, &[3, 4, 5]);
    }

    #[test]
    fn merge_preserves_triangle_strips() {
        let mut first =
            PolyData::from_points(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        first.strips.push_cell(&[0, 1, 2]);

        let mut second =
            PolyData::from_points(vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]]);
        second.strips.push_cell(&[0, 1, 2]);

        let mut pds = PartitionedDataSet::new();
        pds.add_partition("a", first);
        pds.add_partition("b", second);

        let merged = pds.merge();
        assert_eq!(merged.strips.num_cells(), 2);
        assert_eq!(merged.strips.cell(1), &[3, 4, 5]);
    }

    #[test]
    fn merge_preserves_compatible_point_and_cell_data() {
        let mut first = triangle(0.0);
        first.add_scalars("temperature", vec![1.0, 2.0, 3.0]);
        first
            .cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("ids", vec![10], 1)));

        let mut second = triangle(5.0);
        second.add_scalars("temperature", vec![4.0, 5.0, 6.0]);
        second
            .cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("ids", vec![20], 1)));

        let mut pds = PartitionedDataSet::new();
        pds.add_partition("a", first);
        pds.add_partition("b", second);

        let merged = pds.merge();
        let scalars = merged.point_data().get_array("temperature").unwrap();
        assert_eq!(scalars.to_f64_vec(), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let ids = merged.cell_data().get_array("ids").unwrap();
        assert_eq!(ids.to_f64_vec(), vec![10.0, 20.0]);
    }

    #[test]
    fn remove_partition_keeps_names_aligned_when_possible() {
        let mut pds = PartitionedDataSet::new();
        pds.add_partition("a", triangle(0.0));
        pds.add_partition("b", triangle(1.0));
        assert!(pds.has_consistent_names());
        let removed = pds.remove_partition(0).unwrap();
        assert_eq!(removed.0.as_deref(), Some("a"));
        assert_eq!(pds.partition_name(0), Some("b"));
        assert!(pds.has_consistent_names());
    }

    #[test]
    fn remove_null_partitions_preserves_survivor_order() {
        let mut pds = PartitionedDataSet::new();
        pds.set_number_of_partitions(5);
        pds.set_partition(1, triangle(1.0));
        pds.set_partition_name(1, Some("one"));
        pds.set_partition(3, triangle(3.0));
        pds.set_partition_name(3, Some("three"));
        pds.set_partition(4, triangle(4.0));
        pds.set_partition_name(4, Some("four"));

        pds.remove_null_partitions();

        assert_eq!(pds.num_partitions(), 3);
        assert_eq!(pds.partition_name(0), Some("one"));
        assert_eq!(pds.partition_name(1), Some("three"));
        assert_eq!(pds.partition_name(2), Some("four"));
        assert!(pds.has_consistent_names());
    }

    #[test]
    fn collection() {
        let mut coll = PartitionedDataSetCollection::new();
        let mut pds = PartitionedDataSet::new();
        pds.add_partition("part0", triangle(0.0));
        coll.add(pds);

        assert_eq!(coll.num_datasets(), 1);
        assert_eq!(coll.dataset(0).unwrap().num_partitions(), 1);
    }
}
