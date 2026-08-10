use std::{ffi::c_void, ptr};

use crate::common::{
    core::{Object, UnsignedCharArray, VtkIdType, VtkMTimeType},
    data_model::ImageData,
};

pub const VTK_CURSOR_DEFAULT: i32 = 0;
pub const VTK_CURSOR_ARROW: i32 = 1;
pub const VTK_CURSOR_SIZENE: i32 = 2;
pub const VTK_CURSOR_SIZENW: i32 = 3;
pub const VTK_CURSOR_SIZESW: i32 = 4;
pub const VTK_CURSOR_SIZESE: i32 = 5;
pub const VTK_CURSOR_SIZENS: i32 = 6;
pub const VTK_CURSOR_SIZEWE: i32 = 7;
pub const VTK_CURSOR_SIZEALL: i32 = 8;
pub const VTK_CURSOR_HAND: i32 = 9;
pub const VTK_CURSOR_CROSSHAIR: i32 = 10;
pub const VTK_CURSOR_CUSTOM: i32 = 11;

/// VTK: `vtkWindow`.
#[derive(Debug, PartialEq)]
pub struct Window {
    object: Object,
    window_name: Option<String>,
    size: [i32; 2],
    position: [i32; 2],
    mapped: bool,
    show_window: bool,
    use_off_screen_buffers: bool,
    erase: bool,
    double_buffer: bool,
    dpi: i32,
    borders: bool,
    current_cursor: i32,
    cursor_file_name: Option<String>,
    tile_viewport: [f64; 4],
    tile_size: [i32; 2],
    tile_scale: [i32; 2],
}

impl Window {
    /// VTK: `vtkWindow::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkWindow"),
            window_name: Some("Visualization Toolkit".to_string()),
            size: [0, 0],
            position: [0, 0],
            mapped: false,
            show_window: true,
            use_off_screen_buffers: false,
            erase: true,
            double_buffer: false,
            dpi: 72,
            borders: true,
            current_cursor: VTK_CURSOR_DEFAULT,
            cursor_file_name: None,
            tile_viewport: [0.0, 0.0, 1.0, 1.0],
            tile_size: [0, 0],
            tile_scale: [1, 1],
        }
    }

    /// VTK: `vtkWindow::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nErase: {}\nWindow Name: {}\nPosition: ({}, {})\nSize: ({}, {})\nMapped: {}\nShowWindow: {}\nUseOffScreenBuffers: {}\nDouble Buffered: {}\nDPI: {}\nBorders: {}\nCurrent Cursor: {}\nCursorFileName: {}\nTileScale: ({}, {})\nTileViewport: ({}, {}, {}, {})",
            self.object.get_class_name(),
            if self.erase { "On" } else { "Off" },
            self.window_name.as_deref().unwrap_or("(none)"),
            self.position[0],
            self.position[1],
            self.size[0],
            self.size[1],
            self.mapped as i32,
            self.show_window,
            self.use_off_screen_buffers,
            self.double_buffer,
            self.dpi,
            if self.borders { "On" } else { "Off" },
            self.current_cursor,
            self.cursor_file_name.as_deref().unwrap_or("(none)"),
            self.tile_scale[0],
            self.tile_scale[1],
            self.tile_viewport[0],
            self.tile_viewport[1],
            self.tile_viewport[2],
            self.tile_viewport[3],
        )
    }

    /// VTK: `vtkWindow::SetDisplayId`.
    pub fn set_display_id(&mut self, _display_id: *mut c_void) {}

    /// VTK: `vtkWindow::SetWindowId`.
    pub fn set_window_id(&mut self, _window_id: *mut c_void) {}

    /// VTK: `vtkWindow::SetParentId`.
    pub fn set_parent_id(&mut self, _parent_id: *mut c_void) {}

    /// VTK: `vtkWindow::GetGenericDisplayId`.
    pub fn get_generic_display_id(&self) -> *mut c_void {
        ptr::null_mut()
    }

    /// VTK: `vtkWindow::GetGenericWindowId`.
    pub fn get_generic_window_id(&self) -> *mut c_void {
        ptr::null_mut()
    }

    /// VTK: `vtkWindow::GetGenericParentId`.
    pub fn get_generic_parent_id(&self) -> *mut c_void {
        ptr::null_mut()
    }

    /// VTK: `vtkWindow::GetGenericContext`.
    pub fn get_generic_context(&self) -> *mut c_void {
        ptr::null_mut()
    }

    /// VTK: `vtkWindow::GetGenericDrawable`.
    pub fn get_generic_drawable(&self) -> *mut c_void {
        ptr::null_mut()
    }

    /// VTK: `vtkWindow::SetWindowInfo`.
    pub fn set_window_info(&mut self, _info: Option<&str>) {}

    /// VTK: `vtkWindow::SetParentInfo`.
    pub fn set_parent_info(&mut self, _info: Option<&str>) {}

    /// VTK: `vtkWindow::EnsureDisplay`.
    pub fn ensure_display(&self) -> bool {
        true
    }

    /// VTK: `vtkWindow::GetPosition`.
    pub fn get_position(&self) -> [i32; 2] {
        self.position
    }

    /// VTK: `vtkWindow::SetPosition`.
    pub fn set_position(&mut self, x: i32, y: i32) {
        if self.position != [x, y] {
            self.modified();
            self.position = [x, y];
        }
    }

    /// VTK: `vtkWindow::SetPosition`.
    pub fn set_position_array(&mut self, position: [i32; 2]) {
        self.set_position(position[0], position[1]);
    }

    /// VTK: `vtkWindow::GetSize`.
    pub fn get_size(&mut self) -> [i32; 2] {
        self.tile_size[0] = self.size[0] * self.tile_scale[0];
        self.tile_size[1] = self.size[1] * self.tile_scale[1];
        self.tile_size
    }

    /// VTK: `vtkWindow::SetSize`.
    pub fn set_size(&mut self, width: i32, height: i32) {
        if self.size != [width, height] {
            self.size = [width, height];
            self.modified();
        }
    }

    /// VTK: `vtkWindow::SetSize`.
    pub fn set_size_array(&mut self, size: [i32; 2]) {
        self.set_size(size[0], size[1]);
    }

    /// VTK: `vtkWindow::GetActualSize`.
    pub fn get_actual_size(&mut self) -> [i32; 2] {
        self.get_size();
        self.size
    }

    /// VTK: `vtkWindow::GetScreenSize`.
    pub fn get_screen_size(&self) -> Option<[i32; 2]> {
        None
    }

    /// VTK: `vtkWindow::GetMapped`.
    pub fn get_mapped(&self) -> bool {
        self.mapped
    }

    /// VTK: `vtkWindow::SetShowWindow`.
    pub fn set_show_window(&mut self, show_window: bool) {
        if self.show_window != show_window {
            self.show_window = show_window;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetShowWindow`.
    pub fn get_show_window(&self) -> bool {
        self.show_window
    }

    /// VTK: `vtkWindow::ShowWindowOn`.
    pub fn show_window_on(&mut self) {
        self.set_show_window(true);
    }

    /// VTK: `vtkWindow::ShowWindowOff`.
    pub fn show_window_off(&mut self) {
        self.set_show_window(false);
    }

    /// VTK: `vtkWindow::SetUseOffScreenBuffers`.
    pub fn set_use_off_screen_buffers(&mut self, use_off_screen_buffers: bool) {
        if self.use_off_screen_buffers != use_off_screen_buffers {
            self.use_off_screen_buffers = use_off_screen_buffers;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetUseOffScreenBuffers`.
    pub fn get_use_off_screen_buffers(&self) -> bool {
        self.use_off_screen_buffers
    }

    /// VTK: `vtkWindow::UseOffScreenBuffersOn`.
    pub fn use_off_screen_buffers_on(&mut self) {
        self.set_use_off_screen_buffers(true);
    }

    /// VTK: `vtkWindow::UseOffScreenBuffersOff`.
    pub fn use_off_screen_buffers_off(&mut self) {
        self.set_use_off_screen_buffers(false);
    }

    /// VTK: `vtkWindow::SetErase`.
    pub fn set_erase(&mut self, erase: bool) {
        if self.erase != erase {
            self.erase = erase;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetErase`.
    pub fn get_erase(&self) -> bool {
        self.erase
    }

    /// VTK: `vtkWindow::EraseOn`.
    pub fn erase_on(&mut self) {
        self.set_erase(true);
    }

    /// VTK: `vtkWindow::EraseOff`.
    pub fn erase_off(&mut self) {
        self.set_erase(false);
    }

    /// VTK: `vtkWindow::SetDoubleBuffer`.
    pub fn set_double_buffer(&mut self, double_buffer: bool) {
        if self.double_buffer != double_buffer {
            self.double_buffer = double_buffer;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetDoubleBuffer`.
    pub fn get_double_buffer(&self) -> bool {
        self.double_buffer
    }

    /// VTK: `vtkWindow::DoubleBufferOn`.
    pub fn double_buffer_on(&mut self) {
        self.set_double_buffer(true);
    }

    /// VTK: `vtkWindow::DoubleBufferOff`.
    pub fn double_buffer_off(&mut self) {
        self.set_double_buffer(false);
    }

    /// VTK: `vtkWindow::GetWindowName`.
    pub fn get_window_name(&self) -> Option<&str> {
        self.window_name.as_deref()
    }

    /// VTK: `vtkWindow::SetWindowName`.
    pub fn set_window_name(&mut self, window_name: Option<&str>) {
        let window_name = window_name.map(str::to_string);
        if self.window_name != window_name {
            self.window_name = window_name;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::SetIcon`.
    pub fn set_icon(&mut self, _icon: *mut ImageData) {}

    /// VTK: `vtkWindow::Render`.
    pub fn render(&mut self) {}

    /// VTK: `vtkWindow::ReleaseGraphicsResources`.
    pub fn release_graphics_resources(&mut self, _window: *mut Window) {}

    /// VTK: `vtkWindow::GetPixelData`.
    pub fn get_pixel_data(
        &self,
        _x: i32,
        _y: i32,
        _x2: i32,
        _y2: i32,
        _front: i32,
        _right: i32,
    ) -> *mut u8 {
        ptr::null_mut()
    }

    /// VTK: `vtkWindow::GetPixelData`.
    pub fn get_pixel_data_into(
        &self,
        _x: i32,
        _y: i32,
        _x2: i32,
        _y2: i32,
        _front: i32,
        _data: *mut UnsignedCharArray,
        _right: i32,
    ) -> i32 {
        0
    }

    /// VTK: `vtkWindow::GetDPI`.
    pub fn get_dpi(&self) -> i32 {
        self.dpi
    }

    /// VTK: `vtkWindow::SetDPI`.
    pub fn set_dpi(&mut self, dpi: i32) {
        let dpi = dpi.clamp(1, i32::MAX);
        if self.dpi != dpi {
            self.dpi = dpi;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::DetectDPI`.
    pub fn detect_dpi(&mut self) -> bool {
        false
    }

    /// VTK: `vtkWindow::SetOffScreenRendering`.
    pub fn set_off_screen_rendering(&mut self, val: bool) {
        self.set_show_window(!val);
        self.set_use_off_screen_buffers(val);
    }

    /// VTK: `vtkWindow::OffScreenRenderingOn`.
    pub fn off_screen_rendering_on(&mut self) {
        self.set_off_screen_rendering(true);
    }

    /// VTK: `vtkWindow::OffScreenRenderingOff`.
    pub fn off_screen_rendering_off(&mut self) {
        self.set_off_screen_rendering(false);
    }

    /// VTK: `vtkWindow::GetOffScreenRendering`.
    pub fn get_off_screen_rendering(&self) -> bool {
        !self.get_show_window()
    }

    /// VTK: `vtkWindow::MakeCurrent`.
    pub fn make_current(&mut self) {}

    /// VTK: `vtkWindow::ReleaseCurrent`.
    pub fn release_current(&mut self) {}

    /// VTK: `vtkWindow::SetTileScale`.
    pub fn set_tile_scale(&mut self, x: i32, y: i32) {
        if self.tile_scale != [x, y] {
            self.tile_scale = [x, y];
            self.modified();
        }
    }

    /// VTK: `vtkWindow::SetTileScale`.
    pub fn set_tile_scale_array(&mut self, tile_scale: [i32; 2]) {
        self.set_tile_scale(tile_scale[0], tile_scale[1]);
    }

    /// VTK: `vtkWindow::SetTileScale`.
    pub fn set_tile_scale_uniform(&mut self, s: i32) {
        self.set_tile_scale(s, s);
    }

    /// VTK: `vtkWindow::GetTileScale`.
    pub fn get_tile_scale(&self) -> [i32; 2] {
        self.tile_scale
    }

    /// VTK: `vtkWindow::SetTileViewport`.
    pub fn set_tile_viewport(&mut self, xmin: f64, ymin: f64, xmax: f64, ymax: f64) {
        let tile_viewport = [xmin, ymin, xmax, ymax];
        if self.tile_viewport != tile_viewport {
            self.tile_viewport = tile_viewport;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::SetTileViewport`.
    pub fn set_tile_viewport_array(&mut self, tile_viewport: [f64; 4]) {
        self.set_tile_viewport(
            tile_viewport[0],
            tile_viewport[1],
            tile_viewport[2],
            tile_viewport[3],
        );
    }

    /// VTK: `vtkWindow::GetTileViewport`.
    pub fn get_tile_viewport(&self) -> [f64; 4] {
        self.tile_viewport
    }

    /// VTK: `vtkWindow::SetBorders`.
    pub fn set_borders(&mut self, borders: bool) {
        if self.borders != borders {
            self.borders = borders;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetBorders`.
    pub fn get_borders(&self) -> bool {
        self.borders
    }

    /// VTK: `vtkWindow::BordersOn`.
    pub fn borders_on(&mut self) {
        self.set_borders(true);
    }

    /// VTK: `vtkWindow::BordersOff`.
    pub fn borders_off(&mut self) {
        self.set_borders(false);
    }

    /// VTK: `vtkWindow::HideCursor`.
    pub fn hide_cursor(&mut self) {}

    /// VTK: `vtkWindow::ShowCursor`.
    pub fn show_cursor(&mut self) {}

    /// VTK: `vtkWindow::SetCursorPosition`.
    pub fn set_cursor_position(&mut self, _x: i32, _y: i32) {}

    /// VTK: `vtkWindow::SetCurrentCursor`.
    pub fn set_current_cursor(&mut self, current_cursor: i32) {
        if self.current_cursor != current_cursor {
            self.current_cursor = current_cursor;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetCurrentCursor`.
    pub fn get_current_cursor(&self) -> i32 {
        self.current_cursor
    }

    /// VTK: `vtkWindow::SetCursorFileName`.
    pub fn set_cursor_file_name(&mut self, cursor_file_name: Option<&str>) {
        let cursor_file_name = cursor_file_name.map(str::to_string);
        if self.cursor_file_name != cursor_file_name {
            self.cursor_file_name = cursor_file_name;
            self.modified();
        }
    }

    /// VTK: `vtkWindow::GetCursorFileName`.
    pub fn get_cursor_file_name(&self) -> Option<&str> {
        self.cursor_file_name.as_deref()
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

    /// VTK: `vtkWindow::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkWindow" || Object::is_type_of(name)
    }

    /// VTK: `vtkWindow::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkWindow::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkWindow" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkWindow::GetNumberOfGenerationsFromBase`.
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
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}
