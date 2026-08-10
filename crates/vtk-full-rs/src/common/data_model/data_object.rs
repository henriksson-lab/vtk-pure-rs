use super::FieldData;
use crate::common::core::{InformationDataObjectKey, ObjectBaseApi, ObjectBaseHandle, VtkIdType};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

static DATA_OBJECT_KEY: OnceLock<usize> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataObjectType {
    DataObject,
    DataSet,
    CartesianGrid,
    PointSet,
    ImageData,
    StructuredPoints,
    UniformGrid,
    RectilinearGrid,
    StructuredGrid,
    Table,
    Graph,
    MultiBlockDataSet,
    PartitionedDataSet,
    PartitionedDataSetCollection,
    Molecule,
    AbstractElectronicData,
}

/// VTK: `vtkDataObject::AttributeTypes::POINT`.
pub const POINT: i32 = 0;
/// VTK: `vtkDataObject::AttributeTypes::CELL`.
pub const CELL: i32 = 1;
/// VTK: `vtkDataObject::AttributeTypes::FIELD`.
pub const FIELD: i32 = 2;
/// VTK: `vtkDataObject::AttributeTypes::POINT_THEN_CELL`.
pub const POINT_THEN_CELL: i32 = 3;
/// VTK: `vtkDataObject::AttributeTypes::VERTEX`.
pub const VERTEX: i32 = 4;
/// VTK: `vtkDataObject::AttributeTypes::EDGE`.
pub const EDGE: i32 = 5;
/// VTK: `vtkDataObject::AttributeTypes::ROW`.
pub const ROW: i32 = 6;
/// VTK: `vtkDataObject::AttributeTypes::NUMBER_OF_ATTRIBUTE_TYPES`.
pub const NUMBER_OF_ATTRIBUTE_TYPES: i32 = 7;

impl DataObjectType {
    fn class_name(self) -> &'static str {
        match self {
            Self::DataObject => "vtkDataObject",
            Self::DataSet => "vtkDataSet",
            Self::CartesianGrid => "vtkCartesianGrid",
            Self::PointSet => "vtkPointSet",
            Self::ImageData => "vtkImageData",
            Self::StructuredPoints => "vtkStructuredPoints",
            Self::UniformGrid => "vtkUniformGrid",
            Self::RectilinearGrid => "vtkRectilinearGrid",
            Self::StructuredGrid => "vtkStructuredGrid",
            Self::Table => "vtkTable",
            Self::Graph => "vtkGraph",
            Self::MultiBlockDataSet => "vtkMultiBlockDataSet",
            Self::PartitionedDataSet => "vtkPartitionedDataSet",
            Self::PartitionedDataSetCollection => "vtkPartitionedDataSetCollection",
            Self::Molecule => "vtkMolecule",
            Self::AbstractElectronicData => "vtkAbstractElectronicData",
        }
    }

    fn type_id(self) -> i32 {
        match self {
            Self::DataObject => 7,
            Self::DataSet => 8,
            Self::CartesianGrid => 51,
            Self::PointSet => 9,
            Self::ImageData => 6,
            Self::StructuredPoints => 1,
            Self::UniformGrid => 10,
            Self::RectilinearGrid => 3,
            Self::StructuredGrid => 2,
            Self::Table => 19,
            Self::Graph => 20,
            Self::MultiBlockDataSet => 13,
            Self::PartitionedDataSet => 37,
            Self::PartitionedDataSetCollection => 38,
            Self::Molecule => 33,
            Self::AbstractElectronicData => 42,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct DataObjectStorage {
    field_data: FieldData,
    data_released: bool,
    modified_time: u64,
}

/// VTK-shaped base for `vtkDataObject`.
///
/// This stores only the data payload shared by concrete data objects. Pipeline
/// `vtkInformation` keys and executive state are intentionally deferred.
#[derive(Debug)]
pub struct DataObject {
    storage: Arc<DataObjectStorage>,
    object_type: DataObjectType,
    class_name: Arc<str>,
}

/// Shallow-copyable handle for `vtkDataObject*` storage.
#[derive(Clone)]
pub struct DataObjectHandle {
    object: Rc<RefCell<DataObject>>,
}

impl DataObjectHandle {
    pub fn new(object: DataObject) -> Self {
        Self {
            object: Rc::new(RefCell::new(object)),
        }
    }

    pub fn from_rc(object: Rc<RefCell<DataObject>>) -> Self {
        Self { object }
    }

    pub fn as_object_base_handle(&self) -> ObjectBaseHandle {
        ObjectBaseHandle::from_rc(Rc::clone(&self.object))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.object, &other.object)
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, DataObject> {
        self.object.borrow()
    }

    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, DataObject> {
        self.object.borrow_mut()
    }
}

impl std::fmt::Debug for DataObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataObjectHandle")
            .field("class_name", &self.borrow().get_class_name())
            .finish_non_exhaustive()
    }
}

impl Clone for DataObject {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            object_type: self.object_type,
            class_name: Arc::clone(&self.class_name),
        }
    }
}

impl PartialEq for DataObject {
    fn eq(&self, other: &Self) -> bool {
        self.object_type == other.object_type
            && self.class_name == other.class_name
            && (Arc::ptr_eq(&self.storage, &other.storage) || self.storage == other.storage)
    }
}

impl DataObject {
    pub fn new() -> Self {
        Self::with_type(DataObjectType::DataObject)
    }

    /// VTK: `vtkDataObject::DATA_OBJECT`.
    pub fn data_object() -> &'static InformationDataObjectKey {
        let key = *DATA_OBJECT_KEY.get_or_init(|| {
            InformationDataObjectKey::make_key(Some("DATA_OBJECT"), Some("vtkDataObject")) as usize
        });
        unsafe { &*(key as *const InformationDataObjectKey) }
    }

    pub(crate) fn with_type(object_type: DataObjectType) -> Self {
        Self {
            storage: Arc::new(DataObjectStorage {
                field_data: FieldData::new(),
                data_released: false,
                modified_time: 0,
            }),
            object_type,
            class_name: Arc::from(object_type.class_name()),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_class_name(class_name: impl Into<String>) -> Self {
        Self {
            storage: Arc::new(DataObjectStorage {
                field_data: FieldData::new(),
                data_released: false,
                modified_time: 0,
            }),
            object_type: DataObjectType::DataObject,
            class_name: Arc::from(class_name.into()),
        }
    }

    fn storage_mut(&mut self) -> &mut DataObjectStorage {
        Arc::make_mut(&mut self.storage)
    }

    /// VTK: `vtkDataObject::GetDataObjectType`.
    pub fn get_data_object_type(&self) -> i32 {
        self.object_type.type_id()
    }

    pub(crate) fn data_object_type(&self) -> DataObjectType {
        self.object_type
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &str {
        &self.class_name
    }

    /// VTK: `vtkDataObject::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkDataObject"
    }

    /// VTK: `vtkDataObject::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        self.get_class_name() == name || Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        format!("{}({:p})", self.get_class_name(), self)
    }

    /// VTK: `vtkDataObject::GetFieldData`.
    pub fn get_field_data(&self) -> &FieldData {
        &self.storage.field_data
    }

    pub(crate) fn get_field_data_mut(&mut self) -> &mut FieldData {
        self.modified();
        &mut self.storage_mut().field_data
    }

    /// VTK: `vtkDataObject::SetFieldData`.
    pub fn set_field_data(&mut self, field_data: FieldData) {
        let storage = self.storage_mut();
        storage.field_data = field_data;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataObject::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.field_data.initialize();
        storage.data_released = false;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataObject::ReleaseData`.
    pub fn release_data(&mut self) {
        self.initialize();
        self.storage_mut().data_released = true;
    }

    /// VTK: `vtkDataObject::DataHasBeenGenerated`.
    pub fn data_has_been_generated(&mut self) {
        let storage = self.storage_mut();
        storage.data_released = false;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkDataObject::GetDataReleased`.
    pub fn get_data_released(&self) -> bool {
        self.storage.data_released
    }

    /// VTK: `vtkDataObject::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.field_data.get_actual_memory_size()
    }

    /// VTK: `vtkObject::GetMTime` with field data included.
    pub fn get_m_time(&self) -> u64 {
        self.storage
            .modified_time
            .max(self.storage.field_data.get_m_time())
    }

    /// VTK: `vtkDataObject::GetAttributesAsFieldData`.
    pub fn get_attributes_as_field_data(&self, attribute_type: i32) -> Option<&FieldData> {
        match attribute_type {
            FIELD => Some(self.get_field_data()),
            _ => None,
        }
    }

    /// VTK: `vtkDataObject::GetNumberOfElements`.
    pub fn get_number_of_elements(&self, attribute_type: i32) -> VtkIdType {
        match attribute_type {
            FIELD => self.get_field_data().get_number_of_tuples(),
            _ => 0,
        }
    }

    pub fn modified(&mut self) {
        let next = self.storage.modified_time.saturating_add(1);
        self.storage_mut().modified_time = next;
    }

    /// VTK: `vtkDataObject::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Arc::new(DataObjectStorage {
            field_data: source.storage.field_data.shallow_clone(),
            data_released: source.storage.data_released,
            modified_time: self.storage.modified_time.saturating_add(1),
        });
    }

    /// VTK: `vtkDataObject::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.storage = Arc::new(DataObjectStorage {
            field_data: source.storage.field_data.deep_clone(),
            data_released: source.storage.data_released,
            modified_time: self.storage.modified_time.saturating_add(1),
        });
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        let mut output = Self::with_type(self.object_type);
        output.shallow_copy(self);
        output
    }

    /// VTK: `vtkDataObject::NewInstance`.
    pub fn new_instance(&self) -> Self {
        Self::with_type(self.object_type)
    }

    pub(crate) fn deep_clone(&self) -> Self {
        let mut output = Self::with_type(self.object_type);
        output.deep_copy(self);
        output
    }
}

impl ObjectBaseApi for DataObject {
    fn get_class_name(&self) -> &str {
        self.get_class_name()
    }

    fn is_a(&self, name: &str) -> bool {
        self.is_a(name)
    }

    fn get_object_description(&self) -> String {
        self.get_object_description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::data_model::{FieldDataArray, Variant};

    #[test]
    fn shallow_and_deep_copy_field_data_like_vtk_data_object() {
        let mut source = DataObject::new();
        source
            .get_field_data_mut()
            .add_field_data_array(FieldDataArray::from_i64("ids", 1, vec![1]));

        let mut shallow = DataObject::new();
        let mut deep = DataObject::new();
        shallow.shallow_copy(&source);
        deep.deep_copy(&source);

        shallow
            .get_field_data_mut()
            .get_array_mut("ids")
            .unwrap()
            .set_value(0, Variant::I64(2));
        deep.get_field_data_mut()
            .get_array_mut("ids")
            .unwrap()
            .set_value(0, Variant::I64(3));

        assert_eq!(
            source
                .get_field_data()
                .get_field_data_array("ids")
                .unwrap()
                .values_as_variants(),
            vec![Variant::I64(1)]
        );
        assert_eq!(
            shallow
                .get_field_data()
                .get_field_data_array("ids")
                .unwrap()
                .values_as_variants(),
            vec![Variant::I64(2)]
        );
        assert_eq!(
            deep.get_field_data()
                .get_field_data_array("ids")
                .unwrap()
                .values_as_variants(),
            vec![Variant::I64(3)]
        );
    }
}
