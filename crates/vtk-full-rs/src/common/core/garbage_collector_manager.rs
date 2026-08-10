use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::core::DebugLeaksManager;

static MANAGER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// VTK: `vtkGarbageCollectorManager`.
#[derive(Debug)]
pub struct GarbageCollectorManager {
    _debug_leaks_manager: DebugLeaksManager,
}

impl GarbageCollectorManager {
    /// VTK: `vtkGarbageCollectorManager::vtkGarbageCollectorManager`.
    pub fn new() -> Self {
        let debug_leaks_manager = DebugLeaksManager::new();
        if MANAGER_COUNT.fetch_add(1, Ordering::AcqRel) == 0 {
            // VTK calls vtkGarbageCollector::ClassInitialize here; that class is
            // not translated yet, so the target singleton work is deferred.
        }
        Self {
            _debug_leaks_manager: debug_leaks_manager,
        }
    }
}

impl Default for GarbageCollectorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GarbageCollectorManager {
    fn drop(&mut self) {
        if MANAGER_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            // VTK calls vtkGarbageCollector::ClassFinalize here; see constructor.
        }
    }
}
