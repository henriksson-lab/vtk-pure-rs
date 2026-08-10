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
static FINALIZERS: OnceLock<Mutex<Vec<Box<dyn FnOnce() + Send>>>> = OnceLock::new();

/// VTK: `vtkFilteringInformationKeyManager`.
#[derive(Debug)]
pub struct FilteringInformationKeyManager;

impl FilteringInformationKeyManager {
    /// VTK: `vtkFilteringInformationKeyManager::vtkFilteringInformationKeyManager`.
    pub fn new() -> Self {
        if MANAGER_COUNT.fetch_add(1, Ordering::AcqRel) == 0 {
            Self::class_initialize();
        }
        Self
    }

    /// VTK: `vtkFilteringInformationKeyManager::Register`.
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

    /// VTK: `vtkFilteringInformationKeyManager::AddFinalizer`.
    pub fn add_finalizer(finalizer: impl FnOnce() + Send + 'static) {
        finalizers().lock().unwrap().push(Box::new(finalizer));
    }

    /// VTK: `vtkFilteringInformationKeyManager::ClassInitialize`.
    pub(crate) fn class_initialize() {
        let _ = information_keys();
    }

    /// VTK: `vtkFilteringInformationKeyManager::ClassFinalize`.
    pub(crate) fn class_finalize() {
        if let Some(finalizers) = FINALIZERS.get() {
            for finalizer in finalizers.lock().unwrap().drain(..) {
                finalizer();
            }
        }
        if let Some(keys) = INFORMATION_KEYS.get() {
            keys.lock().unwrap().clear();
        }
    }
}

impl Default for FilteringInformationKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FilteringInformationKeyManager {
    fn drop(&mut self) {
        if MANAGER_COUNT.fetch_sub(1, Ordering::AcqRel) == 1 {
            Self::class_finalize();
        }
    }
}

fn information_keys() -> &'static Mutex<Vec<Box<dyn InformationKeyRegistration>>> {
    INFORMATION_KEYS.get_or_init(|| Mutex::new(Vec::new()))
}

fn finalizers() -> &'static Mutex<Vec<Box<dyn FnOnce() + Send>>> {
    FINALIZERS.get_or_init(|| Mutex::new(Vec::new()))
}
