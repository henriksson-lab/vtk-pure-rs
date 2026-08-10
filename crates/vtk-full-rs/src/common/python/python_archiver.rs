use std::{fmt, path::Path};

use crate::common::core::{
    archiver::Archiver,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// Rust representation of the callable Python object used by `vtkPythonArchiver`.
///
/// VTK origin: `PyObject* Object` plus calls to Python methods named
/// `OpenArchive`, `CloseArchive`, `InsertIntoArchive`, and `Contains`.
pub trait PythonArchiverObject {
    /// VTK Python callback: `OpenArchive(self, vtkself)`.
    fn open_archive(&mut self, _vtkself: &mut PythonArchiver) -> Option<i32> {
        None
    }

    /// VTK Python callback: `CloseArchive(self, vtkself)`.
    fn close_archive(&mut self, _vtkself: &mut PythonArchiver) -> Option<i32> {
        None
    }

    /// VTK Python callback:
    /// `InsertIntoArchive(self, vtkself, relativePath, data, size)`.
    fn insert_into_archive(
        &mut self,
        _vtkself: &mut PythonArchiver,
        _relative_path: &str,
        _data: &[u8],
        _size: usize,
    ) -> Option<i32> {
        None
    }

    /// VTK Python callback: `Contains(self, vtkself, relativePath)`.
    fn contains(&mut self, _vtkself: &mut PythonArchiver, _relative_path: &str) -> Option<i32> {
        None
    }
}

/// A `vtkArchiver` implementation delegated to a Python-like object.
///
/// VTK origin: `VTK/Common/Python/vtkPythonArchiver.{h,cxx}`.
pub struct PythonArchiver {
    base: Archiver,
    object: Option<Box<dyn PythonArchiverObject>>,
}

impl PythonArchiver {
    /// VTK: `vtkPythonArchiver::New`.
    pub fn new() -> Self {
        Self {
            base: Archiver::with_class_name("vtkPythonArchiver"),
            object: None,
        }
    }

    /// VTK: `vtkPythonArchiver::SetPythonObject`.
    pub fn set_python_object(&mut self, object: Option<Box<dyn PythonArchiverObject>>) {
        let Some(object) = object else {
            return;
        };
        self.object = Some(object);
    }

    /// VTK: `vtkPythonArchiver::OpenArchive`.
    pub fn open_archive(&mut self) {
        let Some(mut object) = self.object.take() else {
            return;
        };
        let result = object.open_archive(self);
        self.object = Some(object);
        self.check_result("OpenArchive", result);
    }

    /// VTK: `vtkPythonArchiver::CloseArchive`.
    pub fn close_archive(&mut self) {
        let Some(mut object) = self.object.take() else {
            return;
        };
        let result = object.close_archive(self);
        self.object = Some(object);
        self.check_result("CloseArchive", result);
    }

    /// VTK: `vtkPythonArchiver::InsertIntoArchive`.
    pub fn insert_into_archive(&mut self, relative_path: impl AsRef<Path>, data: &[u8]) {
        let Some(mut object) = self.object.take() else {
            return;
        };
        let relative_path = relative_path.as_ref().to_string_lossy().into_owned();
        let result = object.insert_into_archive(self, &relative_path, data, data.len());
        self.object = Some(object);
        self.check_result("InsertIntoArchive", result);
    }

    /// VTK: `vtkPythonArchiver::Contains`.
    pub fn contains(&mut self, relative_path: impl AsRef<Path>) -> bool {
        let Some(mut object) = self.object.take() else {
            return false;
        };
        let relative_path = relative_path.as_ref().to_string_lossy().into_owned();
        let result = object.contains(self, &relative_path);
        self.object = Some(object);
        self.check_result("Contains", result) != 0
    }

    /// VTK macro: `vtkGetStringMacro(ArchiveName)`.
    pub fn get_archive_name(&self) -> Option<&str> {
        self.base.get_archive_name()
    }

    /// VTK macro: `vtkSetStringMacro(ArchiveName)`.
    pub fn set_archive_name(&mut self, archive_name: Option<&str>) {
        self.base.set_archive_name(archive_name);
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.base.get_class_name()
    }

    /// VTK: `vtkPythonArchiver::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkPythonArchiver" || Archiver::is_type_of(name)
    }

    /// VTK: `vtkPythonArchiver::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkPythonArchiver::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkPythonArchiver" => 0,
            "vtkArchiver" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => Archiver::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkPythonArchiver::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        Archiver::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        Archiver::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        Archiver::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        Archiver::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.base.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.base.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.base.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.base.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        Archiver::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.base.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.base.get_m_time()
    }

    /// VTK: `vtkObject::Register`.
    pub fn register(&mut self) {
        self.base.register();
    }

    /// VTK: `vtkObject::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.base.unregister()
    }

    /// VTK: `vtkObject::Delete`.
    pub fn delete(&mut self) -> bool {
        self.base.delete()
    }

    /// VTK: `vtkObject::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.base.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.base.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.base.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.base.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.base.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.base.get_object_description()
    }

    /// VTK: `vtkPythonArchiver::CheckResult`.
    fn check_result(&self, method: &str, result: Option<i32>) -> i32 {
        let Some(code) = result else {
            eprintln!("Failure when calling method: \"{method}\"");
            return 0;
        };
        code
    }
}

impl Default for PythonArchiver {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PythonArchiver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PythonArchiver")
            .field("base", &self.base)
            .field("has_object", &self.object.is_some())
            .finish()
    }
}
