use std::ops::{Add, AddAssign, Index, IndexMut};

use super::vtk_type::VtkIdType;

/// VTK: `vtkBatch<TBatchData>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch<TBatchData> {
    pub begin_id: VtkIdType,
    pub end_id: VtkIdType,
    pub data: TBatchData,
}

impl<TBatchData> Default for Batch<TBatchData>
where
    TBatchData: Default,
{
    fn default() -> Self {
        Self {
            begin_id: 0,
            end_id: 0,
            data: TBatchData::default(),
        }
    }
}

/// VTK: `vtkBatches<TBatchData>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batches<TBatchData> {
    batches: Vec<Batch<TBatchData>>,
    batch_size: u32,
}

impl<TBatchData> Batches<TBatchData> {
    /// VTK: `vtkBatches<TBatchData>::vtkBatches`.
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
            batch_size: 0,
        }
    }

    /// VTK: `vtkBatches<TBatchData>::Initialize`.
    pub fn initialize(&mut self, number_of_elements: VtkIdType)
    where
        TBatchData: Default,
    {
        self.initialize_with_batch_size(number_of_elements, 1000);
    }

    /// VTK: `vtkBatches<TBatchData>::Initialize`.
    pub fn initialize_with_batch_size(&mut self, number_of_elements: VtkIdType, batch_size: u32)
    where
        TBatchData: Default,
    {
        self.batch_size = batch_size;

        let batch_size_signed = VtkIdType::from(batch_size);
        let number_of_batches = ((number_of_elements - 1) / batch_size_signed) + 1;
        self.batches
            .resize_with(vtk_id_to_usize(number_of_batches), Batch::default);
        let last_batch_id = number_of_batches - 1;

        for batch_id in 0..number_of_batches {
            let batch = &mut self.batches[vtk_id_to_usize(batch_id)];
            batch.begin_id = batch_id * batch_size_signed;
            batch.end_id = if batch_id == last_batch_id {
                number_of_elements
            } else {
                (batch_id + 1) * batch_size_signed
            };
        }
    }

    /// VTK: `vtkBatches<TBatchData>::TrimBatches`.
    pub fn trim_batches(
        &mut self,
        mut should_remove_batch: impl FnMut(&Batch<TBatchData>) -> bool,
    ) {
        self.batches.retain(|batch| !should_remove_batch(batch));
    }

    /// VTK: `vtkBatches<TBatchData>::BuildOffsetsAndGetGlobalSum`.
    pub fn build_offsets_and_get_global_sum(&mut self) -> TBatchData
    where
        TBatchData: Add<Output = TBatchData> + AddAssign + Clone + Default,
    {
        if self.batches.is_empty() {
            return TBatchData::default();
        }

        let mut global_sum = TBatchData::default();
        for batch in &self.batches {
            global_sum += batch.data.clone();
        }

        let mut offset = TBatchData::default();
        for batch in &mut self.batches {
            let current_batch_data = batch.data.clone();
            let next_offset = offset.clone() + current_batch_data;
            batch.data = offset;
            offset = next_offset;
        }

        global_sum
    }

    /// VTK: `vtkBatches<TBatchData>::GetNumberOfBatches`.
    pub fn get_number_of_batches(&self) -> VtkIdType {
        self.batches.len() as VtkIdType
    }

    /// VTK: `vtkBatches<TBatchData>::GetBatchSize`.
    pub fn get_batch_size(&self) -> u32 {
        self.batch_size
    }

    /// VTK: `vtkBatches<TBatchData>::begin`.
    pub fn begin(&self) -> std::slice::Iter<'_, Batch<TBatchData>> {
        self.batches.iter()
    }

    /// VTK: `vtkBatches<TBatchData>::begin`.
    pub fn begin_mut(&mut self) -> std::slice::IterMut<'_, Batch<TBatchData>> {
        self.batches.iter_mut()
    }

    /// VTK: `vtkBatches<TBatchData>::cbegin`.
    pub fn cbegin(&self) -> std::slice::Iter<'_, Batch<TBatchData>> {
        self.batches.iter()
    }

    /// VTK: `vtkBatches<TBatchData>::end`.
    pub fn end(&self) -> std::slice::Iter<'_, Batch<TBatchData>> {
        self.batches[self.batches.len()..].iter()
    }

    /// VTK: `vtkBatches<TBatchData>::end`.
    pub fn end_mut(&mut self) -> std::slice::IterMut<'_, Batch<TBatchData>> {
        let len = self.batches.len();
        self.batches[len..].iter_mut()
    }

    /// VTK: `vtkBatches<TBatchData>::cend`.
    pub fn cend(&self) -> std::slice::Iter<'_, Batch<TBatchData>> {
        self.batches[self.batches.len()..].iter()
    }

    pub fn as_slice(&self) -> &[Batch<TBatchData>] {
        &self.batches
    }

    pub fn as_mut_slice(&mut self) -> &mut [Batch<TBatchData>] {
        &mut self.batches
    }
}

impl<TBatchData> Default for Batches<TBatchData> {
    fn default() -> Self {
        Self::new()
    }
}

impl<TBatchData> Index<usize> for Batches<TBatchData> {
    type Output = Batch<TBatchData>;

    /// VTK: `vtkBatches<TBatchData>::operator[]`.
    fn index(&self, index: usize) -> &Self::Output {
        &self.batches[index]
    }
}

impl<TBatchData> IndexMut<usize> for Batches<TBatchData> {
    /// VTK: `vtkBatches<TBatchData>::operator[]`.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.batches[index]
    }
}

fn vtk_id_to_usize(value: VtkIdType) -> usize {
    usize::try_from(value).expect("vtkIdType value must fit usize")
}
