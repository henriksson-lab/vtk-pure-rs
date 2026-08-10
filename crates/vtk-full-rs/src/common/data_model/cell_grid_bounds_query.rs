use crate::common::core::VtkMTimeType;

use super::{BoundingBox, CellGridQuery};

/// VTK: `vtkCellGridBoundsQuery`.
#[derive(Debug, Clone, PartialEq)]
pub struct CellGridBoundsQuery {
    query: CellGridQuery,
    bounds: [f64; 6],
}

impl CellGridBoundsQuery {
    /// VTK: `vtkCellGridBoundsQuery::New`.
    pub fn new() -> Self {
        Self {
            query: CellGridQuery::with_class_name("vtkCellGridBoundsQuery"),
            bounds: uninitialized_bounds(),
        }
    }

    /// VTK: `vtkCellGridBoundsQuery::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Bounds: {} {}, {} {}, {} {}\n",
            self.bounds[0],
            self.bounds[1],
            self.bounds[2],
            self.bounds[3],
            self.bounds[4],
            self.bounds[5]
        )
    }

    /// VTK: `vtkCellGridBoundsQuery::Initialize`.
    pub fn initialize(&mut self) -> bool {
        let ok = self.query.initialize();
        self.bounds = uninitialized_bounds();
        ok
    }

    /// VTK: `vtkCellGridBoundsQuery::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.bounds
    }

    /// VTK: `vtkCellGridBoundsQuery::AddBounds`.
    pub fn add_bounds(&mut self, bbox: &mut BoundingBox) {
        if !bbox.is_valid() {
            return;
        }
        if self.bounds[0] <= self.bounds[1] {
            bbox.add_point([self.bounds[0], self.bounds[2], self.bounds[4]]);
            bbox.add_point([self.bounds[1], self.bounds[3], self.bounds[5]]);
        }
        self.bounds = bbox.get_bounds();
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

    /// VTK: `vtkCellGridQuery::Finalize`.
    pub fn finalize(&mut self) -> bool {
        self.query.finalize()
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

impl Default for CellGridBoundsQuery {
    fn default() -> Self {
        Self::new()
    }
}

fn uninitialized_bounds() -> [f64; 6] {
    [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ]
}
