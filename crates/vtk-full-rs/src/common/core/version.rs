pub const VTK_MAJOR_VERSION: i32 = 9;
pub const VTK_MINOR_VERSION: i32 = 6;
pub const VTK_BUILD_VERSION: i32 = 20260325;
pub const VTK_EPOCH_VERSION: i32 = 20251220;
pub const VTK_VERSION: &str = "9.6.20260325";
pub const VTK_SOURCE_VERSION: &str = "vtk version 9.6.20260325";
pub const VTK_VERSION_FULL: &str = "9.6.1-1255-g00f9418ca6";
pub const VTK_VERSION_NUMBER: u64 = 90620260325;
pub const VTK_VERSION_NUMBER_QUICK: u64 = 90620251220;

/// VTK macro: `VTK_VERSION_CHECK`.
pub fn vtk_version_check(major: i32, minor: i32, build: i32) -> u64 {
    10_000_000_000_u64 * major as u64 + 100_000_000_u64 * minor as u64 + build as u64
}

/// VTK `vtkVersion`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Version;

impl Version {
    /// VTK: `vtkVersion::New`.
    pub fn new() -> Self {
        Self
    }

    /// VTK: `vtkVersion::GetVTKVersion`.
    pub fn get_vtk_version() -> &'static str {
        VTK_VERSION
    }

    /// VTK: `vtkVersion::GetVTKVersionFull`.
    pub fn get_vtk_version_full() -> &'static str {
        VTK_VERSION_FULL
    }

    /// VTK: `vtkVersion::GetVTKMajorVersion`.
    pub fn get_vtk_major_version() -> i32 {
        VTK_MAJOR_VERSION
    }

    /// VTK: `vtkVersion::GetVTKMinorVersion`.
    pub fn get_vtk_minor_version() -> i32 {
        VTK_MINOR_VERSION
    }

    /// VTK: `vtkVersion::GetVTKBuildVersion`.
    pub fn get_vtk_build_version() -> i32 {
        VTK_BUILD_VERSION
    }

    /// VTK: `vtkVersion::GetVTKSourceVersion`.
    pub fn get_vtk_source_version() -> &'static str {
        VTK_SOURCE_VERSION
    }
}

/// VTK C API: `GetVTKVersion`.
pub fn get_vtk_version() -> &'static str {
    Version::get_vtk_version()
}
