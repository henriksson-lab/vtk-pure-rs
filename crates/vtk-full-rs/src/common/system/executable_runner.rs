use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::common::core::{Object, VtkIdType, VtkMTimeType};

/// VTK: `vtkExecutableRunner`.
#[derive(Debug, Clone)]
pub struct ExecutableRunner {
    object: Object,
    right_trim_result: bool,
    timeout: f64,
    command: String,
    return_value: i32,
    execute_in_system_shell: bool,
    arguments: Vec<String>,
    stdout: String,
    stderr: String,
}

impl ExecutableRunner {
    /// VTK: `vtkExecutableRunner::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkExecutableRunner"),
            right_trim_result: true,
            timeout: 5.0,
            command: String::new(),
            return_value: -1,
            execute_in_system_shell: true,
            arguments: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    /// VTK: `vtkExecutableRunner::Execute`.
    pub fn execute(&mut self) {
        let command = self.command.trim_start().to_string();
        if command.is_empty() {
            return;
        }

        let command_to_execute = self.get_command_to_execute();
        let Some((program, args)) = command_to_execute.split_first() else {
            return;
        };

        let spawn_result = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let Ok(mut child) = spawn_result else {
            self.return_value = -1;
            return;
        };

        let stdout_handle = child.stdout.take().map(read_pipe_on_thread);
        let stderr_handle = child.stderr.take().map(read_pipe_on_thread);

        let (status, timed_out) = self.wait_for_exit(&mut child);
        if timed_out {
            let _ = child.kill();
            let _ = child.wait();
            self.return_value = -1;
        } else {
            self.return_value = status
                .and_then(|exit_status| exit_status.code())
                .unwrap_or(-1);
        }

        let mut out = stdout_handle
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default();
        let mut err = stderr_handle
            .and_then(|handle| handle.join().ok())
            .unwrap_or_default();

        if self.right_trim_result {
            rtrim(&mut out);
            rtrim(&mut err);
        }

        self.set_stdout(out);
        self.set_stderr(err);
    }

    /// VTK: `vtkExecutableRunner::GetTimeout`.
    pub fn get_timeout(&self) -> f64 {
        self.timeout
    }

    /// VTK: `vtkExecutableRunner::SetTimeout`.
    pub fn set_timeout(&mut self, timeout: f64) {
        if self.timeout != timeout {
            self.timeout = timeout;
            self.modified();
        }
    }

    /// VTK: `vtkExecutableRunner::GetRightTrimResult`.
    pub fn get_right_trim_result(&self) -> bool {
        self.right_trim_result
    }

    /// VTK: `vtkExecutableRunner::SetRightTrimResult`.
    pub fn set_right_trim_result(&mut self, right_trim_result: bool) {
        if self.right_trim_result != right_trim_result {
            self.right_trim_result = right_trim_result;
            self.modified();
        }
    }

    /// VTK: `vtkExecutableRunner::RightTrimResultOn`.
    pub fn right_trim_result_on(&mut self) {
        self.set_right_trim_result(true);
    }

    /// VTK: `vtkExecutableRunner::RightTrimResultOff`.
    pub fn right_trim_result_off(&mut self) {
        self.set_right_trim_result(false);
    }

    /// VTK: `vtkExecutableRunner::GetCommand`.
    pub fn get_command(&self) -> &str {
        &self.command
    }

    /// VTK: `vtkExecutableRunner::SetCommand`.
    pub fn set_command(&mut self, command: impl Into<String>) {
        let command = command.into();
        if self.command != command {
            self.command = command;
            self.modified();
        }
    }

    /// VTK: `vtkExecutableRunner::GetExecuteInSystemShell`.
    pub fn get_execute_in_system_shell(&self) -> bool {
        self.execute_in_system_shell
    }

    /// VTK: `vtkExecutableRunner::SetExecuteInSystemShell`.
    pub fn set_execute_in_system_shell(&mut self, execute_in_system_shell: bool) {
        if self.execute_in_system_shell != execute_in_system_shell {
            self.execute_in_system_shell = execute_in_system_shell;
            self.modified();
        }
    }

    /// VTK: `vtkExecutableRunner::ExecuteInSystemShellOn`.
    pub fn execute_in_system_shell_on(&mut self) {
        self.set_execute_in_system_shell(true);
    }

    /// VTK: `vtkExecutableRunner::ExecuteInSystemShellOff`.
    pub fn execute_in_system_shell_off(&mut self) {
        self.set_execute_in_system_shell(false);
    }

    /// VTK: `vtkExecutableRunner::AddArgument`.
    pub fn add_argument(&mut self, arg: impl Into<String>) {
        self.arguments.push(arg.into());
        self.modified();
    }

    /// VTK: `vtkExecutableRunner::ClearArguments`.
    pub fn clear_arguments(&mut self) {
        if !self.arguments.is_empty() {
            self.arguments.clear();
            self.modified();
        }
    }

    /// VTK: `vtkExecutableRunner::GetNumberOfArguments`.
    pub fn get_number_of_arguments(&self) -> VtkIdType {
        self.arguments.len() as VtkIdType
    }

    /// VTK: `vtkExecutableRunner::GetStdOut`.
    pub fn get_stdout(&self) -> &str {
        &self.stdout
    }

    /// VTK: `vtkExecutableRunner::GetStdErr`.
    pub fn get_stderr(&self) -> &str {
        &self.stderr
    }

    /// VTK: `vtkExecutableRunner::GetReturnValue`.
    pub fn get_return_value(&self) -> i32 {
        self.return_value
    }

    /// VTK: `vtkExecutableRunner::GetCommandToExecute`.
    pub(crate) fn get_command_to_execute(&self) -> Vec<String> {
        if self.execute_in_system_shell {
            let mut result = Vec::with_capacity(3);
            if cfg!(windows) {
                result.push("cmd.exe".to_string());
                result.push("/c".to_string());
            } else {
                result.push("sh".to_string());
                result.push("-c".to_string());
            }
            result.push(self.command.clone());
            result
        } else {
            let mut result = Vec::with_capacity(self.arguments.len() + 1);
            result.push(self.command.clone());
            result.extend(self.arguments.iter().cloned());
            result
        }
    }

    /// VTK: `vtkExecutableRunner::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Command: {}\nTimeout: {}\nRightTrimResult: {}\n",
            self.get_command(),
            self.get_timeout(),
            self.get_right_trim_result()
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkExecutableRunner::SetStdOut`.
    pub(crate) fn set_stdout(&mut self, stdout: String) {
        self.stdout = stdout;
    }

    /// VTK: `vtkExecutableRunner::SetStdErr`.
    pub(crate) fn set_stderr(&mut self, stderr: String) {
        self.stderr = stderr;
    }

    fn wait_for_exit(&self, child: &mut Child) -> (Option<ExitStatus>, bool) {
        if self.timeout <= 0.0 {
            return (child.wait().ok(), false);
        }

        let timeout = Duration::from_secs_f64(self.timeout);
        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return (Some(status), false),
                Ok(None) if start.elapsed() >= timeout => return (None, true),
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(_) => return (None, false),
            }
        }
    }
}

impl Default for ExecutableRunner {
    fn default() -> Self {
        Self::new()
    }
}

fn read_pipe_on_thread<R>(mut pipe: R) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = String::new();
        let _ = pipe.read_to_string(&mut output);
        output
    })
}

fn rtrim(s: &mut String) {
    let trimmed_len = s.trim_end().len();
    s.truncate(trimmed_len);
}
