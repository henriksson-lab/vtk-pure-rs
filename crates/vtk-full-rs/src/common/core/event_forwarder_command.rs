use std::{ffi::c_void, ptr};

use super::{command::Command, object::Object};

/// VTK: `vtkEventForwarderCommand`.
#[derive(Debug)]
pub struct EventForwarderCommand {
    command: Command,
    target: *mut Object,
}

impl EventForwarderCommand {
    /// VTK: `vtkEventForwarderCommand::New`.
    pub fn new() -> Self {
        Self {
            command: Command::with_class_name("vtkEventForwarderCommand"),
            target: ptr::null_mut(),
        }
    }

    /// VTK: `vtkEventForwarderCommand::Execute`.
    pub fn execute(&mut self, _caller: *mut Object, _event: u64, _call_data: *mut c_void) {
        // VTK forwards to Target->InvokeEvent(event, call_data). The compact
        // crate has not translated vtkObject observer dispatch yet, so this
        // method intentionally preserves only the public command surface.
    }

    /// VTK: `vtkEventForwarderCommand::SetTarget`.
    pub fn set_target(&mut self, obj: *mut Object) {
        self.target = obj;
    }

    /// VTK: `vtkEventForwarderCommand::GetTarget`.
    pub fn get_target(&self) -> *mut Object {
        self.target
    }

    /// VTK: `vtkEventForwarderCommand::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkEventForwarderCommand" || Command::is_type_of(name)
    }

    /// VTK: `vtkEventForwarderCommand::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkEventForwarderCommand::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> i64 {
        match name {
            "vtkEventForwarderCommand" => 0,
            "vtkCommand" => 1,
            "vtkObjectBase" => 2,
            _ => Command::get_number_of_generations_from_base_type(name),
        }
    }

    /// VTK: `vtkEventForwarderCommand::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> i64 {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.command.get_class_name()
    }

    /// VTK: `vtkCommand::UnRegister`.
    pub fn unregister(&mut self) -> bool {
        self.command.unregister()
    }

    /// VTK: `vtkCommand::SetAbortFlag`.
    pub fn set_abort_flag(&mut self, flag: bool) {
        self.command.set_abort_flag(flag);
    }

    /// VTK: `vtkCommand::GetAbortFlag`.
    pub fn get_abort_flag(&self) -> bool {
        self.command.get_abort_flag()
    }

    /// VTK: `vtkCommand::AbortFlagOn`.
    pub fn abort_flag_on(&mut self) {
        self.command.abort_flag_on();
    }

    /// VTK: `vtkCommand::AbortFlagOff`.
    pub fn abort_flag_off(&mut self) {
        self.command.abort_flag_off();
    }

    /// VTK: `vtkCommand::SetPassiveObserver`.
    pub fn set_passive_observer(&mut self, flag: bool) {
        self.command.set_passive_observer(flag);
    }

    /// VTK: `vtkCommand::GetPassiveObserver`.
    pub fn get_passive_observer(&self) -> bool {
        self.command.get_passive_observer()
    }

    /// VTK: `vtkCommand::PassiveObserverOn`.
    pub fn passive_observer_on(&mut self) {
        self.command.passive_observer_on();
    }

    /// VTK: `vtkCommand::PassiveObserverOff`.
    pub fn passive_observer_off(&mut self) {
        self.command.passive_observer_off();
    }
}

impl Default for EventForwarderCommand {
    fn default() -> Self {
        Self::new()
    }
}
