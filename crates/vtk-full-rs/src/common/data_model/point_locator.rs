use super::{BoundingBox, DataSet, IncrementalPointLocator, Locator};
use crate::common::core::{IdList, Points, VtkIdType, VtkMTimeType};

/// VTK: file-local `VTK_INITIAL_SIZE`.
#[allow(dead_code)]
pub(crate) const VTK_POINT_LOCATOR_INITIAL_SIZE: i32 = 1000;

/// VTK: `vtkPointLocator`.
#[derive(Debug, Clone)]
pub struct PointLocator {
    incremental_point_locator: IncrementalPointLocator,
    points: *mut Points,
    divisions: [i32; 3],
    number_of_points_per_bucket: i32,
    hash_table: Vec<Option<IdList>>,
    h: [f64; 3],
    insertion_tol2: f64,
    insertion_point_id: VtkIdType,
    insertion_level: f64,
    hx: f64,
    hy: f64,
    hz: f64,
    fx: f64,
    fy: f64,
    fz: f64,
    bx: f64,
    by: f64,
    bz: f64,
    xd: VtkIdType,
    yd: VtkIdType,
    zd: VtkIdType,
    slice_size: VtkIdType,
}

impl PointLocator {
    /// VTK: `vtkPointLocator::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkPointLocator")
    }

    /// VTK: `vtkPointLocator::vtkPointLocator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            incremental_point_locator: IncrementalPointLocator::with_class_name(class_name),
            points: std::ptr::null_mut(),
            divisions: [50, 50, 50],
            number_of_points_per_bucket: 3,
            hash_table: Vec::new(),
            h: [0.0; 3],
            insertion_tol2: 0.0001,
            insertion_point_id: 0,
            insertion_level: 0.0,
            hx: 0.0,
            hy: 0.0,
            hz: 0.0,
            fx: 0.0,
            fy: 0.0,
            fz: 0.0,
            bx: 0.0,
            by: 0.0,
            bz: 0.0,
            xd: 0,
            yd: 0,
            zd: 0,
            slice_size: 0,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn incremental_point_locator(&self) -> &IncrementalPointLocator {
        &self.incremental_point_locator
    }

    #[allow(dead_code)]
    pub(crate) fn incremental_point_locator_mut(&mut self) -> &mut IncrementalPointLocator {
        &mut self.incremental_point_locator
    }

    #[allow(dead_code)]
    pub(crate) fn locator(&self) -> &Locator {
        self.incremental_point_locator.locator()
    }

    #[allow(dead_code)]
    pub(crate) fn locator_mut(&mut self) -> &mut Locator {
        self.incremental_point_locator.locator_mut()
    }

    #[allow(dead_code)]
    pub(crate) fn bucket_for_point(&self, x: [f64; 3]) -> Option<&IdList> {
        let idx = self.get_bucket_index(x) as usize;
        self.hash_table.get(idx).and_then(Option::as_ref)
    }

    #[allow(dead_code)]
    pub(crate) fn insert_current_point_in_bucket(&mut self, x: [f64; 3]) -> Option<VtkIdType> {
        let idx = self.get_bucket_index(x) as usize;
        if idx >= self.hash_table.len() || self.points.is_null() {
            return None;
        }

        let point_id = self.insertion_point_id;
        let bucket = self.hash_table[idx].get_or_insert_with(IdList::new);
        bucket.reserve((self.number_of_points_per_bucket / 2) as VtkIdType);
        bucket.insert_next_id(point_id);
        unsafe {
            (*self.points).insert_point(point_id, x);
        }
        self.insertion_point_id += 1;
        Some(point_id)
    }

    /// VTK: `vtkPointLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        let points = if self.points.is_null() {
            "(none)".to_string()
        } else {
            format!("{:?}", self.points)
        };
        format!(
            "{}Number of Points Per Bucket: {}\nDivisions: ({}, {}, {})\nPoints: {}\n",
            self.incremental_point_locator.print_self(),
            self.number_of_points_per_bucket,
            self.divisions[0],
            self.divisions[1],
            self.divisions[2],
            points
        )
    }

    /// VTK: `vtkPointLocator::SetDivisions`.
    pub fn set_divisions(&mut self, divisions: [i32; 3]) {
        if self.divisions != divisions {
            self.divisions = divisions;
            self.modified();
        }
    }

    /// VTK: `vtkPointLocator::SetDivisions(int, int, int)`.
    pub fn set_divisions_components(&mut self, i: i32, j: i32, k: i32) {
        self.set_divisions([i, j, k]);
    }

    /// VTK: `vtkPointLocator::GetDivisions`.
    pub fn get_divisions(&self) -> [i32; 3] {
        self.divisions
    }

    /// VTK: `vtkPointLocator::GetDivisions(int[3])`.
    pub fn get_divisions_into(&self, divisions: &mut [i32; 3]) {
        *divisions = self.divisions;
    }

    /// VTK: `vtkPointLocator::SetNumberOfPointsPerBucket`.
    pub fn set_number_of_points_per_bucket(&mut self, number_of_points_per_bucket: i32) {
        let number_of_points_per_bucket = number_of_points_per_bucket.clamp(1, i32::MAX);
        if self.number_of_points_per_bucket != number_of_points_per_bucket {
            self.number_of_points_per_bucket = number_of_points_per_bucket;
            self.modified();
        }
    }

    /// VTK: `vtkPointLocator::GetNumberOfPointsPerBucket`.
    pub fn get_number_of_points_per_bucket(&self) -> i32 {
        self.number_of_points_per_bucket
    }

    /// VTK: `vtkPointLocator::GetPoints`.
    pub fn get_points(&self) -> *mut Points {
        self.points
    }

    /// VTK: `vtkPointLocator::Initialize`.
    pub fn initialize(&mut self) {
        self.points = std::ptr::null_mut();
        self.free_search_structure();
    }

    /// VTK: `vtkPointLocator::FreeSearchStructure`.
    pub fn free_search_structure(&mut self) {
        self.hash_table.clear();
        self.incremental_point_locator
            .abstract_point_locator_mut()
            .set_bounds_internal([
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ]);
        self.incremental_point_locator
            .abstract_point_locator_mut()
            .set_number_of_buckets_internal(0);
        self.h = [0.0; 3];
        self.divisions = [50, 50, 50];
    }

    /// VTK: `vtkPointLocator::ComputePerformanceFactors`.
    pub fn compute_performance_factors(&mut self) {
        let bounds = self.get_bounds();
        self.hx = self.h[0];
        self.hy = self.h[1];
        self.hz = self.h[2];
        self.fx = if self.h[0] != 0.0 {
            1.0 / self.h[0]
        } else {
            0.0
        };
        self.fy = if self.h[1] != 0.0 {
            1.0 / self.h[1]
        } else {
            0.0
        };
        self.fz = if self.h[2] != 0.0 {
            1.0 / self.h[2]
        } else {
            0.0
        };
        self.bx = bounds[0];
        self.by = bounds[2];
        self.bz = bounds[4];
        self.xd = self.divisions[0] as VtkIdType;
        self.yd = self.divisions[1] as VtkIdType;
        self.zd = self.divisions[2] as VtkIdType;
        self.slice_size = self.xd * self.yd;
    }

    /// VTK: `vtkPointLocator::InitPointInsertion(vtkPoints*, const double[6])`.
    pub fn init_point_insertion(&mut self, new_pts: *mut Points, bounds: [f64; 6]) -> i32 {
        self.init_point_insertion_estimated(new_pts, bounds, 0)
    }

    /// VTK: `vtkPointLocator::InitPointInsertion(vtkPoints*, const double[6], vtkIdType)`.
    pub fn init_point_insertion_estimated(
        &mut self,
        new_pts: *mut Points,
        bounds: [f64; 6],
        est_num_pts: VtkIdType,
    ) -> i32 {
        self.insertion_point_id = 0;
        if !self.hash_table.is_empty() {
            self.free_search_structure();
        }
        if new_pts.is_null() {
            return 0;
        }
        self.points = new_pts;

        let mut bbox = BoundingBox::new_with_bounds(bounds);
        let ndivs = if self.get_automatic() && est_num_pts > 0 {
            let num_buckets =
                (est_num_pts as f64 / self.number_of_points_per_bucket as f64) as VtkIdType;
            let divisions = bbox.compute_divisions(num_buckets);
            bbox.set_bounds(divisions.bounds);
            divisions.divisions
        } else {
            bbox.inflate_to_non_zero_volume();
            self.divisions.map(|division| division.max(1))
        };

        self.divisions = ndivs;
        let bounds = bbox.get_bounds();
        self.incremental_point_locator
            .abstract_point_locator_mut()
            .set_bounds_internal(bounds);
        let number_of_buckets =
            ndivs[0] as VtkIdType * ndivs[1] as VtkIdType * ndivs[2] as VtkIdType;
        self.incremental_point_locator
            .abstract_point_locator_mut()
            .set_number_of_buckets_internal(number_of_buckets);
        self.hash_table = vec![None; number_of_buckets.max(0) as usize];

        for axis in 0..3 {
            self.h[axis] = (bounds[2 * axis + 1] - bounds[2 * axis]) / ndivs[axis] as f64;
        }

        self.insertion_tol2 = self.get_tolerance() * self.get_tolerance();
        let mut max_divs = 0;
        let mut hmin = f64::MAX;
        for axis in 0..3 {
            hmin = hmin.min(self.h[axis]);
            max_divs = max_divs.max(self.divisions[axis]);
        }
        self.insertion_level = (self.get_tolerance() / hmin).ceil().min(max_divs as f64);
        self.compute_performance_factors();

        self.insertion_point_id = unsafe { (*new_pts).get_number_of_points() };
        1
    }

    /// VTK: `vtkPointLocator::InsertNextPoint`.
    pub fn insert_next_point(&mut self, x: [f64; 3]) -> VtkIdType {
        let point_id = self.insertion_point_id;
        self.insert_point(point_id, x);
        self.insertion_point_id += 1;
        point_id
    }

    /// VTK: `vtkPointLocator::InsertPoint`.
    pub fn insert_point(&mut self, pt_id: VtkIdType, x: [f64; 3]) {
        let idx = self.get_bucket_index(x) as usize;
        if idx >= self.hash_table.len() || self.points.is_null() {
            return;
        }
        let bucket = self.hash_table[idx].get_or_insert_with(IdList::new);
        bucket.reserve(self.number_of_points_per_bucket as VtkIdType);
        bucket.insert_next_id(pt_id);
        unsafe {
            (*self.points).insert_point(pt_id, x);
        }
    }

    /// VTK: `vtkPointLocator::IsInsertedPoint(double, double, double)`.
    pub fn is_inserted_point_components(&self, x: f64, y: f64, z: f64) -> VtkIdType {
        self.is_inserted_point([x, y, z])
    }

    /// VTK: `vtkPointLocator::IsInsertedPoint(const double[3])`.
    pub fn is_inserted_point(&self, x: [f64; 3]) -> VtkIdType {
        if self.points.is_null() || self.hash_table.is_empty() {
            return -1;
        }
        let ijk = self.get_bucket_indices(x);
        for level in 0..=self.insertion_level as i32 {
            for neighbor in self.bucket_neighbors(ijk, level) {
                let bucket_index = self.bucket_index_from_ijk(neighbor);
                if let Some(bucket) = self
                    .hash_table
                    .get(bucket_index as usize)
                    .and_then(Option::as_ref)
                {
                    for offset in 0..bucket.get_number_of_ids() {
                        let pt_id = bucket.get_id(offset);
                        let point = unsafe { (*self.points).get_point(pt_id) };
                        if distance2_between_points(x, point) <= self.insertion_tol2 {
                            return pt_id;
                        }
                    }
                }
            }
        }
        -1
    }

    /// VTK: `vtkPointLocator::InsertUniquePoint`.
    pub fn insert_unique_point(&mut self, x: [f64; 3], id: &mut VtkIdType) -> i32 {
        let point_id = self.is_inserted_point(x);
        if point_id > -1 {
            *id = point_id;
            0
        } else {
            *id = self.insert_next_point(x);
            1
        }
    }

    /// VTK: `vtkPointLocator::FindClosestInsertedPoint`.
    pub fn find_closest_inserted_point(&self, x: [f64; 3]) -> VtkIdType {
        if self.points.is_null()
            || self.hash_table.is_empty()
            || !BoundingBox::new_with_bounds(self.get_bounds()).contains_point(x)
        {
            return -1;
        }

        let mut closest = -1;
        let mut min_dist2 = f64::MAX;
        for bucket in self.hash_table.iter().flatten() {
            for offset in 0..bucket.get_number_of_ids() {
                let pt_id = bucket.get_id(offset);
                let point = unsafe { (*self.points).get_point(pt_id) };
                let dist2 = distance2_between_points(x, point);
                if dist2 < min_dist2 {
                    min_dist2 = dist2;
                    closest = pt_id;
                }
            }
        }
        closest
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.incremental_point_locator.get_bounds()
    }

    /// VTK: `vtkAbstractPointLocator::GetBounds(double*)`.
    pub fn get_bounds_into(&self, bounds: &mut [f64; 6]) {
        self.incremental_point_locator.get_bounds_into(bounds);
    }

    /// VTK: `vtkAbstractPointLocator::GetNumberOfBuckets`.
    pub fn get_number_of_buckets(&self) -> VtkIdType {
        self.incremental_point_locator.get_number_of_buckets()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.incremental_point_locator.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.incremental_point_locator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.incremental_point_locator.get_m_time()
    }

    /// VTK: `vtkLocator::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut DataSet) {
        self.incremental_point_locator.set_data_set(data_set);
    }

    /// VTK: `vtkLocator::GetDataSet`.
    pub fn get_data_set(&self) -> *mut DataSet {
        self.incremental_point_locator.get_data_set()
    }

    /// VTK: `vtkLocator::SetMaxLevel`.
    pub fn set_max_level(&mut self, max_level: i32) {
        self.incremental_point_locator.set_max_level(max_level);
    }

    /// VTK: `vtkLocator::GetMaxLevel`.
    pub fn get_max_level(&self) -> i32 {
        self.incremental_point_locator.get_max_level()
    }

    /// VTK: `vtkLocator::GetLevel`.
    pub fn get_level(&self) -> i32 {
        self.incremental_point_locator.get_level()
    }

    /// VTK: `vtkLocator::SetAutomatic`.
    pub fn set_automatic(&mut self, automatic: bool) {
        self.incremental_point_locator.set_automatic(automatic);
    }

    /// VTK: `vtkLocator::GetAutomatic`.
    pub fn get_automatic(&self) -> bool {
        self.incremental_point_locator.get_automatic()
    }

    /// VTK: `vtkLocator::AutomaticOn`.
    pub fn automatic_on(&mut self) {
        self.incremental_point_locator.automatic_on();
    }

    /// VTK: `vtkLocator::AutomaticOff`.
    pub fn automatic_off(&mut self) {
        self.incremental_point_locator.automatic_off();
    }

    /// VTK: `vtkLocator::SetTolerance`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.incremental_point_locator.set_tolerance(tolerance);
    }

    /// VTK: `vtkLocator::GetTolerance`.
    pub fn get_tolerance(&self) -> f64 {
        self.incremental_point_locator.get_tolerance()
    }

    /// VTK: `vtkLocator::SetUseExistingSearchStructure`.
    pub fn set_use_existing_search_structure(&mut self, use_existing_search_structure: bool) {
        self.incremental_point_locator
            .set_use_existing_search_structure(use_existing_search_structure);
    }

    /// VTK: `vtkLocator::GetUseExistingSearchStructure`.
    pub fn get_use_existing_search_structure(&self) -> bool {
        self.incremental_point_locator
            .get_use_existing_search_structure()
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOn`.
    pub fn use_existing_search_structure_on(&mut self) {
        self.incremental_point_locator
            .use_existing_search_structure_on();
    }

    /// VTK: `vtkLocator::UseExistingSearchStructureOff`.
    pub fn use_existing_search_structure_off(&mut self) {
        self.incremental_point_locator
            .use_existing_search_structure_off();
    }

    /// VTK: `vtkLocator::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.incremental_point_locator.get_build_time()
    }

    /// VTK: `vtkLocator::UsesGarbageCollector`.
    pub fn uses_garbage_collector(&self) -> bool {
        self.incremental_point_locator.uses_garbage_collector()
    }

    fn get_bucket_indices(&self, x: [f64; 3]) -> [i32; 3] {
        [
            (((x[0] - self.bx) * self.fx) as VtkIdType).clamp(0, self.xd - 1) as i32,
            (((x[1] - self.by) * self.fy) as VtkIdType).clamp(0, self.yd - 1) as i32,
            (((x[2] - self.bz) * self.fz) as VtkIdType).clamp(0, self.zd - 1) as i32,
        ]
    }

    fn get_bucket_index(&self, x: [f64; 3]) -> VtkIdType {
        self.bucket_index_from_ijk(self.get_bucket_indices(x))
    }

    fn bucket_index_from_ijk(&self, ijk: [i32; 3]) -> VtkIdType {
        ijk[0] as VtkIdType + ijk[1] as VtkIdType * self.xd + ijk[2] as VtkIdType * self.slice_size
    }

    fn bucket_neighbors(&self, ijk: [i32; 3], level: i32) -> Vec<[i32; 3]> {
        let mut neighbors = Vec::new();
        for k in (ijk[2] - level).max(0)..=(ijk[2] + level).min(self.divisions[2] - 1) {
            for j in (ijk[1] - level).max(0)..=(ijk[1] + level).min(self.divisions[1] - 1) {
                for i in (ijk[0] - level).max(0)..=(ijk[0] + level).min(self.divisions[0] - 1) {
                    if (i - ijk[0])
                        .abs()
                        .max((j - ijk[1]).abs())
                        .max((k - ijk[2]).abs())
                        == level
                    {
                        neighbors.push([i, j, k]);
                    }
                }
            }
        }
        neighbors
    }
}

impl Default for PointLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PointLocator {
    fn drop(&mut self) {
        self.free_search_structure();
        self.points = std::ptr::null_mut();
    }
}

fn distance2_between_points(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}
