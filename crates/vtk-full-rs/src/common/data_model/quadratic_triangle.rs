use crate::common::core::{
    math::{cross, invert_matrix, normalize},
    IdList, Points, VtkIdType, VTK_DOUBLE_MAX,
};

use super::{Cell, CellBaseApi, CellType, QuadraticEdge, Triangle};

const QUADRATIC_TRIANGLE_LINEAR_TRIS: [[VtkIdType; 3]; 4] =
    [[0, 3, 5], [3, 1, 4], [5, 4, 2], [4, 5, 3]];
const QUADRATIC_TRIANGLE_LOCAL_IDS: [VtkIdType; 12] = [0, 3, 5, 3, 1, 4, 5, 4, 2, 4, 5, 3];
static QUADRATIC_TRIANGLE_CELL_PCOORDS: [f64; 18] = [
    0.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, //
    0.5, 0.0, 0.0, //
    0.5, 0.5, 0.0, //
    0.0, 0.5, 0.0,
];

/// Rust return bundle for VTK `vtkQuadraticTriangle::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticTriangleEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 6],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkQuadraticTriangle::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadraticTriangleIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkQuadraticTriangle`.
#[derive(Debug)]
pub struct QuadraticTriangle {
    cell: Cell,
    edge: QuadraticEdge,
    face: Triangle,
}

impl QuadraticTriangle {
    /// VTK: `vtkQuadraticTriangle::New`.
    pub fn new() -> Self {
        let mut triangle = Self {
            cell: Cell::with_class_name("vtkQuadraticTriangle"),
            edge: QuadraticEdge::new(),
            face: Triangle::new(),
        };
        triangle.cell.get_points_mut().set_number_of_points(6);
        triangle.cell.get_point_ids_mut().set_number_of_ids(6);
        for i in 0..6 {
            triangle.cell.get_points_mut().set_point(i, [0.0, 0.0, 0.0]);
            triangle.cell.get_point_ids_mut().set_id(i, 0);
        }
        triangle
    }

    /// VTK: `vtkQuadraticTriangle::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell.print_self();
        text.push_str("Edge:\n");
        text.push_str(&self.edge.print_self());
        text.push_str("Face:\n");
        text.push_str(&self.face.print_self());
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
            .max(self.edge.get_m_time())
            .max(self.face.get_m_time())
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

    /// VTK: `vtkQuadraticTriangle::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::QuadraticTriangle as i32
    }

    /// VTK: `vtkQuadraticTriangle::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkQuadraticTriangle::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        3
    }

    /// VTK: `vtkQuadraticTriangle::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkQuadraticTriangle::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut QuadraticEdge {
        let edge_id = edge_id.clamp(0, 2) as VtkIdType;
        let p = (edge_id + 1) % 3;

        self.edge
            .cell_mut()
            .get_point_ids_mut()
            .set_id(0, self.cell.get_point_ids().get_id(edge_id));
        self.edge
            .cell_mut()
            .get_point_ids_mut()
            .set_id(1, self.cell.get_point_ids().get_id(p));
        self.edge
            .cell_mut()
            .get_point_ids_mut()
            .set_id(2, self.cell.get_point_ids().get_id(edge_id + 3));

        self.edge
            .cell_mut()
            .get_points_mut()
            .set_point(0, self.cell.get_points().get_point(edge_id));
        self.edge
            .cell_mut()
            .get_points_mut()
            .set_point(1, self.cell.get_points().get_point(p));
        self.edge
            .cell_mut()
            .get_points_mut()
            .set_point(2, self.cell.get_points().get_point(edge_id + 3));

        &mut self.edge
    }

    /// VTK: `vtkQuadraticTriangle::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkQuadraticTriangle::EvaluatePosition`.
    pub fn evaluate_position(
        &mut self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> QuadraticTriangleEvaluatePosition {
        let wants_closest_point = closest_point.is_some();
        let mut closest_point = closest_point;
        let mut return_status = 0;
        let mut sub_id = 0;
        let mut pcoords = [0.0, 0.0, 0.0];
        let mut min_dist2 = VTK_DOUBLE_MAX;

        for i in 0..4 {
            self.set_face_triangle_points(i);
            let status = self.face.evaluate_position(x, true);
            if status.inside != -1
                && (status.dist2 < min_dist2 || (status.dist2 == min_dist2 && return_status == 0))
            {
                return_status = status.inside;
                min_dist2 = status.dist2;
                sub_id = i as i32;
                pcoords[0] = status.pcoords[0];
                pcoords[1] = status.pcoords[1];
            }
        }

        let mut weights = [0.0; 6];
        let mut closest = None;
        if return_status != -1 {
            match sub_id {
                0 => {
                    pcoords[0] /= 2.0;
                    pcoords[1] /= 2.0;
                }
                1 => {
                    pcoords[0] = 0.5 + pcoords[0] / 2.0;
                    pcoords[1] /= 2.0;
                }
                2 => {
                    pcoords[0] /= 2.0;
                    pcoords[1] = 0.5 + pcoords[1] / 2.0;
                }
                _ => {
                    pcoords[0] = 0.5 - pcoords[0] / 2.0;
                    pcoords[1] = 0.5 - pcoords[1] / 2.0;
                }
            }
            pcoords[2] = 0.0;

            if wants_closest_point {
                let (point, triangle_weights) = self.evaluate_location(sub_id, pcoords);
                weights = triangle_weights;
                if let Some(out) = closest_point.as_deref_mut() {
                    *out = point;
                }
                closest = Some(point);
            } else {
                weights = Self::interpolation_functions(pcoords);
            }
        }

        QuadraticTriangleEvaluatePosition {
            inside: return_status,
            sub_id,
            pcoords,
            dist2: min_dist2,
            weights,
            closest_point: if wants_closest_point { closest } else { None },
        }
    }

    /// VTK: `vtkQuadraticTriangle::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 6]) {
        let weights = Self::interpolation_functions(pcoords);
        let mut x = [0.0; 3];
        for i in 0..6 {
            let point = self.cell.get_points().get_point(i);
            for j in 0..3 {
                x[j] += point[j] * weights[i as usize];
            }
        }
        (x, weights)
    }

    /// VTK: `vtkQuadraticTriangle::CellBoundary`.
    pub fn cell_boundary(&self, sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        self.face.cell_boundary(sub_id, pcoords, pts)
    }

    /// VTK: `vtkQuadraticTriangle::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> QuadraticTriangleIntersectWithLine {
        let mut last_miss = None;
        for i in 0..4 {
            self.set_face_triangle_points(i);
            let hit = self.face.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                return QuadraticTriangleIntersectWithLine {
                    intersection: 1,
                    t: hit.t,
                    x: hit.x,
                    pcoords: hit.pcoords,
                    sub_id: 0,
                };
            }
            last_miss = Some(hit);
        }

        let (t, x, pcoords) = last_miss.map_or((0.0, [0.0; 3], [0.0; 3]), |miss| {
            (miss.t, miss.x, miss.pcoords)
        });
        QuadraticTriangleIntersectWithLine {
            intersection: 0,
            t,
            x,
            pcoords,
            sub_id: 0,
        }
    }

    /// VTK: `vtkQuadraticTriangle::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(12);
        for (i, id) in QUADRATIC_TRIANGLE_LOCAL_IDS.iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, *id);
        }
        1
    }

    /// VTK: `vtkQuadraticTriangle::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let dim = dim.max(0) as usize;
        assert!(
            values.len() >= dim * 6,
            "vtkQuadraticTriangle::Derivatives values slice too short"
        );
        assert!(
            derivs.len() >= dim * 3,
            "vtkQuadraticTriangle::Derivatives derivs slice too short"
        );

        let function_derivs = Self::interpolation_derivs(pcoords);
        let mut j0 = [0.0; 3];
        let mut j1 = [0.0; 3];
        for i in 0..6 {
            let point = self.cell.get_points().get_point(i);
            for k in 0..3 {
                j0[k] += point[k] * function_derivs[i as usize];
                j1[k] += point[k] * function_derivs[6 + i as usize];
            }
        }

        let mut j2 = cross(j0, j1);
        if normalize(&mut j2) == 0.0 {
            self.zero_degenerate_derivatives(dim, derivs);
            return;
        }

        let jacobian = vec![
            vec![j0[0], j0[1], j0[2]],
            vec![j1[0], j1[1], j1[2]],
            vec![j2[0], j2[1], j2[2]],
        ];
        let (success, _factored, inverse) = invert_matrix(jacobian, 3);
        if !success {
            self.zero_degenerate_derivatives(dim, derivs);
            return;
        }

        for j in 0..dim {
            let mut sum = [0.0; 2];
            for i in 0..6 {
                let value = values[dim * i + j];
                sum[0] += function_derivs[i] * value;
                sum[1] += function_derivs[6 + i] * value;
            }

            derivs[3 * j] = sum[0] * inverse[0][0] + sum[1] * inverse[0][1];
            derivs[3 * j + 1] = sum[0] * inverse[1][0] + sum[1] * inverse[1][1];
            derivs[3 * j + 2] = sum[0] * inverse[2][0] + sum[1] * inverse[2][1];
        }
    }

    /// VTK: `vtkQuadraticTriangle::GetParametricDistance`.
    pub fn get_parametric_distance(&self, pcoords: [f64; 3]) -> f64 {
        let pc = [pcoords[0], pcoords[1], 1.0 - pcoords[0] - pcoords[1]];
        let mut p_dist_max = 0.0_f64;
        for value in pc {
            let p_dist = if value < 0.0 {
                -value
            } else if value > 1.0 {
                value - 1.0
            } else {
                0.0
            };
            p_dist_max = p_dist.max(p_dist_max);
        }
        p_dist_max
    }

    /// VTK: `vtkQuadraticTriangle::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [1.0 / 3.0, 1.0 / 3.0, 0.0])
    }

    /// VTK: `vtkQuadraticTriangle::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 6] {
        let r = pcoords[0];
        let s = pcoords[1];
        let t = 1.0 - r - s;
        [
            t * (2.0 * t - 1.0),
            r * (2.0 * r - 1.0),
            s * (2.0 * s - 1.0),
            4.0 * r * t,
            4.0 * r * s,
            4.0 * s * t,
        ]
    }

    /// VTK: `vtkQuadraticTriangle::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 12] {
        let r = pcoords[0];
        let s = pcoords[1];
        [
            4.0 * r + 4.0 * s - 3.0,
            4.0 * r - 1.0,
            0.0,
            4.0 - 8.0 * r - 4.0 * s,
            4.0 * s,
            -4.0 * s,
            4.0 * r + 4.0 * s - 3.0,
            0.0,
            4.0 * s - 1.0,
            -4.0 * r,
            4.0 * r,
            4.0 - 8.0 * s - 4.0 * r,
        ]
    }

    /// VTK: `vtkQuadraticTriangle::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        assert!(
            weights.len() >= 6,
            "vtkQuadraticTriangle::InterpolateFunctions weights slice too short"
        );
        weights[..6].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkQuadraticTriangle::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        assert!(
            derivs.len() >= 12,
            "vtkQuadraticTriangle::InterpolateDerivs derivs slice too short"
        );
        derivs[..12].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkQuadraticTriangle::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 18] {
        &QUADRATIC_TRIANGLE_CELL_PCOORDS
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }

    fn set_face_triangle_points(&mut self, sub_id: usize) {
        for i in 0..3 {
            self.face.cell_mut().get_points_mut().set_point(
                i as VtkIdType,
                self.cell
                    .get_points()
                    .get_point(QUADRATIC_TRIANGLE_LINEAR_TRIS[sub_id][i]),
            );
        }
    }

    fn zero_degenerate_derivatives(&self, dim: usize, derivs: &mut [f64]) {
        for j in 0..dim {
            for i in 0..3 {
                let idx = j * dim + i;
                if idx < derivs.len() {
                    derivs[idx] = 0.0;
                }
            }
        }
    }
}

impl Default for QuadraticTriangle {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for QuadraticTriangle {
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
