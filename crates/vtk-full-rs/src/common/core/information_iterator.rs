use std::{
    cell::RefCell,
    ptr,
    rc::{Rc, Weak},
};

use crate::common::core::{Information, InformationHandle, InformationKey, Object, VtkMTimeType};

#[derive(Debug, Clone)]
enum InformationReference {
    Strong(InformationHandle),
    Weak(Weak<RefCell<Information>>),
}

impl InformationReference {
    fn upgrade(&self) -> Option<InformationHandle> {
        match self {
            Self::Strong(info) => Some(Rc::clone(info)),
            Self::Weak(info) => info.upgrade(),
        }
    }
}

/// VTK: `vtkInformationIterator`.
#[derive(Debug)]
pub struct InformationIterator {
    object: Object,
    information: Option<InformationReference>,
    keys: Vec<*mut InformationKey>,
    iterator: usize,
    reference_is_weak: bool,
}

impl InformationIterator {
    /// VTK: `vtkInformationIterator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkInformationIterator"),
            information: None,
            keys: Vec::new(),
            iterator: 0,
            reference_is_weak: false,
        }
    }

    /// VTK: `vtkInformationIterator::SetInformation`.
    pub fn set_information(&mut self, information: Option<InformationHandle>) {
        let changed = match (&self.information, &information) {
            (Some(InformationReference::Strong(current)), Some(next)) => !Rc::ptr_eq(current, next),
            (None, None) => false,
            _ => true,
        };
        self.information = information.map(InformationReference::Strong);
        self.reference_is_weak = false;
        if changed {
            self.modified();
        }
    }

    /// VTK: `vtkInformationIterator::GetInformation`.
    pub fn get_information(&self) -> Option<InformationHandle> {
        self.information
            .as_ref()
            .and_then(InformationReference::upgrade)
    }

    /// VTK: `vtkInformationIterator::SetInformationWeak`.
    pub fn set_information_weak(&mut self, information: Option<&InformationHandle>) {
        let next = information.map(Rc::downgrade);
        let changed = match (&self.information, &next) {
            (Some(InformationReference::Weak(current)), Some(next)) => !current.ptr_eq(next),
            (None, None) => false,
            _ => true,
        };
        self.information = next.map(InformationReference::Weak);
        self.reference_is_weak = true;
        if changed {
            self.modified();
        }
    }

    /// VTK: `vtkInformationIterator::InitTraversal`.
    pub fn init_traversal(&mut self) {
        self.go_to_first_item();
    }

    /// VTK: `vtkInformationIterator::GoToFirstItem`.
    pub fn go_to_first_item(&mut self) {
        self.keys.clear();
        self.iterator = 0;
        if let Some(information) = self.get_information() {
            self.keys = information.borrow().key_ptrs();
        }
    }

    /// VTK: `vtkInformationIterator::GoToNextItem`.
    pub fn go_to_next_item(&mut self) {
        if self.get_information().is_none() {
            return;
        }
        if self.iterator < self.keys.len() {
            self.iterator += 1;
        }
    }

    /// VTK: `vtkInformationIterator::IsDoneWithTraversal`.
    pub fn is_done_with_traversal(&self) -> i32 {
        if self.get_information().is_none() {
            return 1;
        }
        i32::from(self.iterator >= self.keys.len())
    }

    /// VTK: `vtkInformationIterator::GetCurrentKey`.
    pub fn get_current_key(&self) -> *mut InformationKey {
        if self.is_done_with_traversal() != 0 {
            return ptr::null_mut();
        }
        self.keys[self.iterator]
    }

    /// VTK: `vtkInformationIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        output.push_str("\nInformation: ");
        if let Some(information) = self.get_information() {
            output.push('\n');
            output.push_str(&information.borrow().print_self());
        } else {
            output.push_str("(none)\n");
        }
        output
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkInformationIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkInformationIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkInformationIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationIterator::GetNumberOfGenerationsFromBase`.
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

impl Default for InformationIterator {
    fn default() -> Self {
        Self::new()
    }
}
