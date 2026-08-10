use std::{cell::RefCell, rc::Rc};

use crate::common::core::VtkMTimeType;

use super::CellGridQuery;

/// Rust boundary for the `vtkCellAttribute*` methods used by
/// `vtkCellGridRangeQuery`.
pub trait CellAttributeRangeApi {
    /// VTK: `vtkCellAttribute::GetNumberOfComponents`.
    fn get_number_of_components(&self) -> i32;
}

/// Rust equivalent of the `vtkCellAttribute*` stored by `vtkCellGridRangeQuery`.
pub type CellAttributeHandle = Rc<RefCell<dyn CellAttributeRangeApi>>;

/// Placeholder boundary for `vtkCellGrid*` until `vtkCellGrid` range-cache
/// ownership is translated.
pub type CellGridHandle = *mut std::ffi::c_void;

/// VTK: `vtkCellGridRangeQuery::ComponentRange`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentRange {
    pub finite_range_time: VtkMTimeType,
    pub finite_range: [f64; 2],
    pub entire_range_time: VtkMTimeType,
    pub entire_range: [f64; 2],
}

/// VTK: `vtkCellGridRangeQuery`.
#[derive(Clone)]
pub struct CellGridRangeQuery {
    query: CellGridQuery,
    component: i32,
    finite_range: bool,
    cell_grid: CellGridHandle,
    cell_attribute: Option<CellAttributeHandle>,
    ranges: Vec<[f64; 2]>,
}

impl CellGridRangeQuery {
    /// VTK: `vtkCellGridRangeQuery::New`.
    pub fn new() -> Self {
        Self {
            query: CellGridQuery::with_class_name("vtkCellGridRangeQuery"),
            component: -2,
            finite_range: false,
            cell_grid: std::ptr::null_mut(),
            cell_attribute: None,
            ranges: Vec::new(),
        }
    }

    /// VTK: `vtkCellGridRangeQuery::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = self.query.print_self();
        result.push_str(&format!("Component: {}\n", self.component));
        result.push_str(&format!(
            "FiniteRange: {}\n",
            if self.finite_range { "ON" } else { "OFF" }
        ));
        result.push_str(&format!("CellGrid: {:?}\n", self.cell_grid));
        result.push_str(&format!(
            "CellAttribute: {}\n",
            if self.cell_attribute.is_some() {
                "Some"
            } else {
                "None"
            }
        ));
        result.push_str("Ranges:\n");
        for (offset, range) in self.ranges.iter().enumerate() {
            let component = offset as i32 - 2;
            let label = match component {
                -2 => "L2-norm".to_string(),
                -1 => "L1-norm".to_string(),
                _ => format!("Component {component}"),
            };
            result.push_str(&format!("  {label}: {} {}\n", range[0], range[1]));
        }
        result
    }

    /// VTK: `vtkCellGridRangeQuery::SetComponent`.
    pub fn set_component(&mut self, component: i32) {
        if self.component != component {
            self.component = component;
            self.modified();
        }
    }

    /// VTK: `vtkCellGridRangeQuery::GetComponent`.
    pub fn get_component(&self) -> i32 {
        self.component
    }

    /// VTK: `vtkCellGridRangeQuery::SetFiniteRange`.
    pub fn set_finite_range(&mut self, finite_range: bool) {
        if self.finite_range != finite_range {
            self.finite_range = finite_range;
            self.modified();
        }
    }

    /// VTK: `vtkCellGridRangeQuery::GetFiniteRange`.
    pub fn get_finite_range(&self) -> bool {
        self.finite_range
    }

    /// VTK: `vtkCellGridRangeQuery::SetCellGrid`.
    pub fn set_cell_grid(&mut self, cell_grid: CellGridHandle) {
        if self.cell_grid != cell_grid {
            self.cell_grid = cell_grid;
            self.modified();
        }
    }

    /// VTK: `vtkCellGridRangeQuery::GetCellGrid`.
    pub fn get_cell_grid(&self) -> CellGridHandle {
        self.cell_grid
    }

    /// VTK: `vtkCellGridRangeQuery::SetCellAttribute`.
    pub fn set_cell_attribute(&mut self, cell_attribute: Option<CellAttributeHandle>) {
        if option_handle_ptr_eq(&self.cell_attribute, &cell_attribute) {
            return;
        }
        self.cell_attribute = cell_attribute;
        self.modified();
    }

    /// VTK: `vtkCellGridRangeQuery::GetCellAttribute`.
    pub fn get_cell_attribute(&self) -> Option<CellAttributeHandle> {
        self.cell_attribute.clone()
    }

    /// VTK: `vtkCellGridRangeQuery::Initialize`.
    pub fn initialize(&mut self) -> bool {
        let ok = self.query.initialize();
        let Some(cell_attribute) = &self.cell_attribute else {
            return false;
        };

        let number_of_components = cell_attribute.borrow().get_number_of_components();
        if number_of_components < 0 {
            self.ranges.clear();
            return false;
        }

        self.ranges = vec![invalid_accumulation_range(); number_of_components as usize + 2];
        ok
    }

    /// VTK: `vtkCellGridRangeQuery::Finalize`.
    pub fn finalize(&mut self) -> bool {
        self.query.finalize()
    }

    /// VTK: `vtkCellGridRangeQuery::GetRange(int, double*)`.
    pub fn get_range_into(&self, component: i32, range: &mut [f64; 2]) {
        *range = self.get_range(component);
    }

    /// VTK: `vtkCellGridRangeQuery::GetRange(int)`.
    pub fn get_range(&self, component: i32) -> [f64; 2] {
        if !self.is_valid_component(component) {
            return invalid_return_range();
        }
        self.ranges
            .get((component + 2) as usize)
            .copied()
            .unwrap_or_else(invalid_return_range)
    }

    /// VTK: `vtkCellGridRangeQuery::GetRange(double*)`.
    pub fn get_range_current_into(&self, range: &mut [f64; 2]) {
        self.get_range_into(self.component, range);
    }

    /// VTK: `vtkCellGridRangeQuery::GetRange()`.
    pub fn get_range_current(&self) -> [f64; 2] {
        self.get_range(self.component)
    }

    /// VTK: `vtkCellGridRangeQuery::AddRange(const std::array<double, 2>&)`.
    pub fn add_range(&mut self, range: [f64; 2]) {
        self.add_range_for_component(self.component, range);
    }

    /// VTK: `vtkCellGridRangeQuery::AddRange(int, const std::array<double, 2>&)`.
    pub fn add_range_for_component(&mut self, component: i32, range: [f64; 2]) {
        if range[1] < range[0] || !self.is_valid_component(component) {
            return;
        }

        let slot = &mut self.ranges[(component + 2) as usize];
        if slot[1] < slot[0] {
            *slot = range;
            return;
        }

        slot[0] = slot[0].min(range[0]);
        slot[1] = slot[1].max(range[1]);
    }

    /// VTK: `vtkCellGridQuery::StartPass`.
    pub fn start_pass(&mut self) {
        self.query.start_pass();
    }

    /// VTK: `vtkCellGridQuery::GetPass`.
    pub fn get_pass(&self) -> i32 {
        self.query.get_pass()
    }

    /// VTK: `vtkCellGridQuery::IsAnotherPassRequired`.
    pub fn is_another_pass_required(&self) -> bool {
        self.query.is_another_pass_required()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.query.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.query.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.query.get_m_time()
    }

    fn is_valid_component(&self, component: i32) -> bool {
        let Some(cell_attribute) = &self.cell_attribute else {
            return false;
        };
        component >= -2 && component < cell_attribute.borrow().get_number_of_components()
    }
}

impl std::fmt::Debug for CellGridRangeQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellGridRangeQuery")
            .field("query", &self.query)
            .field("component", &self.component)
            .field("finite_range", &self.finite_range)
            .field("cell_grid", &self.cell_grid)
            .field("cell_attribute", &self.cell_attribute.is_some())
            .field("ranges", &self.ranges)
            .finish()
    }
}

impl Default for CellGridRangeQuery {
    fn default() -> Self {
        Self::new()
    }
}

fn invalid_accumulation_range() -> [f64; 2] {
    [f64::INFINITY, f64::NEG_INFINITY]
}

fn invalid_return_range() -> [f64; 2] {
    [1.0, 0.0]
}

fn option_handle_ptr_eq(
    left: &Option<CellAttributeHandle>,
    right: &Option<CellAttributeHandle>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}
