use super::{CellApi, DataSet, GenericCellHandle, Locator, PointSet};
use crate::common::core::{IdList, Points, VtkIdType, VtkMTimeType};

/// VTK virtual API subset used by abstract and concrete cell locators.
pub trait AbstractCellLocatorApi {
    fn abstract_cell_locator(&self) -> &AbstractCellLocator;
    fn abstract_cell_locator_mut(&mut self) -> &mut AbstractCellLocator;

    /// VTK: `vtkAbstractCellLocator::SetDataSet`.
    fn set_data_set(&mut self, point_set: *mut PointSet) {
        self.abstract_cell_locator_mut()
            .locator_mut()
            .set_data_set(point_set.cast::<DataSet>());
    }

    /// VTK: `vtkLocator::BuildLocator`.
    fn build_locator(&mut self);

    /// VTK: `vtkLocator::Initialize`.
    fn initialize(&mut self) {
        self.abstract_cell_locator_mut().free_cell_bounds();
    }

    /// VTK: `vtkAbstractCellLocator::FindCell`.
    #[allow(clippy::too_many_arguments)]
    fn find_cell(
        &mut self,
        x: [f64; 3],
        tol2: f64,
        cell: GenericCellHandle,
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: &mut [f64],
    ) -> VtkIdType {
        self.abstract_cell_locator_mut()
            .find_cell_with_sub_id(x, tol2, cell, sub_id, pcoords, weights)
    }

    /// VTK: `vtkAbstractCellLocator::FindClosestPointWithinRadius`.
    #[allow(clippy::too_many_arguments)]
    fn find_closest_point_within_radius(
        &mut self,
        x: [f64; 3],
        radius: f64,
        closest_point: &mut [f64; 3],
        cell: GenericCellHandle,
        cell_id: &mut VtkIdType,
        sub_id: &mut i32,
        dist2: &mut f64,
        inside: &mut i32,
    ) -> VtkIdType {
        self.abstract_cell_locator_mut()
            .find_closest_point_within_radius_inside(
                x,
                radius,
                closest_point,
                cell,
                cell_id,
                sub_id,
                dist2,
                inside,
            )
    }

    /// VTK: `vtkAbstractCellLocator::InsideCellBounds`.
    fn inside_cell_bounds(&self, x: [f64; 3], cell_id: VtkIdType) -> bool {
        self.abstract_cell_locator().inside_cell_bounds(x, cell_id)
    }
}

/// VTK: `vtkAbstractCellLocator`.
#[derive(Debug, Clone, PartialEq)]
pub struct AbstractCellLocator {
    locator: Locator,
    number_of_cells_per_node: i32,
    retain_cell_lists: bool,
    cache_cell_bounds: bool,
    generic_cell: GenericCellHandle,
    cell_bounds: Option<Vec<f64>>,
    weights_time: VtkMTimeType,
    weights: Vec<f64>,
}

impl AbstractCellLocator {
    /// VTK: `vtkAbstractCellLocator::vtkAbstractCellLocator`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        let mut locator = Locator::with_class_name(class_name);
        locator.set_max_level(8);
        locator.set_use_existing_search_structure(false);
        Self {
            locator,
            number_of_cells_per_node: 32,
            retain_cell_lists: true,
            cache_cell_bounds: true,
            generic_cell: std::ptr::null_mut(),
            cell_bounds: None,
            weights_time: 0,
            weights: Vec::new(),
        }
    }

    /// VTK: `vtkAbstractCellLocator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}Cache Cell Bounds: {}\nRetain Cell Lists: {}\nNumber of Cells Per Bucket: {}\n",
            self.locator.print_self(),
            i32::from(self.cache_cell_bounds),
            if self.retain_cell_lists { "On" } else { "Off" },
            self.number_of_cells_per_node
        )
    }

    /// VTK: `vtkAbstractCellLocator::SetNumberOfCellsPerNode`.
    pub fn set_number_of_cells_per_node(&mut self, number_of_cells_per_node: i32) {
        let number_of_cells_per_node = number_of_cells_per_node.clamp(1, i32::MAX);
        if self.number_of_cells_per_node != number_of_cells_per_node {
            self.number_of_cells_per_node = number_of_cells_per_node;
            self.modified();
        }
    }

    /// VTK: `vtkAbstractCellLocator::GetNumberOfCellsPerNode`.
    pub fn get_number_of_cells_per_node(&self) -> i32 {
        self.number_of_cells_per_node
    }

    /// VTK: `vtkAbstractCellLocator::SetCacheCellBounds`.
    pub fn set_cache_cell_bounds(&mut self, cache_cell_bounds: bool) {
        if self.cache_cell_bounds != cache_cell_bounds {
            self.cache_cell_bounds = cache_cell_bounds;
            self.modified();
        }
    }

    /// VTK: `vtkAbstractCellLocator::GetCacheCellBounds`.
    pub fn get_cache_cell_bounds(&self) -> bool {
        self.cache_cell_bounds
    }

    /// VTK: `vtkAbstractCellLocator::CacheCellBoundsOn`.
    pub fn cache_cell_bounds_on(&mut self) {
        self.set_cache_cell_bounds(true);
    }

    /// VTK: `vtkAbstractCellLocator::CacheCellBoundsOff`.
    pub fn cache_cell_bounds_off(&mut self) {
        self.set_cache_cell_bounds(false);
    }

    /// VTK: `vtkAbstractCellLocator::ComputeCellBounds`.
    pub fn compute_cell_bounds(&mut self) {
        if self.cache_cell_bounds {
            self.free_cell_bounds();
            self.store_cell_bounds();
        }
    }

    /// VTK: `vtkAbstractCellLocator::SetRetainCellLists`.
    pub fn set_retain_cell_lists(&mut self, retain_cell_lists: bool) {
        if self.retain_cell_lists != retain_cell_lists {
            self.retain_cell_lists = retain_cell_lists;
            self.modified();
        }
    }

    /// VTK: `vtkAbstractCellLocator::GetRetainCellLists`.
    pub fn get_retain_cell_lists(&self) -> bool {
        self.retain_cell_lists
    }

    /// VTK: `vtkAbstractCellLocator::RetainCellListsOn`.
    pub fn retain_cell_lists_on(&mut self) {
        self.set_retain_cell_lists(true);
    }

    /// VTK: `vtkAbstractCellLocator::RetainCellListsOff`.
    pub fn retain_cell_lists_off(&mut self) {
        self.set_retain_cell_lists(false);
    }

    /// VTK: `vtkAbstractCellLocator::IntersectWithLine`.
    #[allow(clippy::too_many_arguments)]
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
        t: &mut f64,
        x: &mut [f64; 3],
        pcoords: &mut [f64; 3],
        sub_id: &mut i32,
    ) -> i32 {
        let mut cell_id = -1;
        self.intersect_with_line_cell_id(p1, p2, tol, t, x, pcoords, sub_id, &mut cell_id)
    }

    /// VTK: `vtkAbstractCellLocator::IntersectWithLine`.
    #[allow(clippy::too_many_arguments)]
    pub fn intersect_with_line_cell_id(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
        t: &mut f64,
        x: &mut [f64; 3],
        pcoords: &mut [f64; 3],
        sub_id: &mut i32,
        cell_id: &mut VtkIdType,
    ) -> i32 {
        self.intersect_with_line_cell(
            p1,
            p2,
            tol,
            t,
            x,
            pcoords,
            sub_id,
            cell_id,
            self.generic_cell,
        )
    }

    /// VTK: `vtkAbstractCellLocator::IntersectWithLine`.
    #[allow(clippy::too_many_arguments)]
    pub fn intersect_with_line_cell(
        &mut self,
        _p1: [f64; 3],
        _p2: [f64; 3],
        _tol: f64,
        _t: &mut f64,
        _x: &mut [f64; 3],
        _pcoords: &mut [f64; 3],
        _sub_id: &mut i32,
        _cell_id: &mut VtkIdType,
        _cell: GenericCellHandle,
    ) -> i32 {
        0
    }

    /// VTK: `vtkAbstractCellLocator::IntersectWithLine`.
    pub fn intersect_with_line_points(
        &mut self,
        _p1: [f64; 3],
        _p2: [f64; 3],
        _points: Option<&mut Points>,
        _cell_ids: Option<&mut IdList>,
    ) -> i32 {
        0
    }

    /// VTK: `vtkAbstractCellLocator::IntersectWithLine`.
    pub fn intersect_with_line_points_tol(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
        points: Option<&mut Points>,
        cell_ids: Option<&mut IdList>,
    ) -> i32 {
        self.intersect_with_line_points_cell(p1, p2, tol, points, cell_ids, self.generic_cell)
    }

    /// VTK: `vtkAbstractCellLocator::IntersectWithLine`.
    pub fn intersect_with_line_points_cell(
        &mut self,
        _p1: [f64; 3],
        _p2: [f64; 3],
        _tol: f64,
        _points: Option<&mut Points>,
        _cell_ids: Option<&mut IdList>,
        _cell: GenericCellHandle,
    ) -> i32 {
        0
    }

    /// VTK: `vtkAbstractCellLocator::FindClosestPoint`.
    pub fn find_closest_point(
        &mut self,
        x: [f64; 3],
        closest_point: &mut [f64; 3],
        cell_id: &mut VtkIdType,
        sub_id: &mut i32,
        dist2: &mut f64,
    ) {
        self.find_closest_point_cell(x, closest_point, self.generic_cell, cell_id, sub_id, dist2);
    }

    /// VTK: `vtkAbstractCellLocator::FindClosestPoint`.
    pub fn find_closest_point_cell(
        &mut self,
        x: [f64; 3],
        closest_point: &mut [f64; 3],
        cell: GenericCellHandle,
        cell_id: &mut VtkIdType,
        sub_id: &mut i32,
        dist2: &mut f64,
    ) {
        let mut inside = 0;
        self.find_closest_point_within_radius_inside(
            x,
            f64::INFINITY,
            closest_point,
            cell,
            cell_id,
            sub_id,
            dist2,
            &mut inside,
        );
    }

    /// VTK: `vtkAbstractCellLocator::FindClosestPointWithinRadius`.
    pub fn find_closest_point_within_radius(
        &mut self,
        x: [f64; 3],
        radius: f64,
        closest_point: &mut [f64; 3],
        cell_id: &mut VtkIdType,
        sub_id: &mut i32,
        dist2: &mut f64,
    ) -> VtkIdType {
        let mut inside = 0;
        self.find_closest_point_within_radius_inside(
            x,
            radius,
            closest_point,
            self.generic_cell,
            cell_id,
            sub_id,
            dist2,
            &mut inside,
        )
    }

    /// VTK: `vtkAbstractCellLocator::FindClosestPointWithinRadius`.
    pub fn find_closest_point_within_radius_cell(
        &mut self,
        x: [f64; 3],
        radius: f64,
        closest_point: &mut [f64; 3],
        cell: GenericCellHandle,
        cell_id: &mut VtkIdType,
        sub_id: &mut i32,
        dist2: &mut f64,
    ) -> VtkIdType {
        let mut inside = 0;
        self.find_closest_point_within_radius_inside(
            x,
            radius,
            closest_point,
            cell,
            cell_id,
            sub_id,
            dist2,
            &mut inside,
        )
    }

    /// VTK: `vtkAbstractCellLocator::FindClosestPointWithinRadius`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_closest_point_within_radius_inside(
        &mut self,
        _x: [f64; 3],
        _radius: f64,
        _closest_point: &mut [f64; 3],
        _cell: GenericCellHandle,
        _cell_id: &mut VtkIdType,
        _sub_id: &mut i32,
        _dist2: &mut f64,
        _inside: &mut i32,
    ) -> VtkIdType {
        0
    }

    /// VTK: `vtkAbstractCellLocator::FindCellsWithinBounds`.
    pub fn find_cells_within_bounds(&self, _bbox: [f64; 6], _cells: Option<&mut IdList>) {}

    /// VTK: `vtkAbstractCellLocator::FindCellsAlongLine`.
    pub fn find_cells_along_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tolerance: f64,
        cells: Option<&mut IdList>,
    ) {
        self.intersect_with_line_points_cell(p1, p2, tolerance, None, cells, std::ptr::null_mut());
    }

    /// VTK: `vtkAbstractCellLocator::FindCellsAlongPlane`.
    pub fn find_cells_along_plane(
        &self,
        _o: [f64; 3],
        _n: [f64; 3],
        _tolerance: f64,
        _cells: Option<&mut IdList>,
    ) {
    }

    /// VTK: `vtkAbstractCellLocator::FindCell`.
    pub fn find_cell(&mut self, x: [f64; 3]) -> VtkIdType {
        self.update_internal_weights();
        let dist2 = 0.0;
        let mut pcoords = [0.0; 3];
        let mut weights = std::mem::take(&mut self.weights);
        let result = self.find_cell_with_cell(
            x,
            dist2,
            self.generic_cell,
            &mut pcoords,
            weights.as_mut_slice(),
        );
        self.weights = weights;
        result
    }

    /// VTK: `vtkAbstractCellLocator::FindCell`.
    pub fn find_cell_with_cell(
        &mut self,
        x: [f64; 3],
        tol2: f64,
        cell: GenericCellHandle,
        pcoords: &mut [f64; 3],
        weights: &mut [f64],
    ) -> VtkIdType {
        let mut sub_id = 0;
        self.find_cell_with_sub_id(x, tol2, cell, &mut sub_id, pcoords, weights)
    }

    /// VTK: `vtkAbstractCellLocator::FindCell`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_cell_with_sub_id(
        &mut self,
        _x: [f64; 3],
        _tol2: f64,
        _cell: GenericCellHandle,
        _sub_id: &mut i32,
        _pcoords: &mut [f64; 3],
        _weights: &mut [f64],
    ) -> VtkIdType {
        -1
    }

    /// VTK: `vtkAbstractCellLocator::InsideCellBounds`.
    pub fn inside_cell_bounds(&self, x: [f64; 3], cell_id: VtkIdType) -> bool {
        if self.cache_cell_bounds {
            if let Some(bounds) = self.get_cell_bounds(cell_id) {
                return Self::is_in_bounds(bounds, x, 0.0);
            }
        }
        false
    }

    /// VTK: `vtkAbstractCellLocator::ShallowCopy`.
    pub fn shallow_copy(&mut self, _locator: &Self) {}

    /// VTK: `vtkAbstractCellLocator::StoreCellBounds`.
    pub fn store_cell_bounds(&mut self) -> bool {
        if self.cell_bounds.is_some() || self.locator.get_data_set().is_null() {
            return false;
        }
        false
    }

    /// VTK: `vtkAbstractCellLocator::FreeCellBounds`.
    pub fn free_cell_bounds(&mut self) {
        self.cell_bounds = None;
    }

    /// VTK: `vtkAbstractCellLocator::UpdateInternalWeights`.
    pub fn update_internal_weights(&mut self) {
        if self.weights_time > self.get_m_time() || self.locator.get_data_set().is_null() {
            return;
        }
        self.weights.clear();
        self.weights_time = self.get_m_time();
    }

    /// VTK: `vtkAbstractCellLocator::IsInBounds`.
    pub fn is_in_bounds(bounds: [f64; 6], x: [f64; 3], tol: f64) -> bool {
        (bounds[0] - tol) <= x[0]
            && x[0] <= (bounds[1] + tol)
            && (bounds[2] - tol) <= x[1]
            && x[1] <= (bounds[3] + tol)
            && (bounds[4] - tol) <= x[2]
            && x[2] <= (bounds[5] + tol)
    }

    /// VTK: `vtkAbstractCellLocator::GetCellBounds`.
    pub fn get_cell_bounds(&self, cell_id: VtkIdType) -> Option<[f64; 6]> {
        if cell_id < 0 {
            return None;
        }
        let offset = cell_id as usize * 6;
        self.cell_bounds
            .as_ref()
            .and_then(|bounds| bounds.get(offset..offset + 6))
            .map(|bounds| {
                [
                    bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5],
                ]
            })
    }

    /// VTK base access used by translated subclasses.
    pub fn locator(&self) -> &Locator {
        &self.locator
    }

    /// VTK base access used by translated subclasses.
    pub fn locator_mut(&mut self) -> &mut Locator {
        &mut self.locator
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
}

impl Default for AbstractCellLocator {
    fn default() -> Self {
        Self::with_class_name("vtkAbstractCellLocator")
    }
}

impl AbstractCellLocatorApi for AbstractCellLocator {
    fn abstract_cell_locator(&self) -> &AbstractCellLocator {
        self
    }

    fn abstract_cell_locator_mut(&mut self) -> &mut AbstractCellLocator {
        self
    }

    fn build_locator(&mut self) {}
}

impl CellApi for AbstractCellLocator {
    fn evaluate_position(
        &mut self,
        _x: [f64; 3],
        _closest_point: &mut [f64; 3],
        _sub_id: &mut i32,
        _pcoords: &mut [f64; 3],
        _dist2: &mut f64,
        _weights: &mut [f64],
    ) -> i32 {
        0
    }
}
