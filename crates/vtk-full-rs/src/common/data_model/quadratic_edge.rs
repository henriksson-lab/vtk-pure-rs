use crate::common::core::{IdList, Points, VtkIdType, VTK_DOUBLE_MAX};

use super::{Cell, CellBaseApi, CellType, Line};

const QUADRATIC_EDGE_LINEAR_LINES: [[VtkIdType; 2]; 2] = [[0, 2], [2, 1]];
static QUADRATIC_EDGE_CELL_PCOORDS: [f64; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 0.0, 0.0];

/// Rust return bundle for VTK `vtkQuadraticEdge::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticEdgeEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 3],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkQuadraticEdge::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticEdgeIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkQuadraticEdge`.
#[derive(Debug)]
pub struct QuadraticEdge {
    cell: Cell,
    line: Line,
}

impl QuadraticEdge {
    /// VTK: `vtkQuadraticEdge::New`.
    pub fn new() -> Self {
        let mut edge = Self {
            cell: Cell::with_class_name("vtkQuadraticEdge"),
            line: Line::new(),
        };
        edge.cell.get_points_mut().set_number_of_points(3);
        edge.cell.get_point_ids_mut().set_number_of_ids(3);
        for i in 0..3 {
            edge.cell.get_points_mut().set_point(i, [0.0, 0.0, 0.0]);
            edge.cell.get_point_ids_mut().set_id(i, 0);
        }
        edge
    }

    /// VTK: `vtkQuadraticEdge::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell.print_self();
        text.push_str("Line:\n");
        text.push_str(&self.line.print_self());
        text
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.cell.get_m_time().max(self.line.get_m_time())
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

    /// VTK: `vtkQuadraticEdge::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::QuadraticEdge as i32
    }

    /// VTK: `vtkQuadraticEdge::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        1
    }

    /// VTK: `vtkQuadraticEdge::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        0
    }

    /// VTK: `vtkQuadraticEdge::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkQuadraticEdge::GetEdge`.
    pub fn get_edge(&self, _edge_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkQuadraticEdge::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkQuadraticEdge::CellBoundary`.
    pub fn cell_boundary(&self, sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        self.line.cell_boundary(sub_id, pcoords, pts)
    }

    /// VTK: `vtkQuadraticEdge::EvaluatePosition`.
    pub fn evaluate_position(
        &mut self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> QuadraticEdgeEvaluatePosition {
        let wants_closest_point = closest_point.is_some();
        let mut closest_point = closest_point;
        let mut return_status = -1;
        let mut sub_id = 0;
        let mut pcoords = [0.0, 0.0, 0.0];
        let mut min_dist2 = VTK_DOUBLE_MAX;

        for i in 0..2 {
            self.set_line_segment_points(i);
            let status = self.line.evaluate_position(x, None);
            if status.inside != -1
                && (status.dist2 < min_dist2 || (status.dist2 == min_dist2 && return_status == 0))
            {
                return_status = status.inside;
                min_dist2 = status.dist2;
                sub_id = i as i32;
                pcoords[0] = status.pcoords[0];
            }
        }

        let mut weights = [0.0; 3];
        let mut closest = None;
        if return_status != -1 {
            if sub_id == 0 {
                pcoords[0] /= 2.0;
            } else {
                pcoords[0] = 0.5 + pcoords[0] / 2.0;
            }

            if wants_closest_point {
                let (point, edge_weights) = self.evaluate_location(sub_id, pcoords);
                weights = edge_weights;
                if let Some(out) = closest_point.as_deref_mut() {
                    *out = point;
                }
                closest = Some(point);
            } else {
                weights = Self::interpolation_functions(pcoords);
            }
        }

        QuadraticEdgeEvaluatePosition {
            inside: return_status,
            sub_id,
            pcoords,
            dist2: min_dist2,
            weights,
            closest_point: if wants_closest_point { closest } else { None },
        }
    }

    /// VTK: `vtkQuadraticEdge::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 3]) {
        let a0 = self.cell.get_points().get_point(0);
        let a1 = self.cell.get_points().get_point(1);
        let a2 = self.cell.get_points().get_point(2);
        let weights = Self::interpolation_functions(pcoords);

        let mut x = [0.0; 3];
        for i in 0..3 {
            x[i] = a0[i] * weights[0] + a1[i] * weights[1] + a2[i] * weights[2];
        }
        (x, weights)
    }

    /// VTK: `vtkQuadraticEdge::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> QuadraticEdgeIntersectWithLine {
        let mut last_miss = None;
        for sub_id in 0..2 {
            self.set_line_segment_points(sub_id);
            let hit = self.line.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                return QuadraticEdgeIntersectWithLine {
                    intersection: 1,
                    t: hit.t,
                    x: hit.x,
                    pcoords: hit.pcoords,
                    sub_id: sub_id as i32,
                };
            }
            last_miss = Some(hit);
        }

        let (t, x, pcoords) = last_miss.map_or((0.0, [0.0; 3], [0.0; 3]), |miss| {
            (miss.t, miss.x, miss.pcoords)
        });
        QuadraticEdgeIntersectWithLine {
            intersection: 0,
            t,
            x,
            pcoords,
            sub_id: 2,
        }
    }

    /// VTK: `vtkQuadraticEdge::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(4);
        pt_ids.set_id(0, 0);
        pt_ids.set_id(1, 2);
        pt_ids.set_id(2, 2);
        pt_ids.set_id(3, 1);
        1
    }

    /// VTK: `vtkQuadraticEdge::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        _pcoords: [f64; 3],
        _values: &[f64],
        _dim: i32,
        _derivs: &mut [f64],
    ) {
    }

    /// VTK: `vtkQuadraticEdge::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.5, 0.0, 0.0])
    }

    /// VTK: `vtkQuadraticEdge::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 3] {
        let r = pcoords[0];
        [
            2.0 * (r - 0.5) * (r - 1.0),
            2.0 * r * (r - 0.5),
            4.0 * r * (1.0 - r),
        ]
    }

    /// VTK: `vtkQuadraticEdge::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 3] {
        let r = pcoords[0];
        [4.0 * r - 3.0, 4.0 * r - 1.0, 4.0 - r * 8.0]
    }

    /// VTK: `vtkQuadraticEdge::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        assert!(
            weights.len() >= 3,
            "vtkQuadraticEdge::InterpolateFunctions weights slice too short"
        );
        weights[..3].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkQuadraticEdge::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        assert!(
            derivs.len() >= 3,
            "vtkQuadraticEdge::InterpolateDerivs derivs slice too short"
        );
        derivs[..3].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkQuadraticEdge::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 9] {
        &QUADRATIC_EDGE_CELL_PCOORDS
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }

    fn set_line_segment_points(&mut self, sub_id: usize) {
        for i in 0..2 {
            self.line.cell_mut().get_points_mut().set_point(
                i as VtkIdType,
                self.cell
                    .get_points()
                    .get_point(QUADRATIC_EDGE_LINEAR_LINES[sub_id][i]),
            );
        }
    }
}

impl Default for QuadraticEdge {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for QuadraticEdge {
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
