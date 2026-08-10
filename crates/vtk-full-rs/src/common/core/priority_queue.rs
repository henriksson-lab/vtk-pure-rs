use super::{
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

pub const VTK_DOUBLE_MAX: f64 = 1.0e299;

/// VTK: `vtkPriorityQueue::Item`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriorityQueueItem {
    pub priority: f64,
    pub id: VtkIdType,
}

/// VTK: `vtkPriorityQueue`.
#[derive(Debug, Clone, PartialEq)]
pub struct PriorityQueue {
    object: Object,
    item_location: Vec<VtkIdType>,
    array: Vec<PriorityQueueItem>,
    size: VtkIdType,
    extend: VtkIdType,
}

impl PriorityQueue {
    /// VTK: `vtkPriorityQueue::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkPriorityQueue"),
            item_location: Vec::new(),
            array: Vec::new(),
            size: 0,
            extend: 1000,
        }
    }

    /// VTK: `vtkPriorityQueue::Allocate`.
    pub fn allocate(&mut self, size: VtkIdType, extend: VtkIdType) {
        self.item_location.clear();
        let capacity = positive_count_or_one(size);
        self.item_location.reserve(capacity);
        self.array.clear();
        self.array = Vec::with_capacity(capacity);
        self.size = capacity as VtkIdType;
        self.extend = if extend > 0 { extend } else { 1 };
    }

    /// VTK: `vtkPriorityQueue::Insert`.
    pub fn insert(&mut self, priority: f64, id: VtkIdType) {
        let Some(id_idx) = vtk_id_to_usize(id) else {
            return;
        };

        if id_idx < self.item_location.len() && self.item_location[id_idx] != -1 {
            return;
        }

        let new_location = self.array.len();
        if new_location as VtkIdType >= self.size {
            self.resize(new_location as VtkIdType + 1);
        }

        self.array.push(PriorityQueueItem { priority, id });

        let old_max_id = self.item_location.len() as VtkIdType - 1;
        if id_idx >= self.item_location.len() {
            self.item_location.resize(id_idx + 1, -1);
        }
        self.item_location[id_idx] = new_location as VtkIdType;
        for i in (old_max_id + 1)..id {
            if let Some(i) = vtk_id_to_usize(i) {
                self.item_location[i] = -1;
            }
        }

        self.percolate_up(new_location);
    }

    /// VTK: `vtkPriorityQueue::Pop`.
    pub fn pop(&mut self, location: VtkIdType) -> VtkIdType {
        self.pop_with_priority(location).map_or(-1, |(id, _)| id)
    }

    /// VTK: `vtkPriorityQueue::Pop(location, priority)`.
    pub fn pop_with_priority(&mut self, location: VtkIdType) -> Option<(VtkIdType, f64)> {
        let location = vtk_id_to_usize(location)?;
        if self.array.is_empty() || location >= self.array.len() {
            return None;
        }

        let removed = self.array[location];
        if let Some(id_idx) = vtk_id_to_usize(removed.id) {
            self.item_location[id_idx] = -1;
        }

        let last = self.array.pop().expect("non-empty queue");
        if location < self.array.len() {
            self.array[location] = last;
            if let Some(id_idx) = vtk_id_to_usize(last.id) {
                self.item_location[id_idx] = location as VtkIdType;
            }
            self.percolate_down(location);
            self.percolate_up(location);
        }

        Some((removed.id, removed.priority))
    }

    /// VTK: `vtkPriorityQueue::Peek`.
    pub fn peek(&self, location: VtkIdType) -> VtkIdType {
        self.peek_with_priority(location).map_or(-1, |(id, _)| id)
    }

    /// VTK: `vtkPriorityQueue::Peek(location, priority)`.
    pub fn peek_with_priority(&self, location: VtkIdType) -> Option<(VtkIdType, f64)> {
        let location = vtk_id_to_usize(location)?;
        let item = self.array.get(location)?;
        Some((item.id, item.priority))
    }

    /// VTK: `vtkPriorityQueue::DeleteId`.
    pub fn delete_id(&mut self, id: VtkIdType) -> f64 {
        let Some(id_idx) = vtk_id_to_usize(id) else {
            return VTK_DOUBLE_MAX;
        };
        if id_idx >= self.item_location.len() {
            return VTK_DOUBLE_MAX;
        }
        let location = self.item_location[id_idx];
        if location == -1 {
            return VTK_DOUBLE_MAX;
        }
        self.pop_with_priority(location)
            .map_or(VTK_DOUBLE_MAX, |(_, priority)| priority)
    }

    /// VTK: `vtkPriorityQueue::GetPriority`.
    pub fn get_priority(&self, id: VtkIdType) -> f64 {
        let Some(id_idx) = vtk_id_to_usize(id) else {
            return VTK_DOUBLE_MAX;
        };
        if id_idx >= self.item_location.len() {
            return VTK_DOUBLE_MAX;
        }
        let location = self.item_location[id_idx];
        if location == -1 {
            return VTK_DOUBLE_MAX;
        }
        vtk_id_to_usize(location)
            .and_then(|location| self.array.get(location))
            .map_or(VTK_DOUBLE_MAX, |item| item.priority)
    }

    /// VTK: `vtkPriorityQueue::GetNumberOfItems`.
    pub fn get_number_of_items(&self) -> VtkIdType {
        self.array.len() as VtkIdType
    }

    /// VTK: `vtkPriorityQueue::Reset`.
    pub fn reset(&mut self) {
        self.array.clear();
        self.item_location.clear();
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkPriorityQueue::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkPriorityQueue" || Object::is_type_of(name)
    }

    /// VTK: `vtkPriorityQueue::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkPriorityQueue::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkPriorityQueue" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkPriorityQueue::GetNumberOfGenerationsFromBase`.
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
        self.object.get_m_time()
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

    /// VTK: `vtkPriorityQueue::Resize`.
    fn resize(&mut self, size: VtkIdType) {
        let mut new_size = if size >= self.size {
            self.size + size
        } else {
            size
        };
        if new_size <= 0 {
            new_size = 1;
        }
        self.size = new_size;
        let capacity = positive_count_or_one(new_size);
        if capacity > self.array.capacity() {
            self.array.reserve(capacity - self.array.capacity());
        }
    }

    fn percolate_up(&mut self, mut i: usize) {
        while i > 0 {
            let parent = (i - 1) / 2;
            if self.array[i].priority >= self.array[parent].priority {
                break;
            }
            self.swap_items(i, parent);
            i = parent;
        }
    }

    fn percolate_down(&mut self, mut i: usize) {
        loop {
            let left = 2 * i + 1;
            if left >= self.array.len() {
                break;
            }
            let right = left + 1;
            let child = if right >= self.array.len()
                || self.array[left].priority < self.array[right].priority
            {
                left
            } else {
                right
            };
            if self.array[i].priority <= self.array[child].priority {
                break;
            }
            self.swap_items(i, child);
            i = child;
        }
    }

    fn swap_items(&mut self, a: usize, b: usize) {
        self.array.swap(a, b);
        for location in [a, b] {
            if let Some(id_idx) = vtk_id_to_usize(self.array[location].id) {
                self.item_location[id_idx] = location as VtkIdType;
            }
        }
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn positive_count_or_one(value: VtkIdType) -> usize {
    if value > 0 {
        value as usize
    } else {
        1
    }
}

fn vtk_id_to_usize(value: VtkIdType) -> Option<usize> {
    (value >= 0).then_some(value as usize)
}
