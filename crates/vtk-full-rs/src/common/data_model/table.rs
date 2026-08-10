use crate::common::core::{AnyArray, VtkIdType};

use super::{DataObject, DataObjectType, DataSetAttributes, FieldData, FieldDataArray, ROW};
use std::sync::Arc;

/// Shared storage for table row data.
#[derive(Debug, Clone, PartialEq)]
struct TableStorage {
    data_object: DataObject,
    row_data: DataSetAttributes,
}

/// Columnar data table backed by row data arrays.
///
/// VTK origin: `VTK/Common/DataModel/vtkTable.{h,cxx}`.
///
/// VTK stores `vtkTable` columns in its `RowData` (`vtkDataSetAttributes`).
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    storage: Arc<TableStorage>,
}

impl Table {
    /// VTK: `vtkTable::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(TableStorage {
                data_object: DataObject::with_type(DataObjectType::Table),
                row_data: DataSetAttributes::new(),
            }),
        }
    }

    /// VTK: `vtkTable::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.data_object.initialize();
        storage.row_data.initialize();
    }

    pub fn get_field_data(&self) -> &FieldData {
        self.storage.data_object.get_field_data()
    }

    /// VTK: `vtkTable::GetRowData`.
    pub fn get_row_data(&self) -> &DataSetAttributes {
        &self.storage.row_data
    }

    /// VTK: `vtkTable::SetRowData`.
    pub fn set_row_data(&mut self, row_data: DataSetAttributes) {
        self.storage_mut().row_data = row_data;
    }

    /// VTK: `vtkTable::GetNumberOfColumns`.
    pub fn get_number_of_columns(&self) -> VtkIdType {
        VtkIdType::from(self.storage.row_data.get_number_of_arrays())
    }

    /// VTK: `vtkTable::GetNumberOfRows`.
    ///
    /// Like VTK, the table row count is derived from column zero.
    pub fn get_number_of_rows(&self) -> VtkIdType {
        self.get_field_data_column(0)
            .map_or(0, |column| column.get_number_of_tuples() as VtkIdType)
    }

    /// VTK: `vtkTable::GetAttributesAsFieldData`.
    pub fn get_attributes_as_field_data(&self, attribute_type: i32) -> Option<&FieldData> {
        match attribute_type {
            ROW => Some(self.get_row_data().field_data()),
            _ => self
                .storage
                .data_object
                .get_attributes_as_field_data(attribute_type),
        }
    }

    /// VTK: `vtkTable::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            ROW => self.get_number_of_rows().max(0),
            _ => self
                .storage
                .data_object
                .get_number_of_elements(attribute_type),
        }
    }

    /// VTK: `vtkTable::SetNumberOfRows`.
    ///
    /// VTK delegates this to `RowData->SetNumberOfTuples(n)`.
    pub fn set_number_of_rows(&mut self, rows: VtkIdType) {
        self.storage_mut().row_data.set_number_of_tuples(rows);
    }

    /// VTK: `vtkTable::SqueezeRows`.
    pub fn squeeze_rows(&mut self) {
        self.storage_mut().row_data.squeeze();
    }

    /// VTK: `vtkTable::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.data_object.get_actual_memory_size()
            + self.storage.row_data.get_actual_memory_size()
    }

    /// VTK: `vtkTable::AddColumn`.
    ///
    /// VTK reports an error and leaves the table unchanged when a non-empty
    /// table receives a column whose tuple count does not match the row count.
    pub fn add_column(&mut self, column: AnyArray) {
        self.add_field_data_column(FieldDataArray::from_any_array(column))
    }

    pub(crate) fn add_field_data_column(&mut self, column: FieldDataArray) {
        if self.get_number_of_columns() > 0
            && column.get_number_of_tuples() as VtkIdType != self.get_number_of_rows()
        {
            return;
        }
        self.storage_mut().row_data.add_field_data_array(column);
    }

    /// VTK: `vtkTable::RemoveColumn(vtkIdType)`.
    pub fn remove_column(&mut self, column: VtkIdType) {
        self.remove_field_data_column(column);
    }

    pub(crate) fn remove_field_data_column(&mut self, column: VtkIdType) -> Option<FieldDataArray> {
        let Some(column) = vtk_id_to_index(column) else {
            return None;
        };
        self.storage_mut()
            .row_data
            .remove_field_data_array_by_index(column)
    }

    /// VTK: `vtkTable::RemoveColumnByName`.
    pub fn remove_column_by_name(&mut self, name: &str) {
        self.remove_field_data_column_by_name(name);
    }

    pub(crate) fn remove_field_data_column_by_name(
        &mut self,
        name: &str,
    ) -> Option<FieldDataArray> {
        self.storage_mut().row_data.remove_field_data_array(name)
    }

    /// VTK: `vtkTable::RemoveAllColumns`.
    pub fn remove_all_columns(&mut self) {
        while self
            .storage_mut()
            .row_data
            .remove_field_data_array_by_index(0)
            .is_some()
        {}
    }

    /// VTK: `vtkTable::GetColumn(vtkIdType)`.
    pub fn get_column(&self, column: VtkIdType) -> Option<&AnyArray> {
        self.get_field_data_column(column)
            .map(FieldDataArray::get_data)
    }

    pub(crate) fn get_field_data_column(&self, column: VtkIdType) -> Option<&FieldDataArray> {
        let column = vtk_id_to_index(column)?;
        self.storage.row_data.get_field_data_array_by_index(column)
    }

    /// VTK: `vtkTable::GetColumnByName`.
    pub fn get_column_by_name(&self, name: &str) -> Option<&AnyArray> {
        self.get_field_data_column_by_name(name)
            .map(FieldDataArray::get_data)
    }

    pub(crate) fn get_field_data_column_by_name(&self, name: &str) -> Option<&FieldDataArray> {
        self.storage.row_data.get_field_data_array(name)
    }

    /// VTK: `vtkTable::GetColumnName`.
    pub fn get_column_name(&self, column: VtkIdType) -> Option<&str> {
        let column = i32::try_from(column).ok()?;
        self.storage.row_data.field_data().get_array_name(column)
    }

    /// VTK: `vtkTable::GetColumnIndex`.
    pub fn get_column_index(&self, name: &str) -> VtkIdType {
        self.storage
            .row_data
            .field_data()
            .arrays()
            .iter()
            .position(|array| array.get_name() == name)
            .map_or(-1, |index| index as VtkIdType)
    }

    /// VTK: `vtkTable::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        let data_object = source.storage.data_object.deep_clone();
        let mut row_data = DataSetAttributes::new();
        row_data.deep_copy(&source.storage.row_data);
        self.storage = Arc::new(TableStorage {
            data_object,
            row_data,
        });
    }

    /// VTK: `vtkTable::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Arc::new(TableStorage {
            data_object: source.storage.data_object.shallow_clone(),
            row_data: source.storage.row_data.shallow_clone(),
        });
    }

    fn storage_mut(&mut self) -> &mut TableStorage {
        Arc::make_mut(&mut self.storage)
    }
}

fn vtk_id_to_index(id: VtkIdType) -> Option<usize> {
    usize::try_from(id).ok()
}
