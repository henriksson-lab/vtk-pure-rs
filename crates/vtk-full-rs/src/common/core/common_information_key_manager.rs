use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex, OnceLock,
};

use crate::common::core::{
    information_key::InformationKeyRegistration, InformationKey, InformationKeyLookup,
};

static MANAGER_COUNT: AtomicUsize = AtomicUsize::new(0);
static INFORMATION_KEYS: OnceLock<Mutex<Vec<Box<dyn InformationKeyRegistration>>>> =
    OnceLock::new();

/// VTK: `vtkCommonInformationKeyManager`.
#[derive(Debug)]
pub struct CommonInformationKeyManager;

impl CommonInformationKeyManager {
    /// VTK: `vtkCommonInformationKeyManager::vtkCommonInformationKeyManager`.
    pub fn new() -> Self {
        if MANAGER_COUNT.fetch_add(1, Ordering::AcqRel) == 0 {
            Self::class_initialize();
        }
        Self
    }

    /// VTK: `vtkCommonInformationKeyManager::Register`.
    pub fn register(key: InformationKey) -> *mut InformationKey {
        Self::register_owned(key)
    }

    pub(crate) fn register_owned<T>(key: T) -> *mut T
    where
        T: InformationKeyRegistration + 'static,
    {
        Self::class_initialize();
        let mut key = Box::new(key);
        let key_pointer = key.as_mut() as *mut T;
        if let (Some(name), Some(location)) = (
            key.information_key().get_name().map(str::to_owned),
            key.information_key().get_location().map(str::to_owned),
        ) {
            InformationKeyLookup::register_key(key.information_key_mut(), &name, &location);
        }
        information_keys().lock().unwrap().push(key);
        key_pointer
    }

    /// VTK: `vtkCommonInformationKeyManager::ClassInitialize`.
    pub(crate) fn class_initialize() {
        let _ = information_keys();
    }

    /// VTK: `vtkCommonInformationKeyManager::ClassFinalize`.
    pub(crate) fn class_finalize() {
        InformationKeyLookup::clear_keys();
        if let Some(keys) = INFORMATION_KEYS.get() {
            keys.lock().unwrap().clear();
        }
    }
}

impl Default for CommonInformationKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CommonInformationKeyManager {
    fn drop(&mut self) {
        if MANAGER_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            Self::class_finalize();
        }
    }
}

fn information_keys() -> &'static Mutex<Vec<Box<dyn InformationKeyRegistration>>> {
    INFORMATION_KEYS.get_or_init(|| Mutex::new(Vec::new()))
}
