use crate::common::core::{
    math::{
        cross, determinant3x3_from_columns, distance2_between_points, dot, invert_matrix,
        normalize, subtract,
    },
    IdList, Points, VtkIdType,
};

use super::{Cell, Cell3D, Cell3DApi, CellBaseApi, CellType, Line, Quad};

const VTK_DIVERGED: f64 = 1.0e6;
const VTK_HEX_MAX_ITERATION: usize = 10;
const VTK_HEX_CONVERGED: f64 = 1.0e-5;
const VTK_HEX_OUTSIDE_CELL_TOLERANCE: f64 = 1.0e-6;

/// Rust return bundle for VTK `vtkHexahedron::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexahedronEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 8],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkHexahedron::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexahedronIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkHexahedron`.
#[derive(Debug)]
pub struct Hexahedron {
    cell_3d: Cell3D,
    line: Line,
    quad: Quad,
}

impl Hexahedron {
    /// VTK: `vtkHexahedron::NumberOfPoints`.
    pub const NUMBER_OF_POINTS: VtkIdType = 8;
    /// VTK: `vtkHexahedron::NumberOfEdges`.
    pub const NUMBER_OF_EDGES: VtkIdType = 12;
    /// VTK: `vtkHexahedron::NumberOfFaces`.
    pub const NUMBER_OF_FACES: VtkIdType = 6;
    /// VTK: `vtkHexahedron::MaximumFaceSize`.
    pub const MAXIMUM_FACE_SIZE: VtkIdType = 4;
    /// VTK: `vtkHexahedron::MaximumValence`.
    pub const MAXIMUM_VALENCE: VtkIdType = 3;

    /// VTK: `vtkHexahedron::New`.
    pub fn new() -> Self {
        let mut hexahedron = Self {
            cell_3d: Cell3D::with_class_name("vtkHexahedron"),
            line: Line::new(),
            quad: Quad::new(),
        };
        hexahedron
            .cell_3d
            .cell_mut()
            .get_points_mut()
            .set_number_of_points(Self::NUMBER_OF_POINTS);
        hexahedron
            .cell_3d
            .cell_mut()
            .get_point_ids_mut()
            .set_number_of_ids(Self::NUMBER_OF_POINTS);
        for i in 0..Self::NUMBER_OF_POINTS {
            hexahedron
                .cell_3d
                .cell_mut()
                .get_points_mut()
                .set_point(i, [0.0, 0.0, 0.0]);
            hexahedron
                .cell_3d
                .cell_mut()
                .get_point_ids_mut()
                .set_id(i, 0);
        }
        hexahedron
    }

    /// VTK: `vtkHexahedron::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nLine:\n{}\nQuad:\n{}",
            self.cell_3d.print_self(),
            self.line.print_self(),
            self.quad.print_self()
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell_3d.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.cell_3d
            .get_m_time()
            .max(self.line.get_m_time())
            .max(self.quad.get_m_time())
    }

    /// VTK: `vtkCell::GetPoints`.
    pub fn get_points(&self) -> &Points {
        self.cell_3d.cell().get_points()
    }

    /// VTK: `vtkCell::GetPointIds`.
    pub fn get_point_ids(&self) -> &IdList {
        self.cell_3d.cell().get_point_ids()
    }

    /// VTK: `vtkCell::GetPointId`.
    pub fn get_point_id(&self, pt_id: i32) -> VtkIdType {
        self.cell_3d.cell().get_point_id(pt_id)
    }

    /// VTK: `vtkCell::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.cell_3d.cell().get_number_of_points()
    }

    /// VTK: `vtkCell::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.cell_3d.cell().get_bounds()
    }

    /// VTK: `vtkCell::GetLength2`.
    pub fn get_length2(&self) -> f64 {
        self.cell_3d.cell().get_length2()
    }

    /// VTK: `vtkCell::Initialize`.
    pub fn initialize(&mut self) {
        self.cell_3d.cell_mut().initialize()
    }

    /// VTK: `vtkCell::Initialize(int, const vtkIdType*, vtkPoints*)`.
    pub fn initialize_with_point_ids(&mut self, npts: i32, pts: &[VtkIdType], p: &Points) {
        self.cell_3d
            .cell_mut()
            .initialize_with_point_ids(npts, pts, p)
    }

    /// VTK: `vtkCell::Initialize(int, vtkPoints*)`.
    pub fn initialize_from_points(&mut self, npts: i32, p: &Points) {
        self.cell_3d.cell_mut().initialize_from_points(npts, p)
    }

    /// VTK: `vtkCell::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.cell_3d.cell_mut().shallow_copy(source.cell_3d.cell());
    }

    /// VTK: `vtkCell::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.cell_3d.cell_mut().deep_copy(source.cell_3d.cell());
    }

    /// VTK: `vtkCell3D::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        self.cell_3d.get_cell_dimension()
    }

    /// VTK: `vtkCell3D::SetMergeTolerance`.
    pub fn set_merge_tolerance(&mut self, merge_tolerance: f64) {
        self.cell_3d.set_merge_tolerance(merge_tolerance);
    }

    /// VTK: `vtkCell3D::GetMergeTolerance`.
    pub fn get_merge_tolerance(&self) -> f64 {
        self.cell_3d.get_merge_tolerance()
    }

    /// VTK: `vtkHexahedron::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Hexahedron as i32
    }

    /// VTK: `vtkHexahedron::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        Self::NUMBER_OF_EDGES as i32
    }

    /// VTK: `vtkHexahedron::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        Self::NUMBER_OF_FACES as i32
    }

    /// VTK: `vtkHexahedron::EvaluatePosition`.
    pub fn evaluate_position(&self, x: [f64; 3]) -> HexahedronEvaluatePosition {
        let mut pcoords = [0.5, 0.5, 0.5];
        let mut params = pcoords;
        let mut weights = [0.0; 8];
        let diagonals = [[0, 6], [1, 7], [2, 4], [3, 5]];
        let mut longest_diagonal: f64 = 0.0;
        for diagonal in diagonals {
            let pt0 = self.get_points().get_point(diagonal[0]);
            let pt1 = self.get_points().get_point(diagonal[1]);
            longest_diagonal = longest_diagonal.max(distance2_between_points(pt0, pt1));
        }
        let volume_bound = longest_diagonal * longest_diagonal.sqrt();
        let determinant_tolerance = if 1.0e-20 < 0.00001 * volume_bound {
            1.0e-20
        } else {
            0.00001 * volume_bound
        };

        let mut converged = false;
        for _iteration in 0..VTK_HEX_MAX_ITERATION {
            weights = Self::interpolation_functions(pcoords);
            let derivs = Self::interpolation_derivs(pcoords);
            let mut fcol = [0.0; 3];
            let mut rcol = [0.0; 3];
            let mut scol = [0.0; 3];
            let mut tcol = [0.0; 3];
            for i in 0..Self::NUMBER_OF_POINTS as usize {
                let coord = self.get_points().get_point(i as VtkIdType);
                for j in 0..3 {
                    fcol[j] += coord[j] * weights[i];
                    rcol[j] += coord[j] * derivs[i];
                    scol[j] += coord[j] * derivs[8 + i];
                    tcol[j] += coord[j] * derivs[16 + i];
                }
            }
            for i in 0..3 {
                fcol[i] -= x[i];
            }

            let determinant = determinant3x3_from_columns(rcol, scol, tcol);
            if determinant.abs() < determinant_tolerance {
                return HexahedronEvaluatePosition {
                    inside: -1,
                    sub_id: 0,
                    pcoords,
                    dist2: 0.0,
                    weights,
                    closest_point: None,
                };
            }

            pcoords[0] = params[0] - determinant3x3_from_columns(fcol, scol, tcol) / determinant;
            pcoords[1] = params[1] - determinant3x3_from_columns(rcol, fcol, tcol) / determinant;
            pcoords[2] = params[2] - determinant3x3_from_columns(rcol, scol, fcol) / determinant;

            if (pcoords[0] - params[0]).abs() < VTK_HEX_CONVERGED
                && (pcoords[1] - params[1]).abs() < VTK_HEX_CONVERGED
                && (pcoords[2] - params[2]).abs() < VTK_HEX_CONVERGED
            {
                converged = true;
                break;
            }
            if pcoords[0].abs() > VTK_DIVERGED
                || pcoords[1].abs() > VTK_DIVERGED
                || pcoords[2].abs() > VTK_DIVERGED
            {
                return HexahedronEvaluatePosition {
                    inside: -1,
                    sub_id: 0,
                    pcoords,
                    dist2: 0.0,
                    weights,
                    closest_point: None,
                };
            }
            params = pcoords;
        }

        if !converged {
            return HexahedronEvaluatePosition {
                inside: -1,
                sub_id: 0,
                pcoords,
                dist2: 0.0,
                weights,
                closest_point: None,
            };
        }

        weights = Self::interpolation_functions(pcoords);
        let lower_limit = 0.0 - VTK_HEX_OUTSIDE_CELL_TOLERANCE;
        let upper_limit = 1.0 + VTK_HEX_OUTSIDE_CELL_TOLERANCE;
        if pcoords[0] >= lower_limit
            && pcoords[0] <= upper_limit
            && pcoords[1] >= lower_limit
            && pcoords[1] <= upper_limit
            && pcoords[2] >= lower_limit
            && pcoords[2] <= upper_limit
        {
            HexahedronEvaluatePosition {
                inside: 1,
                sub_id: 0,
                pcoords,
                dist2: 0.0,
                weights,
                closest_point: Some(x),
            }
        } else {
            let pc = [
                pcoords[0].clamp(0.0, 1.0),
                pcoords[1].clamp(0.0, 1.0),
                pcoords[2].clamp(0.0, 1.0),
            ];
            let (closest, _) = self.evaluate_location(0, pc);
            HexahedronEvaluatePosition {
                inside: 0,
                sub_id: 0,
                pcoords,
                dist2: distance2_between_points(closest, x),
                weights,
                closest_point: Some(closest),
            }
        }
    }

    /// VTK: `vtkHexahedron::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 8] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        let tm = 1.0 - pcoords[2];
        let rm_x_sm = rm * sm;
        let p0_x_sm = pcoords[0] * sm;
        let p0_x_p1 = pcoords[0] * pcoords[1];
        let rm_x_p1 = rm * pcoords[1];
        [
            rm_x_sm * tm,
            p0_x_sm * tm,
            p0_x_p1 * tm,
            rm_x_p1 * tm,
            rm_x_sm * pcoords[2],
            p0_x_sm * pcoords[2],
            p0_x_p1 * pcoords[2],
            rm_x_p1 * pcoords[2],
        ]
    }

    /// VTK: `vtkHexahedron::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        weights[..8].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkHexahedron::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 24] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        let tm = 1.0 - pcoords[2];
        let mut derivs = [0.0; 24];
        derivs[0] = -sm * tm;
        derivs[1] = -derivs[0];
        derivs[2] = pcoords[1] * tm;
        derivs[3] = -derivs[2];
        derivs[4] = -sm * pcoords[2];
        derivs[5] = -derivs[4];
        derivs[6] = pcoords[1] * pcoords[2];
        derivs[7] = -derivs[6];
        derivs[8] = -rm * tm;
        derivs[9] = -pcoords[0] * tm;
        derivs[10] = -derivs[9];
        derivs[11] = -derivs[8];
        derivs[12] = -rm * pcoords[2];
        derivs[13] = -pcoords[0] * pcoords[2];
        derivs[14] = -derivs[13];
        derivs[15] = -derivs[12];
        derivs[16] = -rm * sm;
        derivs[17] = -pcoords[0] * sm;
        derivs[18] = -pcoords[0] * pcoords[1];
        derivs[19] = -rm * pcoords[1];
        derivs[20] = -derivs[16];
        derivs[21] = -derivs[17];
        derivs[22] = -derivs[18];
        derivs[23] = -derivs[19];
        derivs
    }

    /// VTK: `vtkHexahedron::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        derivs[..24].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkHexahedron::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 8]) {
        let weights = Self::interpolation_functions(pcoords);
        let mut x = [0.0; 3];
        for i in 0..Self::NUMBER_OF_POINTS as usize {
            let pt = self.get_points().get_point(i as VtkIdType);
            for j in 0..3 {
                x[j] += pt[j] * weights[i];
            }
        }
        (x, weights)
    }

    /// VTK: `vtkHexahedron::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let t1 = pcoords[0] - pcoords[1];
        let t2 = 1.0 - pcoords[0] - pcoords[1];
        let t3 = pcoords[1] - pcoords[2];
        let t4 = 1.0 - pcoords[1] - pcoords[2];
        let t5 = pcoords[2] - pcoords[0];
        let t6 = 1.0 - pcoords[2] - pcoords[0];
        pts.set_number_of_ids(Self::MAXIMUM_FACE_SIZE);
        let face = if t3 >= 0.0 && t4 >= 0.0 && t5 < 0.0 && t6 >= 0.0 {
            [0, 1, 2, 3]
        } else if t1 >= 0.0 && t2 < 0.0 && t5 < 0.0 && t6 < 0.0 {
            [1, 2, 6, 5]
        } else if t1 >= 0.0 && t2 >= 0.0 && t3 < 0.0 && t4 >= 0.0 {
            [0, 1, 5, 4]
        } else if t3 < 0.0 && t4 < 0.0 && t5 >= 0.0 && t6 < 0.0 {
            [4, 5, 6, 7]
        } else if t1 < 0.0 && t2 >= 0.0 && t5 >= 0.0 && t6 >= 0.0 {
            [0, 4, 7, 3]
        } else {
            [2, 3, 7, 6]
        };
        for (i, local_id) in face.into_iter().enumerate() {
            pts.set_id(i as VtkIdType, self.get_point_ids().get_id(local_id));
        }
        pcoords.iter().all(|p| *p >= 0.0 && *p <= 1.0) as i32
    }

    /// VTK: `vtkHexahedron::GetCentroid`.
    pub fn get_centroid(&self) -> (bool, [f64; 3]) {
        Self::compute_centroid(self.get_points(), None)
    }

    /// VTK: `vtkHexahedron::ComputeCentroid`.
    pub fn compute_centroid(points: &Points, point_ids: Option<&[VtkIdType]>) -> (bool, [f64; 3]) {
        let face0 = Self::face_point_ids(0, point_ids);
        let face1 = Self::face_point_ids(1, point_ids);
        let (ok0, mut centroid) = polygon_centroid4(points, face0);
        let (ok1, p) = polygon_centroid4(points, face1);
        for i in 0..3 {
            centroid[i] = 0.5 * (centroid[i] + p[i]);
        }
        (ok0 && ok1, centroid)
    }

    /// VTK: `vtkHexahedron::GetEdgeToAdjacentFacesArray`.
    pub fn get_edge_to_adjacent_faces_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGE_TO_ADJACENT_FACES[edge_id as usize]
    }

    /// VTK: `vtkHexahedron::GetFaceToAdjacentFacesArray`.
    pub fn get_face_to_adjacent_faces_array(face_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &FACE_TO_ADJACENT_FACES[face_id as usize]
    }

    /// VTK: `vtkHexahedron::GetPointToIncidentEdgesArray`.
    pub fn get_point_to_incident_edges_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_INCIDENT_EDGES[point_id as usize]
    }

    /// VTK: `vtkHexahedron::GetPointToIncidentFacesArray`.
    pub fn get_point_to_incident_faces_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_INCIDENT_FACES[point_id as usize]
    }

    /// VTK: `vtkHexahedron::GetPointToOneRingPointsArray`.
    pub fn get_point_to_one_ring_points_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_ONE_RING_POINTS[point_id as usize]
    }

    /// VTK: `vtkHexahedron::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkHexahedron::GetFaceArray`.
    pub fn get_face_array(face_id: VtkIdType) -> &'static [VtkIdType; 5] {
        &FACES[face_id as usize]
    }

    /// VTK: `vtkHexahedron::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut Line {
        let verts = *Self::get_edge_array(edge_id as VtkIdType);
        let point_ids = [
            self.get_point_ids().get_id(verts[0]),
            self.get_point_ids().get_id(verts[1]),
        ];
        let points = [
            self.get_points().get_point(verts[0]),
            self.get_points().get_point(verts[1]),
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

    /// VTK: `vtkHexahedron::GetFace`.
    pub fn get_face(&mut self, face_id: i32) -> &mut Quad {
        let verts = *Self::get_face_array(face_id as VtkIdType);
        let point_ids = [
            self.get_point_ids().get_id(verts[0]),
            self.get_point_ids().get_id(verts[1]),
            self.get_point_ids().get_id(verts[2]),
            self.get_point_ids().get_id(verts[3]),
        ];
        let points = [
            self.get_points().get_point(verts[0]),
            self.get_points().get_point(verts[1]),
            self.get_points().get_point(verts[2]),
            self.get_points().get_point(verts[3]),
        ];
        for i in 0..Self::MAXIMUM_FACE_SIZE as usize {
            self.quad
                .cell_mut()
                .get_point_ids_mut()
                .set_id(i as VtkIdType, point_ids[i]);
            self.quad
                .cell_mut()
                .get_points_mut()
                .set_point(i as VtkIdType, points[i]);
        }
        &mut self.quad
    }

    /// VTK: `vtkHexahedron::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> HexahedronIntersectWithLine {
        let mut result = HexahedronIntersectWithLine {
            intersection: 0,
            t: f64::MAX,
            x: [0.0; 3],
            pcoords: [0.0; 3],
            sub_id: 0,
        };
        for face_num in 0..Self::NUMBER_OF_FACES as usize {
            let verts = FACES[face_num];
            let points = [
                self.get_points().get_point(verts[0]),
                self.get_points().get_point(verts[1]),
                self.get_points().get_point(verts[2]),
                self.get_points().get_point(verts[3]),
            ];
            for i in 0..Self::MAXIMUM_FACE_SIZE as usize {
                self.quad
                    .cell_mut()
                    .get_points_mut()
                    .set_point(i as VtkIdType, points[i]);
            }
            let quad_hit = self.quad.intersect_with_line(p1, p2, tol);
            if quad_hit.intersection != 0 {
                result.intersection = 1;
                if quad_hit.t < result.t {
                    result.t = quad_hit.t;
                    result.x = quad_hit.x;
                    result.sub_id = quad_hit.sub_id;
                    result.pcoords = match face_num {
                        0 => [0.0, quad_hit.pcoords[0], 0.0],
                        1 => [1.0, quad_hit.pcoords[0], 0.0],
                        2 => [quad_hit.pcoords[0], 0.0, quad_hit.pcoords[1]],
                        3 => [quad_hit.pcoords[0], 1.0, quad_hit.pcoords[1]],
                        4 => [quad_hit.pcoords[0], quad_hit.pcoords[1], 0.0],
                        _ => [quad_hit.pcoords[0], quad_hit.pcoords[1], 1.0],
                    };
                }
            }
        }
        result
    }

    /// VTK: `vtkHexahedron::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(20);
        let ids = if index % 2 != 0 {
            [0, 1, 3, 4, 1, 4, 5, 6, 1, 4, 6, 3, 1, 3, 6, 2, 3, 6, 7, 4]
        } else {
            [2, 1, 5, 0, 0, 2, 3, 7, 2, 5, 6, 7, 0, 7, 4, 5, 0, 2, 7, 5]
        };
        for (i, id) in ids.into_iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, id);
        }
        1
    }

    /// VTK: `vtkHexahedron::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let (_success, jacobian_inverse, function_derivs) = self.jacobian_inverse(pcoords);
        for k in 0..dim as usize {
            let mut sum = [0.0; 3];
            for i in 0..Self::NUMBER_OF_POINTS as usize {
                sum[0] += function_derivs[i] * values[dim as usize * i + k];
                sum[1] += function_derivs[8 + i] * values[dim as usize * i + k];
                sum[2] += function_derivs[16 + i] * values[dim as usize * i + k];
            }
            for j in 0..3 {
                derivs[3 * k + j] = sum[0] * jacobian_inverse[j][0]
                    + sum[1] * jacobian_inverse[j][1]
                    + sum[2] * jacobian_inverse[j][2];
            }
        }
    }

    /// VTK: `vtkHexahedron::JacobianInverse`.
    pub fn jacobian_inverse(&self, pcoords: [f64; 3]) -> (i32, [[f64; 3]; 3], [f64; 24]) {
        let derivs = Self::interpolation_derivs(pcoords);
        let mut m = [[0.0; 3]; 3];
        for j in 0..Self::NUMBER_OF_POINTS as usize {
            let x = self.get_points().get_point(j as VtkIdType);
            for i in 0..3 {
                m[0][i] += x[i] * derivs[j];
                m[1][i] += x[i] * derivs[8 + j];
                m[2][i] += x[i] * derivs[16 + j];
            }
        }
        let (success, _factored, inverse) =
            invert_matrix(vec![m[0].to_vec(), m[1].to_vec(), m[2].to_vec()], 3);
        if !success {
            return (0, [[0.0; 3]; 3], derivs);
        }
        (
            1,
            [
                [inverse[0][0], inverse[0][1], inverse[0][2]],
                [inverse[1][0], inverse[1][1], inverse[1][2]],
                [inverse[2][0], inverse[2][1], inverse[2][2]],
            ],
            derivs,
        )
    }

    /// VTK: `vtkHexahedron::GetPointToOneRingPoints`.
    pub fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_one_ring_points_array(point_id),
        )
    }

    /// VTK: `vtkHexahedron::GetPointToIncidentFaces`.
    pub fn get_point_to_incident_faces(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_faces_array(point_id),
        )
    }

    /// VTK: `vtkHexahedron::GetPointToIncidentEdges`.
    pub fn get_point_to_incident_edges(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_edges_array(point_id),
        )
    }

    /// VTK: `vtkHexahedron::GetFaceToAdjacentFaces`.
    pub fn get_face_to_adjacent_faces(
        &self,
        face_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            Self::MAXIMUM_FACE_SIZE,
            Self::get_face_to_adjacent_faces_array(face_id),
        )
    }

    /// VTK: `vtkHexahedron::GetEdgeToAdjacentFaces`.
    pub fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_to_adjacent_faces_array(edge_id)
    }

    /// VTK: `vtkHexahedron::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_array(edge_id)
    }

    /// VTK: `vtkHexahedron::GetFacePoints`.
    pub fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType; 5]) {
        (Self::MAXIMUM_FACE_SIZE, Self::get_face_array(face_id))
    }

    /// VTK: `vtkHexahedron::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 24] {
        &HEXAHEDRON_CELL_PCOORDS
    }

    pub(crate) fn cell_3d(&self) -> &Cell3D {
        &self.cell_3d
    }

    pub(crate) fn cell_3d_mut(&mut self) -> &mut Cell3D {
        &mut self.cell_3d
    }

    pub(crate) fn cell(&self) -> &Cell {
        self.cell_3d.cell()
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        self.cell_3d.cell_mut()
    }

    fn face_point_ids(face_id: usize, point_ids: Option<&[VtkIdType]>) -> [VtkIdType; 4] {
        let face = FACES[face_id];
        [
            point_ids.map_or(face[0], |ids| ids[face[0] as usize]),
            point_ids.map_or(face[1], |ids| ids[face[1] as usize]),
            point_ids.map_or(face[2], |ids| ids[face[2] as usize]),
            point_ids.map_or(face[3], |ids| ids[face[3] as usize]),
        ]
    }
}

impl Default for Hexahedron {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for Hexahedron {
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

impl Cell3DApi for Hexahedron {
    fn cell_3d(&self) -> &Cell3D {
        self.cell_3d()
    }

    fn cell_3d_mut(&mut self) -> &mut Cell3D {
        self.cell_3d_mut()
    }

    fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        self.get_edge_points(edge_id)
    }

    fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType]) {
        let (count, pts) = self.get_face_points(face_id);
        (count, pts.as_slice())
    }

    fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        self.get_edge_to_adjacent_faces(edge_id)
    }

    fn get_face_to_adjacent_faces(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType]) {
        let (count, face_ids) = self.get_face_to_adjacent_faces(face_id);
        (count, face_ids.as_slice())
    }

    fn get_point_to_incident_edges(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType]) {
        let (count, edge_ids) = self.get_point_to_incident_edges(point_id);
        (count, edge_ids.as_slice())
    }

    fn get_point_to_incident_faces(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType]) {
        let (count, face_ids) = self.get_point_to_incident_faces(point_id);
        (count, face_ids.as_slice())
    }

    fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType]) {
        let (count, point_ids) = self.get_point_to_one_ring_points(point_id);
        (count, point_ids.as_slice())
    }

    fn get_centroid(&self) -> (bool, [f64; 3]) {
        self.get_centroid()
    }

    fn is_inside_out(&self) -> bool {
        false
    }
}

fn polygon_centroid4(points: &Points, ids: [VtkIdType; 4]) -> (bool, [f64; 3]) {
    let mut normal = [0.0; 3];
    for i in 0..4 {
        let p0 = points.get_point(ids[i]);
        let p1 = points.get_point(ids[(i + 1) % 4]);
        normal[0] += (p0[1] - p1[1]) * (p0[2] + p1[2]);
        normal[1] += (p0[2] - p1[2]) * (p0[0] + p1[0]);
        normal[2] += (p0[0] - p1[0]) * (p0[1] + p1[1]);
    }
    if normalize(&mut normal) == 0.0 {
        return (false, [0.0; 3]);
    }

    let mut xx = [0.0; 3];
    for id in ids {
        let point = points.get_point(id);
        for i in 0..3 {
            xx[i] += 0.25 * point[i];
        }
    }

    let mut total_area = 0.0;
    let mut accum = [0.0; 3];
    let mut pp = points.get_point(ids[3]);
    for id in ids {
        let qq = points.get_point(id);
        let pq = [
            0.5 * (pp[0] + qq[0]),
            0.5 * (pp[1] + qq[1]),
            0.5 * (pp[2] + qq[2]),
        ];
        let ctr = [
            (1.0 / 3.0) * xx[0] + (2.0 / 3.0) * pq[0],
            (1.0 / 3.0) * xx[1] + (2.0 / 3.0) * pq[1],
            (1.0 / 3.0) * xx[2] + (2.0 / 3.0) * pq[2],
        ];
        let area = dot(cross(subtract(pp, xx), subtract(qq, xx)), normal) / 2.0;
        for i in 0..3 {
            accum[i] += area * ctr[i];
        }
        total_area += area;
        pp = qq;
    }
    if total_area == 0.0 {
        return (false, [0.0; 3]);
    }
    for value in &mut accum {
        *value /= total_area;
    }
    (true, accum)
}

const EDGES: [[VtkIdType; 2]; 12] = [
    [0, 1],
    [1, 2],
    [3, 2],
    [0, 3],
    [4, 5],
    [5, 6],
    [7, 6],
    [4, 7],
    [0, 4],
    [1, 5],
    [3, 7],
    [2, 6],
];

const FACES: [[VtkIdType; 5]; 6] = [
    [0, 4, 7, 3, -1],
    [1, 2, 6, 5, -1],
    [0, 1, 5, 4, -1],
    [3, 7, 6, 2, -1],
    [0, 3, 2, 1, -1],
    [4, 5, 6, 7, -1],
];

const EDGE_TO_ADJACENT_FACES: [[VtkIdType; 2]; 12] = [
    [2, 4],
    [1, 4],
    [3, 4],
    [0, 4],
    [2, 5],
    [1, 5],
    [3, 5],
    [0, 5],
    [0, 2],
    [1, 2],
    [0, 3],
    [1, 3],
];

const FACE_TO_ADJACENT_FACES: [[VtkIdType; 4]; 6] = [
    [4, 2, 5, 3],
    [4, 3, 5, 2],
    [4, 1, 5, 0],
    [0, 5, 1, 4],
    [0, 3, 1, 2],
    [2, 1, 0, 3],
];

const POINT_TO_INCIDENT_EDGES: [[VtkIdType; 3]; 8] = [
    [0, 8, 3],
    [0, 1, 9],
    [1, 2, 11],
    [2, 3, 10],
    [7, 8, 4],
    [4, 9, 5],
    [5, 11, 6],
    [6, 10, 7],
];

const POINT_TO_INCIDENT_FACES: [[VtkIdType; 3]; 8] = [
    [2, 0, 4],
    [4, 1, 2],
    [4, 3, 1],
    [4, 0, 3],
    [5, 2, 0],
    [2, 1, 5],
    [1, 3, 5],
    [3, 0, 5],
];

const POINT_TO_ONE_RING_POINTS: [[VtkIdType; 3]; 8] = [
    [1, 4, 3],
    [0, 2, 5],
    [1, 3, 6],
    [2, 0, 7],
    [5, 7, 0],
    [4, 1, 6],
    [5, 2, 7],
    [6, 3, 4],
];

const HEXAHEDRON_CELL_PCOORDS: [f64; 24] = [
    0.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    1.0, 1.0, 0.0, //
    0.0, 1.0, 0.0, //
    0.0, 0.0, 1.0, //
    1.0, 0.0, 1.0, //
    1.0, 1.0, 1.0, //
    0.0, 1.0, 1.0, //
];
