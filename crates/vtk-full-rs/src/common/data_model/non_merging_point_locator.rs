use super::{DataSet, PointLocator};
use crate::common::core::{Points, VtkIdType, VtkMTimeType};

/// VTK: `vtkNonMergingPointLocator`.
#[derive(Debug, Clone)]
pub struct NonMergingPointLocator {
    point_locator: PointLocator,
}

impl NonMergingPointLocator {
    /// VTK: `vtkNonMergingPointLocator::New`.
    pub fn new() -> Self {
        Self {
            point_locator: PointLocator::with_class_name("vtkNonMergingPointLocator"),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn point_locator(&self) -> &PointLocator {
        &self.point_locator
    }

    #[allow(dead_code)]
    pub(crate) fn point_locator_mut(&mut self) -> &mut PointLocator {
        &mut self.point_locator
    }

    /// VTK: `vtkNonMergingPointLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.point_locator.print_self()
    }

    /// VTK: `vtkNonMergingPointLocator::IsInsertedPoint(const double[3])`.
    pub fn is_inserted_point(&self, _x: [f64; 3]) -> VtkIdType {
        -1
    }

    /// VTK: `vtkNonMergingPointLocator::IsInsertedPoint(double, double, double)`.
    pub fn is_inserted_point_components(&self, _x: f64, _y: f64, _z: f64) -> VtkIdType {
        -1
    }

    /// VTK: `vtkNonMergingPointLocator::InsertUniquePoint`.
    pub fn insert_unique_point(&mut self, x: [f64; 3], pt_id: &mut VtkIdType) -> i32 {
        let points = self.point_locator.get_points();
        assert!(
            !points.is_null(),
            "vtkNonMergingPointLocator requires InitPointInsertion before InsertUniquePoint"
        );
        *pt_id = unsafe { (*points).insert_next_point(x) };
        1
    }

    /// VTK: `vtkPointLocator::SetDivisions`.
    pub fn set_divisions(&mut self, divisions: [i32; 3]) {
        self.point_locator.set_divisions(divisions);
    }

    /// VTK: `vtkPointLocator::SetDivisions(int, int, int)`.
    pub fn set_divisions_components(&mut self, i: i32, j: i32, k: i32) {
        self.point_locator.set_divisions_components(i, j, k);
    }

    /// VTK: `vtkPointLocator::GetDivisions`.
    pub fn get_divisions(&self) -> [i32; 3] {
        self.point_locator.get_divisions()
    }

    /// VTK: `vtkPointLocator::GetDivisions(int[3])`.
    pub fn get_divisions_into(&self, divisions: &mut [i32; 3]) {
        self.point_locator.get_divisions_into(divisions);
    }

    /// VTK: `vtkPointLocator::SetNumberOfPointsPerBucket`.
    pub fn set_number_of_points_per_bucket(&mut self, number_of_points_per_bucket: i32) {
        self.point_locator
            .set_number_of_points_per_bucket(number_of_points_per_bucket);
    }

    /// VTK: `vtkPointLocator::GetNumberOfPointsPerBucket`.
    pub fn get_number_of_points_per_bucket(&self) -> i32 {
        self.point_locator.get_number_of_points_per_bucket()
    }

    /// VTK: `vtkPointLocator::GetPoints`.
    pub fn get_points(&self) -> *mut Points {
        self.point_locator.get_points()
    }

    /// VTK: `vtkPointLocator::Initialize`.
    pub fn initialize(&mut self) {
        self.point_locator.initialize();
    }

    /// VTK: `vtkPointLocator::FreeSearchStructure`.
    pub fn free_search_structure(&mut self) {
        self.point_locator.free_search_structure();
    }

    /// VTK: `vtkPointLocator::ComputePerformanceFactors`.
    pub fn compute_performance_factors(&mut self) {
        self.point_locator.compute_performance_factors();
    }

    /// VTK: `vtkPointLocator::InitPointInsertion(vtkPoints*, const double[6])`.
    pub fn init_point_insertion(&mut self, new_pts: *mut Points, bounds: [f64; 6]) -> i32 {
        self.point_locator.init_point_insertion(new_pts, bounds)
    }

    /// VTK: `vtkPointLocator::InitPointInsertion(vtkPoints*, const double[6], vtkIdType)`.
    pub fn init_point_insertion_estimated(
        &mut self,
        new_pts: *mut Points,
        bounds: [f64; 6],
        est_num_pts: VtkIdType,
    ) -> i32 {
        self.point_locator
            .init_point_insertion_estimated(new_pts, bounds, est_num_pts)
    }

    /// VTK: `vtkPointLocator::InsertNextPoint`.
    pub fn insert_next_point(&mut self, x: [f64; 3]) -> VtkIdType {
        self.point_locator.insert_next_point(x)
    }

    /// VTK: `vtkPointLocator::InsertPoint`.
    pub fn insert_point(&mut self, pt_id: VtkIdType, x: [f64; 3]) {
        self.point_locator.insert_point(pt_id, x);
    }

    /// VTK: `vtkPointLocator::FindClosestInsertedPoint`.
    pub fn find_closest_inserted_point(&self, x: [f64; 3]) -> VtkIdType {
        self.point_locator.find_closest_inserted_point(x)
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.point_locator.get_bounds()
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds(double*)`.
    pub fn get_bounds_into(&self, bounds: &mut [f64; 6]) {
        self.point_locator.get_bounds_into(bounds);
    }

    /// VTK: `vtkAbstractPointLocator::GetNumberOfBuckets`.
    pub fn get_number_of_buckets(&self) -> VtkIdType {
        self.point_locator.get_number_of_buckets()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.point_locator.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.point_locator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.point_locator.get_m_time()
    }

    /// VTK: `vtkLocator::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut DataSet) {
        self.point_locator.set_data_set(data_set);
    }

    /// VTK: `vtkLocator::GetDataSet`.
    pub fn get_data_set(&self) -> *mut DataSet {
        self.point_locator.get_data_set()
    }

    /// VTK: `vtkLocator::SetMaxLevel`.
    pub fn set_max_level(&mut self, max_level: i32) {
        self.point_locator.set_max_level(max_level);
    }

    /// VTK: `vtkLocator::GetMaxLevel`.
    pub fn get_max_level(&self) -> i32 {
        self.point_locator.get_max_level()
    }

    /// VTK: `vtkLocator::GetLevel`.
    pub fn get_level(&self) -> i32 {
        self.point_locator.get_level()
    }

    /// VTK: `vtkLocator::SetAutomatic`.
    pub fn set_automatic(&mut self, automatic: bool) {
        self.point_locator.set_automatic(automatic);
    }

    /// VTK: `vtkLocator::GetAutomatic`.
    pub fn get_automatic(&self) -> bool {
        self.point_locator.get_automatic()
    }

    /// VTK: `vtkLocator::AutomaticOn`.
    pub fn automatic_on(&mut self) {
        self.point_locator.automatic_on();
    }

    /// VTK: `vtkLocator::AutomaticOff`.
    pub fn automatic_off(&mut self) {
        self.point_locator.automatic_off();
    }

    /// VTK: `vtkLocator::SetTolerance`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.point_locator.set_tolerance(tolerance);
    }

    /// VTK: `vtkLocator::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.point_locator.get_tolerance()
    }

    /// VTK: `vtkLocator::SetUseExistingSearchStructure`.
    pub fn set_use_existing_search_structure(&mut self, use_existing_search_structure: bool) {
        self.point_locator
            .set_use_existing_search_structure(use_existing_search_structure);
    }

    /// VTK: `vtkLocator::GetUseExistingSearchStructure`.
    pub fn get_use_existing_search_structure(&self) -> bool {
        self.point_locator.get_use_existing_search_structure()
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOn`.
    pub fn use_existing_search_structure_on(&mut self) {
        self.point_locator.use_existing_search_structure_on();
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOff`.
    pub fn use_existing_search_structure_off(&mut self) {
        self.point_locator.use_existing_search_structure_off();
    }

    /// VTK: `vtkLocator::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.point_locator.get_build_time()
    }

    /// VTK: `vtkLocator::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        self.point_locator.uses_garbage_collector()
    }
}

impl Default for NonMergingPointLocator {
    fn default() -> Self {
        Self::new()
    }
}
