use std::{cell::RefCell, fmt, rc::Rc};

use crate::common::core::{InformationKey, Object, ObjectBase};

/// Rust handle for APIs that store an arbitrary `vtkObjectBase*`.
pub trait ObjectBaseApi: fmt::Debug {
    /// VTK: `vtkObjectBase::GetClassName`.
    fn get_class_name(&self) -> &str;

    /// VTK: `vtkObjectBase::IsA`.
    fn is_a(&self, name: &str) -> bool;

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    fn get_object_description(&self) -> String;

    /// VTK: `vtkObjectBase::PrintSelf`.
    fn print_self(&self) -> String {
        self.get_object_description()
    }
}

/// Shallow-copyable dynamic object handle for `vtkObjectBase*` storage.
#[derive(Clone)]
pub struct ObjectBaseHandle {
    object: Rc<RefCell<dyn ObjectBaseApi>>,
}

impl ObjectBaseHandle {
    pub fn new<T: ObjectBaseApi + 'static>(object: T) -> Self {
        Self {
            object: Rc::new(RefCell::new(object)),
        }
    }

    pub fn from_rc<T: ObjectBaseApi + 'static>(object: Rc<RefCell<T>>) -> Self {
        Self { object }
    }

    pub fn as_ptr(&self) -> *const RefCell<dyn ObjectBaseApi> {
        Rc::as_ptr(&self.object)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.object, &other.object)
    }

    pub fn get_class_name(&self) -> String {
        self.object.borrow().get_class_name().to_owned()
    }

    pub fn is_a(&self, name: &str) -> bool {
        self.object.borrow().is_a(name)
    }

    pub fn get_object_description(&self) -> String {
        self.object.borrow().get_object_description()
    }

    pub fn print_self(&self) -> String {
        self.object.borrow().print_self()
    }
}

impl fmt::Debug for ObjectBaseHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectBaseHandle")
            .field("class_name", &self.get_class_name())
            .finish_non_exhaustive()
    }
}

impl ObjectBaseApi for ObjectBase {
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

impl ObjectBaseApi for Object {
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

impl ObjectBaseApi for InformationKey {
    fn get_class_name(&self) -> &str {
        self.get_class_name()
    }

    fn is_a(&self, name: &str) -> bool {
        self.is_a(name)
    }

    fn get_object_description(&self) -> String {
        self.get_object_description()
    }

    fn print_self(&self) -> String {
        self.print_self()
    }
}
