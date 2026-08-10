use std::path::{Path, PathBuf};

use crate::common::core::{Object, StringArray, VtkIdType, VtkMTimeType};

/// VTK: `vtkDirectory`.
#[derive(Debug, Clone)]
pub struct Directory {
    object: Object,
    path: Option<PathBuf>,
    files: StringArray,
}

impl Directory {
    /// VTK: `vtkDirectory::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkDirectory"),
            path: None,
            files: StringArray::new(),
        }
    }

    /// VTK: `vtkDirectory::CleanUpFilesAndPath`.
    pub(crate) fn clean_up_files_and_path(&mut self) {
        self.files.reset();
        self.path = None;
    }

    /// VTK: `vtkDirectory::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = format!("Files: ({:p})\n", &self.files);
        let Some(path) = &self.path else {
            result.push_str("Directory not open\n");
            return result;
        };

        result.push_str(&format!("Directory for: {}\n", path.display()));
        result.push_str("Contains the following files:\n");
        for index in 0..self.files.get_number_of_values() {
            result.push_str(self.files.get_value(index));
            result.push('\n');
        }
        result
    }

    /// VTK: `vtkDirectory::Open`.
    pub fn open(&mut self, dir: Option<&str>) -> i32 {
        self.clean_up_files_and_path();

        let Some(dir) = dir else {
            return 0;
        };
        let path = PathBuf::from(dir);
        let Ok(entries) = std::fs::read_dir(&path) else {
            return 0;
        };

        for entry in entries.flatten() {
            self.files
                .insert_next_value(entry.file_name().to_string_lossy().into_owned());
        }
        self.path = Some(path);
        1
    }

    /// VTK: `vtkDirectory::GetNumberOfFiles`.
    pub fn get_number_of_files(&self) -> VtkIdType {
        self.files.get_number_of_values()
    }

    /// VTK: `vtkDirectory::GetFile`.
    pub fn get_file(&self, index: VtkIdType) -> Option<&str> {
        if index < 0 || index >= self.files.get_number_of_values() {
            None
        } else {
            Some(self.files.get_value(index))
        }
    }

    /// VTK: `vtkDirectory::FileIsDirectory`.
    pub fn file_is_directory(&self, name: Option<&str>) -> i32 {
        let Some(name) = name else {
            return 0;
        };
        let name_path = Path::new(name);
        let full_path = if name_path.is_absolute() {
            name_path.to_path_buf()
        } else if let Some(path) = &self.path {
            path.join(name_path)
        } else {
            name_path.to_path_buf()
        };
        i32::from(full_path.is_dir())
    }

    /// VTK: `vtkDirectory::GetFiles`.
    pub fn get_files(&self) -> &StringArray {
        &self.files
    }

    /// VTK: `vtkDirectory::GetCurrentWorkingDirectory`.
    pub fn get_current_working_directory(len: u32) -> Option<String> {
        let directory = std::env::current_dir().ok()?;
        let value = directory.to_string_lossy().into_owned();
        if value.len() < len as usize {
            Some(value)
        } else {
            None
        }
    }

    /// VTK: `vtkDirectory::MakeDirectory`.
    pub fn make_directory(dir: &str) -> i32 {
        i32::from(std::fs::create_dir_all(dir).is_ok())
    }

    /// VTK: `vtkDirectory::DeleteDirectory`.
    pub fn delete_directory(dir: &str) -> i32 {
        i32::from(std::fs::remove_dir(dir).is_ok())
    }

    /// VTK: `vtkDirectory::Rename`.
    pub fn rename(oldname: &str, newname: &str) -> i32 {
        i32::from(std::fs::rename(oldname, newname).is_ok())
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time().max(self.files.get_m_time())
    }
}

impl Default for Directory {
    fn default() -> Self {
        Self::new()
    }
}
