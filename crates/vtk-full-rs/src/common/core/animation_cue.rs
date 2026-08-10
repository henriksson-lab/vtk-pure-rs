use crate::common::core::{Object, VtkIdType, VtkMTimeType};

pub const TIMEMODE_NORMALIZED: i32 = 0;
pub const TIMEMODE_RELATIVE: i32 = 1;

const UNINITIALIZED: i32 = 0;
const INACTIVE: i32 = 1;
const ACTIVE: i32 = 2;

/// VTK: `vtkAnimationCue::PlayDirection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayDirection {
    Backward,
    Forward,
}

/// VTK: `vtkAnimationCue::AnimationCueInfo`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationCueInfo {
    pub start_time: f64,
    pub end_time: f64,
    pub animation_time: f64,
    pub delta_time: f64,
    pub clock_time: f64,
}

/// VTK: `vtkAnimationCue`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationCue {
    object: Object,
    start_time: f64,
    end_time: f64,
    time_mode: i32,
    direction: PlayDirection,
    animation_time: f64,
    delta_time: f64,
    clock_time: f64,
    cue_state: i32,
}

impl AnimationCue {
    /// VTK: `vtkAnimationCue::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkAnimationCue"),
            start_time: 0.0,
            end_time: 0.0,
            time_mode: TIMEMODE_RELATIVE,
            direction: PlayDirection::Forward,
            animation_time: 0.0,
            delta_time: 0.0,
            clock_time: 0.0,
            cue_state: UNINITIALIZED,
        }
    }

    /// VTK: `vtkAnimationCue::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nStartTime: {}\nEndTime: {}\nCueState: {}\nTimeMode: {}\nAnimationTime: {}\nDeltaTime: {}\nClockTime: {}\nDirection: {}",
            self.object.get_class_name(),
            self.start_time,
            self.end_time,
            self.cue_state,
            self.time_mode,
            self.animation_time,
            self.delta_time,
            self.clock_time,
            match self.direction {
                PlayDirection::Backward => "Backward",
                PlayDirection::Forward => "Forward",
            },
        )
    }

    /// VTK: `vtkAnimationCue::SetTimeMode`.
    pub fn set_time_mode(&mut self, mode: i32) {
        self.time_mode = mode;
    }

    /// VTK: `vtkAnimationCue::GetTimeMode`.
    pub fn get_time_mode(&self) -> i32 {
        self.time_mode
    }

    /// VTK: `vtkAnimationCue::SetTimeModeToRelative`.
    pub fn set_time_mode_to_relative(&mut self) {
        self.set_time_mode(TIMEMODE_RELATIVE);
    }

    /// VTK: `vtkAnimationCue::SetTimeModeToNormalized`.
    pub fn set_time_mode_to_normalized(&mut self) {
        self.set_time_mode(TIMEMODE_NORMALIZED);
    }

    /// VTK: `vtkAnimationCue::SetStartTime`.
    pub fn set_start_time(&mut self, start_time: f64) {
        if self.start_time != start_time {
            self.start_time = start_time;
            self.modified();
        }
    }

    /// VTK: `vtkAnimationCue::GetStartTime`.
    pub fn get_start_time(&self) -> f64 {
        self.start_time
    }

    /// VTK: `vtkAnimationCue::SetEndTime`.
    pub fn set_end_time(&mut self, end_time: f64) {
        if self.end_time != end_time {
            self.end_time = end_time;
            self.modified();
        }
    }

    /// VTK: `vtkAnimationCue::GetEndTime`.
    pub fn get_end_time(&self) -> f64 {
        self.end_time
    }

    /// VTK: `vtkAnimationCue::Tick`.
    pub fn tick(&mut self, currenttime: f64, deltatime: f64, clocktime: f64) {
        if self.check_start_cue(currenttime) {
            self.cue_state = ACTIVE;
            self.start_cue_internal();
        }

        if self.cue_state == ACTIVE && currenttime <= self.end_time {
            self.tick_internal(currenttime, deltatime, clocktime);
        }

        if self.check_end_cue(currenttime) {
            self.end_cue_internal();
            self.cue_state = INACTIVE;
        }
    }

    /// VTK: `vtkAnimationCue::Initialize`.
    pub fn initialize(&mut self) {
        self.cue_state = UNINITIALIZED;
    }

    /// VTK: `vtkAnimationCue::Finalize`.
    pub fn finalize(&mut self) {
        if self.cue_state == ACTIVE {
            self.end_cue_internal();
        }
        self.cue_state = INACTIVE;
    }

    /// VTK: `vtkAnimationCue::GetAnimationTime`.
    pub fn get_animation_time(&self) -> f64 {
        self.animation_time
    }

    /// VTK: `vtkAnimationCue::GetDeltaTime`.
    pub fn get_delta_time(&self) -> f64 {
        self.delta_time
    }

    /// VTK: `vtkAnimationCue::GetClockTime`.
    pub fn get_clock_time(&self) -> f64 {
        self.clock_time
    }

    /// VTK: `vtkAnimationCue::SetDirection`.
    pub fn set_direction(&mut self, direction: PlayDirection) {
        if self.direction != direction {
            self.direction = direction;
            self.modified();
        }
    }

    /// VTK: `vtkAnimationCue::GetDirection`.
    pub fn get_direction(&self) -> PlayDirection {
        self.direction
    }

    /// VTK: `vtkAnimationCue::StartCueInternal`.
    pub(crate) fn start_cue_internal(&mut self) {
        let _info = AnimationCueInfo {
            start_time: self.start_time,
            end_time: self.end_time,
            animation_time: 0.0,
            delta_time: 0.0,
            clock_time: 0.0,
        };
    }

    /// VTK: `vtkAnimationCue::TickInternal`.
    pub(crate) fn tick_internal(&mut self, currenttime: f64, deltatime: f64, clocktime: f64) {
        let _info = AnimationCueInfo {
            start_time: self.start_time,
            end_time: self.end_time,
            animation_time: currenttime,
            delta_time: deltatime,
            clock_time: clocktime,
        };

        self.animation_time = currenttime;
        self.delta_time = deltatime;
        self.clock_time = clocktime;

        self.animation_time = 0.0;
        self.delta_time = 0.0;
        self.clock_time = 0.0;
    }

    /// VTK: `vtkAnimationCue::EndCueInternal`.
    pub(crate) fn end_cue_internal(&mut self) {
        let _info = AnimationCueInfo {
            start_time: self.start_time,
            end_time: self.end_time,
            animation_time: self.end_time,
            delta_time: 0.0,
            clock_time: 0.0,
        };
    }

    /// VTK: `vtkAnimationCue::CheckStartCue`.
    pub(crate) fn check_start_cue(&self, currenttime: f64) -> bool {
        if self.direction == PlayDirection::Forward {
            currenttime >= self.start_time && self.cue_state == UNINITIALIZED
        } else {
            currenttime <= self.end_time && self.cue_state == UNINITIALIZED
        }
    }

    /// VTK: `vtkAnimationCue::CheckEndCue`.
    pub(crate) fn check_end_cue(&self, currenttime: f64) -> bool {
        if self.direction == PlayDirection::Forward {
            currenttime >= self.end_time && self.cue_state == ACTIVE
        } else {
            currenttime <= self.start_time && self.cue_state == ACTIVE
        }
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

    /// VTK: `vtkAnimationCue::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkAnimationCue" || Object::is_type_of(name)
    }

    /// VTK: `vtkAnimationCue::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkAnimationCue::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkAnimationCue" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkAnimationCue::GetNumberOfGenerationsFromBase`.
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

impl Default for AnimationCue {
    fn default() -> Self {
        Self::new()
    }
}
