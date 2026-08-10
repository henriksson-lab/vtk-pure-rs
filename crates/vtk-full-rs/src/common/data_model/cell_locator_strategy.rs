use std::cell::RefCell;
use std::rc::Rc;

use super::{
    AbstractCellLocator, AbstractCellLocatorApi, CellApi, FindCellStrategy, FindCellStrategyApi,
    GenericCellHandle, PointSet,
};
use crate::common::core::{VtkIdType, VtkMTimeType};

/// Rust equivalent of `vtkAbstractCellLocator*` for locator-backed
/// `vtkFindCellStrategy` dispatch.
pub type AbstractCellLocatorHandle = Rc<RefCell<dyn AbstractCellLocatorApi>>;

/// VTK: `vtkCellLocatorStrategy`.
#[derive(Clone)]
pub struct CellLocatorStrategy {
    find_cell_strategy: FindCellStrategy,
    cell_locator: Option<AbstractCellLocatorHandle>,
}

impl CellLocatorStrategy {
    /// VTK: `vtkCellLocatorStrategy::New`.
    pub fn new() -> Self {
        Self::with_class_name("vtkCellLocatorStrategy")
    }

    /// VTK: `vtkCellLocatorStrategy::vtkCellLocatorStrategy`.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            find_cell_strategy: FindCellStrategy::with_class_name(class_name),
            cell_locator: None,
        }
    }

    /// VTK: `vtkCellLocatorStrategy::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}CellLocator: {}\n",
            self.find_cell_strategy.print_self(),
            if self.cell_locator.is_some() {
                "set"
            } else {
                "(none)"
            }
        )
    }

    /// VTK: `vtkCellLocatorStrategy::SetCellLocator`.
    pub fn set_cell_locator(&mut self, cell_locator: Option<AbstractCellLocatorHandle>) {
        let changed = match (&self.cell_locator, &cell_locator) {
            (Some(current), Some(next)) => !Rc::ptr_eq(current, next),
            (None, None) => false,
            _ => true,
        };

        if changed {
            self.cell_locator = cell_locator;
            self.find_cell_strategy.set_owns_locator(true);
            self.modified();
        }
    }

    /// VTK: `vtkCellLocatorStrategy::GetCellLocator`.
    pub fn get_cell_locator(&self) -> Option<AbstractCellLocatorHandle> {
        self.cell_locator.as_ref().map(Rc::clone)
    }

    /// VTK typed base accessor for callers that need an already translated
    /// `vtkAbstractCellLocator` value.
    pub fn set_abstract_cell_locator(
        &mut self,
        cell_locator: Option<Rc<RefCell<AbstractCellLocator>>>,
    ) {
        self.set_cell_locator(cell_locator.map(|locator| locator as AbstractCellLocatorHandle));
    }

    /// VTK: `vtkCellLocatorStrategy::Initialize`.
    pub fn initialize(&mut self, ps: Option<&mut PointSet>) -> i32 {
        if let Some(point_set) = ps.as_ref() {
            let cached = !self.find_cell_strategy.get_point_set().is_null()
                && self.find_cell_strategy.get_point_set().cast_const()
                    == *point_set as *const PointSet
                && self.find_cell_strategy.get_m_time()
                    < self.find_cell_strategy.get_initialize_time();
            if cached {
                return 1;
            }
        }

        let Some(ps) = ps else {
            return 0;
        };

        if self.find_cell_strategy.initialize(Some(ps)) == 0 {
            return 0;
        }

        let ps_locator = ps.get_cell_locator();
        if let Some(locator) = ps_locator {
            let changed = self
                .cell_locator
                .as_ref()
                .map(|current| !Rc::ptr_eq(current, &locator))
                .unwrap_or(true);
            if changed {
                self.cell_locator = Some(locator);
                self.find_cell_strategy.set_owns_locator(false);
            }
            if !self.find_cell_strategy.get_is_a_copy() {
                if let Some(locator) = self.cell_locator.as_ref() {
                    locator.borrow_mut().build_locator();
                }
            }
        } else if let Some(locator) = self.cell_locator.as_ref() {
            if self.find_cell_strategy.get_owns_locator() {
                let point_set = self.find_cell_strategy.get_point_set();
                let mut locator = locator.borrow_mut();
                locator.set_data_set(point_set);
                locator.build_locator();
            } else if !self.find_cell_strategy.get_is_a_copy() {
                locator.borrow_mut().build_locator();
            }
        } else {
            ps.build_cell_locator();
            if let Some(locator) = ps.get_cell_locator() {
                self.cell_locator = Some(locator);
                self.find_cell_strategy.set_owns_locator(false);
            } else {
                return 0;
            }
        }

        self.find_cell_strategy.initialize_time_modified();
        1
    }

    /// VTK: `vtkCellLocatorStrategy::FindCell`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_cell(
        &mut self,
        x: [f64; 3],
        cell: Option<&mut dyn CellApi>,
        gencell: GenericCellHandle,
        cell_id: VtkIdType,
        tol2: f64,
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: &mut [f64],
    ) -> VtkIdType {
        if let Some(cell) = cell {
            if cell_id >= 0 {
                let mut closest_point = [0.0; 3];
                let mut dist2 = 0.0;
                if cell.evaluate_position(
                    x,
                    &mut closest_point,
                    sub_id,
                    pcoords,
                    &mut dist2,
                    weights,
                ) == 1
                    && dist2 <= tol2
                {
                    return cell_id;
                }
            }
        }

        self.cell_locator
            .as_ref()
            .map(|locator| {
                locator
                    .borrow_mut()
                    .find_cell(x, tol2, gencell, sub_id, pcoords, weights)
            })
            .unwrap_or(-1)
    }

    /// VTK: `vtkCellLocatorStrategy::FindClosestPointWithinRadius`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_closest_point_within_radius(
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
        self.cell_locator
            .as_ref()
            .map(|locator| {
                locator.borrow_mut().find_closest_point_within_radius(
                    x,
                    radius,
                    closest_point,
                    cell,
                    cell_id,
                    sub_id,
                    dist2,
                    inside,
                )
            })
            .unwrap_or(0)
    }

    /// VTK: `vtkCellLocatorStrategy::InsideCellBounds`.
    pub fn inside_cell_bounds(&self, x: [f64; 3], cell_id: VtkIdType) -> bool {
        self.cell_locator
            .as_ref()
            .map(|locator| locator.borrow().inside_cell_bounds(x, cell_id))
            .unwrap_or(false)
    }

    /// VTK: `vtkCellLocatorStrategy::CopyParameters`.
    pub fn copy_parameters(&mut self, from: &Self) {
        self.find_cell_strategy
            .copy_parameters(from.find_cell_strategy());
        if let Some(locator) = from.cell_locator.as_ref() {
            self.cell_locator = Some(Rc::clone(locator));
            self.find_cell_strategy.set_owns_locator(false);
        }
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.find_cell_strategy.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.find_cell_strategy.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.find_cell_strategy.get_m_time()
    }

    /// VTK base access used by translated subclasses and strategy copies.
    pub fn find_cell_strategy(&self) -> &FindCellStrategy {
        &self.find_cell_strategy
    }

    /// VTK base access used by translated subclasses.
    pub fn find_cell_strategy_mut(&mut self) -> &mut FindCellStrategy {
        &mut self.find_cell_strategy
    }
}

impl Default for CellLocatorStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CellLocatorStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellLocatorStrategy")
            .field("find_cell_strategy", &self.find_cell_strategy)
            .field("cell_locator", &self.cell_locator.is_some())
            .finish()
    }
}

impl PartialEq for CellLocatorStrategy {
    fn eq(&self, other: &Self) -> bool {
        self.find_cell_strategy == other.find_cell_strategy
            && match (&self.cell_locator, &other.cell_locator) {
                (Some(lhs), Some(rhs)) => Rc::ptr_eq(lhs, rhs),
                (None, None) => true,
                _ => false,
            }
    }
}

impl FindCellStrategyApi for CellLocatorStrategy {
    fn find_cell_strategy(&self) -> &FindCellStrategy {
        self.find_cell_strategy()
    }

    fn find_cell_strategy_mut(&mut self) -> &mut FindCellStrategy {
        self.find_cell_strategy_mut()
    }

    fn find_cell(
        &mut self,
        x: [f64; 3],
        cell: Option<&mut dyn CellApi>,
        gencell: GenericCellHandle,
        cell_id: VtkIdType,
        tol2: f64,
        sub_id: &mut i32,
        pcoords: &mut [f64; 3],
        weights: &mut [f64],
    ) -> VtkIdType {
        self.find_cell(x, cell, gencell, cell_id, tol2, sub_id, pcoords, weights)
    }

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
        self.find_closest_point_within_radius(
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

    fn inside_cell_bounds(&self, x: [f64; 3], cell_id: VtkIdType) -> bool {
        self.inside_cell_bounds(x, cell_id)
    }
}

impl Drop for CellLocatorStrategy {
    fn drop(&mut self) {
        if self.find_cell_strategy.get_owns_locator() {
            self.cell_locator = None;
        }
    }
}
