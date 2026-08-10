use crate::common::core::{
    math::{cross, determinant2x2, distance2_between_points, dot, normalize},
    IdList, Points, VtkIdType,
};

use super::{Cell, CellBaseApi, CellType, Line, Plane, Triangle};

const VTK_DIVERGED: f64 = 1.0e6;
const VTK_QUAD_MAX_ITERATION: usize = 20;
const VTK_QUAD_CONVERGED: f64 = 1.0e-4;

/// Rust return bundle for VTK `vtkQuad::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 4],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkQuad::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

#[derive(Debug, Clone, Copy)]
struct Intersection {
    intersected: bool,
    sub_id: i32,
    x: [f64; 3],
    pcoords: [f64; 3],
    t: f64,
}

impl Intersection {
    fn from_triangle(triangle: &mut Triangle, p1: [f64; 3], p2: [f64; 3], tol: f64) -> Self {
        let result = triangle.intersect_with_line(p1, p2, tol);
        Self {
            intersected: result.intersection != 0,
            sub_id: result.sub_id,
            x: result.x,
            pcoords: result.pcoords,
            t: result.t,
        }
    }
}

/// VTK: `vtkQuad`.
#[derive(Debug)]
pub struct Quad {
    cell: Cell,
    line: Line,
    triangle: Triangle,
}

impl Quad {
    /// VTK: `vtkQuad::New`.
    pub fn new() -> Self {
        let mut quad = Self {
            cell: Cell::with_class_name("vtkQuad"),
            line: Line::new(),
            triangle: Triangle::new(),
        };
        quad.cell.get_points_mut().set_number_of_points(4);
        quad.cell.get_point_ids_mut().set_number_of_ids(4);
        for i in 0..4 {
            quad.cell.get_points_mut().set_point(i, [0.0, 0.0, 0.0]);
            quad.cell.get_point_ids_mut().set_id(i, 0);
        }
        quad
    }

    /// VTK: `vtkQuad::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell.print_self();
        text.push_str("\nLine:\n");
        text.push_str(&self.line.print_self());
        text.push_str("\nTriangle:\n");
        text.push_str(&self.triangle.print_self());
        text
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.cell
            .get_m_time()
            .max(self.line.get_m_time())
            .max(self.triangle.get_m_time())
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
        self.cell.shallow_copy(&source.cell);
    }

    /// VTK: `vtkCell::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.cell.deep_copy(&source.cell);
    }

    /// VTK: `vtkQuad::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Quad as i32
    }

    /// VTK: `vtkQuad::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkQuad::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        4
    }

    /// VTK: `vtkQuad::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkQuad::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut Line {
        let edge_id_plus_one = if edge_id + 1 > 3 { 0 } else { edge_id + 1 };
        let point_ids = [
            self.cell.get_point_ids().get_id(edge_id as VtkIdType),
            self.cell
                .get_point_ids()
                .get_id(edge_id_plus_one as VtkIdType),
        ];
        let points = [
            self.cell.get_points().get_point(edge_id as VtkIdType),
            self.cell
                .get_points()
                .get_point(edge_id_plus_one as VtkIdType),
        ];

        for i in 0..2 {
            self.line
                .cell_mut()
                .get_point_ids_mut()
                .set_id(i as VtkIdType, point_ids[i]);
            self.line
                .cell_mut()
                .get_points_mut()
                .set_point(i as VtkIdType, points[i]);
        }
        &mut self.line
    }

    /// VTK: `vtkQuad::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkQuad::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.5, 0.5, 0.0])
    }

    /// VTK: `vtkQuad::EvaluatePosition`.
    pub fn evaluate_position(&self, x: [f64; 3]) -> QuadEvaluatePosition {
        let mut sub_id = 0;
        let mut pcoords = [0.5, 0.5, 0.0];
        let mut params = [0.5, 0.5];
        let mut weights = [0.0; 4];

        let pts = [
            self.cell.get_points().get_point(0),
            self.cell.get_points().get_point(1),
            self.cell.get_points().get_point(2),
            self.cell.get_points().get_point(3),
        ];
        let n = self.compute_normal_from_first_non_collinear_triangle(pts[0], pts[1], pts[2]);
        let cp = Plane::project_point(x, pts[0], n);

        let mut idx = 0;
        let mut max_component = 0.0;
        for (i, value) in n.iter().enumerate() {
            if value.abs() > max_component {
                max_component = value.abs();
                idx = i;
            }
        }

        let mut indices = [0usize; 2];
        let mut j = 0;
        for i in 0..3 {
            if i != idx {
                indices[j] = i;
                j += 1;
            }
        }

        let mut converged = false;
        for _iteration in 0..VTK_QUAD_MAX_ITERATION {
            weights = Self::interpolation_functions(pcoords);
            let derivs = Self::interpolation_derivs(pcoords);
            let mut fcol = [0.0; 2];
            let mut rcol = [0.0; 2];
            let mut scol = [0.0; 2];

            for i in 0..4 {
                fcol[0] += pts[i][indices[0]] * weights[i];
                rcol[0] += pts[i][indices[0]] * derivs[i];
                scol[0] += pts[i][indices[0]] * derivs[i + 4];
                fcol[1] += pts[i][indices[1]] * weights[i];
                rcol[1] += pts[i][indices[1]] * derivs[i];
                scol[1] += pts[i][indices[1]] * derivs[i + 4];
            }

            fcol[0] -= cp[indices[0]];
            fcol[1] -= cp[indices[1]];
            let det = determinant2x2(rcol[0], scol[0], rcol[1], scol[1]);
            if det == 0.0 {
                return QuadEvaluatePosition {
                    inside: -1,
                    sub_id,
                    pcoords,
                    dist2: 0.0,
                    weights,
                    closest_point: None,
                };
            }

            pcoords[0] = params[0] - determinant2x2(fcol[0], scol[0], fcol[1], scol[1]) / det;
            pcoords[1] = params[1] - determinant2x2(rcol[0], fcol[0], rcol[1], fcol[1]) / det;

            if (pcoords[0] - params[0]).abs() < VTK_QUAD_CONVERGED
                && (pcoords[1] - params[1]).abs() < VTK_QUAD_CONVERGED
            {
                converged = true;
                break;
            }
            if pcoords[0].abs() > VTK_DIVERGED || pcoords[1].abs() > VTK_DIVERGED {
                return QuadEvaluatePosition {
                    inside: -1,
                    sub_id,
                    pcoords,
                    dist2: 0.0,
                    weights,
                    closest_point: None,
                };
            }
            params[0] = pcoords[0];
            params[1] = pcoords[1];
        }

        if !converged {
            return QuadEvaluatePosition {
                inside: -1,
                sub_id,
                pcoords,
                dist2: 0.0,
                weights,
                closest_point: None,
            };
        }

        weights = Self::interpolation_functions(pcoords);
        if pcoords[0] >= -0.001
            && pcoords[0] <= 1.001
            && pcoords[1] >= -0.001
            && pcoords[1] <= 1.001
        {
            QuadEvaluatePosition {
                inside: 1,
                sub_id,
                pcoords,
                dist2: distance2_between_points(cp, x),
                weights,
                closest_point: Some(cp),
            }
        } else {
            let (dist2, closest) = Self::closest_boundary_distance2(x, pts, pcoords);
            sub_id = 0;
            QuadEvaluatePosition {
                inside: 0,
                sub_id,
                pcoords,
                dist2,
                weights,
                closest_point: Some(closest),
            }
        }
    }

    /// VTK: `vtkQuad::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 4]) {
        let weights = Self::interpolation_functions(pcoords);
        let mut x = [0.0; 3];
        for i in 0..4 {
            let pt = self.cell.get_points().get_point(i);
            for j in 0..3 {
                x[j] += pt[j] * weights[i as usize];
            }
        }
        (x, weights)
    }

    /// VTK: `vtkQuad::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 4] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        [
            rm * sm,
            pcoords[0] * sm,
            pcoords[0] * pcoords[1],
            rm * pcoords[1],
        ]
    }

    /// VTK: `vtkQuad::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        weights[..4].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkQuad::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 8] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        [
            -sm,
            sm,
            pcoords[1],
            -pcoords[1],
            -rm,
            -pcoords[0],
            pcoords[0],
            rm,
        ]
    }

    /// VTK: `vtkQuad::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        derivs[..8].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkQuad::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let t1 = pcoords[0] - pcoords[1];
        let t2 = 1.0 - pcoords[0] - pcoords[1];
        pts.set_number_of_ids(2);
        let edge = if t1 >= 0.0 && t2 >= 0.0 {
            [0, 1]
        } else if t1 >= 0.0 && t2 < 0.0 {
            [1, 2]
        } else if t1 < 0.0 && t2 < 0.0 {
            [2, 3]
        } else {
            [3, 0]
        };
        pts.set_id(0, self.cell.get_point_ids().get_id(edge[0]));
        pts.set_id(1, self.cell.get_point_ids().get_id(edge[1]));
        (pcoords[0] >= 0.0 && pcoords[0] <= 1.0 && pcoords[1] >= 0.0 && pcoords[1] <= 1.0) as i32
    }

    /// VTK: `vtkQuad::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkQuad::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> QuadIntersectWithLine {
        let d1 = distance2_between_points(
            self.cell.get_points().get_point(0),
            self.cell.get_points().get_point(2),
        );
        let d2 = distance2_between_points(
            self.cell.get_points().get_point(1),
            self.cell.get_points().get_point(3),
        );
        let diagonal_case = if d1 == d2 {
            let mut max_id = 0;
            let mut max_idx = 0;
            for i in 0..4 {
                let id = self.cell.get_point_ids().get_id(i);
                if id > max_id {
                    max_id = id;
                    max_idx = i;
                }
            }
            if max_idx == 0 || max_idx == 2 {
                0
            } else {
                1
            }
        } else if d1 < d2 {
            0
        } else {
            1
        };

        let mut res = Intersection {
            intersected: false,
            sub_id: -1,
            x: [0.0; 3],
            pcoords: [0.0; 3],
            t: -1.0,
        };

        match diagonal_case {
            0 => {
                self.set_triangle_points([0, 1, 2]);
                let first = Intersection::from_triangle(&mut self.triangle, p1, p2, tol);
                self.set_triangle_points([2, 3, 0]);
                let second = Intersection::from_triangle(&mut self.triangle, p1, p2, tol);
                if first.intersected && (!second.intersected || first.t <= second.t) {
                    res = first;
                    res.pcoords[0] += res.pcoords[1];
                } else if second.intersected {
                    res = second;
                    res.pcoords[0] = 1.0 - (res.pcoords[0] + res.pcoords[1]);
                    res.pcoords[1] = 1.0 - res.pcoords[1];
                }
            }
            _ => {
                self.set_triangle_points([0, 1, 3]);
                let first = Intersection::from_triangle(&mut self.triangle, p1, p2, tol);
                self.set_triangle_points([2, 3, 1]);
                let second = Intersection::from_triangle(&mut self.triangle, p1, p2, tol);
                if first.intersected && (!second.intersected || first.t <= second.t) {
                    res = first;
                } else if second.intersected {
                    res = second;
                    res.pcoords[0] = 1.0 - res.pcoords[0];
                    res.pcoords[1] = 1.0 - res.pcoords[1];
                }
            }
        }

        QuadIntersectWithLine {
            intersection: res.intersected as i32,
            t: res.t,
            x: res.x,
            pcoords: res.pcoords,
            sub_id: res.sub_id,
        }
    }

    /// VTK: `vtkQuad::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        let d1 = distance2_between_points(
            self.cell.get_points().get_point(0),
            self.cell.get_points().get_point(2),
        );
        let d2 = distance2_between_points(
            self.cell.get_points().get_point(1),
            self.cell.get_points().get_point(3),
        );
        let ids = if d1 <= d2 {
            [0, 1, 2, 0, 2, 3]
        } else {
            [0, 1, 3, 1, 2, 3]
        };
        pt_ids.set_number_of_ids(6);
        for (i, id) in ids.into_iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, id);
        }
        1
    }

    /// VTK: `vtkQuad::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: usize,
        derivs: &mut [f64],
    ) {
        let x0 = self.cell.get_points().get_point(0);
        let x1 = self.cell.get_points().get_point(1);
        let x2 = self.cell.get_points().get_point(2);
        let x3 = self.cell.get_points().get_point(3);
        let n = self.compute_normal_from_first_non_collinear_triangle(x0, x1, x2);
        let mut v10 = [0.0; 3];
        let mut vec20 = [0.0; 3];
        let mut vec30 = [0.0; 3];
        for i in 0..3 {
            v10[i] = x1[i] - x0[i];
            vec20[i] = x2[i] - x0[i];
            vec30[i] = x3[i] - x0[i];
        }

        let mut v20 = cross(n, v10);
        let len_x = normalize(&mut v10);
        if len_x <= 0.0 || normalize(&mut v20) <= 0.0 {
            Self::zero_derivs(dim, derivs);
            return;
        }

        let v0 = [0.0, 0.0];
        let v1 = [len_x, 0.0];
        let v2 = [dot(vec20, v10), dot(vec20, v20)];
        let v3 = [dot(vec30, v10), dot(vec30, v20)];
        let func_derivs = Self::interpolation_derivs(pcoords);
        let j00 = v0[0] * func_derivs[0]
            + v1[0] * func_derivs[1]
            + v2[0] * func_derivs[2]
            + v3[0] * func_derivs[3];
        let j01 = v0[1] * func_derivs[0]
            + v1[1] * func_derivs[1]
            + v2[1] * func_derivs[2]
            + v3[1] * func_derivs[3];
        let j10 = v0[0] * func_derivs[4]
            + v1[0] * func_derivs[5]
            + v2[0] * func_derivs[6]
            + v3[0] * func_derivs[7];
        let j11 = v0[1] * func_derivs[4]
            + v1[1] * func_derivs[5]
            + v2[1] * func_derivs[6]
            + v3[1] * func_derivs[7];
        let det = determinant2x2(j00, j01, j10, j11);
        if det == 0.0 {
            Self::zero_derivs(dim, derivs);
            return;
        }
        let ji = [[j11 / det, -j01 / det], [-j10 / det, j00 / det]];

        for j in 0..dim {
            let mut sum = [0.0; 2];
            for i in 0..4 {
                sum[0] += func_derivs[i] * values[dim * i + j];
                sum[1] += func_derivs[4 + i] * values[dim * i + j];
            }
            let d_by_dx = sum[0] * ji[0][0] + sum[1] * ji[0][1];
            let d_by_dy = sum[0] * ji[1][0] + sum[1] * ji[1][1];
            derivs[3 * j] = d_by_dx * v10[0] + d_by_dy * v20[0];
            derivs[3 * j + 1] = d_by_dx * v10[1] + d_by_dy * v20[1];
            derivs[3 * j + 2] = d_by_dx * v10[2] + d_by_dy * v20[2];
        }
    }

    /// VTK: `vtkQuad::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 12] {
        &QUAD_CELL_PCOORDS
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }

    fn compute_normal_from_first_non_collinear_triangle(
        &self,
        pt1: [f64; 3],
        pt2: [f64; 3],
        pt3: [f64; 3],
    ) -> [f64; 3] {
        let mut n = Triangle::compute_normal(pt1, pt2, pt3);
        if n[0] == 0.0 && n[1] == 0.0 && n[2] == 0.0 {
            n = Triangle::compute_normal(pt2, pt3, self.cell.get_points().get_point(3));
        }
        n
    }

    fn closest_boundary_distance2(
        x: [f64; 3],
        pts: [[f64; 3]; 4],
        pcoords: [f64; 3],
    ) -> (f64, [f64; 3]) {
        if pcoords[0] < 0.0 && pcoords[1] < 0.0 {
            (distance2_between_points(x, pts[0]), pts[0])
        } else if pcoords[0] > 1.0 && pcoords[1] < 0.0 {
            (distance2_between_points(x, pts[1]), pts[1])
        } else if pcoords[0] > 1.0 && pcoords[1] > 1.0 {
            (distance2_between_points(x, pts[2]), pts[2])
        } else if pcoords[0] < 0.0 && pcoords[1] > 1.0 {
            (distance2_between_points(x, pts[3]), pts[3])
        } else if pcoords[0] < 0.0 {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pts[0], pts[3]);
            (dist2, closest)
        } else if pcoords[0] > 1.0 {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pts[1], pts[2]);
            (dist2, closest)
        } else if pcoords[1] < 0.0 {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pts[0], pts[1]);
            (dist2, closest)
        } else {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pts[2], pts[3]);
            (dist2, closest)
        }
    }

    fn set_triangle_points(&mut self, ids: [VtkIdType; 3]) {
        for (i, id) in ids.into_iter().enumerate() {
            self.triangle
                .cell_mut()
                .get_points_mut()
                .set_point(i as VtkIdType, self.cell.get_points().get_point(id));
        }
    }

    fn zero_derivs(dim: usize, derivs: &mut [f64]) {
        for j in 0..dim {
            for i in 0..3 {
                derivs[3 * j + i] = 0.0;
            }
        }
    }
}

impl CellBaseApi for Quad {
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

const EDGES: [[VtkIdType; 2]; 4] = [[0, 1], [1, 2], [3, 2], [0, 3]];

const QUAD_CELL_PCOORDS: [f64; 12] = [
    0.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    1.0, 1.0, 0.0, //
    0.0, 1.0, 0.0,
];
