use crate::common::core::{
    math::{distance2_between_points, dot},
    IdList, Points, VtkIdType,
};

use super::{Cell, CellBaseApi, CellType};

/// Rust return bundle for VTK `vtkVertex::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 1],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkVertex::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VertexIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkVertex`.
#[derive(Debug)]
pub struct Vertex {
    cell: Cell,
}

impl Vertex {
    /// VTK: `vtkVertex::New`.
    pub fn new() -> Self {
        let mut vertex = Self {
            cell: Cell::with_class_name("vtkVertex"),
        };
        vertex.cell.get_points_mut().set_number_of_points(1);
        vertex.cell.get_point_ids_mut().set_number_of_ids(1);
        vertex.cell.get_points_mut().set_point(0, [0.0, 0.0, 0.0]);
        vertex.cell.get_point_ids_mut().set_id(0, 0);
        vertex
    }

    /// VTK: `vtkVertex::PrintSelf`.
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

    /// VTK: `vtkVertex::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Vertex as i32
    }

    /// VTK: `vtkVertex::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        0
    }

    /// VTK: `vtkVertex::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        0
    }

    /// VTK: `vtkVertex::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkVertex::GetEdge`.
    pub fn get_edge(&self, _edge_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkVertex::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkVertex::Inflate`.
    pub fn inflate(&mut self, _dist: f64) -> i32 {
        0
    }

    /// VTK: `vtkVertex::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> VertexEvaluatePosition {
        let point = self.cell.get_points().get_point(0);
        if let Some(closest_point) = closest_point {
            *closest_point = point;
        }

        let dist2 = distance2_between_points(point, x);
        let inside = (dist2 == 0.0) as i32;
        VertexEvaluatePosition {
            inside,
            sub_id: 0,
            pcoords: [if inside != 0 { 0.0 } else { -1.0 }, 0.0, 0.0],
            dist2,
            weights: [1.0],
            closest_point: Some(point),
        }
    }

    /// VTK: `vtkVertex::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, _pcoords: [f64; 3]) -> ([f64; 3], [f64; 1]) {
        (self.cell.get_points().get_point(0), [1.0])
    }

    /// VTK: `vtkVertex::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        pts.set_number_of_ids(1);
        pts.set_id(0, self.cell.get_point_ids().get_id(0));
        (pcoords[0] == 0.0) as i32
    }

    /// VTK: `vtkVertex::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.0, 0.0, 0.0])
    }

    /// VTK: `vtkVertex::IntersectWithLine`.
    pub fn intersect_with_line(
        &self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> VertexIntersectWithLine {
        let point = self.cell.get_points().get_point(0);
        let ray = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let ray_factor = dot(ray, ray);
        if ray_factor == 0.0 {
            return VertexIntersectWithLine {
                intersection: 0,
                t: 0.0,
                x: [0.0; 3],
                pcoords: [0.0, 0.0, 0.0],
                sub_id: 0,
            };
        }

        let t = dot(ray, [point[0] - p1[0], point[1] - p1[1], point[2] - p1[2]]) / ray_factor;
        if (0.0..=1.0).contains(&t) {
            let proj = [p1[0] + t * ray[0], p1[1] + t * ray[1], p1[2] + t * ray[2]];
            if (0..3).all(|i| (point[i] - proj[i]).abs() <= tol) {
                return VertexIntersectWithLine {
                    intersection: 1,
                    t,
                    x: point,
                    pcoords: [0.0, 0.0, 0.0],
                    sub_id: 0,
                };
            }
        }

        VertexIntersectWithLine {
            intersection: 0,
            t,
            x: [0.0; 3],
            pcoords: [-1.0, 0.0, 0.0],
            sub_id: 0,
        }
    }

    /// VTK: `vtkVertex::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(1);
        pt_ids.set_id(0, 0);
        1
    }

    /// VTK: `vtkVertex::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        _pcoords: [f64; 3],
        _values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let dim = dim.max(0) as usize;
        let required = if dim == 0 { 0 } else { (dim - 1) * dim + 3 };
        assert!(
            derivs.len() >= required,
            "vtkVertex::Derivatives derivs slice too short"
        );
        for i in 0..dim {
            let idx = i * dim;
            derivs[idx] = 0.0;
            derivs[idx + 1] = 0.0;
            derivs[idx + 2] = 0.0;
        }
    }

    /// VTK: `vtkVertex::InterpolationFunctions`.
    pub fn interpolation_functions(_pcoords: [f64; 3]) -> [f64; 1] {
        [1.0]
    }

    /// VTK: `vtkVertex::InterpolationDerivs`.
    pub fn interpolation_derivs(_pcoords: [f64; 3]) -> [f64; 3] {
        [0.0, 0.0, 0.0]
    }

    /// VTK: `vtkVertex::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        assert!(
            !weights.is_empty(),
            "vtkVertex::InterpolateFunctions weights slice too short"
        );
        weights[0] = Self::interpolation_functions(pcoords)[0];
    }

    /// VTK: `vtkVertex::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        assert!(
            derivs.len() >= 3,
            "vtkVertex::InterpolateDerivs derivs slice too short"
        );
        derivs[..3].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkVertex::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 3] {
        &VERTEX_CELL_PCOORDS
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for Vertex {
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

const VERTEX_CELL_PCOORDS: [f64; 3] = [0.0, 0.0, 0.0];
