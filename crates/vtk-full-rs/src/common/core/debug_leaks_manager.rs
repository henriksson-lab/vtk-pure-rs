use std::sync::atomic::{AtomicUsize, Ordering};

static MANAGER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// VTK: `vtkDebugLeaksManager`.
#[derive(Debug)]
pub struct DebugLeaksManager;

impl DebugLeaksManager {
    /// VTK: `vtkDebugLeaksManager::vtkDebugLeaksManager`.
    pub fn new() -> Self {
        if MANAGER_COUNT.fetch_add(1, Ordering::AcqRel) == 0 {
            // VTK calls vtkDebugLeaks::ClassInitialize here; that class is not
            // translated yet, so the target singleton work is deferred.
        }
        Self
    }
}

impl Default for DebugLeaksManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DebugLeaksManager {
    fn drop(&mut self) {
        if MANAGER_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            // VTK calls vtkDebugLeaks::ClassFinalize here; see constructor.
        }
    }
}
