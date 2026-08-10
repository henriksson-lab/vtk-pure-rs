use crate::common::{
    core::{AnyArray, VtkDataType, VtkIdType},
    data_model::BoundingBox,
};
use std::sync::Arc;

/// VTK scalar type id for single precision coordinates.
///
/// VTK origin: `VTK_FLOAT`, the default backing array type for `vtkPoints`.
pub const VTK_FLOAT: i32 = 10;

/// VTK scalar type id for double precision coordinates.
///
/// VTK origin: `VTK_DOUBLE`, returned by `vtkPoints::GetDataType` when backed
/// by a double array.
pub const VTK_DOUBLE: i32 = 11;

/// Storage for 3D point coordinates.
///
/// VTK origin: selected audited symbols from `VTK/Common/Core/vtkPoints.cxx`
/// and `VTK/Common/Core/vtkPoints.h`.
#[derive(Debug, Clone)]
pub struct Points {
    storage: Arc<PointsStorage>,
}

#[derive(Debug, Clone)]
struct PointsStorage {
    data: AnyArray,
    modified_time: u64,
    bounds: BoundingBox,
    compute_time: u64,
}

impl PartialEq for Points {
    fn eq(&self, other: &Self) -> bool {
        self.storage.data == other.storage.data
    }
}

impl Points {
    pub fn new() -> Self {
        Self::with_data_type_and_capacity(VTK_FLOAT, 0)
    }

    /// VTK: `vtkPoints::New(int)`.
    pub fn new_with_data_type(data_type: i32) -> Self {
        Self::with_data_type_and_capacity(
            VtkDataType::from_id(data_type)
                .filter(|data_type| data_type.is_numeric())
                .map_or(VTK_FLOAT, VtkDataType::id),
            0,
        )
    }

    fn storage_mut(&mut self) -> &mut PointsStorage {
        Arc::make_mut(&mut self.storage)
    }

    /// VTK: `vtkPoints::InsertNextPoint`.
    pub fn insert_next_point(&mut self, point: [f64; 3]) -> VtkIdType {
        let id = self.get_number_of_points();
        self.insert_point(id, point);
        id
    }

    /// VTK: `vtkPoints::GetPoint`.
    pub fn get_point(&self, idx: VtkIdType) -> [f64; 3] {
        tuple_to_point(
            self.storage
                .data
                .numeric_tuple_as_f64_checked(point_id_to_index(idx))
                .expect("vtkPoints backing array must be numeric"),
        )
    }

    /// VTK: `vtkPoints::SetPoint`.
    pub fn set_point(&mut self, idx: VtkIdType, point: [f64; 3]) {
        let storage = self.storage_mut();
        storage
            .data
            .insert_numeric_tuple_from_f64_checked(point_id_to_index(idx), &point)
            .expect("vtkPoints backing array must be numeric");
    }

    /// VTK: `vtkPoints::InsertPoint`.
    pub fn insert_point(&mut self, idx: VtkIdType, point: [f64; 3]) {
        self.set_point(idx, point);
        self.modified();
    }

    /// VTK: `vtkPoints::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.storage.data.get_number_of_tuples() as VtkIdType
    }

    /// VTK: `vtkPoints::GetData`.
    pub fn get_data(&self) -> &AnyArray {
        &self.storage.data
    }

    /// VTK: `vtkPoints::GetData`.
    pub fn get_data_mut(&mut self) -> &mut AnyArray {
        &mut self.storage_mut().data
    }

    /// VTK: `vtkPoints::SetData`.
    pub fn set_data(&mut self, data: &AnyArray) {
        if !data.is_numeric() || data.get_number_of_components() != 3 {
            return;
        }
        let mut data = data.shallow_clone();
        if data.get_name().is_empty() {
            data.set_name("Points");
        }
        let storage = self.storage_mut();
        storage.data = data;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPoints::Allocate`.
    pub fn allocate(&mut self, size: VtkIdType, ext: VtkIdType) -> bool {
        if size < 0 || ext < 0 {
            return false;
        }
        if size > self.storage.data.get_number_of_tuples() {
            let storage = self.storage_mut();
            storage.data.reserve_tuples(size.saturating_add(ext));
            storage.modified_time = storage.modified_time.saturating_add(1);
        }
        true
    }

    /// VTK: `vtkPoints::Reserve`.
    pub fn reserve(&mut self, capacity: VtkIdType) -> bool {
        if capacity < 0 {
            return false;
        }
        let storage = self.storage_mut();
        storage.data.reserve_tuples(capacity);
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    /// VTK: `vtkPoints::Resize`.
    pub fn resize(&mut self, num_points: VtkIdType) -> bool {
        if num_points < 0 {
            return false;
        }
        let num_points = num_points as usize;
        let storage = self.storage_mut();
        storage.data.set_number_of_tuples(num_points as VtkIdType);
        storage.modified_time = storage.modified_time.saturating_add(1);
        true
    }

    /// VTK: `vtkPoints::SetNumberOfPoints`.
    pub fn set_number_of_points(&mut self, num_points: VtkIdType) {
        self.resize(num_points);
    }

    /// VTK: `vtkPoints::Initialize`.
    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.data.initialize();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPoints::Reset`.
    pub fn reset(&mut self) {
        let storage = self.storage_mut();
        storage.data.reset();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPoints::Squeeze`.
    pub fn squeeze(&mut self) {
        let storage = self.storage_mut();
        storage.data.squeeze();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPoints::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        let modified_time = self.storage.modified_time.saturating_add(1);
        self.storage = Arc::new(PointsStorage {
            data: other.storage.data.deep_clone(),
            modified_time,
            bounds: BoundingBox::empty(),
            compute_time: 0,
        });
    }

    /// VTK: `vtkPoints::ShallowCopy`.
    pub fn shallow_copy(&mut self, other: &Self) {
        self.set_data(other.get_data());
    }

    /// VTK: `vtkPoints::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.data.get_actual_memory_size()
    }

    /// VTK: `vtkPoints::GetDataType`.
    pub fn get_data_type(&self) -> i32 {
        self.storage.data.get_data_type().id()
    }

    /// VTK: `vtkPoints::SetDataType`.
    pub fn set_data_type(&mut self, data_type: i32) {
        if !is_supported_point_data_type(data_type) {
            return;
        }
        if data_type == self.get_data_type() {
            return;
        }

        let storage = self.storage_mut();
        storage.data = point_array_for_data_type(data_type, 0).expect("supported point data type");
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPoints::SetDataTypeToBit`.
    pub fn set_data_type_to_bit(&mut self) {
        self.set_data_type(VtkDataType::VTK_BIT);
    }

    /// VTK: `vtkPoints::SetDataTypeToChar`.
    pub fn set_data_type_to_char(&mut self) {
        self.set_data_type(VtkDataType::VTK_CHAR);
    }

    /// VTK: `vtkPoints::SetDataTypeToUnsignedChar`.
    pub fn set_data_type_to_unsigned_char(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_CHAR);
    }

    /// VTK: `vtkPoints::SetDataTypeToShort`.
    pub fn set_data_type_to_short(&mut self) {
        self.set_data_type(VtkDataType::VTK_SHORT);
    }

    /// VTK: `vtkPoints::SetDataTypeToUnsignedShort`.
    pub fn set_data_type_to_unsigned_short(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_SHORT);
    }

    /// VTK: `vtkPoints::SetDataTypeToInt`.
    pub fn set_data_type_to_int(&mut self) {
        self.set_data_type(VtkDataType::VTK_INT);
    }

    /// VTK: `vtkPoints::SetDataTypeToUnsignedInt`.
    pub fn set_data_type_to_unsigned_int(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_INT);
    }

    /// VTK: `vtkPoints::SetDataTypeToLong`.
    pub fn set_data_type_to_long(&mut self) {
        self.set_data_type(VtkDataType::VTK_LONG);
    }

    /// VTK: `vtkPoints::SetDataTypeToUnsignedLong`.
    pub fn set_data_type_to_unsigned_long(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_LONG);
    }

    /// VTK: `vtkPoints::SetDataTypeToFloat`.
    pub fn set_data_type_to_float(&mut self) {
        self.set_data_type(VtkDataType::VTK_FLOAT);
    }

    /// VTK: `vtkPoints::SetDataTypeToDouble`.
    pub fn set_data_type_to_double(&mut self) {
        self.set_data_type(VtkDataType::VTK_DOUBLE);
    }

    /// VTK: `vtkPoints::Modified`.
    pub fn modified(&mut self) {
        let storage = self.storage_mut();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkPoints::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.storage
            .modified_time
            .max(self.storage.data.get_m_time())
    }

    /// VTK: `vtkPoints::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        if self.storage.compute_time >= self.storage.modified_time {
            self.storage.bounds.get_bounds()
        } else {
            compute_bounds_from_data(&self.storage.data).get_bounds()
        }
    }

    /// VTK: `vtkPoints::ComputeBounds`.
    pub fn compute_bounds(&mut self) {
        let bounds = compute_bounds_from_data(&self.storage.data);
        let storage = self.storage_mut();
        storage.bounds = bounds;
        storage.compute_time = storage.modified_time;
    }

    /// VTK: `vtkPoints::GetPoints(vtkIdList*, vtkPoints*)`.
    pub fn get_points(&self, point_ids: &[VtkIdType], out_points: &mut Self) {
        out_points.set_number_of_points(point_ids.len() as VtkIdType);
        for (out_id, &point_id) in point_ids.iter().enumerate() {
            out_points.set_point(out_id as VtkIdType, self.get_point(point_id));
        }
    }

    /// VTK: `vtkPoints::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "vtkPoints {{ number_of_points: {}, data_type: {}, actual_memory_size_kib: {}, m_time: {} }}",
            self.get_number_of_points(),
            self.get_data_type(),
            self.get_actual_memory_size(),
            self.get_m_time()
        )
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage)
    }

    fn with_data_type_and_capacity(data_type: i32, capacity: usize) -> Self {
        Self {
            storage: Arc::new(PointsStorage {
                data: point_array_for_data_type(data_type, capacity)
                    .expect("supported vtkPoints data type"),
                modified_time: 0,
                bounds: BoundingBox::empty(),
                compute_time: 0,
            }),
        }
    }
}

fn is_supported_point_data_type(data_type: i32) -> bool {
    VtkDataType::from_id(data_type).is_some_and(|data_type| data_type.is_numeric())
}

fn point_array_for_data_type(data_type: i32, capacity: usize) -> Option<AnyArray> {
    let data_type = VtkDataType::from_id(data_type)?;
    if !data_type.is_numeric() {
        return None;
    }
    let mut data = AnyArray::create_array(data_type)?;
    data.set_name("Points");
    data.set_number_of_components(3);
    data.reserve_tuples(capacity as VtkIdType);
    Some(data)
}

fn tuple_to_point(tuple: Vec<f64>) -> [f64; 3] {
    [tuple[0], tuple[1], tuple[2]]
}

fn point_id_to_index(point_id: VtkIdType) -> usize {
    usize::try_from(point_id).expect("vtkPoints id must be non-negative")
}

fn compute_bounds_from_data(data: &AnyArray) -> BoundingBox {
    let mut bounds = BoundingBox::empty();
    for idx in 0..data.get_number_of_tuples() {
        let point = tuple_to_point(
            data.numeric_tuple_as_f64_checked(idx as usize)
                .expect("vtkPoints backing array must be numeric"),
        );
        bounds.add_point(point);
    }
    bounds
}
