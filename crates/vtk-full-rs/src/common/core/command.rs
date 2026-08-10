use super::object_base::ObjectBase;

/// VTK: `vtkCommand::EventIds`.
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventId {
    NoEvent = 0,
    AnyEvent,
    DeleteEvent,
    StartEvent,
    EndEvent,
    RenderEvent,
    ProgressEvent,
    PickEvent,
    StartPickEvent,
    EndPickEvent,
    AbortCheckEvent,
    ExitEvent,
    LeftButtonPressEvent,
    LeftButtonReleaseEvent,
    MiddleButtonPressEvent,
    MiddleButtonReleaseEvent,
    RightButtonPressEvent,
    RightButtonReleaseEvent,
    EnterEvent,
    LeaveEvent,
    KeyPressEvent,
    KeyReleaseEvent,
    CharEvent,
    ExposeEvent,
    ConfigureEvent,
    TimerEvent,
    MouseMoveEvent,
    MouseWheelForwardEvent,
    MouseWheelBackwardEvent,
    ActiveCameraEvent,
    CreateCameraEvent,
    ResetCameraEvent,
    ResetCameraClippingRangeEvent,
    ModifiedEvent,
    WindowLevelEvent,
    StartWindowLevelEvent,
    EndWindowLevelEvent,
    ResetWindowLevelEvent,
    SetOutputEvent,
    ErrorEvent,
    WarningEvent,
    StartInteractionEvent,
    DropFilesEvent,
    UpdateDropLocationEvent,
    InteractionEvent,
    EndInteractionEvent,
    EnableEvent,
    DisableEvent,
    CreateTimerEvent,
    DestroyTimerEvent,
    PlacePointEvent,
    DeletePointEvent,
    PlaceWidgetEvent,
    CursorChangedEvent,
    ExecuteInformationEvent,
    RenderWindowMessageEvent,
    WrongTagEvent,
    StartAnimationCueEvent,
    ResliceAxesChangedEvent,
    AnimationCueTickEvent,
    EndAnimationCueEvent,
    VolumeMapperRenderEndEvent,
    VolumeMapperRenderProgressEvent,
    VolumeMapperRenderStartEvent,
    VolumeMapperComputeGradientsEndEvent,
    VolumeMapperComputeGradientsProgressEvent,
    VolumeMapperComputeGradientsStartEvent,
    WidgetModifiedEvent,
    WidgetValueChangedEvent,
    WidgetActivateEvent,
    ConnectionCreatedEvent,
    ConnectionClosedEvent,
    DomainModifiedEvent,
    PropertyModifiedEvent,
    UpdateEvent,
    RegisterEvent,
    UnRegisterEvent,
    UpdateInformationEvent,
    AnnotationChangedEvent,
    SelectionChangedEvent,
    UpdatePropertyEvent,
    ViewProgressEvent,
    UpdateDataEvent,
    CurrentChangedEvent,
    ComputeVisiblePropBoundsEvent,
    TDxMotionEvent,
    TDxButtonPressEvent,
    TDxButtonReleaseEvent,
    HoverEvent,
    LoadStateEvent,
    SaveStateEvent,
    StateChangedEvent,
    WindowMakeCurrentEvent,
    WindowIsCurrentEvent,
    WindowFrameEvent,
    HighlightEvent,
    WindowSupportsOpenGLEvent,
    WindowIsDirectEvent,
    WindowStereoTypeChangedEvent,
    WindowResizeEvent,
    UncheckedPropertyModifiedEvent,
    UpdateShaderEvent,
    MessageEvent,
    StartSwipeEvent,
    SwipeEvent,
    EndSwipeEvent,
    StartPinchEvent,
    PinchEvent,
    EndPinchEvent,
    StartRotateEvent,
    RotateEvent,
    EndRotateEvent,
    StartPanEvent,
    PanEvent,
    EndPanEvent,
    TapEvent,
    LongTapEvent,
    FourthButtonPressEvent,
    FourthButtonReleaseEvent,
    FifthButtonPressEvent,
    FifthButtonReleaseEvent,
    Move3DEvent,
    Button3DEvent,
    TextEvent,
    LeftButtonDoubleClickEvent,
    MiddleButtonDoubleClickEvent,
    RightButtonDoubleClickEvent,
    MouseWheelLeftEvent,
    MouseWheelRightEvent,
    ViewerMovement3DEvent,
    Menu3DEvent,
    NextPose3DEvent,
    Clip3DEvent,
    PositionProp3DEvent,
    Pick3DEvent,
    Select3DEvent,
    Elevation3DEvent,
    BufferChangedEvent,
    UserEvent = 1000,
}

impl EventId {
    pub fn from_id(id: u64) -> Option<Self> {
        Some(match id {
            0 => Self::NoEvent,
            1 => Self::AnyEvent,
            2 => Self::DeleteEvent,
            3 => Self::StartEvent,
            4 => Self::EndEvent,
            5 => Self::RenderEvent,
            6 => Self::ProgressEvent,
            7 => Self::PickEvent,
            8 => Self::StartPickEvent,
            9 => Self::EndPickEvent,
            10 => Self::AbortCheckEvent,
            11 => Self::ExitEvent,
            12 => Self::LeftButtonPressEvent,
            13 => Self::LeftButtonReleaseEvent,
            14 => Self::MiddleButtonPressEvent,
            15 => Self::MiddleButtonReleaseEvent,
            16 => Self::RightButtonPressEvent,
            17 => Self::RightButtonReleaseEvent,
            18 => Self::EnterEvent,
            19 => Self::LeaveEvent,
            20 => Self::KeyPressEvent,
            21 => Self::KeyReleaseEvent,
            22 => Self::CharEvent,
            23 => Self::ExposeEvent,
            24 => Self::ConfigureEvent,
            25 => Self::TimerEvent,
            26 => Self::MouseMoveEvent,
            27 => Self::MouseWheelForwardEvent,
            28 => Self::MouseWheelBackwardEvent,
            29 => Self::ActiveCameraEvent,
            30 => Self::CreateCameraEvent,
            31 => Self::ResetCameraEvent,
            32 => Self::ResetCameraClippingRangeEvent,
            33 => Self::ModifiedEvent,
            34 => Self::WindowLevelEvent,
            35 => Self::StartWindowLevelEvent,
            36 => Self::EndWindowLevelEvent,
            37 => Self::ResetWindowLevelEvent,
            38 => Self::SetOutputEvent,
            39 => Self::ErrorEvent,
            40 => Self::WarningEvent,
            41 => Self::StartInteractionEvent,
            42 => Self::DropFilesEvent,
            43 => Self::UpdateDropLocationEvent,
            44 => Self::InteractionEvent,
            45 => Self::EndInteractionEvent,
            46 => Self::EnableEvent,
            47 => Self::DisableEvent,
            48 => Self::CreateTimerEvent,
            49 => Self::DestroyTimerEvent,
            50 => Self::PlacePointEvent,
            51 => Self::DeletePointEvent,
            52 => Self::PlaceWidgetEvent,
            53 => Self::CursorChangedEvent,
            54 => Self::ExecuteInformationEvent,
            55 => Self::RenderWindowMessageEvent,
            56 => Self::WrongTagEvent,
            57 => Self::StartAnimationCueEvent,
            58 => Self::ResliceAxesChangedEvent,
            59 => Self::AnimationCueTickEvent,
            60 => Self::EndAnimationCueEvent,
            61 => Self::VolumeMapperRenderEndEvent,
            62 => Self::VolumeMapperRenderProgressEvent,
            63 => Self::VolumeMapperRenderStartEvent,
            64 => Self::VolumeMapperComputeGradientsEndEvent,
            65 => Self::VolumeMapperComputeGradientsProgressEvent,
            66 => Self::VolumeMapperComputeGradientsStartEvent,
            67 => Self::WidgetModifiedEvent,
            68 => Self::WidgetValueChangedEvent,
            69 => Self::WidgetActivateEvent,
            70 => Self::ConnectionCreatedEvent,
            71 => Self::ConnectionClosedEvent,
            72 => Self::DomainModifiedEvent,
            73 => Self::PropertyModifiedEvent,
            74 => Self::UpdateEvent,
            75 => Self::RegisterEvent,
            76 => Self::UnRegisterEvent,
            77 => Self::UpdateInformationEvent,
            78 => Self::AnnotationChangedEvent,
            79 => Self::SelectionChangedEvent,
            80 => Self::UpdatePropertyEvent,
            81 => Self::ViewProgressEvent,
            82 => Self::UpdateDataEvent,
            83 => Self::CurrentChangedEvent,
            84 => Self::ComputeVisiblePropBoundsEvent,
            85 => Self::TDxMotionEvent,
            86 => Self::TDxButtonPressEvent,
            87 => Self::TDxButtonReleaseEvent,
            88 => Self::HoverEvent,
            89 => Self::LoadStateEvent,
            90 => Self::SaveStateEvent,
            91 => Self::StateChangedEvent,
            92 => Self::WindowMakeCurrentEvent,
            93 => Self::WindowIsCurrentEvent,
            94 => Self::WindowFrameEvent,
            95 => Self::HighlightEvent,
            96 => Self::WindowSupportsOpenGLEvent,
            97 => Self::WindowIsDirectEvent,
            98 => Self::WindowStereoTypeChangedEvent,
            99 => Self::WindowResizeEvent,
            100 => Self::UncheckedPropertyModifiedEvent,
            101 => Self::UpdateShaderEvent,
            102 => Self::MessageEvent,
            103 => Self::StartSwipeEvent,
            104 => Self::SwipeEvent,
            105 => Self::EndSwipeEvent,
            106 => Self::StartPinchEvent,
            107 => Self::PinchEvent,
            108 => Self::EndPinchEvent,
            109 => Self::StartRotateEvent,
            110 => Self::RotateEvent,
            111 => Self::EndRotateEvent,
            112 => Self::StartPanEvent,
            113 => Self::PanEvent,
            114 => Self::EndPanEvent,
            115 => Self::TapEvent,
            116 => Self::LongTapEvent,
            117 => Self::FourthButtonPressEvent,
            118 => Self::FourthButtonReleaseEvent,
            119 => Self::FifthButtonPressEvent,
            120 => Self::FifthButtonReleaseEvent,
            121 => Self::Move3DEvent,
            122 => Self::Button3DEvent,
            123 => Self::TextEvent,
            124 => Self::LeftButtonDoubleClickEvent,
            125 => Self::MiddleButtonDoubleClickEvent,
            126 => Self::RightButtonDoubleClickEvent,
            127 => Self::MouseWheelLeftEvent,
            128 => Self::MouseWheelRightEvent,
            129 => Self::ViewerMovement3DEvent,
            130 => Self::Menu3DEvent,
            131 => Self::NextPose3DEvent,
            132 => Self::Clip3DEvent,
            133 => Self::PositionProp3DEvent,
            134 => Self::Pick3DEvent,
            135 => Self::Select3DEvent,
            136 => Self::Elevation3DEvent,
            137 => Self::BufferChangedEvent,
            1000 => Self::UserEvent,
            _ => return None,
        })
    }

    pub fn id(self) -> u64 {
        self as u64
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::NoEvent => "NoEvent",
            Self::AnyEvent => "AnyEvent",
            Self::DeleteEvent => "DeleteEvent",
            Self::StartEvent => "StartEvent",
            Self::EndEvent => "EndEvent",
            Self::RenderEvent => "RenderEvent",
            Self::ProgressEvent => "ProgressEvent",
            Self::PickEvent => "PickEvent",
            Self::StartPickEvent => "StartPickEvent",
            Self::EndPickEvent => "EndPickEvent",
            Self::AbortCheckEvent => "AbortCheckEvent",
            Self::ExitEvent => "ExitEvent",
            Self::LeftButtonPressEvent => "LeftButtonPressEvent",
            Self::LeftButtonReleaseEvent => "LeftButtonReleaseEvent",
            Self::MiddleButtonPressEvent => "MiddleButtonPressEvent",
            Self::MiddleButtonReleaseEvent => "MiddleButtonReleaseEvent",
            Self::RightButtonPressEvent => "RightButtonPressEvent",
            Self::RightButtonReleaseEvent => "RightButtonReleaseEvent",
            Self::EnterEvent => "EnterEvent",
            Self::LeaveEvent => "LeaveEvent",
            Self::KeyPressEvent => "KeyPressEvent",
            Self::KeyReleaseEvent => "KeyReleaseEvent",
            Self::CharEvent => "CharEvent",
            Self::ExposeEvent => "ExposeEvent",
            Self::ConfigureEvent => "ConfigureEvent",
            Self::TimerEvent => "TimerEvent",
            Self::MouseMoveEvent => "MouseMoveEvent",
            Self::MouseWheelForwardEvent => "MouseWheelForwardEvent",
            Self::MouseWheelBackwardEvent => "MouseWheelBackwardEvent",
            Self::ActiveCameraEvent => "ActiveCameraEvent",
            Self::CreateCameraEvent => "CreateCameraEvent",
            Self::ResetCameraEvent => "ResetCameraEvent",
            Self::ResetCameraClippingRangeEvent => "ResetCameraClippingRangeEvent",
            Self::ModifiedEvent => "ModifiedEvent",
            Self::WindowLevelEvent => "WindowLevelEvent",
            Self::StartWindowLevelEvent => "StartWindowLevelEvent",
            Self::EndWindowLevelEvent => "EndWindowLevelEvent",
            Self::ResetWindowLevelEvent => "ResetWindowLevelEvent",
            Self::SetOutputEvent => "SetOutputEvent",
            Self::ErrorEvent => "ErrorEvent",
            Self::WarningEvent => "WarningEvent",
            Self::StartInteractionEvent => "StartInteractionEvent",
            Self::DropFilesEvent => "DropFilesEvent",
            Self::UpdateDropLocationEvent => "UpdateDropLocationEvent",
            Self::InteractionEvent => "InteractionEvent",
            Self::EndInteractionEvent => "EndInteractionEvent",
            Self::EnableEvent => "EnableEvent",
            Self::DisableEvent => "DisableEvent",
            Self::CreateTimerEvent => "CreateTimerEvent",
            Self::DestroyTimerEvent => "DestroyTimerEvent",
            Self::PlacePointEvent => "PlacePointEvent",
            Self::DeletePointEvent => "DeletePointEvent",
            Self::PlaceWidgetEvent => "PlaceWidgetEvent",
            Self::CursorChangedEvent => "CursorChangedEvent",
            Self::ExecuteInformationEvent => "ExecuteInformationEvent",
            Self::RenderWindowMessageEvent => "RenderWindowMessageEvent",
            Self::WrongTagEvent => "WrongTagEvent",
            Self::StartAnimationCueEvent => "StartAnimationCueEvent",
            Self::ResliceAxesChangedEvent => "ResliceAxesChangedEvent",
            Self::AnimationCueTickEvent => "AnimationCueTickEvent",
            Self::EndAnimationCueEvent => "EndAnimationCueEvent",
            Self::VolumeMapperRenderEndEvent => "VolumeMapperRenderEndEvent",
            Self::VolumeMapperRenderProgressEvent => "VolumeMapperRenderProgressEvent",
            Self::VolumeMapperRenderStartEvent => "VolumeMapperRenderStartEvent",
            Self::VolumeMapperComputeGradientsEndEvent => "VolumeMapperComputeGradientsEndEvent",
            Self::VolumeMapperComputeGradientsProgressEvent => {
                "VolumeMapperComputeGradientsProgressEvent"
            }
            Self::VolumeMapperComputeGradientsStartEvent => {
                "VolumeMapperComputeGradientsStartEvent"
            }
            Self::WidgetModifiedEvent => "WidgetModifiedEvent",
            Self::WidgetValueChangedEvent => "WidgetValueChangedEvent",
            Self::WidgetActivateEvent => "WidgetActivateEvent",
            Self::ConnectionCreatedEvent => "ConnectionCreatedEvent",
            Self::ConnectionClosedEvent => "ConnectionClosedEvent",
            Self::DomainModifiedEvent => "DomainModifiedEvent",
            Self::PropertyModifiedEvent => "PropertyModifiedEvent",
            Self::UpdateEvent => "UpdateEvent",
            Self::RegisterEvent => "RegisterEvent",
            Self::UnRegisterEvent => "UnRegisterEvent",
            Self::UpdateInformationEvent => "UpdateInformationEvent",
            Self::AnnotationChangedEvent => "AnnotationChangedEvent",
            Self::SelectionChangedEvent => "SelectionChangedEvent",
            Self::UpdatePropertyEvent => "UpdatePropertyEvent",
            Self::ViewProgressEvent => "ViewProgressEvent",
            Self::UpdateDataEvent => "UpdateDataEvent",
            Self::CurrentChangedEvent => "CurrentChangedEvent",
            Self::ComputeVisiblePropBoundsEvent => "ComputeVisiblePropBoundsEvent",
            Self::TDxMotionEvent => "TDxMotionEvent",
            Self::TDxButtonPressEvent => "TDxButtonPressEvent",
            Self::TDxButtonReleaseEvent => "TDxButtonReleaseEvent",
            Self::HoverEvent => "HoverEvent",
            Self::LoadStateEvent => "LoadStateEvent",
            Self::SaveStateEvent => "SaveStateEvent",
            Self::StateChangedEvent => "StateChangedEvent",
            Self::WindowMakeCurrentEvent => "WindowMakeCurrentEvent",
            Self::WindowIsCurrentEvent => "WindowIsCurrentEvent",
            Self::WindowFrameEvent => "WindowFrameEvent",
            Self::HighlightEvent => "HighlightEvent",
            Self::WindowSupportsOpenGLEvent => "WindowSupportsOpenGLEvent",
            Self::WindowIsDirectEvent => "WindowIsDirectEvent",
            Self::WindowStereoTypeChangedEvent => "WindowStereoTypeChangedEvent",
            Self::WindowResizeEvent => "WindowResizeEvent",
            Self::UncheckedPropertyModifiedEvent => "UncheckedPropertyModifiedEvent",
            Self::UpdateShaderEvent => "UpdateShaderEvent",
            Self::MessageEvent => "MessageEvent",
            Self::StartSwipeEvent => "StartSwipeEvent",
            Self::SwipeEvent => "SwipeEvent",
            Self::EndSwipeEvent => "EndSwipeEvent",
            Self::StartPinchEvent => "StartPinchEvent",
            Self::PinchEvent => "PinchEvent",
            Self::EndPinchEvent => "EndPinchEvent",
            Self::StartRotateEvent => "StartRotateEvent",
            Self::RotateEvent => "RotateEvent",
            Self::EndRotateEvent => "EndRotateEvent",
            Self::StartPanEvent => "StartPanEvent",
            Self::PanEvent => "PanEvent",
            Self::EndPanEvent => "EndPanEvent",
            Self::TapEvent => "TapEvent",
            Self::LongTapEvent => "LongTapEvent",
            Self::FourthButtonPressEvent => "FourthButtonPressEvent",
            Self::FourthButtonReleaseEvent => "FourthButtonReleaseEvent",
            Self::FifthButtonPressEvent => "FifthButtonPressEvent",
            Self::FifthButtonReleaseEvent => "FifthButtonReleaseEvent",
            Self::Move3DEvent => "Move3DEvent",
            Self::Button3DEvent => "Button3DEvent",
            Self::TextEvent => "TextEvent",
            Self::LeftButtonDoubleClickEvent => "LeftButtonDoubleClickEvent",
            Self::MiddleButtonDoubleClickEvent => "MiddleButtonDoubleClickEvent",
            Self::RightButtonDoubleClickEvent => "RightButtonDoubleClickEvent",
            Self::MouseWheelLeftEvent => "MouseWheelLeftEvent",
            Self::MouseWheelRightEvent => "MouseWheelRightEvent",
            Self::ViewerMovement3DEvent => "ViewerMovement3DEvent",
            Self::Menu3DEvent => "Menu3DEvent",
            Self::NextPose3DEvent => "NextPose3DEvent",
            Self::Clip3DEvent => "Clip3DEvent",
            Self::PositionProp3DEvent => "PositionProp3DEvent",
            Self::Pick3DEvent => "Pick3DEvent",
            Self::Select3DEvent => "Select3DEvent",
            Self::Elevation3DEvent => "Elevation3DEvent",
            Self::BufferChangedEvent => "BufferChangedEvent",
            Self::UserEvent => "UserEvent",
        }
    }
}

/// VTK: `vtkCommand`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    base: ObjectBase,
    abort_flag: bool,
    passive_observer: bool,
}

impl Command {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self::with_class_name("vtkCommand")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        let mut command = Self {
            base: ObjectBase::with_class_name(class_name),
            abort_flag: false,
            passive_observer: false,
        };
        command.base.initialize_object_base();
        command
    }

    /// VTK: `vtkCommand::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.base.unregister()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.base.get_class_name()
    }

    /// VTK: `vtkCommand::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCommand" || ObjectBase::is_type_of(name)
    }

    /// VTK: `vtkCommand::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCommand::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkCommand" => 0,
            "vtkObjectBase" => 1,
            _ => ObjectBase::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkCommand::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkCommand::GetStringFromEventId`.
    pub fn get_string_from_event_id(event: u64) -> &'static str {
        EventId::from_id(event).map_or("NoEvent", EventId::name)
    }

    /// VTK: `vtkCommand::GetEventIdFromString`.
    pub fn get_event_id_from_string(event: Option<&str>) -> u64 {
        let Some(event) = event else {
            return EventId::NoEvent.id();
        };
        ALL_EVENTS
            .iter()
            .find_map(|candidate| (candidate.name() == event).then_some(candidate.id()))
            .unwrap_or(EventId::NoEvent.id())
    }

    /// VTK: `vtkCommand::EventHasData`.
    pub fn event_has_data(event: u64) -> bool {
        matches!(
            EventId::from_id(event),
            Some(
                EventId::Button3DEvent
                    | EventId::Move3DEvent
                    | EventId::ViewerMovement3DEvent
                    | EventId::Menu3DEvent
                    | EventId::NextPose3DEvent
                    | EventId::Clip3DEvent
                    | EventId::PositionProp3DEvent
                    | EventId::Pick3DEvent
                    | EventId::Select3DEvent
                    | EventId::Elevation3DEvent
            )
        )
    }

    /// VTK: `vtkCommand::SetAbortFlag`.
    pub fn set_abort_flag(&mut self, flag: bool) {
        self.abort_flag = flag;
    }

    /// VTK: `vtkCommand::GetAbortFlag`.
    pub fn get_abort_flag(&self) -> bool {
        self.abort_flag
    }

    /// VTK: `vtkCommand::AbortFlagOn`.
    pub fn abort_flag_on(&mut self) {
        self.set_abort_flag(true);
    }

    /// VTK: `vtkCommand::AbortFlagOff`.
    pub fn abort_flag_off(&mut self) {
        self.set_abort_flag(false);
    }

    /// VTK: `vtkCommand::SetPassiveObserver`.
    pub fn set_passive_observer(&mut self, flag: bool) {
        self.passive_observer = flag;
    }

    /// VTK: `vtkCommand::GetPassiveObserver`.
    pub fn get_passive_observer(&self) -> bool {
        self.passive_observer
    }

    /// VTK: `vtkCommand::PassiveObserverOn`.
    pub fn passive_observer_on(&mut self) {
        self.set_passive_observer(true);
    }

    /// VTK: `vtkCommand::PassiveObserverOff`.
    pub fn passive_observer_off(&mut self) {
        self.set_passive_observer(false);
    }

    /// VTK: `vtkCommand::GetDebugClassName`.
    pub fn get_debug_class_name(&self) -> &'static str {
        "vtkCommand or subclass"
    }
}

const ALL_EVENTS: &[EventId] = &[
    EventId::AnyEvent,
    EventId::DeleteEvent,
    EventId::StartEvent,
    EventId::EndEvent,
    EventId::RenderEvent,
    EventId::ProgressEvent,
    EventId::PickEvent,
    EventId::StartPickEvent,
    EventId::EndPickEvent,
    EventId::AbortCheckEvent,
    EventId::ExitEvent,
    EventId::LeftButtonPressEvent,
    EventId::LeftButtonReleaseEvent,
    EventId::MiddleButtonPressEvent,
    EventId::MiddleButtonReleaseEvent,
    EventId::RightButtonPressEvent,
    EventId::RightButtonReleaseEvent,
    EventId::EnterEvent,
    EventId::LeaveEvent,
    EventId::KeyPressEvent,
    EventId::KeyReleaseEvent,
    EventId::CharEvent,
    EventId::ExposeEvent,
    EventId::ConfigureEvent,
    EventId::TimerEvent,
    EventId::MouseMoveEvent,
    EventId::MouseWheelForwardEvent,
    EventId::MouseWheelBackwardEvent,
    EventId::ActiveCameraEvent,
    EventId::CreateCameraEvent,
    EventId::ResetCameraEvent,
    EventId::ResetCameraClippingRangeEvent,
    EventId::ModifiedEvent,
    EventId::WindowLevelEvent,
    EventId::StartWindowLevelEvent,
    EventId::EndWindowLevelEvent,
    EventId::ResetWindowLevelEvent,
    EventId::SetOutputEvent,
    EventId::ErrorEvent,
    EventId::WarningEvent,
    EventId::StartInteractionEvent,
    EventId::DropFilesEvent,
    EventId::UpdateDropLocationEvent,
    EventId::InteractionEvent,
    EventId::EndInteractionEvent,
    EventId::EnableEvent,
    EventId::DisableEvent,
    EventId::CreateTimerEvent,
    EventId::DestroyTimerEvent,
    EventId::PlacePointEvent,
    EventId::DeletePointEvent,
    EventId::PlaceWidgetEvent,
    EventId::CursorChangedEvent,
    EventId::ExecuteInformationEvent,
    EventId::RenderWindowMessageEvent,
    EventId::WrongTagEvent,
    EventId::StartAnimationCueEvent,
    EventId::ResliceAxesChangedEvent,
    EventId::AnimationCueTickEvent,
    EventId::EndAnimationCueEvent,
    EventId::VolumeMapperRenderEndEvent,
    EventId::VolumeMapperRenderProgressEvent,
    EventId::VolumeMapperRenderStartEvent,
    EventId::VolumeMapperComputeGradientsEndEvent,
    EventId::VolumeMapperComputeGradientsProgressEvent,
    EventId::VolumeMapperComputeGradientsStartEvent,
    EventId::WidgetModifiedEvent,
    EventId::WidgetValueChangedEvent,
    EventId::WidgetActivateEvent,
    EventId::ConnectionCreatedEvent,
    EventId::ConnectionClosedEvent,
    EventId::DomainModifiedEvent,
    EventId::PropertyModifiedEvent,
    EventId::UpdateEvent,
    EventId::RegisterEvent,
    EventId::UnRegisterEvent,
    EventId::UpdateInformationEvent,
    EventId::AnnotationChangedEvent,
    EventId::SelectionChangedEvent,
    EventId::UpdatePropertyEvent,
    EventId::ViewProgressEvent,
    EventId::UpdateDataEvent,
    EventId::CurrentChangedEvent,
    EventId::ComputeVisiblePropBoundsEvent,
    EventId::TDxMotionEvent,
    EventId::TDxButtonPressEvent,
    EventId::TDxButtonReleaseEvent,
    EventId::HoverEvent,
    EventId::LoadStateEvent,
    EventId::SaveStateEvent,
    EventId::StateChangedEvent,
    EventId::WindowMakeCurrentEvent,
    EventId::WindowIsCurrentEvent,
    EventId::WindowFrameEvent,
    EventId::HighlightEvent,
    EventId::WindowSupportsOpenGLEvent,
    EventId::WindowIsDirectEvent,
    EventId::WindowStereoTypeChangedEvent,
    EventId::WindowResizeEvent,
    EventId::UncheckedPropertyModifiedEvent,
    EventId::UpdateShaderEvent,
    EventId::MessageEvent,
    EventId::StartSwipeEvent,
    EventId::SwipeEvent,
    EventId::EndSwipeEvent,
    EventId::StartPinchEvent,
    EventId::PinchEvent,
    EventId::EndPinchEvent,
    EventId::StartRotateEvent,
    EventId::RotateEvent,
    EventId::EndRotateEvent,
    EventId::StartPanEvent,
    EventId::PanEvent,
    EventId::EndPanEvent,
    EventId::TapEvent,
    EventId::LongTapEvent,
    EventId::FourthButtonPressEvent,
    EventId::FourthButtonReleaseEvent,
    EventId::FifthButtonPressEvent,
    EventId::FifthButtonReleaseEvent,
    EventId::Move3DEvent,
    EventId::Button3DEvent,
    EventId::TextEvent,
    EventId::LeftButtonDoubleClickEvent,
    EventId::MiddleButtonDoubleClickEvent,
    EventId::RightButtonDoubleClickEvent,
    EventId::MouseWheelLeftEvent,
    EventId::MouseWheelRightEvent,
    EventId::ViewerMovement3DEvent,
    EventId::Menu3DEvent,
    EventId::NextPose3DEvent,
    EventId::Clip3DEvent,
    EventId::PositionProp3DEvent,
    EventId::Pick3DEvent,
    EventId::Select3DEvent,
    EventId::Elevation3DEvent,
    EventId::BufferChangedEvent,
    EventId::UserEvent,
];
