use std::ffi::c_void;

use super::DataSet;
use crate::common::core::{Object, VtkMTimeType, VTK_DOUBLE_MAX};

/// VTK: `vtkPolyData*`.
pub type PolyDataHandle = *mut c_void;

/// VTK: `vtkLocator`.
#[derive(Debug, Clone, PartialEq)]
pub struct Locator {
    object: Object,
    data_set: *mut DataSet,
    use_existing_search_structure: bool,
    automatic: bool,
    tolerance: f64,
    max_level: i32,
    level: i32,
    build_time: VtkMTimeType,
}

impl Locator {
    /// VTK: `vtkLocator::vtkLocator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            data_set: std::ptr::null_mut(),
            use_existing_search_structure: false,
            automatic: true,
            tolerance: 0.001,
            max_level: 8,
            level: 8,
            build_time: 0,
        }
    }

    /// VTK: `vtkLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "DataSet: {}\nAutomatic: {}\nTolerance: {}\nBuild Time: {}\nMaxLevel: {}\nLevel: {}\nUseExistingSearchStructure: {}\n",
            if self.data_set.is_null() { "(none)" } else { "set" },
            if self.automatic { "On" } else { "Off" },
            self.tolerance,
            self.build_time,
            self.max_level,
            self.level,
            i32::from(self.use_existing_search_structure)
        )
    }

    /// VTK: `vtkLocator::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut DataSet) {
        if self.data_set != data_set {
            self.data_set = data_set;
            self.modified();
        }
    }

    /// VTK: `vtkLocator::GetDataSet`.
    pub fn get_data_set(&self) -> *mut DataSet {
        self.data_set
    }

    /// VTK: `vtkLocator::SetMaxLevel`.
    pub fn set_max_level(&mut self, max_level: i32) {
        let max_level = max_level.clamp(0, i32::MAX);
        if self.max_level != max_level {
            self.max_level = max_level;
            self.modified();
        }
    }

    /// VTK: `vtkLocator::GetMaxLevel`.
    pub fn get_max_level(&self) -> i32 {
        self.max_level
    }

    /// VTK: `vtkLocator::GetLevel`.
    pub fn get_level(&self) -> i32 {
        self.level
    }

    /// VTK: `vtkLocator::SetAutomatic`.
    pub fn set_automatic(&mut self, automatic: bool) {
        if self.automatic != automatic {
            self.automatic = automatic;
            self.modified();
        }
    }

    /// VTK: `vtkLocator::GetAutomatic`.
    pub fn get_automatic(&self) -> bool {
        self.automatic
    }

    /// VTK: `vtkLocator::AutomaticOn`.
    pub fn automatic_on(&mut self) {
        self.set_automatic(true);
    }

    /// VTK: `vtkLocator::AutomaticOff`.
    pub fn automatic_off(&mut self) {
        self.set_automatic(false);
    }

    /// VTK: `vtkLocator::SetTolerance`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        let tolerance = tolerance.clamp(0.0, VTK_DOUBLE_MAX);
        if self.tolerance != tolerance {
            self.tolerance = tolerance;
            self.modified();
        }
    }

    /// VTK: `vtkLocator::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.tolerance
    }

    /// VTK: `vtkLocator::SetUseExistingSearchStructure`.
    pub fn set_use_existing_search_structure(&mut self, use_existing_search_structure: bool) {
        if self.use_existing_search_structure != use_existing_search_structure {
            self.use_existing_search_structure = use_existing_search_structure;
            self.modified();
        }
    }

    /// VTK: `vtkLocator::GetUseExistingSearchStructure`.
    pub fn get_use_existing_search_structure(&self) -> bool {
        self.use_existing_search_structure
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOn`.
    pub fn use_existing_search_structure_on(&mut self) {
        self.set_use_existing_search_structure(true);
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOff`.
    pub fn use_existing_search_structure_off(&mut self) {
        self.set_use_existing_search_structure(false);
    }

    /// VTK: `vtkLocator::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.build_time
    }

    /// VTK: `vtkLocator::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        true
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

impl Default for Locator {
    fn default() -> Self {
        Self::with_class_name("vtkLocator")
    }
}

impl Drop for Locator {
    fn drop(&mut self) {
        self.set_data_set(std::ptr::null_mut());
    }
}

/// VTK pure virtual and virtual-default API for `vtkLocator`.
pub trait LocatorApi {
    fn locator(&self) -> &Locator;
    fn locator_mut(&mut self) -> &mut Locator;

    /// VTK: `vtkLocator::BuildLocator`.
    fn build_locator(&mut self);

    /// VTK: `vtkLocator::ForceBuildLocator`.
    fn force_build_locator(&mut self) {}

    /// VTK: `vtkLocator::FreeSearchStructure`.
    fn free_search_structure(&mut self);

    /// VTK: `vtkLocator::GenerateRepresentation`.
    fn generate_representation(&mut self, level: i32, pd: PolyDataHandle);

    /// VTK: `vtkLocator::BuildLocatorInternal`.
    fn build_locator_internal(&mut self) {}

    /// VTK: `vtkLocator::Initialize`.
    fn initialize(&mut self) {
        self.free_search_structure();
    }

    /// VTK: `vtkLocator::Update`.
    fn update(&mut self) {
        let locator = self.locator();
        let data_set = locator.get_data_set();
        if data_set.is_null() {
            return;
        }

        let data_set_m_time = unsafe { (*data_set).get_m_time() };
        if locator.get_m_time() > locator.get_build_time()
            || data_set_m_time > locator.get_build_time()
        {
            self.build_locator();
        }
    }
}
