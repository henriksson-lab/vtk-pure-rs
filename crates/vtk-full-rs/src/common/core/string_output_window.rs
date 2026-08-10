use super::{output_window::OutputWindow, vtk_type::VtkMTimeType};

/// VTK: `vtkStringOutputWindow`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringOutputWindow {
    output_window: OutputWindow,
    output: String,
}

impl StringOutputWindow {
    /// VTK: `vtkStringOutputWindow::New`.
    pub fn new() -> Self {
        let mut window = Self {
            output_window: OutputWindow::with_class_name("vtkStringOutputWindow"),
            output: String::new(),
        };
        window.initialize();
        window
    }

    /// VTK: `vtkStringOutputWindow::Initialize`.
    fn initialize(&mut self) {
        self.output.clear();
    }

    /// VTK: `vtkStringOutputWindow::DisplayText`.
    pub fn display_text(&mut self, text: Option<&str>) {
        let Some(text) = text else {
            return;
        };
        self.output.push_str(text);
        self.output.push('\n');
    }

    /// VTK: `vtkStringOutputWindow::GetOutput`.
    pub fn get_output(&self) -> String {
        self.output.clone()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.output_window.get_class_name()
    }

    /// VTK: `vtkStringOutputWindow::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkStringOutputWindow" || OutputWindow::is_type_of(name)
    }

    /// VTK: `vtkStringOutputWindow::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkStringOutputWindow::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkStringOutputWindow" => 0,
            "vtkOutputWindow" => 1,
            "vtkObject" => 2,
            "vtkObjectBase" => 3,
            _ => OutputWindow::get_number_of_generations_from_base_type(name),
        }
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

    /// VTK: `vtkStringOutputWindow::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
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
}

impl Default for StringOutputWindow {
    fn default() -> Self {
        Self::new()
    }
}
