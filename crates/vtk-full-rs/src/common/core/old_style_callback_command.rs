use std::{ffi::c_void, ptr};

use super::{callback_command::ClientDataDeleteCallback, command::Command};

pub type OldStyleCallback = fn(*mut c_void);

/// VTK: `vtkOldStyleCallbackCommand`.
#[derive(Debug)]
pub struct OldStyleCallbackCommand {
    command: Command,
    client_data: *mut c_void,
    callback: Option<OldStyleCallback>,
    client_data_delete_callback: Option<ClientDataDeleteCallback>,
}

impl OldStyleCallbackCommand {
    /// VTK: `vtkOldStyleCallbackCommand::New`.
    pub fn new() -> Self {
        Self {
            command: Command::new(),
            client_data: ptr::null_mut(),
            callback: None,
            client_data_delete_callback: None,
        }
    }

    /// VTK: `vtkOldStyleCallbackCommand::Execute`.
    pub fn execute(&mut self) {
        if let Some(callback) = self.callback {
            callback(self.client_data);
        }
    }

    /// VTK: `vtkOldStyleCallbackCommand::SetClientData`.
    pub fn set_client_data(&mut self, client_data: *mut c_void) {
        self.client_data = client_data;
    }

    /// VTK: `vtkOldStyleCallbackCommand::SetCallback`.
    pub fn set_callback(&mut self, callback: Option<OldStyleCallback>) {
        self.callback = callback;
    }

    /// VTK: `vtkOldStyleCallbackCommand::SetClientDataDeleteCallback`.
    pub fn set_client_data_delete_callback(&mut self, callback: Option<ClientDataDeleteCallback>) {
        self.client_data_delete_callback = callback;
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

impl Default for OldStyleCallbackCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OldStyleCallbackCommand {
    fn drop(&mut self) {
        if let Some(callback) = self.client_data_delete_callback {
            callback(self.client_data);
        }
    }
}
