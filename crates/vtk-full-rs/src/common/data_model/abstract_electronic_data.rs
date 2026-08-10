use super::{DataObject, DataObjectType, FieldData, ImageData};
use crate::common::core::{VtkIdType, VtkMTimeType};

/// VTK: `vtkAbstractElectronicData`.
///
/// This stores the translated `vtkDataObject` base state and `Padding` ivar.
/// The pure virtual molecular-orbital API is represented by
/// `AbstractElectronicDataApi`.
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractElectronicData {
    data_object: DataObject,
    padding: f64,
}

impl AbstractElectronicData {
    /// VTK: `vtkAbstractElectronicData::vtkAbstractElectronicData`.
    pub(crate) fn with_type(object_type: DataObjectType) -> Self {
        Self {
            data_object: DataObject::with_type(object_type),
            padding: 0.0,
        }
    }

    /// VTK: `vtkAbstractElectronicData::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Padding: {}\n", self.padding)
    }

    /// VTK: `vtkAbstractElectronicData::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        self.data_object.get_data_object_type()
    }

    /// VTK: `vtkAbstractElectronicData::DeepCopy`.
    pub fn deep_copy(&mut self, obj: &Self) {
        self.data_object.deep_copy(&obj.data_object);
        self.padding = obj.padding;
    }

    /// VTK: `vtkAbstractElectronicData::GetPadding`.
    pub fn get_padding(&self) -> f64 {
        self.padding
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &str {
        self.data_object.get_class_name()
    }

    /// VTK: `vtkDataObject::GetFieldData`.
    pub fn get_field_data(&self) -> &FieldData {
        self.data_object.get_field_data()
    }

    /// VTK: `vtkDataObject::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.data_object.get_actual_memory_size()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.data_object.get_m_time()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.data_object.modified();
    }
}

impl Default for AbstractElectronicData {
    fn default() -> Self {
        Self::with_type(DataObjectType::AbstractElectronicData)
    }
}

/// VTK pure virtual and inline helper API for `vtkAbstractElectronicData`.
pub trait AbstractElectronicDataApi {
    /// VTK: `vtkAbstractElectronicData::GetNumberOfMOs`.
    fn get_number_of_mos(&mut self) -> VtkIdType;

    /// VTK: `vtkAbstractElectronicData::GetNumberOfElectrons`.
    fn get_number_of_electrons(&mut self) -> VtkIdType;

    /// VTK: `vtkAbstractElectronicData::GetMO`.
    fn get_mo(&mut self, orbital_number: VtkIdType) -> *mut ImageData;

    /// VTK: `vtkAbstractElectronicData::GetElectronDensity`.
    fn get_electron_density(&mut self) -> *mut ImageData;

    /// VTK: `vtkAbstractElectronicData::GetHOMO`.
    fn get_homo(&mut self) -> *mut ImageData {
        let orbital_number = self.get_homo_orbital_number();
        self.get_mo(orbital_number)
    }

    /// VTK: `vtkAbstractElectronicData::GetLUMO`.
    fn get_lumo(&mut self) -> *mut ImageData {
        let orbital_number = self.get_lumo_orbital_number();
        self.get_mo(orbital_number)
    }

    /// VTK: `vtkAbstractElectronicData::GetHOMOOrbitalNumber`.
    fn get_homo_orbital_number(&mut self) -> VtkIdType {
        (self.get_number_of_electrons() / 2) - 1
    }

    /// VTK: `vtkAbstractElectronicData::GetLUMOOrbitalNumber`.
    fn get_lumo_orbital_number(&mut self) -> VtkIdType {
        self.get_number_of_electrons() / 2
    }

    /// VTK: `vtkAbstractElectronicData::IsHOMO`.
    fn is_homo(&mut self, orbital_number: VtkIdType) -> bool {
        orbital_number == self.get_homo_orbital_number()
    }

    /// VTK: `vtkAbstractElectronicData::IsLUMO`.
    fn is_lumo(&mut self, orbital_number: VtkIdType) -> bool {
        orbital_number == self.get_lumo_orbital_number()
    }
}
