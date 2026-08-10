use std::hint::spin_loop;
use std::sync::atomic::{AtomicBool, Ordering};

/// VTK/Common/Core/vtkAtomicMutex.h
#[derive(Debug)]
pub struct AtomicMutex {
    locked: AtomicBool,
}

impl AtomicMutex {
    /// VTK: vtkAtomicMutex::vtkAtomicMutex()
    #[must_use]
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    /// VTK: vtkAtomicMutex::lock()
    pub fn lock(&self) {
        loop {
            if !self.locked.swap(true, Ordering::Acquire) {
                return;
            }

            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
    }

    /// VTK: vtkAtomicMutex::unlock()
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

impl Clone for AtomicMutex {
    /// VTK: vtkAtomicMutex::vtkAtomicMutex(const vtkAtomicMutex&)
    fn clone(&self) -> Self {
        Self {
            locked: AtomicBool::new(self.locked.load(Ordering::Acquire)),
        }
    }
}

impl Default for AtomicMutex {
    fn default() -> Self {
        Self::new()
    }
}
