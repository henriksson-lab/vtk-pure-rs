use std::fmt;

use super::{
    any_array::AnyArray, minimal_standard_random_sequence::MinimalStandardRandomSequence,
    object::Object, random_sequence::RandomSequence, time_stamp::TimeStamp, vtk_type::VtkIdType,
};

/// VTK: `vtkRandomPool`.
pub struct RandomPool {
    object: Object,
    generate_time: TimeStamp,
    sequence: Option<Box<dyn RandomSequence>>,
    size: VtkIdType,
    number_of_components: i32,
    chunk_size: VtkIdType,
    total_size: VtkIdType,
    pool: Vec<f64>,
}

impl RandomPool {
    /// VTK: `vtkRandomPool::New`.
    pub fn new() -> Self {
        let mut generate_time = TimeStamp::new();
        generate_time.modified();

        let mut object = Object::new();
        object.modified();

        Self {
            object,
            generate_time,
            sequence: Some(Box::new(MinimalStandardRandomSequence::new())),
            size: 0,
            number_of_components: 1,
            chunk_size: 10000,
            total_size: 0,
            pool: Vec::new(),
        }
    }

    /// VTK: `vtkRandomPool::SetSequence`.
    pub fn set_sequence(&mut self, sequence: Option<Box<dyn RandomSequence>>) {
        self.sequence = sequence;
        self.modified();
    }

    /// VTK: `vtkRandomPool::GetSequence`.
    pub fn get_sequence(&self) -> Option<&dyn RandomSequence> {
        self.sequence.as_deref()
    }

    /// VTK: `vtkRandomPool::SetSize`.
    pub fn set_size(&mut self, size: VtkIdType) {
        let size = size.clamp(1, VtkIdType::MAX);
        if self.size != size {
            self.size = size;
            self.modified();
        }
    }

    /// VTK: `vtkRandomPool::GetSize`.
    pub fn get_size(&self) -> VtkIdType {
        self.size
    }

    /// VTK: `vtkRandomPool::SetNumberOfComponents`.
    pub fn set_number_of_components(&mut self, number_of_components: i32) {
        let number_of_components = number_of_components.clamp(1, i32::MAX);
        if self.number_of_components != number_of_components {
            self.number_of_components = number_of_components;
            self.modified();
        }
    }

    /// VTK: `vtkRandomPool::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.number_of_components
    }

    /// VTK: `vtkRandomPool::GetTotalSize`.
    pub fn get_total_size(&self) -> VtkIdType {
        self.size * VtkIdType::from(self.number_of_components)
    }

    /// VTK: `vtkRandomPool::GeneratePool`.
    pub fn generate_pool(&mut self) -> Option<&[f64]> {
        if self.generate_time.get_m_time() > self.object.get_m_time() {
            return Some(&self.pool);
        }

        self.total_size = self.size * VtkIdType::from(self.number_of_components);
        if self.total_size <= 0 || self.sequence.is_none() {
            self.size = 1000;
            self.total_size = 1000;
            self.number_of_components = 1;
        }

        self.chunk_size = self.chunk_size.max(1000);
        let total_size = usize::try_from(self.total_size).ok()?;
        self.pool.clear();
        self.pool.resize(total_size, 0.0);

        let sequence = self.sequence.as_deref_mut()?;
        sequence.initialize(31415);
        for value in &mut self.pool {
            *value = sequence.get_value();
            sequence.next();
        }

        self.generate_time.modified();
        Some(&self.pool)
    }

    /// VTK: `vtkRandomPool::GetPool`.
    pub fn get_pool(&self) -> &[f64] {
        &self.pool
    }

    /// VTK: `vtkRandomPool::GetValue(vtkIdType i)`.
    pub fn get_value(&self, i: VtkIdType) -> f64 {
        let total_size = usize::try_from(self.total_size).expect("pool must be generated first");
        let index = usize::try_from(i.rem_euclid(self.total_size))
            .expect("modulo-reduced pool index must fit usize");
        self.pool[index % total_size]
    }

    /// VTK: `vtkRandomPool::GetValue(vtkIdType i, int compNum)`.
    pub fn get_value_component(&self, i: VtkIdType, comp_num: i32) -> f64 {
        let total_size = usize::try_from(self.total_size).expect("pool must be generated first");
        let flat = VtkIdType::from(comp_num) + VtkIdType::from(self.number_of_components) * i;
        let index = usize::try_from(flat.rem_euclid(self.total_size))
            .expect("modulo-reduced pool index must fit usize");
        self.pool[index % total_size]
    }

    /// VTK: `vtkRandomPool::PopulateDataArray(vtkDataArray*, double, double)`.
    pub fn populate_data_array(
        &mut self,
        data_array: &mut AnyArray,
        min_range: f64,
        max_range: f64,
    ) -> bool {
        if !data_array.is_data_array() {
            return false;
        }

        let size = data_array.get_number_of_tuples();
        let number_of_components = data_array.get_number_of_components();
        self.set_size(size);
        self.set_number_of_components(number_of_components);

        let Some(pool) = self.generate_pool() else {
            return false;
        };
        let range = max_range - min_range;
        let number_of_components = usize::try_from(number_of_components)
            .expect("number of components must be non-negative");
        let number_of_tuples = usize::try_from(size).expect("tuple count must fit usize");

        for tuple_id in 0..number_of_tuples {
            let start = tuple_id * number_of_components;
            let tuple: Vec<_> = pool[start..start + number_of_components]
                .iter()
                .map(|value| min_range + value * range)
                .collect();
            if data_array
                .insert_numeric_tuple_from_f64_checked(tuple_id, &tuple)
                .is_err()
            {
                return false;
            }
        }

        true
    }

    /// VTK: `vtkRandomPool::PopulateDataArray(vtkDataArray*, int, double, double)`.
    pub fn populate_data_array_component(
        &mut self,
        data_array: &mut AnyArray,
        comp_num: i32,
        min_range: f64,
        max_range: f64,
    ) -> bool {
        if !data_array.is_data_array() {
            return false;
        }

        let size = data_array.get_number_of_tuples();
        let number_of_components = data_array.get_number_of_components();
        let comp_num = comp_num.clamp(0, number_of_components - 1);
        self.set_size(size);
        self.set_number_of_components(number_of_components);

        let Some(pool) = self.generate_pool() else {
            return false;
        };
        let range = max_range - min_range;
        let number_of_components = usize::try_from(number_of_components)
            .expect("number of components must be non-negative");
        let comp_num = usize::try_from(comp_num).expect("component must be non-negative");
        let number_of_tuples = usize::try_from(size).expect("tuple count must fit usize");

        for tuple_id in 0..number_of_tuples {
            let value_id = tuple_id * number_of_components + comp_num;
            let mut tuple = match data_array.numeric_tuple_as_f64_checked(tuple_id) {
                Ok(tuple) => tuple,
                Err(_) => return false,
            };
            tuple[comp_num] = min_range + pool[value_id] * range;
            if data_array
                .insert_numeric_tuple_from_f64_checked(tuple_id, &tuple)
                .is_err()
            {
                return false;
            }
        }

        true
    }

    /// VTK: `vtkRandomPool::SetChunkSize`.
    pub fn set_chunk_size(&mut self, chunk_size: VtkIdType) {
        let chunk_size = chunk_size.clamp(1000, VtkIdType::from(i32::MAX));
        if self.chunk_size != chunk_size {
            self.chunk_size = chunk_size;
            self.modified();
        }
    }

    /// VTK: `vtkRandomPool::GetChunkSize`.
    pub fn get_chunk_size(&self) -> VtkIdType {
        self.chunk_size
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
    pub fn get_m_time(&self) -> u64 {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkRandomPool"
    }

    /// VTK: `vtkRandomPool::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkRandomPool" || Object::is_type_of(name)
    }

    /// VTK: `vtkRandomPool::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkRandomPool::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkRandomPool" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkRandomPool::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.object.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.object.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.object.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
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

impl Default for RandomPool {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for RandomPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RandomPool")
            .field("object", &self.object)
            .field("generate_time", &self.generate_time)
            .field("has_sequence", &self.sequence.is_some())
            .field("size", &self.size)
            .field("number_of_components", &self.number_of_components)
            .field("chunk_size", &self.chunk_size)
            .field("total_size", &self.total_size)
            .field("pool_len", &self.pool.len())
            .finish()
    }
}
