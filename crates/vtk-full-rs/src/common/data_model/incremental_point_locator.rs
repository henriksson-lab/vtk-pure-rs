use super::{AbstractPointLocator, AbstractPointLocatorApi, DataSet, Locator};
use crate::common::core::{Points, VtkIdType, VtkMTimeType};

/// VTK: `vtkIncrementalPointLocator`.
///
/// This stores the translated `vtkAbstractPointLocator` base state. The
/// incremental insertion API is represented by `IncrementalPointLocatorApi`.
#[derive(Debug, Clone, PartialEq)]
pub struct IncrementalPointLocator {
    abstract_point_locator: AbstractPointLocator,
}

impl IncrementalPointLocator {
    /// VTK: `vtkIncrementalPointLocator::vtkIncrementalPointLocator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            abstract_point_locator: AbstractPointLocator::with_class_name(class_name),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn abstract_point_locator(&self) -> &AbstractPointLocator {
        &self.abstract_point_locator
    }

    #[allow(dead_code)]
    pub(crate) fn abstract_point_locator_mut(&mut self) -> &mut AbstractPointLocator {
        &mut self.abstract_point_locator
    }

    #[allow(dead_code)]
    pub(crate) fn locator(&self) -> &Locator {
        self.abstract_point_locator.locator()
    }

    #[allow(dead_code)]
    pub(crate) fn locator_mut(&mut self) -> &mut Locator {
        self.abstract_point_locator.locator_mut()
    }

    /// VTK: `vtkIncrementalPointLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.abstract_point_locator.print_self()
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.abstract_point_locator.get_bounds()
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds(double*)`.
    pub fn get_bounds_into(&self, bounds: &mut [f64; 6]) {
        self.abstract_point_locator.get_bounds_into(bounds);
    }

    /// VTK: `vtkAbstractPointLocator::GetNumberOfBuckets`.
    pub fn get_number_of_buckets(&self) -> VtkIdType {
        self.abstract_point_locator.get_number_of_buckets()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.abstract_point_locator.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.abstract_point_locator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.abstract_point_locator.get_m_time()
    }

    /// VTK: `vtkLocator::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut DataSet) {
        self.abstract_point_locator.set_data_set(data_set);
    }

    /// VTK: `vtkLocator::GetDataSet`.
    pub fn get_data_set(&self) -> *mut DataSet {
        self.abstract_point_locator.get_data_set()
    }

    /// VTK: `vtkLocator::SetMaxLevel`.
    pub fn set_max_level(&mut self, max_level: i32) {
        self.abstract_point_locator.set_max_level(max_level);
    }

    /// VTK: `vtkLocator::GetMaxLevel`.
    pub fn get_max_level(&self) -> i32 {
        self.abstract_point_locator.get_max_level()
    }

    /// VTK: `vtkLocator::GetLevel`.
    pub fn get_level(&self) -> i32 {
        self.abstract_point_locator.get_level()
    }

    /// VTK: `vtkLocator::SetAutomatic`.
    pub fn set_automatic(&mut self, automatic: bool) {
        self.abstract_point_locator.set_automatic(automatic);
    }

    /// VTK: `vtkLocator::GetAutomatic`.
    pub fn get_automatic(&self) -> bool {
        self.abstract_point_locator.get_automatic()
    }

    /// VTK: `vtkLocator::AutomaticOn`.
    pub fn automatic_on(&mut self) {
        self.abstract_point_locator.automatic_on();
    }

    /// VTK: `vtkLocator::AutomaticOff`.
    pub fn automatic_off(&mut self) {
        self.abstract_point_locator.automatic_off();
    }

    /// VTK: `vtkLocator::SetTolerance`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.abstract_point_locator.set_tolerance(tolerance);
    }

    /// VTK: `vtkLocator::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.abstract_point_locator.get_tolerance()
    }

    /// VTK: `vtkLocator::SetUseExistingSearchStructure`.
    pub fn set_use_existing_search_structure(&mut self, use_existing_search_structure: bool) {
        self.abstract_point_locator
            .set_use_existing_search_structure(use_existing_search_structure);
    }

    /// VTK: `vtkLocator::GetUseExistingSearchStructure`.
    pub fn get_use_existing_search_structure(&self) -> bool {
        self.abstract_point_locator
            .get_use_existing_search_structure()
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOn`.
    pub fn use_existing_search_structure_on(&mut self) {
        self.abstract_point_locator
            .use_existing_search_structure_on();
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOff`.
    pub fn use_existing_search_structure_off(&mut self) {
        self.abstract_point_locator
            .use_existing_search_structure_off();
    }

    /// VTK: `vtkLocator::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.abstract_point_locator.get_build_time()
    }

    /// VTK: `vtkLocator::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        self.abstract_point_locator.uses_garbage_collector()
    }
}

impl Default for IncrementalPointLocator {
    fn default() -> Self {
        Self::with_class_name("vtkIncrementalPointLocator")
    }
}

/// VTK pure virtual API for `vtkIncrementalPointLocator`.
pub trait IncrementalPointLocatorApi: AbstractPointLocatorApi {
    /// VTK: `vtkIncrementalPointLocator::FindClosestInsertedPoint`.
    fn find_closest_inserted_point(&mut self, x: [f64; 3]) -> VtkIdType;

    /// VTK: `vtkIncrementalPointLocator::InitPointInsertion(vtkPoints*, const double[6])`.
    fn init_point_insertion(&mut self, new_pts: *mut Points, bounds: [f64; 6]) -> i32;

    /// VTK: `vtkIncrementalPointLocator::InitPointInsertion(vtkPoints*, const double[6], vtkIdType)`.
    fn init_point_insertion_estimated(
        &mut self,
        new_pts: *mut Points,
        bounds: [f64; 6],
        est_size: VtkIdType,
    ) -> i32;

    /// VTK: `vtkIncrementalPointLocator::IsInsertedPoint(double, double, double)`.
    fn is_inserted_point_components(&mut self, x: f64, y: f64, z: f64) -> VtkIdType;

    /// VTK: `vtkIncrementalPointLocator::IsInsertedPoint(const double[3])`.
    fn is_inserted_point(&mut self, x: [f64; 3]) -> VtkIdType;

    /// VTK: `vtkIncrementalPointLocator::InsertUniquePoint`.
    fn insert_unique_point(&mut self, x: [f64; 3], pt_id: &mut VtkIdType) -> i32;

    /// VTK: `vtkIncrementalPointLocator::InsertPoint`.
    fn insert_point(&mut self, pt_id: VtkIdType, x: [f64; 3]);

    /// VTK: `vtkIncrementalPointLocator::InsertNextPoint`.
    fn insert_next_point(&mut self, x: [f64; 3]) -> VtkIdType;
}
