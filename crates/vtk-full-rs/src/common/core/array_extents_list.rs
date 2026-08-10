use std::ops::{Index, IndexMut};

use super::array_extents::ArrayExtents;
use super::vtk_type::VtkIdType;

/// VTK/Common/Core/vtkArrayExtentsList.h
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArrayExtentsList {
    storage: Vec<ArrayExtents>,
}

impl ArrayExtentsList {
    /// VTK: vtkArrayExtentsList::vtkArrayExtentsList()
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// VTK: vtkArrayExtentsList::vtkArrayExtentsList(const vtkArrayExtents&)
    #[must_use]
    pub fn new_1(i: ArrayExtents) -> Self {
        Self { storage: vec![i] }
    }

    /// VTK: vtkArrayExtentsList::vtkArrayExtentsList(const vtkArrayExtents&, const vtkArrayExtents&)
    #[must_use]
    pub fn new_2(i: ArrayExtents, j: ArrayExtents) -> Self {
        Self {
            storage: vec![i, j],
        }
    }

    /// VTK: vtkArrayExtentsList::vtkArrayExtentsList(const vtkArrayExtents&, const vtkArrayExtents&, const vtkArrayExtents&)
    #[must_use]
    pub fn new_3(i: ArrayExtents, j: ArrayExtents, k: ArrayExtents) -> Self {
        Self {
            storage: vec![i, j, k],
        }
    }

    /// VTK: vtkArrayExtentsList::vtkArrayExtentsList(const vtkArrayExtents&, const vtkArrayExtents&, const vtkArrayExtents&, const vtkArrayExtents&)
    #[must_use]
    pub fn new_4(i: ArrayExtents, j: ArrayExtents, k: ArrayExtents, l: ArrayExtents) -> Self {
        Self {
            storage: vec![i, j, k, l],
        }
    }

    /// VTK: vtkArrayExtentsList::GetCount()
    #[must_use]
    pub fn get_count(&self) -> VtkIdType {
        self.storage.len() as VtkIdType
    }

    /// VTK: vtkArrayExtentsList::SetCount(vtkIdType)
    pub fn set_count(&mut self, count: VtkIdType) {
        let count = vtk_id_to_usize(count);
        self.storage.clear();
        self.storage.resize(count, ArrayExtents::new());
    }
}

impl Index<VtkIdType> for ArrayExtentsList {
    type Output = ArrayExtents;

    /// VTK: vtkArrayExtentsList::operator[](vtkIdType) const
    fn index(&self, index: VtkIdType) -> &Self::Output {
        &self.storage[vtk_id_to_usize(index)]
    }
}

impl IndexMut<VtkIdType> for ArrayExtentsList {
    /// VTK: vtkArrayExtentsList::operator[](vtkIdType)
    fn index_mut(&mut self, index: VtkIdType) -> &mut Self::Output {
        &mut self.storage[vtk_id_to_usize(index)]
    }
}

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    assert!(id >= 0, "VTK array extents-list index must be non-negative");
    id as usize
}
