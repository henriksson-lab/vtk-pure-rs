use super::{Locator, LocatorApi};
use crate::common::core::{IdList, VtkIdType, VtkMTimeType};

/// VTK: `vtkAbstractPointLocator`.
///
/// This stores the translated `vtkLocator` base state plus the immediate
/// abstract-point-locator state. The point-query API is represented by
/// `AbstractPointLocatorApi`.
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractPointLocator {
    locator: Locator,
    bounds: [f64; 6],
    number_of_buckets: VtkIdType,
}

impl AbstractPointLocator {
    /// VTK: `vtkAbstractPointLocator::vtkAbstractPointLocator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            locator: Locator::with_class_name(class_name),
            bounds: [0.0; 6],
            number_of_buckets: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn locator(&self) -> &Locator {
        &self.locator
    }

    #[allow(dead_code)]
    pub(crate) fn locator_mut(&mut self) -> &mut Locator {
        &mut self.locator
    }

    #[allow(dead_code)]
    pub(crate) fn set_bounds_internal(&mut self, bounds: [f64; 6]) {
        self.bounds = bounds;
    }

    #[allow(dead_code)]
    pub(crate) fn set_number_of_buckets_internal(&mut self, number_of_buckets: VtkIdType) {
        self.number_of_buckets = number_of_buckets;
    }

    /// VTK: `vtkAbstractPointLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Bounds[0]: {}\nBounds[1]: {}\nBounds[2]: {}\nBounds[3]: {}\nBounds[4]: {}\nBounds[5]: {}\nNumber of Buckets: {}\n",
            self.bounds[0],
            self.bounds[1],
            self.bounds[2],
            self.bounds[3],
            self.bounds[4],
            self.bounds[5],
            self.number_of_buckets
        )
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.bounds
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds(double*)`.
    pub fn get_bounds_into(&self, bounds: &mut [f64; 6]) {
        *bounds = self.bounds;
    }

    /// VTK: `vtkAbstractPointLocator::GetNumberOfBuckets`.
    pub fn get_number_of_buckets(&self) -> VtkIdType {
        self.number_of_buckets
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.locator.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.locator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.locator.get_m_time()
    }

    /// VTK: `vtkLocator::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut super::DataSet) {
        self.locator.set_data_set(data_set);
    }

    /// VTK: `vtkLocator::GetDataSet`.
    pub fn get_data_set(&self) -> *mut super::DataSet {
        self.locator.get_data_set()
    }

    /// VTK: `vtkLocator::SetMaxLevel`.
    pub fn set_max_level(&mut self, max_level: i32) {
        self.locator.set_max_level(max_level);
    }

    /// VTK: `vtkLocator::GetMaxLevel`.
    pub fn get_max_level(&self) -> i32 {
        self.locator.get_max_level()
    }

    /// VTK: `vtkLocator::GetLevel`.
    pub fn get_level(&self) -> i32 {
        self.locator.get_level()
    }

    /// VTK: `vtkLocator::SetAutomatic`.
    pub fn set_automatic(&mut self, automatic: bool) {
        self.locator.set_automatic(automatic);
    }

    /// VTK: `vtkLocator::GetAutomatic`.
    pub fn get_automatic(&self) -> bool {
        self.locator.get_automatic()
    }

    /// VTK: `vtkLocator::AutomaticOn`.
    pub fn automatic_on(&mut self) {
        self.locator.automatic_on();
    }

    /// VTK: `vtkLocator::AutomaticOff`.
    pub fn automatic_off(&mut self) {
        self.locator.automatic_off();
    }

    /// VTK: `vtkLocator::SetTolerance`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.locator.set_tolerance(tolerance);
    }

    /// VTK: `vtkLocator::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.locator.get_tolerance()
    }

    /// VTK: `vtkLocator::SetUseExistingSearchStructure`.
    pub fn set_use_existing_search_structure(&mut self, use_existing_search_structure: bool) {
        self.locator
            .set_use_existing_search_structure(use_existing_search_structure);
    }

    /// VTK: `vtkLocator::GetUseExistingSearchStructure`.
    pub fn get_use_existing_search_structure(&self) -> bool {
        self.locator.get_use_existing_search_structure()
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOn`.
    pub fn use_existing_search_structure_on(&mut self) {
        self.locator.use_existing_search_structure_on();
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOff`.
    pub fn use_existing_search_structure_off(&mut self) {
        self.locator.use_existing_search_structure_off();
    }

    /// VTK: `vtkLocator::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.locator.get_build_time()
    }

    /// VTK: `vtkLocator::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        self.locator.uses_garbage_collector()
    }
}

impl Default for AbstractPointLocator {
    fn default() -> Self {
        Self::with_class_name("vtkAbstractPointLocator")
    }
}

/// VTK pure virtual and overload helper API for `vtkAbstractPointLocator`.
pub trait AbstractPointLocatorApi: LocatorApi {
    /// VTK: `vtkAbstractPointLocator::FindClosestPoint(const double[3])`.
    fn find_closest_point(&mut self, x: [f64; 3]) -> VtkIdType;

    /// VTK: `vtkAbstractPointLocator::FindClosestPoint(double, double, double)`.
    fn find_closest_point_components(&mut self, x: f64, y: f64, z: f64) -> VtkIdType {
        self.find_closest_point([x, y, z])
    }

    /// VTK: `vtkAbstractPointLocator::FindClosestPointWithinRadius`.
    fn find_closest_point_within_radius(
        &mut self,
        radius: f64,
        x: [f64; 3],
        dist2: &mut f64,
    ) -> VtkIdType;

    /// VTK: `vtkAbstractPointLocator::FindClosestNPoints(int, const double[3], vtkIdList*)`.
    fn find_closest_n_points(&mut self, n: i32, x: [f64; 3], result: &mut IdList);

    /// VTK: `vtkAbstractPointLocator::FindClosestNPoints(int, double, double, double, vtkIdList*)`.
    fn find_closest_n_points_components(
        &mut self,
        n: i32,
        x: f64,
        y: f64,
        z: f64,
        result: &mut IdList,
    ) {
        self.find_closest_n_points(n, [x, y, z], result);
    }

    /// VTK: `vtkAbstractPointLocator::FindPointsWithinRadius(double, const double[3], vtkIdList*)`.
    fn find_points_within_radius(&mut self, radius: f64, x: [f64; 3], result: &mut IdList);

    /// VTK: `vtkAbstractPointLocator::FindPointsWithinRadius(double, double, double, double, vtkIdList*)`.
    fn find_points_within_radius_components(
        &mut self,
        radius: f64,
        x: f64,
        y: f64,
        z: f64,
        result: &mut IdList,
    ) {
        self.find_points_within_radius(radius, [x, y, z], result);
    }
}
