use std::sync::atomic::{AtomicU64, Ordering};

use super::vtk_type::VtkMTimeType;

static GLOBAL_TIME_STAMP: AtomicU64 = AtomicU64::new(0);

/// VTK `vtkTimeStamp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeStamp {
    modified_time: VtkMTimeType,
}

impl TimeStamp {
    /// VTK: `vtkTimeStamp::New`.
    pub fn new() -> Self {
        Self { modified_time: 0 }
    }

    /// VTK: `vtkTimeStamp::Modified`.
    pub fn modified(&mut self) {
        self.modified_time = GLOBAL_TIME_STAMP
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
    }

    /// VTK: `vtkTimeStamp::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.modified_time
    }
}

impl Default for TimeStamp {
    fn default() -> Self {
        Self::new()
    }
}

impl From<TimeStamp> for VtkMTimeType {
    fn from(value: TimeStamp) -> Self {
        value.modified_time
    }
}

impl From<&TimeStamp> for VtkMTimeType {
    fn from(value: &TimeStamp) -> Self {
        value.modified_time
    }
}
