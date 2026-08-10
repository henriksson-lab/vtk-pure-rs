use std::{ffi::c_void, ptr};

use super::{command::Command, object::Object};

pub type CallbackCommandCallback = fn(*mut Object, u64, *mut c_void, *mut c_void);
pub type ClientDataDeleteCallback = fn(*mut c_void);

/// VTK: `vtkCallbackCommand`.
#[derive(Debug)]
pub struct CallbackCommand {
    command: Command,
    abort_flag_on_execute: bool,
    client_data: *mut c_void,
    callback: Option<CallbackCommandCallback>,
    client_data_delete_callback: Option<ClientDataDeleteCallback>,
}

impl CallbackCommand {
    /// VTK: `vtkCallbackCommand::New`.
    pub fn new() -> Self {
        Self {
            command: Command::new(),
            abort_flag_on_execute: false,
            client_data: ptr::null_mut(),
            callback: None,
            client_data_delete_callback: None,
        }
    }

    /// VTK: `vtkCallbackCommand::Execute`.
    pub fn execute(&mut self, caller: *mut Object, event: u64, call_data: *mut c_void) {
        if let Some(callback) = self.callback {
            callback(caller, event, self.client_data, call_data);
            if self.abort_flag_on_execute {
                self.abort_flag_on();
            }
        }
    }

    /// VTK: `vtkCallbackCommand::SetClientData`.
    pub fn set_client_data(&mut self, client_data: *mut c_void) {
        self.client_data = client_data;
    }

    /// VTK: `vtkCallbackCommand::GetClientData`.
    pub fn get_client_data(&self) -> *mut c_void {
        self.client_data
    }

    /// VTK: `vtkCallbackCommand::SetCallback`.
    pub fn set_callback(&mut self, callback: Option<CallbackCommandCallback>) {
        self.callback = callback;
    }

    /// VTK: `vtkCallbackCommand::SetClientDataDeleteCallback`.
    pub fn set_client_data_delete_callback(&mut self, callback: Option<ClientDataDeleteCallback>) {
        self.client_data_delete_callback = callback;
    }

    /// VTK: `vtkCallbackCommand::SetAbortFlagOnExecute`.
    pub fn set_abort_flag_on_execute(&mut self, flag: bool) {
        self.abort_flag_on_execute = flag;
    }

    /// VTK: `vtkCallbackCommand::GetAbortFlagOnExecute`.
    pub fn get_abort_flag_on_execute(&self) -> bool {
        self.abort_flag_on_execute
    }

    /// VTK: `vtkCallbackCommand::AbortFlagOnExecuteOn`.
    pub fn abort_flag_on_execute_on(&mut self) {
        self.set_abort_flag_on_execute(true);
    }

    /// VTK: `vtkCallbackCommand::AbortFlagOnExecuteOff`.
    pub fn abort_flag_on_execute_off(&mut self) {
        self.set_abort_flag_on_execute(false);
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

impl Default for CallbackCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CallbackCommand {
    fn drop(&mut self) {
        if let Some(callback) = self.client_data_delete_callback {
            callback(self.client_data);
        }
    }
}
