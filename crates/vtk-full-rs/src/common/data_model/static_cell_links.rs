use crate::common::core::{VtkIdType, VtkMTimeType};

use super::{
    AbstractCellLinks, AbstractCellLinksApi, AbstractCellLinksHandle, CellLinksTypes, DataSetApi,
    StaticCellLinksTemplate,
};

/// VTK: `vtkStaticCellLinks`.
#[derive(Clone, Debug)]
pub struct StaticCellLinks {
    abstract_cell_links: AbstractCellLinks,
    implementation: StaticCellLinksTemplate<VtkIdType>,
}

impl StaticCellLinks {
    /// VTK: `vtkStaticCellLinks::New`.
    pub fn new() -> Self {
        let implementation = StaticCellLinksTemplate::new();
        let mut abstract_cell_links = AbstractCellLinks::with_class_name("vtkStaticCellLinks");
        abstract_cell_links.set_type(CellLinksTypes::STATIC_CELL_LINKS_IDTYPE);
        Self {
            abstract_cell_links,
            implementation,
        }
    }

    /// VTK: `vtkStaticCellLinks::BuildLinks`.
    ///
    /// This is the faithful Rust entry point when the concrete dataset object is
    /// available as `DataSetApi`. The inherited no-argument `build_links`
    /// cannot recover concrete virtual topology from the current raw
    /// `vtkDataSet` base pointer representation.
    pub fn build_links_from_data_set(&mut self, data_set: &dyn DataSetApi) {
        if self.implementation.get_actual_memory_size() != 0
            && self.get_build_time() > self.get_m_time()
            && self.get_build_time() > data_set.data_set().get_m_time()
        {
            return;
        }
        self.implementation.build_links(data_set);
        self.abstract_cell_links.build_time_modified();
    }

    /// VTK: `vtkStaticCellLinks::GetNumberOfCells`.
    pub fn get_number_of_cells(&self, pt_id: VtkIdType) -> VtkIdType {
        self.implementation.get_number_of_cells(pt_id)
    }

    /// VTK: `vtkStaticCellLinks::GetNcells`.
    pub fn get_ncells(&self, pt_id: VtkIdType) -> VtkIdType {
        self.implementation.get_ncells(pt_id)
    }

    /// VTK: `vtkStaticCellLinks::GetCells`.
    pub fn get_cells(&self, pt_id: VtkIdType) -> &[VtkIdType] {
        self.implementation.get_cells(pt_id)
    }

    /// VTK: `vtkStaticCellLinks::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}Implementation: {:p}\n",
            self.abstract_cell_links.print_self(),
            &self.implementation
        )
    }

    /// VTK: `vtkAbstractCellLinks::SetDataSet`.
    pub fn set_data_set(&mut self, data_set: *mut super::DataSet) {
        self.abstract_cell_links.set_data_set(data_set);
    }

    /// VTK: `vtkAbstractCellLinks::GetDataSet`.
    pub fn get_data_set(&self) -> *mut super::DataSet {
        self.abstract_cell_links.get_data_set()
    }

    /// VTK: `vtkAbstractCellLinks::GetType`.
    pub fn get_type(&self) -> i32 {
        self.abstract_cell_links.get_type()
    }

    /// VTK: `vtkAbstractCellLinks::GetBuildTime`.
    pub fn get_build_time(&self) -> VtkMTimeType {
        self.abstract_cell_links.get_build_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.abstract_cell_links.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.abstract_cell_links.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.abstract_cell_links.get_m_time()
    }
}

impl Default for StaticCellLinks {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractCellLinksApi for StaticCellLinks {
    fn build_links(&mut self) {
        if self.get_data_set().is_null() {
            return;
        }
        self.modified();
    }

    fn initialize(&mut self) {
        self.implementation.initialize();
        self.modified();
    }

    fn squeeze(&mut self) {}

    fn reset(&mut self) {}

    fn get_actual_memory_size(&mut self) -> u64 {
        self.implementation.get_actual_memory_size()
    }

    fn deep_copy(&mut self, src: AbstractCellLinksHandle) {
        let Some(src) = (unsafe { (src as *const StaticCellLinks).as_ref() }) else {
            return;
        };
        if src.get_class_name() != "vtkStaticCellLinks" {
            return;
        }
        self.implementation.deep_copy(&src.implementation);
        self.abstract_cell_links.build_time_modified();
    }

    fn shallow_copy(&mut self, src: AbstractCellLinksHandle) {
        let Some(src) = (unsafe { (src as *const StaticCellLinks).as_ref() }) else {
            return;
        };
        if src.get_class_name() != "vtkStaticCellLinks" {
            return;
        }
        self.implementation.shallow_copy(&src.implementation);
        self.abstract_cell_links.build_time_modified();
    }

    fn select_cells(&mut self, min_max_degree: [VtkIdType; 2], cell_selection: &mut [u8]) {
        self.implementation
            .select_cells(min_max_degree, cell_selection);
    }
}
