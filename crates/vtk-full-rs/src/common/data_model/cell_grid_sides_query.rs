use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use crate::common::core::{IdTypeArray, StringToken, VtkIdType, VtkMTimeType};

use super::{CellGridQuery, CellGridSidesCache};

/// VTK: `vtkCellGridSidesQuery::SideFlags`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SideFlags;

impl SideFlags {
    pub const VERTICES_OF_EDGES: i32 = 0x01;
    pub const VERTICES_OF_SURFACES: i32 = 0x02;
    pub const EDGES_OF_SURFACES: i32 = 0x04;
    pub const VERTICES_OF_VOLUMES: i32 = 0x08;
    pub const EDGES_OF_VOLUMES: i32 = 0x10;
    pub const SURFACES_OF_VOLUMES: i32 = 0x20;
    pub const SURFACES_OF_INPUTS: i32 = 0x20;
    pub const EDGES_OF_INPUTS: i32 = 0x14;
    pub const VERTICES_OF_INPUTS: i32 = 0x0b;
    pub const ALL_SIDES: i32 = 0x3f;
    pub const NEXT_LOWEST_DIMENSION: i32 = 0x25;
}

/// VTK: `vtkCellGridSidesQuery::PassWork`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PassWork {
    HashSides = 0,
    Summarize = 1,
    GenerateSideSets = 2,
}

impl PassWork {
    fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Summarize,
            2 => Self::GenerateSideSets,
            _ => Self::HashSides,
        }
    }
}

/// VTK: `vtkCellGridSidesQuery::SummaryStrategy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryStrategy {
    Winding,
    AnyOccurrence,
    Boundary,
}

/// VTK: `vtkCellGridSidesQuery::SelectionMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Input,
    Output,
}

/// VTK: `vtkCellGridSidesQuery::SideSetArray`.
#[derive(Debug, Clone, PartialEq)]
pub struct SideSetArray {
    pub cell_type: StringToken,
    pub side_shape: StringToken,
    pub sides: IdTypeArray,
}

pub type SideIdSet = BTreeSet<i32>;
pub type SidesByCellId = HashMap<VtkIdType, SideIdSet>;
pub type SidesByShape = HashMap<StringToken, SidesByCellId>;
pub type SidesByCellType = HashMap<StringToken, SidesByShape>;

/// VTK: `vtkCellGridSidesQuery`.
#[derive(Debug, Clone, PartialEq)]
pub struct CellGridSidesQuery {
    query: CellGridQuery,
    preserve_renderable_inputs: bool,
    omit_sides_for_renderable_inputs: bool,
    output_dimension_control: i32,
    selection_type: SelectionMode,
    strategy: SummaryStrategy,
    side_cache: Option<Rc<RefCell<CellGridSidesCache>>>,
    temporary_side_cache: bool,
    sides: SidesByCellType,
}

impl CellGridSidesQuery {
    /// VTK: `vtkCellGridSidesQuery::New`.
    pub fn new() -> Self {
        Self {
            query: CellGridQuery::with_class_name("vtkCellGridSidesQuery"),
            preserve_renderable_inputs: false,
            omit_sides_for_renderable_inputs: false,
            output_dimension_control: SideFlags::SURFACES_OF_INPUTS,
            selection_type: SelectionMode::Input,
            strategy: SummaryStrategy::Boundary,
            side_cache: None,
            temporary_side_cache: false,
            sides: HashMap::new(),
        }
    }

    /// VTK: `vtkCellGridSidesQuery::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "SideCache: {}\nSides: {}\nPreserveRenderableInputs: {}\nOmitSidesForRenderableInputs: {}\nOutputDimensionControl: {:x}\nSelectionType: {}\nSummaryStrategy: {}\n",
            if self.side_cache.is_some() { "Some" } else { "None" },
            self.sides.len(),
            if self.preserve_renderable_inputs { "Y" } else { "N" },
            if self.omit_sides_for_renderable_inputs { "Y" } else { "N" },
            self.output_dimension_control,
            Self::selection_mode_to_label(self.selection_type).data(),
            Self::summary_strategy_to_label(self.strategy).data()
        )
    }

    /// VTK: `vtkCellGridSidesQuery::SetPreserveRenderableInputs`.
    pub fn set_preserve_renderable_inputs(&mut self, value: bool) {
        self.preserve_renderable_inputs = value;
    }

    /// VTK: `vtkCellGridSidesQuery::GetPreserveRenderableInputs`.
    pub fn get_preserve_renderable_inputs(&self) -> bool {
        self.preserve_renderable_inputs
    }

    /// VTK: `vtkCellGridSidesQuery::PreserveRenderableInputsOn`.
    pub fn preserve_renderable_inputs_on(&mut self) {
        self.set_preserve_renderable_inputs(true);
    }

    /// VTK: `vtkCellGridSidesQuery::PreserveRenderableInputsOff`.
    pub fn preserve_renderable_inputs_off(&mut self) {
        self.set_preserve_renderable_inputs(false);
    }

    /// VTK: `vtkCellGridSidesQuery::SetOmitSidesForRenderableInputs`.
    pub fn set_omit_sides_for_renderable_inputs(&mut self, value: bool) {
        self.omit_sides_for_renderable_inputs = value;
    }

    /// VTK: `vtkCellGridSidesQuery::GetOmitSidesForRenderableInputs`.
    pub fn get_omit_sides_for_renderable_inputs(&self) -> bool {
        self.omit_sides_for_renderable_inputs
    }

    /// VTK: `vtkCellGridSidesQuery::OmitSidesForRenderableInputsOn`.
    pub fn omit_sides_for_renderable_inputs_on(&mut self) {
        self.set_omit_sides_for_renderable_inputs(true);
    }

    /// VTK: `vtkCellGridSidesQuery::OmitSidesForRenderableInputsOff`.
    pub fn omit_sides_for_renderable_inputs_off(&mut self) {
        self.set_omit_sides_for_renderable_inputs(false);
    }

    /// VTK: `vtkCellGridSidesQuery::SetOutputDimensionControl`.
    pub fn set_output_dimension_control(&mut self, value: i32) {
        self.output_dimension_control = value;
    }

    /// VTK: `vtkCellGridSidesQuery::GetOutputDimensionControl`.
    pub fn get_output_dimension_control(&self) -> i32 {
        self.output_dimension_control
    }

    /// VTK: `vtkCellGridSidesQuery::OutputDimensionControlOn`.
    pub fn output_dimension_control_on(&mut self) {
        self.set_output_dimension_control(1);
    }

    /// VTK: `vtkCellGridSidesQuery::OutputDimensionControlOff`.
    pub fn output_dimension_control_off(&mut self) {
        self.set_output_dimension_control(0);
    }

    /// VTK: `vtkCellGridSidesQuery::SetStrategy`.
    pub fn set_strategy(&mut self, strategy: SummaryStrategy) {
        self.strategy = strategy;
    }

    /// VTK: `vtkCellGridSidesQuery::SetStrategy(int)`.
    pub fn set_strategy_i32(&mut self, strategy: i32) {
        self.set_strategy(match strategy {
            0 => SummaryStrategy::Winding,
            1 => SummaryStrategy::AnyOccurrence,
            _ => SummaryStrategy::Boundary,
        });
    }

    /// VTK: `vtkCellGridSidesQuery::GetStrategy`.
    pub fn get_strategy(&self) -> SummaryStrategy {
        self.strategy
    }

    /// VTK: `vtkCellGridSidesQuery::SetStrategyToWinding`.
    pub fn set_strategy_to_winding(&mut self) {
        self.set_strategy(SummaryStrategy::Winding);
    }

    /// VTK: `vtkCellGridSidesQuery::SetStrategyToAnyOccurrence`.
    pub fn set_strategy_to_any_occurrence(&mut self) {
        self.set_strategy(SummaryStrategy::AnyOccurrence);
    }

    /// VTK: `vtkCellGridSidesQuery::SetStrategyToBoundary`.
    pub fn set_strategy_to_boundary(&mut self) {
        self.set_strategy(SummaryStrategy::Boundary);
    }

    /// VTK: `vtkCellGridSidesQuery::SetSelectionType`.
    pub fn set_selection_type(&mut self, selection_type: SelectionMode) {
        self.selection_type = selection_type;
    }

    /// VTK: `vtkCellGridSidesQuery::SetSelectionType(int)`.
    pub fn set_selection_type_i32(&mut self, selection_type: i32) {
        self.set_selection_type(match selection_type {
            1 => SelectionMode::Output,
            _ => SelectionMode::Input,
        });
    }

    /// VTK: `vtkCellGridSidesQuery::GetSelectionType`.
    pub fn get_selection_type(&self) -> SelectionMode {
        self.selection_type
    }

    /// VTK: `vtkCellGridSidesQuery::Initialize`.
    pub fn initialize(&mut self) -> bool {
        let ok = self.query.initialize();
        if self.side_cache.is_none() {
            self.temporary_side_cache = true;
            self.side_cache = Some(Rc::new(RefCell::new(CellGridSidesCache::new())));
        } else if let Some(side_cache) = &self.side_cache {
            if self.get_m_time() > side_cache.borrow().get_m_time() {
                side_cache.borrow_mut().initialize();
            }
        }
        ok
    }

    /// VTK: `vtkCellGridSidesQuery::StartPass`.
    pub fn start_pass(&mut self) {
        self.query.start_pass();
        if PassWork::from_i32(self.query.get_pass()) == PassWork::Summarize {
            self.sides.clear();
        }
    }

    /// VTK: `vtkCellGridSidesQuery::IsAnotherPassRequired`.
    pub fn is_another_pass_required(&self) -> bool {
        self.query.get_pass() < PassWork::GenerateSideSets as i32
    }

    /// VTK: `vtkCellGridSidesQuery::Finalize`.
    pub fn finalize(&mut self) -> bool {
        self.sides.clear();
        if self.temporary_side_cache {
            self.side_cache = None;
            self.temporary_side_cache = false;
        }
        true
    }

    /// VTK: `vtkCellGridSidesQuery::GetSides`.
    pub fn get_sides(&mut self) -> &mut SidesByCellType {
        &mut self.sides
    }

    /// VTK: `vtkCellGridSidesQuery::GetSideSetArrays`.
    pub fn get_side_set_arrays(&self, cell_type: StringToken) -> Vec<SideSetArray> {
        let mut result = Vec::new();
        let Some(side_shapes) = self.sides.get(&cell_type) else {
            return result;
        };

        for (side_shape, entries) in side_shapes {
            let side_count: VtkIdType = entries.values().map(|set| set.len() as VtkIdType).sum();
            let mut side_array = IdTypeArray::new();
            side_array.set_name("conn");
            side_array.set_number_of_components(2);
            side_array.set_number_of_tuples(side_count);
            let mut side_id = 0;
            for (cell_id, side_ids) in entries {
                for side in side_ids {
                    side_array.set_typed_tuple(side_id, &[*cell_id, VtkIdType::from(*side)]);
                    side_id += 1;
                }
            }
            result.push(SideSetArray {
                cell_type,
                side_shape: *side_shape,
                sides: side_array,
            });
        }

        result
    }

    /// VTK: `vtkCellGridSidesQuery::SelectionModeToLabel`.
    pub fn selection_mode_to_label(mode: SelectionMode) -> StringToken {
        match mode {
            SelectionMode::Input => StringToken::new_from_str("Input"),
            SelectionMode::Output => StringToken::new_from_str("Output"),
        }
    }

    /// VTK: `vtkCellGridSidesQuery::SelectionModeFromLabel`.
    pub fn selection_mode_from_label(token: StringToken) -> SelectionMode {
        match token.get_id() {
            id if id == StringToken::string_hash("Output") => SelectionMode::Output,
            _ => SelectionMode::Input,
        }
    }

    /// VTK: `vtkCellGridSidesQuery::SummaryStrategyToLabel`.
    pub fn summary_strategy_to_label(strategy: SummaryStrategy) -> StringToken {
        match strategy {
            SummaryStrategy::Winding => StringToken::new_from_str("Winding"),
            SummaryStrategy::AnyOccurrence => StringToken::new_from_str("AnyOccurrence"),
            SummaryStrategy::Boundary => StringToken::new_from_str("Boundary"),
        }
    }

    /// VTK: `vtkCellGridSidesQuery::SummaryStrategyFromLabel`.
    pub fn summary_strategy_from_label(token: StringToken) -> SummaryStrategy {
        match token.get_id() {
            id if id == StringToken::string_hash("Winding") => SummaryStrategy::Winding,
            id if id == StringToken::string_hash("AnyOccurrence") => SummaryStrategy::AnyOccurrence,
            _ => SummaryStrategy::Boundary,
        }
    }

    /// VTK: `vtkCellGridSidesQuery::GetSideCache`.
    pub fn get_side_cache(&self) -> Option<Rc<RefCell<CellGridSidesCache>>> {
        self.side_cache.clone()
    }

    /// VTK: `vtkCellGridSidesQuery::SetSideCache`.
    pub fn set_side_cache(&mut self, cache: Option<Rc<RefCell<CellGridSidesCache>>>) {
        if same_cache(&self.side_cache, &cache) {
            return;
        }
        self.side_cache = cache;
        self.temporary_side_cache = self.side_cache.is_none();
        self.modified();
    }

    /// VTK: `vtkCellGridQuery::GetPass`.
    pub fn get_pass(&self) -> i32 {
        self.query.get_pass()
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
}

impl Default for CellGridSidesQuery {
    fn default() -> Self {
        Self::new()
    }
}

fn same_cache(
    left: &Option<Rc<RefCell<CellGridSidesCache>>>,
    right: &Option<Rc<RefCell<CellGridSidesCache>>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}
