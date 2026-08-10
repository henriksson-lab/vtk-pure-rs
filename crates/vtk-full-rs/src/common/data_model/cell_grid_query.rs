use crate::common::core::{Object, VtkMTimeType};

/// VTK: `vtkCellGridQuery`.
#[derive(Debug, Clone, PartialEq)]
pub struct CellGridQuery {
    object: Object,
    pass: i32,
}

impl CellGridQuery {
    /// VTK: `vtkCellGridQuery::vtkCellGridQuery`.
    pub(crate) fn new() -> Self {
        Self::with_class_name("vtkCellGridQuery")
    }

    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            pass: -1,
        }
    }

    /// VTK: `vtkCellGridQuery::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("Pass: {}\n", self.pass)
    }

    /// VTK: `vtkCellGridQuery::Initialize`.
    pub fn initialize(&mut self) -> bool {
        self.pass = -1;
        true
    }

    /// VTK: `vtkCellGridQuery::StartPass`.
    pub fn start_pass(&mut self) {
        self.pass += 1;
    }

    /// VTK: `vtkCellGridQuery::GetPass`.
    pub fn get_pass(&self) -> i32 {
        self.pass
    }

    /// VTK: `vtkCellGridQuery::IsAnotherPassRequired`.
    pub fn is_another_pass_required(&self) -> bool {
        false
    }

    /// VTK: `vtkCellGridQuery::Finalize`.
    pub fn finalize(&mut self) -> bool {
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

impl Default for CellGridQuery {
    fn default() -> Self {
        Self::new()
    }
}
