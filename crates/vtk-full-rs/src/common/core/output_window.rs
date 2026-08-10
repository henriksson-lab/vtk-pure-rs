use std::io::{self, Write};
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::{object::Object, vtk_type::VtkMTimeType};

static OUTPUT_WINDOW_INSTANCE: OnceLock<Mutex<OutputWindow>> = OnceLock::new();

/// VTK: `vtkOutputWindow::DisplayModes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DisplayMode {
    Default = -1,
    Never = 0,
    Always = 1,
    AlwaysStderr = 2,
}

impl DisplayMode {
    fn clamp(value: i32) -> Self {
        match value {
            i32::MIN..=-2 => Self::Default,
            -1 => Self::Default,
            0 => Self::Never,
            1 => Self::Always,
            2..=i32::MAX => Self::AlwaysStderr,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageType {
    Text,
    Error,
    Warning,
    GenericWarning,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamType {
    Null,
    StdOutput,
    StdError,
}

/// VTK: `vtkOutputWindow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputWindow {
    object: Object,
    prompt_user: bool,
    current_message_type: MessageType,
    display_mode: DisplayMode,
    in_standard_macros: i32,
}

impl OutputWindow {
    /// VTK: `vtkOutputWindow::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkOutputWindow")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            prompt_user: false,
            current_message_type: MessageType::Text,
            display_mode: DisplayMode::Default,
            in_standard_macros: 0,
        }
    }

    /// VTK: `vtkOutputWindow::GetInstance`.
    pub fn get_instance() -> MutexGuard<'static, OutputWindow> {
        OUTPUT_WINDOW_INSTANCE
            .get_or_init(|| Mutex::new(Self::new()))
            .lock()
            .expect("vtkOutputWindow singleton mutex poisoned")
    }

    /// VTK: `vtkOutputWindow::SetInstance`.
    pub fn set_instance(instance: OutputWindow) {
        let mut current = Self::get_instance();
        *current = instance;
    }

    /// VTK: `vtkOutputWindow::DisplayText`.
    pub fn display_text(&mut self, text: Option<&str>) {
        let Some(text) = text else {
            return;
        };

        match self.get_display_stream(self.current_message_type) {
            StreamType::StdOutput => {
                let _ = io::stdout().write_all(text.as_bytes());
            }
            StreamType::StdError => {
                let _ = io::stderr().write_all(text.as_bytes());
            }
            StreamType::Null => {}
        }
    }

    /// VTK: `vtkOutputWindow::DisplayErrorText`.
    pub fn display_error_text(&mut self, text: Option<&str>) {
        let previous = self.current_message_type;
        self.current_message_type = MessageType::Error;
        self.display_text(text);
        self.current_message_type = previous;
    }

    /// VTK: `vtkOutputWindow::DisplayWarningText`.
    pub fn display_warning_text(&mut self, text: Option<&str>) {
        let previous = self.current_message_type;
        self.current_message_type = MessageType::Warning;
        self.display_text(text);
        self.current_message_type = previous;
    }

    /// VTK: `vtkOutputWindow::DisplayGenericWarningText`.
    pub fn display_generic_warning_text(&mut self, text: Option<&str>) {
        let previous = self.current_message_type;
        self.current_message_type = MessageType::GenericWarning;
        self.display_text(text);
        self.current_message_type = previous;
    }

    /// VTK: `vtkOutputWindow::DisplayDebugText`.
    pub fn display_debug_text(&mut self, text: Option<&str>) {
        let previous = self.current_message_type;
        self.current_message_type = MessageType::Debug;
        self.display_text(text);
        self.current_message_type = previous;
    }

    /// VTK: `vtkOutputWindow::PromptUserOn`.
    pub fn prompt_user_on(&mut self) {
        self.set_prompt_user(true);
    }

    /// VTK: `vtkOutputWindow::PromptUserOff`.
    pub fn prompt_user_off(&mut self) {
        self.set_prompt_user(false);
    }

    /// VTK: `vtkOutputWindow::SetPromptUser`.
    pub fn set_prompt_user(&mut self, prompt_user: bool) {
        self.prompt_user = prompt_user;
    }

    /// VTK: `vtkOutputWindow::GetPromptUser`.
    pub fn get_prompt_user(&self) -> bool {
        self.prompt_user
    }

    /// VTK: `vtkOutputWindow::SetDisplayMode`.
    pub fn set_display_mode(&mut self, display_mode: i32) {
        self.display_mode = DisplayMode::clamp(display_mode);
    }

    /// VTK: `vtkOutputWindow::GetDisplayMode`.
    pub fn get_display_mode(&self) -> i32 {
        self.display_mode as i32
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToDefault`.
    pub fn set_display_mode_to_default(&mut self) {
        self.set_display_mode(DisplayMode::Default as i32);
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToNever`.
    pub fn set_display_mode_to_never(&mut self) {
        self.set_display_mode(DisplayMode::Never as i32);
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToAlways`.
    pub fn set_display_mode_to_always(&mut self) {
        self.set_display_mode(DisplayMode::Always as i32);
    }

    /// VTK: `vtkOutputWindow::SetDisplayModeToAlwaysStdErr`.
    pub fn set_display_mode_to_always_std_err(&mut self) {
        self.set_display_mode(DisplayMode::AlwaysStderr as i32);
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkOutputWindow::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkOutputWindow" || Object::is_type_of(name)
    }

    /// VTK: `vtkOutputWindow::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkOutputWindow::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkOutputWindow" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkOutputWindow::GetNumberOfGenerationsFromBase`.
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

    fn get_display_stream(&self, message_type: MessageType) -> StreamType {
        match self.display_mode {
            DisplayMode::Default => {
                let _standard_macro_depth = self.in_standard_macros;
                match message_type {
                    MessageType::Text => StreamType::StdOutput,
                    _ => StreamType::StdError,
                }
            }
            DisplayMode::Always => match message_type {
                MessageType::Text => StreamType::StdOutput,
                _ => StreamType::StdError,
            },
            DisplayMode::AlwaysStderr => StreamType::StdError,
            DisplayMode::Never => StreamType::Null,
        }
    }
}

impl Default for OutputWindow {
    fn default() -> Self {
        Self::new()
    }
}

/// VTK: `vtkOutputWindowDisplayText`.
pub fn vtk_output_window_display_text(message: Option<&str>) {
    OutputWindow::get_instance().display_text(message);
}

/// VTK: `vtkOutputWindowDisplayErrorText`.
pub fn vtk_output_window_display_error_text(message: Option<&str>) {
    OutputWindow::get_instance().display_error_text(message);
}

/// VTK: `vtkOutputWindowDisplayWarningText`.
pub fn vtk_output_window_display_warning_text(message: Option<&str>) {
    OutputWindow::get_instance().display_warning_text(message);
}

/// VTK: `vtkOutputWindowDisplayGenericWarningText`.
pub fn vtk_output_window_display_generic_warning_text(message: Option<&str>) {
    OutputWindow::get_instance().display_generic_warning_text(message);
}

/// VTK: `vtkOutputWindowDisplayDebugText`.
pub fn vtk_output_window_display_debug_text(message: Option<&str>) {
    OutputWindow::get_instance().display_debug_text(message);
}
