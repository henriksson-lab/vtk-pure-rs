use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use super::{
    object::Object,
    vtk_type::{VtkIdType, VtkMTimeType},
};

/// VTK: `vtkArchiver`.
#[derive(Debug)]
pub struct Archiver {
    object: Object,
    archive_name: Option<String>,
}

impl Archiver {
    /// VTK: `vtkArchiver::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkArchiver")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            archive_name: None,
        }
    }

    /// VTK macro: `vtkGetStringMacro(ArchiveName)`.
    pub fn get_archive_name(&self) -> Option<&str> {
        self.archive_name.as_deref()
    }

    /// VTK macro: `vtkSetStringMacro(ArchiveName)`.
    pub fn set_archive_name(&mut self, archive_name: Option<&str>) {
        self.archive_name = archive_name.map(str::to_string);
        self.modified();
    }

    /// VTK: `vtkArchiver::OpenArchive`.
    pub fn open_archive(&self) {
        let Some(archive_name) = self.archive_name.as_deref() else {
            eprintln!("Please specify ArchiveName to use");
            return;
        };

        if fs::create_dir_all(archive_name).is_err() {
            eprintln!("Can not create directory {archive_name}");
        }
    }

    /// VTK: `vtkArchiver::CloseArchive`.
    pub fn close_archive(&self) {}

    /// VTK: `vtkArchiver::InsertIntoArchive`.
    pub fn insert_into_archive(&self, relative_path: impl AsRef<Path>, data: &[u8]) {
        let path = self.archive_path(relative_path.as_ref());
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut out) = File::create(path) {
            let _ = out.write_all(data);
        }
    }

    /// VTK: `vtkArchiver::Contains`.
    pub fn contains(&self, relative_path: impl AsRef<Path>) -> bool {
        let path = self.archive_path(relative_path.as_ref());
        path.parent().is_some_and(Path::is_dir)
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkArchiver::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkArchiver" || Object::is_type_of(name)
    }

    /// VTK: `vtkArchiver::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkArchiver::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkArchiver" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkArchiver::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
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

    fn archive_path(&self, relative_path: &Path) -> PathBuf {
        match self.archive_name.as_deref() {
            Some(archive_name) => Path::new(archive_name).join(relative_path),
            None => PathBuf::from("/").join(relative_path),
        }
    }
}

impl Default for Archiver {
    fn default() -> Self {
        Self::new()
    }
}
