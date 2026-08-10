use std::path::Path;

use crate::common::core::{FileOutputWindow, VtkMTimeType};

const DEFAULT_XML_FILE_NAME: &str = "vtkMessageLog.xml";

/// VTK: `vtkXMLFileOutputWindow`.
#[derive(Debug)]
pub struct XMLFileOutputWindow {
    file_output_window: FileOutputWindow,
}

impl XMLFileOutputWindow {
    /// VTK: `vtkXMLFileOutputWindow::New`.
    pub fn new() -> Self {
        Self {
            file_output_window: FileOutputWindow::with_class_name("vtkXMLFileOutputWindow"),
        }
    }

    /// VTK: `vtkXMLFileOutputWindow::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.file_output_window.get_class_name().to_string()
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayText`.
    pub fn display_text(&mut self, text: Option<&str>) {
        self.display_xml("Text", text);
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayErrorText`.
    pub fn display_error_text(&mut self, text: Option<&str>) {
        self.display_xml("Error", text);
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayWarningText`.
    pub fn display_warning_text(&mut self, text: Option<&str>) {
        self.display_xml("Warning", text);
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayGenericWarningText`.
    pub fn display_generic_warning_text(&mut self, text: Option<&str>) {
        self.display_xml("GenericWarning", text);
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayDebugText`.
    pub fn display_debug_text(&mut self, text: Option<&str>) {
        self.display_xml("Debug", text);
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayTag`.
    pub fn display_tag(&mut self, text: Option<&str>) {
        let Some(text) = text else {
            return;
        };

        self.initialize();
        self.file_output_window.write_line(text);
    }

    /// VTK: `vtkXMLFileOutputWindow::Initialize`.
    pub(crate) fn initialize(&mut self) {
        let needs_xml_declaration =
            !self.file_output_window.is_stream_open() && !self.file_output_window.get_append();
        self.file_output_window
            .initialize_with_default_file_name(DEFAULT_XML_FILE_NAME);
        if needs_xml_declaration {
            self.file_output_window
                .write_line("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>");
        }
    }

    /// VTK: `vtkXMLFileOutputWindow::DisplayXML`.
    pub(crate) fn display_xml(&mut self, tag: &str, text: Option<&str>) {
        let Some(text) = text else {
            return;
        };

        let xml_text = escape_xml_text(text);
        self.initialize();
        self.file_output_window
            .write_line(&format!("<{tag}>{xml_text}</{tag}>"));
    }

    /// VTK: `vtkFileOutputWindow::SetFileName`.
    pub fn set_file_name(&mut self, file_name: Option<&Path>) {
        self.file_output_window.set_file_name(file_name);
    }

    /// VTK: `vtkFileOutputWindow::GetFileName`.
    pub fn get_file_name(&self) -> Option<&Path> {
        self.file_output_window.get_file_name()
    }

    /// VTK: `vtkFileOutputWindow::SetFlush`.
    pub fn set_flush(&mut self, flush: bool) {
        self.file_output_window.set_flush(flush);
    }

    /// VTK: `vtkFileOutputWindow::GetFlush`.
    pub fn get_flush(&self) -> bool {
        self.file_output_window.get_flush()
    }

    /// VTK: `vtkFileOutputWindow::FlushOn`.
    pub fn flush_on(&mut self) {
        self.file_output_window.flush_on();
    }

    /// VTK: `vtkFileOutputWindow::FlushOff`.
    pub fn flush_off(&mut self) {
        self.file_output_window.flush_off();
    }

    /// VTK: `vtkFileOutputWindow::SetAppend`.
    pub fn set_append(&mut self, append: bool) {
        self.file_output_window.set_append(append);
    }

    /// VTK: `vtkFileOutputWindow::GetAppend`.
    pub fn get_append(&self) -> bool {
        self.file_output_window.get_append()
    }

    /// VTK: `vtkFileOutputWindow::AppendOn`.
    pub fn append_on(&mut self) {
        self.file_output_window.append_on();
    }

    /// VTK: `vtkFileOutputWindow::AppendOff`.
    pub fn append_off(&mut self) {
        self.file_output_window.append_off();
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.file_output_window.get_class_name()
    }

    /// VTK: `vtkXMLFileOutputWindow::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkXMLFileOutputWindow" || FileOutputWindow::is_type_of(name)
    }

    /// VTK: `vtkXMLFileOutputWindow::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkXMLFileOutputWindow::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkXMLFileOutputWindow" => 0,
            "vtkFileOutputWindow" => 1,
            "vtkOutputWindow" => 2,
            "vtkObject" => 3,
            "vtkObjectBase" => 4,
            _ => FileOutputWindow::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkXMLFileOutputWindow::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkOutputWindow::PromptUserOn`.
    pub fn prompt_user_on(&mut self) {
        self.file_output_window.prompt_user_on();
    }

    /// VTK: `vtkOutputWindow::PromptUserOff`.
    pub fn prompt_user_off(&mut self) {
        self.file_output_window.prompt_user_off();
    }

    /// VTK: `vtkOutputWindow::SetPromptUser`.
    pub fn set_prompt_user(&mut self, prompt_user: bool) {
        self.file_output_window.set_prompt_user(prompt_user);
    }

    /// VTK: `vtkOutputWindow::GetPromptUser`.
    pub fn get_prompt_user(&self) -> bool {
        self.file_output_window.get_prompt_user()
    }

    /// VTK: `vtkOutputWindow::SetDisplayMode`.
    pub fn set_display_mode(&mut self, display_mode: i32) {
        self.file_output_window.set_display_mode(display_mode);
    }

    /// VTK: `vtkOutputWindow::GetDisplayMode`.
    pub fn get_display_mode(&self) -> i32 {
        self.file_output_window.get_display_mode()
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToDefault`.
    pub fn set_display_mode_to_default(&mut self) {
        self.file_output_window.set_display_mode_to_default();
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToNever`.
    pub fn set_display_mode_to_never(&mut self) {
        self.file_output_window.set_display_mode_to_never();
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToAlways`.
    pub fn set_display_mode_to_always(&mut self) {
        self.file_output_window.set_display_mode_to_always();
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToAlwaysStdErr`.
    pub fn set_display_mode_to_always_std_err(&mut self) {
        self.file_output_window.set_display_mode_to_always_std_err();
    }

    /// VTK: `vtkObject::SetGlobalWarningDisplay`.
    pub fn set_global_warning_display(value: bool) {
        FileOutputWindow::set_global_warning_display(value);
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOn`.
    pub fn global_warning_display_on() {
        FileOutputWindow::global_warning_display_on();
    }

    /// VTK: `vtkObject::GlobalWarningDisplayOff`.
    pub fn global_warning_display_off() {
        FileOutputWindow::global_warning_display_off();
    }

    /// VTK: `vtkObject::GetGlobalWarningDisplay`.
    pub fn get_global_warning_display() -> bool {
        FileOutputWindow::get_global_warning_display()
    }

    /// VTK: `vtkObject::DebugOn`.
    pub fn debug_on(&mut self) {
        self.file_output_window.debug_on();
    }

    /// VTK: `vtkObject::DebugOff`.
    pub fn debug_off(&mut self) {
        self.file_output_window.debug_off();
    }

    /// VTK: `vtkObject::GetDebug`.
    pub fn get_debug(&self) -> bool {
        self.file_output_window.get_debug()
    }

    /// VTK: `vtkObject::SetDebug`.
    pub fn set_debug(&mut self, debug: bool) {
        self.file_output_window.set_debug(debug);
    }

    /// VTK: `vtkObject::BreakOnError`.
    pub fn break_on_error() {
        FileOutputWindow::break_on_error();
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.file_output_window.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.file_output_window.get_m_time()
    }

    /// VTK: `vtkObjectBase::Register`.
    pub fn register(&mut self) {
        self.file_output_window.register();
    }

    /// VTK: `vtkObjectBase::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.file_output_window.unregister()
    }

    /// VTK: `vtkObjectBase::Delete`.
    pub fn delete(&mut self) -> bool {
        self.file_output_window.delete()
    }

    /// VTK: `vtkObjectBase::FastDelete`.
    pub fn fast_delete(&mut self) -> bool {
        self.file_output_window.fast_delete()
    }

    /// VTK: `vtkObjectBase::GetReferenceCount`.
    pub fn get_reference_count(&self) -> i32 {
        self.file_output_window.get_reference_count()
    }

    /// VTK: `vtkObjectBase::SetReferenceCount`.
    pub fn set_reference_count(&mut self, reference_count: i32) {
        self.file_output_window.set_reference_count(reference_count);
    }

    /// VTK: `vtkObject::SetObjectName`.
    pub fn set_object_name(&mut self, object_name: impl Into<String>) {
        self.file_output_window.set_object_name(object_name);
    }

    /// VTK: `vtkObject::GetObjectName`.
    pub fn get_object_name(&self) -> &str {
        self.file_output_window.get_object_name()
    }

    /// VTK: `vtkObject::GetObjectDescription`.
    pub fn get_object_description(&self) -> String {
        self.file_output_window.get_object_description()
    }
}

impl Default for XMLFileOutputWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_xml_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
