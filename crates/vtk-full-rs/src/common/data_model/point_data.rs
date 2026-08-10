use crate::common::{
    core::{AnyArray, VtkIdType, VtkMTimeType},
    data_model::{DataSetAttributes, DataSetAttributesError, HIDDENPOINT},
};

/// Point attribute data.
///
/// VTK origin: `VTK/Common/DataModel/vtkPointData.h` and
/// `VTK/Common/DataModel/vtkPointData.cxx`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointData {
    attributes: DataSetAttributes,
}

impl PointData {
    /// VTK: `vtkPointData::New`.
    pub fn new() -> Self {
        let mut attributes = DataSetAttributes::new();
        attributes.set_ghosts_to_skip(HIDDENPOINT);
        Self { attributes }
    }

    /// VTK: `vtkPointData::ExtendedNew`.
    pub fn extended_new() -> Self {
        Self::new()
    }

    /// VTK: `vtkPointData::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.attributes.print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkPointData"
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.attributes.get_m_time()
    }

    /// VTK: `vtkFieldData::GetGhostsToSkip`.
    pub fn get_ghosts_to_skip(&self) -> u8 {
        self.attributes.get_ghosts_to_skip()
    }

    /// VTK: `vtkFieldData::SetGhostsToSkip`.
    pub fn set_ghosts_to_skip(&mut self, ghosts_to_skip: u8) {
        self.attributes.set_ghosts_to_skip(ghosts_to_skip);
    }

    /// VTK: `vtkFieldData::AddArray`.
    pub fn add_array(&mut self, array: AnyArray) -> i32 {
        self.attributes.add_array(array)
    }

    /// VTK: `vtkFieldData::RemoveArray(const char*)`.
    pub fn remove_array(&mut self, name: &str) {
        self.attributes.remove_array(name);
    }

    /// VTK: `vtkDataSetAttributes::RemoveArray(int)`.
    pub fn remove_array_by_index(&mut self, index: i32) {
        self.attributes.remove_array_by_index(index);
    }

    /// VTK: `vtkFieldData::GetNumberOfArrays`.
    pub fn get_number_of_arrays(&self) -> i32 {
        self.attributes.get_number_of_arrays()
    }

    /// VTK: `vtkFieldData::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.attributes.get_number_of_components()
    }

    /// VTK: `vtkFieldData::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> VtkIdType {
        self.attributes.get_number_of_tuples()
    }

    /// VTK: `vtkFieldData::GetArray(const char*)`.
    pub fn get_array(&self, name: &str) -> Option<&AnyArray> {
        self.attributes.get_array(name)
    }

    /// VTK: `vtkFieldData::GetArray(int)`.
    pub fn get_array_by_index(&self, index: i32) -> Option<&AnyArray> {
        self.attributes.get_array_by_index(index)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(const char*)`.
    pub fn get_abstract_array(&self, name: &str) -> Option<&AnyArray> {
        self.attributes.get_abstract_array(name)
    }

    /// VTK: `vtkFieldData::GetAbstractArray(int)`.
    pub fn get_abstract_array_by_index(&self, index: i32) -> Option<&AnyArray> {
        self.attributes.get_abstract_array_by_index(index)
    }

    /// VTK: `vtkFieldData::GetArrayName`.
    pub fn get_array_name(&self, index: i32) -> Option<&str> {
        self.attributes.get_array_name(index)
    }

    /// VTK: `vtkDataSetAttributes::SetScalars`.
    pub fn set_scalars(&mut self, array: Option<AnyArray>) -> i32 {
        self.attributes.set_scalars(array)
    }

    /// VTK: `vtkDataSetAttributes::GetScalars`.
    pub fn get_scalars(&self) -> Option<&AnyArray> {
        self.attributes.get_scalars()
    }

    /// VTK: `vtkDataSetAttributes::SetVectors`.
    pub fn set_vectors(&mut self, array: Option<AnyArray>) -> i32 {
        self.attributes.set_vectors(array)
    }

    /// VTK: `vtkDataSetAttributes::GetVectors`.
    pub fn get_vectors(&self) -> Option<&AnyArray> {
        self.attributes.get_vectors()
    }

    /// VTK: `vtkDataSetAttributes::Initialize`.
    pub fn initialize(&mut self) {
        self.attributes.initialize();
    }

    /// VTK: `vtkDataSetAttributes::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.attributes.deep_copy(&source.attributes);
    }

    /// VTK: `vtkDataSetAttributes::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.attributes.shallow_copy(&source.attributes);
    }

    /// VTK: `vtkDataSetAttributes::CopyAllocate(vtkDataSetAttributes*, vtkIdType, vtkIdType, int)`.
    pub fn copy_allocate(
        &mut self,
        source: &Self,
        size: VtkIdType,
        ext: VtkIdType,
        shallow_copy_arrays: i32,
    ) {
        self.attributes
            .copy_allocate_from(&source.attributes, size, ext, shallow_copy_arrays);
    }

    /// VTK: `vtkDataSetAttributes::InterpolateAllocate(vtkDataSetAttributes*, vtkIdType, vtkIdType, int)`.
    pub fn interpolate_allocate(
        &mut self,
        source: &Self,
        size: VtkIdType,
        ext: VtkIdType,
        shallow_copy_arrays: i32,
    ) {
        self.attributes.interpolate_allocate_from(
            &source.attributes,
            size,
            ext,
            shallow_copy_arrays,
        );
    }

    /// VTK: `vtkDataSetAttributes::SetupForCopy`.
    pub fn setup_for_copy(&mut self, source: &Self) {
        self.attributes.setup_for_copy(&source.attributes);
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdType, vtkIdType)`.
    pub fn copy_data(
        &mut self,
        source: &Self,
        from_id: VtkIdType,
        to_id: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .copy_data_from(&source.attributes, from_id, to_id)
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdList*, vtkIdList*)`.
    pub fn copy_data_from_ids(
        &mut self,
        source: &Self,
        from_ids: &[VtkIdType],
        to_ids: &[VtkIdType],
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .copy_data_from_ids(&source.attributes, from_ids, to_ids)
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdList*, vtkIdType)`.
    pub fn copy_data_from_ids_to_start(
        &mut self,
        source: &Self,
        from_ids: &[VtkIdType],
        dest_start: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .copy_data_from_ids_to_start(&source.attributes, from_ids, dest_start)
    }

    /// VTK: `vtkDataSetAttributes::CopyData(vtkDataSetAttributes*, vtkIdType, vtkIdType, vtkIdType)`.
    pub fn copy_data_range(
        &mut self,
        source: &Self,
        dst_start: VtkIdType,
        n: VtkIdType,
        src_start: VtkIdType,
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .copy_data_range_from(&source.attributes, dst_start, n, src_start)
    }

    /// VTK: `vtkDataSetAttributes::CopyStructuredData`.
    pub fn copy_structured_data(
        &mut self,
        source: &Self,
        in_ext: [i32; 6],
        out_ext: [i32; 6],
        set_size: bool,
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .copy_structured_data(&source.attributes, in_ext, out_ext, set_size)
    }

    /// VTK: `vtkDataSetAttributes::InterpolatePoint`.
    pub fn interpolate_point(
        &mut self,
        source: &Self,
        to_id: VtkIdType,
        pt_ids: &[VtkIdType],
        weights: &[f64],
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .interpolate_point_from(&source.attributes, to_id, pt_ids, weights)
    }

    /// VTK: `vtkDataSetAttributes::InterpolateEdge`.
    pub fn interpolate_edge(
        &mut self,
        source: &Self,
        to_id: VtkIdType,
        p1: VtkIdType,
        p2: VtkIdType,
        t: f64,
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .interpolate_edge(&source.attributes, to_id, p1, p2, t)
    }

    /// VTK: `vtkDataSetAttributes::InterpolateTime`.
    pub fn interpolate_time(
        &mut self,
        source1: &Self,
        source2: &Self,
        id: VtkIdType,
        t: f64,
    ) -> Result<(), DataSetAttributesError> {
        self.attributes
            .interpolate_time(&source1.attributes, &source2.attributes, id, t)
    }
}

impl Default for PointData {
    fn default() -> Self {
        Self::new()
    }
}
