use crate::common::core::{IdList, Points, VtkIdType};

use super::{Cell, CellBaseApi, CellType};

/// Rust return bundle for VTK `vtkEmptyCell::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyCellEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkEmptyCell::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmptyCellIntersectWithLine {
    pub intersection: i32,
}

/// VTK: `vtkEmptyCell`.
#[derive(Debug)]
pub struct EmptyCell {
    cell: Cell,
}

impl EmptyCell {
    /// VTK: `vtkEmptyCell::New`.
    pub fn new() -> Self {
        Self {
            cell: Cell::with_class_name("vtkEmptyCell"),
        }
    }

    /// VTK: `vtkEmptyCell::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.cell.print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.cell.get_m_time()
    }

    /// VTK: `vtkCell::GetPoints`.
    pub fn get_points(&self) -> &Points {
        self.cell.get_points()
    }

    /// VTK: `vtkCell::GetPointIds`.
    pub fn get_point_ids(&self) -> &IdList {
        self.cell.get_point_ids()
    }

    /// VTK: `vtkCell::GetPointId`.
    pub fn get_point_id(&self, pt_id: i32) -> VtkIdType {
        self.cell.get_point_id(pt_id)
    }

    /// VTK: `vtkCell::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.cell.get_number_of_points()
    }

    /// VTK: `vtkCell::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.cell.get_bounds()
    }

    /// VTK: `vtkCell::GetLength2`.
    pub fn get_length2(&self) -> f64 {
        self.cell.get_length2()
    }

    /// VTK: `vtkCell::ComputeBoundingSphere`.
    pub fn compute_bounding_sphere(&self) -> ([f64; 3], f64) {
        self.cell.compute_bounding_sphere()
    }

    /// VTK: `vtkCell::Initialize`.
    pub fn initialize(&mut self) {
        self.cell.initialize()
    }

    /// VTK: `vtkCell::Initialize(int, const vtkIdType*, vtkPoints*)`.
    pub fn initialize_with_point_ids(&mut self, npts: i32, pts: &[VtkIdType], p: &Points) {
        self.cell.initialize_with_point_ids(npts, pts, p)
    }

    /// VTK: `vtkCell::Initialize(int, vtkPoints*)`.
    pub fn initialize_from_points(&mut self, npts: i32, p: &Points) {
        self.cell.initialize_from_points(npts, p)
    }

    /// VTK: `vtkCell::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.cell.shallow_copy(&source.cell)
    }

    /// VTK: `vtkCell::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.cell.deep_copy(&source.cell)
    }

    /// VTK: `vtkEmptyCell::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Empty as i32
    }

    /// VTK: `vtkEmptyCell::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        0
    }

    /// VTK: `vtkEmptyCell::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        0
    }

    /// VTK: `vtkEmptyCell::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkEmptyCell::GetEdge`.
    pub fn get_edge(&self, _edge_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkEmptyCell::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkEmptyCell::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        _x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> EmptyCellEvaluatePosition {
        if let Some(closest_point) = closest_point {
            *closest_point = [0.0, 0.0, 0.0];
        }
        EmptyCellEvaluatePosition {
            inside: 0,
            sub_id: 0,
            pcoords: [-1.0, -1.0, -1.0],
            dist2: -1.0,
            closest_point: Some([0.0, 0.0, 0.0]),
        }
    }

    /// VTK: `vtkEmptyCell::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, _pcoords: [f64; 3]) -> ([f64; 3], Vec<f64>) {
        ([0.0, 0.0, 0.0], Vec::new())
    }

    /// VTK: `vtkEmptyCell::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, _pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        pts.reset();
        0
    }

    /// VTK: `vtkEmptyCell::Contour`.
    pub fn contour(&self) {}

    /// VTK: `vtkEmptyCell::Clip`.
    pub fn clip(&self) {}

    /// VTK: `vtkEmptyCell::IntersectWithLine`.
    pub fn intersect_with_line(
        &self,
        _p1: [f64; 3],
        _p2: [f64; 3],
        _tol: f64,
    ) -> EmptyCellIntersectWithLine {
        EmptyCellIntersectWithLine { intersection: 0 }
    }

    /// VTK: `vtkEmptyCell::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.reset();
        1
    }

    /// VTK: `vtkEmptyCell::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        _pcoords: [f64; 3],
        _values: &[f64],
        _dim: i32,
        _derivs: &mut [f64],
    ) {
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }
}

impl Default for EmptyCell {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for EmptyCell {
    fn cell(&self) -> &Cell {
        self.cell()
    }

    fn cell_mut(&mut self) -> &mut Cell {
        self.cell_mut()
    }

    fn get_cell_type(&self) -> i32 {
        self.get_cell_type()
    }

    fn get_cell_dimension(&self) -> i32 {
        self.get_cell_dimension()
    }

    fn get_number_of_edges(&self) -> i32 {
        self.get_number_of_edges()
    }

    fn get_number_of_faces(&self) -> i32 {
        self.get_number_of_faces()
    }

    fn triangulate_local_ids(&self, index: i32, pt_ids: &mut IdList) -> i32 {
        self.triangulate_local_ids(index, pt_ids)
    }
}
