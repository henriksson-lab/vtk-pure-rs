use std::ffi::{c_void, CStr, CString};
use std::path::{Component, Path, PathBuf};

use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkResourceFileLocator`.
#[derive(Debug, Clone)]
pub struct ResourceFileLocator {
    object: Object,
    log_verbosity: i32,
}

impl ResourceFileLocator {
    /// VTK: `vtkResourceFileLocator::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkResourceFileLocator"),
            log_verbosity: 9,
        }
    }

    /// VTK: `vtkResourceFileLocator::Locate`.
    pub fn locate(&self, anchor: &str, landmark: &str, default_dir: &str) -> String {
        self.locate_with_prefixes(anchor, &[""], landmark, default_dir)
    }

    /// VTK: `vtkResourceFileLocator::Locate`.
    pub fn locate_with_prefixes(
        &self,
        anchor: &str,
        landmark_prefixes: &[impl AsRef<str>],
        landmark: &str,
        default_dir: &str,
    ) -> String {
        let mut path_components = split_path_components(anchor);
        while !path_components.is_empty() {
            let curanchor = join_path_components(&path_components);
            for curprefix in landmark_prefixes {
                let curprefix = curprefix.as_ref();
                let landmarkdir = if curprefix.is_empty() {
                    curanchor.clone()
                } else {
                    curanchor.join(curprefix)
                };
                let landmarktocheck = landmarkdir.join(landmark);
                if landmarktocheck.exists() {
                    return landmarkdir.to_string_lossy().into_owned();
                }
            }
            path_components.pop();
        }
        default_dir.to_string()
    }

    /// VTK: `vtkResourceFileLocator::GetLibraryPathForAddress`.
    pub fn get_library_path_for_address(ptr: *const c_void) -> String {
        library_path_for_address(ptr).unwrap_or_default()
    }

    /// VTK: `vtkResourceFileLocator::GetCurrentExecutablePath`.
    pub fn get_current_executable_path() -> String {
        std::env::current_exe()
            .ok()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// VTK: `vtkResourceFileLocator::GetLibraryPathForSymbolUnix`.
    pub fn get_library_path_for_symbol_unix(symbolname: Option<&str>) -> String {
        let Some(symbolname) = symbolname else {
            return String::new();
        };
        let Ok(symbolname) = CString::new(symbolname) else {
            return String::new();
        };
        library_path_for_symbol_unix(&symbolname).unwrap_or_default()
    }

    /// VTK: `vtkResourceFileLocator::GetLibraryPathForSymbolWin32`.
    pub fn get_library_path_for_symbol_win32(fptr: *const c_void) -> String {
        #[cfg(windows)]
        {
            Self::get_library_path_for_address(fptr)
        }
        #[cfg(not(windows))]
        {
            let _ = fptr;
            String::new()
        }
    }

    /// VTK: `vtkResourceFileLocator::SetLogVerbosity`.
    pub fn set_log_verbosity(&mut self, log_verbosity: i32) {
        if self.log_verbosity != log_verbosity {
            self.log_verbosity = log_verbosity;
            self.modified();
        }
    }

    /// VTK: `vtkResourceFileLocator::GetLogVerbosity`.
    pub fn get_log_verbosity(&self) -> i32 {
        self.log_verbosity
    }

    /// VTK: `vtkResourceFileLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("LogVerbosity: {}\n", self.log_verbosity)
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
}

impl Default for ResourceFileLocator {
    fn default() -> Self {
        Self::new()
    }
}

fn split_path_components(anchor: &str) -> Vec<Component<'_>> {
    Path::new(anchor).components().collect()
}

fn join_path_components(components: &[Component<'_>]) -> PathBuf {
    let mut path = PathBuf::new();
    for component in components {
        path.push(component.as_os_str());
    }
    path
}

#[cfg(any(unix, target_os = "macos"))]
fn library_path_for_address(ptr: *const c_void) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    let mut info = std::mem::MaybeUninit::<libc::Dl_info>::uninit();
    let found = unsafe { libc::dladdr(ptr, info.as_mut_ptr()) };
    if found == 0 {
        return None;
    }

    let info = unsafe { info.assume_init() };
    if info.dli_fname.is_null() {
        return None;
    }

    Some(
        unsafe { CStr::from_ptr(info.dli_fname) }
            .to_string_lossy()
            .into_owned(),
    )
}

#[cfg(not(any(unix, target_os = "macos")))]
fn library_path_for_address(_ptr: *const c_void) -> Option<String> {
    None
}

#[cfg(any(unix, target_os = "macos"))]
fn library_path_for_symbol_unix(symbolname: &CStr) -> Option<String> {
    let ptr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, symbolname.as_ptr()) };
    library_path_for_address(ptr.cast_const())
}

#[cfg(not(any(unix, target_os = "macos")))]
fn library_path_for_symbol_unix(_symbolname: &CStr) -> Option<String> {
    None
}
