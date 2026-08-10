use std::ffi::CStr;

const ERROR_STRINGS: [&str; 10] = [
    "NoError",
    "FileNotFoundError",
    "CannotOpenFileError",
    "UnrecognizedFileTypeError",
    "PrematureEndOfFileError",
    "FileFormatError",
    "NoFileNameError",
    "OutOfDiskSpaceError",
    "UnknownError",
    "UserError",
];

/// VTK: `vtkErrorCode`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ErrorCode;

impl ErrorCode {
    /// VTK: `vtkErrorCode::NoError`.
    pub const NO_ERROR: u64 = 0;
    /// VTK: `vtkErrorCode::FirstVTKErrorCode`.
    pub const FIRST_VTK_ERROR_CODE: u64 = 20000;
    /// VTK: `vtkErrorCode::FileNotFoundError`.
    pub const FILE_NOT_FOUND_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 1;
    /// VTK: `vtkErrorCode::CannotOpenFileError`.
    pub const CANNOT_OPEN_FILE_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 2;
    /// VTK: `vtkErrorCode::UnrecognizedFileTypeError`.
    pub const UNRECOGNIZED_FILE_TYPE_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 3;
    /// VTK: `vtkErrorCode::PrematureEndOfFileError`.
    pub const PREMATURE_END_OF_FILE_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 4;
    /// VTK: `vtkErrorCode::FileFormatError`.
    pub const FILE_FORMAT_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 5;
    /// VTK: `vtkErrorCode::NoFileNameError`.
    pub const NO_FILE_NAME_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 6;
    /// VTK: `vtkErrorCode::OutOfDiskSpaceError`.
    pub const OUT_OF_DISK_SPACE_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 7;
    /// VTK: `vtkErrorCode::UnknownError`.
    pub const UNKNOWN_ERROR: u64 = Self::FIRST_VTK_ERROR_CODE + 8;
    /// VTK: `vtkErrorCode::UserError`.
    pub const USER_ERROR: u64 = 40000;

    /// VTK: `vtkErrorCode::GetStringFromErrorCode`.
    pub fn get_string_from_error_code(error: u64) -> String {
        if error < Self::FIRST_VTK_ERROR_CODE {
            return system_error_string(error);
        }

        let shifted = error - Self::FIRST_VTK_ERROR_CODE;
        if let Some(value) = ERROR_STRINGS.get(shifted as usize) {
            (*value).to_string()
        } else if shifted == Self::USER_ERROR {
            "UserError".to_string()
        } else {
            "NoError".to_string()
        }
    }

    /// VTK: `vtkErrorCode::GetErrorCodeFromString`.
    pub fn get_error_code_from_string(error: &str) -> u64 {
        for (index, value) in ERROR_STRINGS.iter().enumerate() {
            if *value == error {
                return index as u64;
            }
        }

        if error == "UserError" {
            Self::USER_ERROR
        } else {
            Self::NO_ERROR
        }
    }

    /// VTK: `vtkErrorCode::GetLastSystemError`.
    pub fn get_last_system_error() -> u64 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u64
    }
}

fn system_error_string(error: u64) -> String {
    // VTK calls C strerror for system errno values below FirstVTKErrorCode.
    let ptr = unsafe { libc::strerror(error as libc::c_int) };
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned()
    }
}
