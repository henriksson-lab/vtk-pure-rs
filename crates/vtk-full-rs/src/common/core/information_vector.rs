use std::{cell::RefCell, rc::Rc};

use crate::common::core::{Information, InformationHandle, Object, VtkIdType, VtkMTimeType};

pub type InformationVectorHandle = Rc<RefCell<InformationVector>>;

/// VTK: `vtkInformationVector`.
#[derive(Debug)]
pub struct InformationVector {
    object: Object,
    vector: Vec<InformationHandle>,
}

impl InformationVector {
    /// VTK: `vtkInformationVector::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkInformationVector"),
            vector: Vec::new(),
        }
    }

    /// VTK: `vtkInformationVector::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        output.push_str("\nNumber of Information Objects: ");
        output.push_str(&self.get_number_of_information_objects().to_string());
        output.push_str("\nInformation Objects:");
        for info in &self.vector {
            let info = info.borrow();
            output.push('\n');
            output.push_str(&info.get_object_description());
            output.push_str(":\n");
            output.push_str(&info.print_self());
        }
        output
    }

    /// VTK: `vtkInformationVector::GetNumberOfInformationObjects`.
    pub fn get_number_of_information_objects(&self) -> i32 {
        self.vector.len() as i32
    }

    /// VTK: `vtkInformationVector::SetNumberOfInformationObjects`.
    pub fn set_number_of_information_objects(&mut self, new_number: i32) {
        let new_number = new_number.max(0) as usize;
        let old_number = self.vector.len();
        if new_number > old_number {
            self.vector
                .resize_with(new_number, || Rc::new(RefCell::new(Information::new())));
        } else if new_number < old_number {
            self.vector.truncate(new_number);
        }
    }

    /// VTK: `vtkInformationVector::SetInformationObject`.
    pub fn set_information_object(&mut self, index: i32, new_info: Option<InformationHandle>) {
        if index < 0 {
            return;
        }
        let index = index as usize;
        match new_info {
            Some(new_info) if index < self.vector.len() => {
                if !Rc::ptr_eq(&self.vector[index], &new_info) {
                    self.vector[index] = new_info;
                }
            }
            Some(new_info) => {
                if index > self.vector.len() {
                    self.set_number_of_information_objects(index as i32);
                }
                self.vector.push(new_info);
            }
            None if index + 1 < self.vector.len() => {
                self.vector[index] = Rc::new(RefCell::new(Information::new()));
            }
            None if index + 1 == self.vector.len() => {
                self.set_number_of_information_objects(index as i32);
            }
            None => {}
        }
    }

    /// VTK: `vtkInformationVector::GetInformationObject`.
    pub fn get_information_object(&self, index: i32) -> Option<InformationHandle> {
        if index < 0 {
            return None;
        }
        self.vector.get(index as usize).cloned()
    }

    /// VTK: `vtkInformationVector::Append`.
    pub fn append(&mut self, info: Option<InformationHandle>) {
        let index = self.get_number_of_information_objects();
        self.set_information_object(index, info);
    }

    /// VTK: `vtkInformationVector::Remove(vtkInformation*)`.
    pub fn remove(&mut self, info: &InformationHandle) {
        let mut index = 0;
        while index < self.vector.len() {
            if Rc::ptr_eq(&self.vector[index], info) {
                self.vector.remove(index);
            } else {
                index += 1;
            }
        }
    }

    /// VTK: `vtkInformationVector::Remove(int)`.
    pub fn remove_by_index(&mut self, index: i32) {
        if index >= 0 && (index as usize) < self.vector.len() {
            self.vector.remove(index as usize);
        }
    }

    /// VTK: `vtkInformationVector::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        true
    }

    /// VTK: `vtkInformationVector::Copy`.
    pub fn copy(&mut self, from: Option<&InformationVector>, deep: bool) {
        let Some(from) = from else {
            self.set_number_of_information_objects(0);
            return;
        };
        if deep {
            self.set_number_of_information_objects(from.get_number_of_information_objects());
            for index in 0..from.vector.len() {
                self.vector[index]
                    .borrow_mut()
                    .copy(Some(&from.vector[index].borrow()), true);
            }
        } else {
            self.set_number_of_information_objects(0);
            for info in &from.vector {
                self.append(Some(info.clone()));
            }
        }
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkInformationVector::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkInformationVector" || Object::is_type_of(name)
    }

    /// VTK: `vtkInformationVector::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkInformationVector::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkInformationVector" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkInformationVector::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
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

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for InformationVector {
    fn default() -> Self {
        Self::new()
    }
}
