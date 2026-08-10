use std::{ffi::c_char, fmt, mem, ptr};

/// VTK: `vtkOStrStreamWrapper`.
#[derive(Debug)]
pub struct OStrStreamWrapper {
    stream: Vec<u8>,
    result: Option<Vec<c_char>>,
    frozen: bool,
}

impl OStrStreamWrapper {
    /// VTK: `vtkOStrStreamWrapper::vtkOStrStreamWrapper`.
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            result: None,
            frozen: false,
        }
    }

    /// VTK: `vtkOStrStreamWrapper::str`.
    pub fn str(&mut self) -> *mut c_char {
        if self.result.is_none() {
            let nul = self
                .stream
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(self.stream.len());
            let mut result = Vec::with_capacity(nul + 1);
            result.extend(self.stream[..nul].iter().map(|byte| *byte as c_char));
            result.push(0);
            self.result = Some(result);
            self.freeze();
        }

        self.result
            .as_mut()
            .map_or(ptr::null_mut(), |result| result.as_mut_ptr())
    }

    /// VTK: `vtkOStrStreamWrapper::rdbuf`.
    pub fn rdbuf(&mut self) -> &mut Self {
        self
    }

    /// VTK: `vtkOStrStreamWrapper::freeze`.
    pub fn freeze(&mut self) {
        self.freeze_with_flag(1);
    }

    /// VTK: `vtkOStrStreamWrapper::freeze(int)`.
    pub fn freeze_with_flag(&mut self, frozen: i32) {
        self.frozen = frozen != 0;
    }

    /// VTK: `vtkOStreamWrapper::write`.
    pub fn write(&mut self, bytes: &[u8], size: usize) -> &mut Self {
        let count = size.min(bytes.len());
        self.stream.extend_from_slice(&bytes[..count]);
        self
    }
}

impl Default for OStrStreamWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for OStrStreamWrapper {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.stream.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl Drop for OStrStreamWrapper {
    fn drop(&mut self) {
        if self.frozen {
            if let Some(result) = self.result.take() {
                mem::forget(result);
            }
        }
    }
}
