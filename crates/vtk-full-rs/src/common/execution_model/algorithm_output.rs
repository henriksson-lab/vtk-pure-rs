use std::{cell::RefCell, fmt, rc::Rc};

use crate::common::core::{Object, ObjectBaseApi, VtkMTimeType};

/// Rust handle for APIs that store a `vtkAlgorithm*`.
pub trait AlgorithmApi: ObjectBaseApi {}

/// Shallow-copyable dynamic handle for `vtkAlgorithm*` storage.
#[derive(Clone)]
pub struct AlgorithmHandle {
    algorithm: Rc<RefCell<dyn AlgorithmApi>>,
}

impl AlgorithmHandle {
    pub fn new<T: AlgorithmApi + 'static>(algorithm: T) -> Self {
        Self {
            algorithm: Rc::new(RefCell::new(algorithm)),
        }
    }

    pub fn from_rc<T: AlgorithmApi + 'static>(algorithm: Rc<RefCell<T>>) -> Self {
        Self { algorithm }
    }

    pub fn as_ptr(&self) -> *const RefCell<dyn AlgorithmApi> {
        Rc::as_ptr(&self.algorithm)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.algorithm, &other.algorithm)
    }

    pub fn get_class_name(&self) -> String {
        self.algorithm.borrow().get_class_name().to_owned()
    }

    pub fn is_a(&self, name: &str) -> bool {
        self.algorithm.borrow().is_a(name)
    }

    pub fn get_object_description(&self) -> String {
        self.algorithm.borrow().get_object_description()
    }
}

impl fmt::Debug for AlgorithmHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlgorithmHandle")
            .field("class_name", &self.get_class_name())
            .finish_non_exhaustive()
    }
}

/// VTK: `vtkAlgorithmOutput`.
#[derive(Debug)]
pub struct AlgorithmOutput {
    object: Object,
    index: i32,
    producer: Option<AlgorithmHandle>,
}

impl AlgorithmOutput {
    /// VTK: `vtkAlgorithmOutput::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkAlgorithmOutput"),
            index: 0,
            producer: None,
        }
    }

    /// VTK: `vtkAlgorithmOutput::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut output = self.object.get_object_description();
        output.push_str("\nProducer: ");
        if let Some(producer) = &self.producer {
            output.push_str(&format!("{:p}", producer.as_ptr()));
        } else {
            output.push_str("(none)");
        }
        output.push_str("\nIndex: ");
        output.push_str(&self.index.to_string());
        output
    }

    /// VTK: `vtkAlgorithmOutput::SetIndex`.
    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    /// VTK: `vtkAlgorithmOutput::GetIndex`.
    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// VTK: `vtkAlgorithmOutput::GetProducer`.
    pub fn get_producer(&self) -> Option<AlgorithmHandle> {
        self.producer.clone()
    }

    /// VTK: `vtkAlgorithmOutput::SetProducer`.
    pub fn set_producer(&mut self, producer: Option<AlgorithmHandle>) {
        self.producer = producer;
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

    /// VTK: `vtkAlgorithmOutput::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkAlgorithmOutput" || Object::is_type_of(name)
    }

    /// VTK: `vtkAlgorithmOutput::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkObjectBase::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.object.get_object_description()
    }
}

impl Default for AlgorithmOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectBaseApi for AlgorithmOutput {
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
