use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use super::{output_window::OutputWindow, vtk_type::VtkMTimeType};

const DEFAULT_FILE_NAME: &str = "vtkMessageLog.log";

/// VTK: `vtkFileOutputWindow`.
#[derive(Debug)]
pub struct FileOutputWindow {
    output_window: OutputWindow,
    file_name: Option<PathBuf>,
    stream: Option<BufWriter<File>>,
    flush: bool,
    append: bool,
}

impl FileOutputWindow {
    /// VTK: `vtkFileOutputWindow::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkFileOutputWindow")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            output_window: OutputWindow::with_class_name(class_name),
            file_name: None,
            stream: None,
            append: false,
            flush: false,
        }
    }

    /// VTK: `vtkFileOutputWindow::DisplayText`.
    pub fn display_text(&mut self, text: Option<&str>) {
        let Some(text) = text else {
            return;
        };

        if self.stream.is_none() {
            self.initialize();
        }

        if let Some(stream) = self.stream.as_mut() {
            let _ = writeln!(stream, "{text}");
            if self.flush {
                let _ = stream.flush();
            }
        }
    }

    /// VTK: `vtkFileOutputWindow::SetFileName`.
    pub fn set_file_name(&mut self, file_name: Option<&Path>) {
        self.file_name = file_name.map(Path::to_path_buf);
        self.stream = None;
    }

    /// VTK: `vtkFileOutputWindow::GetFileName`.
    pub fn get_file_name(&self) -> Option<&Path> {
        self.file_name.as_deref()
    }

    /// VTK: `vtkFileOutputWindow::SetFlush`.
    pub fn set_flush(&mut self, flush: bool) {
        self.flush = flush;
    }

    /// VTK: `vtkFileOutputWindow::GetFlush`.
    pub fn get_flush(&self) -> bool {
        self.flush
    }

    /// VTK: `vtkFileOutputWindow::FlushOn`.
    pub fn flush_on(&mut self) {
        self.set_flush(true);
    }

    /// VTK: `vtkFileOutputWindow::FlushOff`.
    pub fn flush_off(&mut self) {
        self.set_flush(false);
    }

    /// VTK: `vtkFileOutputWindow::SetAppend`.
    pub fn set_append(&mut self, append: bool) {
        self.append = append;
        self.stream = None;
    }

    /// VTK: `vtkFileOutputWindow::GetAppend`.
    pub fn get_append(&self) -> bool {
        self.append
    }

    /// VTK: `vtkFileOutputWindow::AppendOn`.
    pub fn append_on(&mut self) {
        self.set_append(true);
    }

    /// VTK: `vtkFileOutputWindow::AppendOff`.
    pub fn append_off(&mut self) {
        self.set_append(false);
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.output_window.get_class_name()
    }

    /// VTK: `vtkFileOutputWindow::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkFileOutputWindow" || OutputWindow::is_type_of(name)
    }

    /// VTK: `vtkFileOutputWindow::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkFileOutputWindow::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkFileOutputWindow" => 0,
            "vtkOutputWindow" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => OutputWindow::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkFileOutputWindow::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkOutputWindow::PromptUserOn`.
    pub fn prompt_user_on(&mut self) {
        self.output_window.prompt_user_on();
    }

    /// VTK: `vtkOutputWindow::PromptUserOff`.
    pub fn prompt_user_off(&mut self) {
        self.output_window.prompt_user_off();
    }

    /// VTK: `vtkOutputWindow::SetPromptUser`.
    pub fn set_prompt_user(&mut self, prompt_user: bool) {
        self.output_window.set_prompt_user(prompt_user);
    }

    /// VTK: `vtkOutputWindow::GetPromptUser`.
    pub fn get_prompt_user(&self) -> bool {
        self.output_window.get_prompt_user()
    }

    /// VTK: `vtkOutputWindow::SetDisplayMode`.
    pub fn set_display_mode(&mut self, display_mode: i32) {
        self.output_window.set_display_mode(display_mode);
    }

    /// VTK: `vtkOutputWindow::GetDisplayMode`.
    pub fn get_display_mode(&self) -> i32 {
        self.output_window.get_display_mode()
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToDefault`.
    pub fn set_display_mode_to_default(&mut self) {
        self.output_window.set_display_mode_to_default();
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToNever`.
    pub fn set_display_mode_to_never(&mut self) {
        self.output_window.set_display_mode_to_never();
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToAlways`.
    pub fn set_display_mode_to_always(&mut self) {
        self.output_window.set_display_mode_to_always();
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToAlwaysStdErr`.
    pub fn set_display_mode_to_always_std_err(&mut self) {
        self.output_window.set_display_mode_to_always_std_err();
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        OutputWindow::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        OutputWindow::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        OutputWindow::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        OutputWindow::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.output_window.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.output_window.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.output_window.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.output_window.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        OutputWindow::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.output_window.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.output_window.get_m_time()
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.output_window.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.output_window.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.output_window.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.output_window.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.output_window.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.output_window.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.output_window.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.output_window.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.output_window.get_object_description()
    }

    /// VTK: `vtkFileOutputWindow::Initialize`.
    fn initialize(&mut self) {
        self.initialize_with_default_file_name(DEFAULT_FILE_NAME);
    }

    pub(crate) fn initialize_with_default_file_name(&mut self, default_file_name: &str) {
        if self.stream.is_some() {
            return;
        }

        if self.file_name.is_none() {
            self.file_name = Some(PathBuf::from(default_file_name));
        }

        let Some(file_name) = self.file_name.as_ref() else {
            return;
        };

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        if self.append {
            options.append(true);
        } else {
            options.truncate(true);
        }

        if let Ok(file) = options.open(file_name) {
            self.stream = Some(BufWriter::new(file));
        }
    }

    pub(crate) fn write_line(&mut self, text: &str) {
        if let Some(stream) = self.stream.as_mut() {
            let _ = writeln!(stream, "{text}");
            if self.flush {
                let _ = stream.flush();
            }
        }
    }

    pub(crate) fn is_stream_open(&self) -> bool {
        self.stream.is_some()
    }
}

impl Default for FileOutputWindow {
    fn default() -> Self {
        Self::new()
    }
}
