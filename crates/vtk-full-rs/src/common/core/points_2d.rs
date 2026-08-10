use std::sync::Arc;

use super::{
    any_array::AnyArray,
    object::Object,
    time_stamp::TimeStamp,
    vtk_type::{VtkDataType, VtkIdType, VtkMTimeType},
};

const VTK_DOUBLE_MAX: f64 = 1.0e299;

/// VTK: `vtkPoints2D`.
#[derive(Debug, Clone, PartialEq)]
pub struct Points2D {
    object: Object,
    storage: Arc<Points2DStorage>,
}

#[derive(Debug, Clone, PartialEq)]
struct Points2DStorage {
    data: AnyArray,
    bounds: [f64; 4],
    compute_time: TimeStamp,
}

impl Points2D {
    /// VTK: `vtkPoints2D::New`.
    pub fn new() -> Self {
        Self::new_with_data_type(VtkDataType::VTK_FLOAT)
    }

    /// VTK: `vtkPoints2D::New(int)`.
    pub fn new_with_data_type(data_type: i32) -> Self {
        let mut points = Self {
            object: Object::with_class_name("vtkPoints2D"),
            storage: Arc::new(Points2DStorage {
                data: points_2d_array_for_data_type(VtkDataType::VTK_FLOAT)
                    .expect("VTK_FLOAT must be supported"),
                bounds: empty_bounds_2d(),
                compute_time: TimeStamp::new(),
            }),
        };
        points.set_data_type(data_type);
        points
    }

    fn storage_mut(&mut self) -> &mut Points2DStorage {
        Arc::make_mut(&mut self.storage)
    }

    /// VTK: `vtkPoints2D::Allocate`.
    pub fn allocate(&mut self, size: VtkIdType, ext: VtkIdType) -> bool {
        let number_of_components = self.storage.data.get_number_of_components() as VtkIdType;
        let values = size.saturating_mul(number_of_components);
        let extra = ext.saturating_mul(number_of_components);
        let storage = self.storage_mut();
        storage.data.initialize();
        let ok = storage.data.reserve_values(values.saturating_add(extra));
        self.modified();
        ok
    }

    /// VTK: `vtkPoints2D::Initialize`.
    pub fn initialize(&mut self) {
        self.storage_mut().data.initialize();
        self.modified();
    }

    /// VTK: `vtkPoints2D::SetData`.
    pub fn set_data(&mut self, data: &AnyArray) {
        if !data.is_numeric()
            || data.get_number_of_components() != self.storage.data.get_number_of_components()
        {
            return;
        }
        let mut data = data.shallow_clone();
        if data.get_name().is_empty() {
            data.set_name("Points2D");
        }
        self.storage_mut().data = data;
        self.modified();
    }

    /// VTK: `vtkPoints2D::GetData`.
    pub fn get_data(&self) -> &AnyArray {
        &self.storage.data
    }

    /// VTK: `vtkPoints2D::GetDataType`.
    pub fn get_data_type(&self) -> i32 {
        self.storage.data.get_data_type().id()
    }

    /// VTK: `vtkPoints2D::SetDataType`.
    pub fn set_data_type(&mut self, data_type: i32) {
        if data_type == self.get_data_type() {
            return;
        }
        let Some(data) = points_2d_array_for_data_type(data_type) else {
            return;
        };
        self.storage_mut().data = data;
        self.modified();
    }

    /// VTK: `vtkPoints2D::SetDataTypeToBit`.
    pub fn set_data_type_to_bit(&mut self) {
        self.set_data_type(VtkDataType::VTK_BIT);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToChar`.
    pub fn set_data_type_to_char(&mut self) {
        self.set_data_type(VtkDataType::VTK_CHAR);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToUnsignedChar`.
    pub fn set_data_type_to_unsigned_char(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_CHAR);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToShort`.
    pub fn set_data_type_to_short(&mut self) {
        self.set_data_type(VtkDataType::VTK_SHORT);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToUnsignedShort`.
    pub fn set_data_type_to_unsigned_short(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_SHORT);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToInt`.
    pub fn set_data_type_to_int(&mut self) {
        self.set_data_type(VtkDataType::VTK_INT);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToUnsignedInt`.
    pub fn set_data_type_to_unsigned_int(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_INT);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToLong`.
    pub fn set_data_type_to_long(&mut self) {
        self.set_data_type(VtkDataType::VTK_LONG);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToUnsignedLong`.
    pub fn set_data_type_to_unsigned_long(&mut self) {
        self.set_data_type(VtkDataType::VTK_UNSIGNED_LONG);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToFloat`.
    pub fn set_data_type_to_float(&mut self) {
        self.set_data_type(VtkDataType::VTK_FLOAT);
    }

    /// VTK: `vtkPoints2D::SetDataTypeToDouble`.
    pub fn set_data_type_to_double(&mut self) {
        self.set_data_type(VtkDataType::VTK_DOUBLE);
    }

    /// VTK: `vtkPoints2D::Squeeze`.
    pub fn squeeze(&mut self) {
        self.storage_mut().data.squeeze();
    }

    /// VTK: `vtkPoints2D::Reset`.
    pub fn reset(&mut self) {
        self.storage_mut().data.reset();
        self.modified();
    }

    /// VTK: `vtkPoints2D::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Self) {
        if other.storage.data.get_number_of_components()
            != self.storage.data.get_number_of_components()
        {
            return;
        }
        let data = other.storage.data.deep_clone();
        self.storage_mut().data = data;
        self.modified();
    }

    /// VTK: `vtkPoints2D::ShallowCopy`.
    pub fn shallow_copy(&mut self, other: &Self) {
        self.set_data(other.get_data());
    }

    /// VTK: `vtkPoints2D::GetActualMemorySize`.
    pub fn get_actual_memory_size(&self) -> usize {
        self.storage.data.get_actual_memory_size()
    }

    /// VTK: `vtkPoints2D::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.storage.data.get_number_of_tuples()
    }

    /// VTK: `vtkPoints2D::GetPoint`.
    pub fn get_point(&self, id: VtkIdType) -> [f64; 2] {
        let tuple = self
            .storage
            .data
            .numeric_tuple_as_f64_checked(vtk_id_to_usize(id))
            .expect("vtkPoints2D backing array must be numeric");
        [tuple[0], tuple[1]]
    }

    /// VTK: `vtkPoints2D::GetPoint(id, x)`.
    pub fn get_point_into(&self, id: VtkIdType, x: &mut [f64; 2]) {
        *x = self.get_point(id);
    }

    /// VTK: `vtkPoints2D::SetPoint`.
    pub fn set_point(&mut self, id: VtkIdType, point: [f64; 2]) {
        let components = self.storage.data.get_number_of_components().max(2) as usize;
        let mut tuple = vec![0.0; components];
        tuple[0] = point[0];
        tuple[1] = point[1];
        self.storage_mut()
            .data
            .insert_numeric_tuple_from_f64_checked(vtk_id_to_usize(id), &tuple)
            .expect("vtkPoints2D backing array must be numeric");
    }

    /// VTK: `vtkPoints2D::SetPoint(id, x, y)`.
    pub fn set_point_xy(&mut self, id: VtkIdType, x: f64, y: f64) {
        self.set_point(id, [x, y]);
    }

    /// VTK: `vtkPoints2D::InsertPoint`.
    pub fn insert_point(&mut self, id: VtkIdType, point: [f64; 2]) {
        self.set_point(id, point);
    }

    /// VTK: `vtkPoints2D::InsertPoint(id, x, y)`.
    pub fn insert_point_xy(&mut self, id: VtkIdType, x: f64, y: f64) {
        self.insert_point(id, [x, y]);
    }

    /// VTK: `vtkPoints2D::InsertNextPoint`.
    pub fn insert_next_point(&mut self, point: [f64; 2]) -> VtkIdType {
        let id = self.get_number_of_points();
        self.insert_point(id, point);
        id
    }

    /// VTK: `vtkPoints2D::InsertNextPoint(x, y)`.
    pub fn insert_next_point_xy(&mut self, x: f64, y: f64) -> VtkIdType {
        self.insert_next_point([x, y])
    }

    /// VTK: `vtkPoints2D::RemovePoint`.
    pub fn remove_point(&mut self, id: VtkIdType) {
        self.storage_mut().data.remove_tuple(id);
    }

    /// VTK: `vtkPoints2D::SetNumberOfPoints`.
    pub fn set_number_of_points(&mut self, num_points: VtkIdType) {
        let storage = self.storage_mut();
        storage.data.set_number_of_components(2);
        storage.data.set_number_of_tuples(num_points);
        self.modified();
    }

    /// VTK: `vtkPoints2D::Resize`.
    pub fn resize(&mut self, num_points: VtkIdType) -> bool {
        if num_points != self.storage.data.get_number_of_tuples() {
            let storage = self.storage_mut();
            storage.data.set_number_of_components(3);
            storage.data.set_number_of_tuples(num_points);
            self.modified();
        }
        true
    }

    /// VTK: `vtkPoints2D::Reserve`.
    pub fn reserve(&mut self, num_points: VtkIdType) -> bool {
        if num_points != self.storage.data.get_number_of_tuples() {
            let storage = self.storage_mut();
            storage.data.set_number_of_components(3);
            let ok = storage.data.reserve_tuples(num_points);
            self.modified();
            return ok;
        }
        true
    }

    /// VTK: `vtkPoints2D::GetPoints`.
    pub fn get_points(&self, point_ids: &[VtkIdType], out_points: &mut Self) {
        for (i, point_id) in point_ids.iter().copied().enumerate() {
            out_points.insert_point(i as VtkIdType, self.get_point(point_id));
        }
    }

    /// VTK: `vtkPoints2D::ComputeBounds`.
    pub fn compute_bounds(&mut self) {
        if self.get_m_time() > self.storage.compute_time.get_m_time() {
            let mut bounds = empty_bounds_2d();
            for i in 0..self.get_number_of_points() {
                let point = self.get_point(i);
                for j in 0..2 {
                    bounds[2 * j] = bounds[2 * j].min(point[j]);
                    bounds[2 * j + 1] = bounds[2 * j + 1].max(point[j]);
                }
            }
            let storage = self.storage_mut();
            storage.bounds = bounds;
            storage.compute_time.modified();
        }
    }

    /// VTK: `vtkPoints2D::GetBounds`.
    pub fn get_bounds(&mut self) -> [f64; 4] {
        self.compute_bounds();
        self.storage.bounds
    }

    /// VTK: `vtkPoints2D::GetBounds(bounds)`.
    pub fn get_bounds_into(&mut self, bounds: &mut [f64; 4]) {
        *bounds = self.get_bounds();
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkPoints2D::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkPoints2D" || Object::is_type_of(name)
    }

    /// VTK: `vtkPoints2D::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkPoints2D::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkPoints2D" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkPoints2D::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        Object::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        Object::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        Object::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        Object::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.object.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.object.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.object.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.object.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        Object::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time().max(self.storage.data.get_m_time())
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.object.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.object.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.object.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.object.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.object.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for Points2D {
    fn default() -> Self {
        Self::new()
    }
}

fn empty_bounds_2d() -> [f64; 4] {
    [
        VTK_DOUBLE_MAX,
        -VTK_DOUBLE_MAX,
        VTK_DOUBLE_MAX,
        -VTK_DOUBLE_MAX,
    ]
}

fn points_2d_array_for_data_type(data_type: i32) -> Option<AnyArray> {
    let data_type = VtkDataType::from_id(data_type)?;
    if !data_type.is_numeric() {
        return None;
    }
    let mut data = AnyArray::create_array(data_type)?;
    data.set_number_of_components(2);
    data.set_name("Points2D");
    Some(data)
}

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkPoints2D id must be non-negative")
}
