use crate::common::core::{
    math::{cross, determinant3x3_from_columns, distance2_between_points, dot, invert_matrix},
    IdList, Points, VtkIdType,
};

use super::{Cell, Cell3D, Cell3DApi, CellBaseApi, CellType, Line, Quad, Triangle};

const VTK_DIVERGED: f64 = 1.0e6;
const VTK_WEDGE_MAX_ITERATION: usize = 10;
const VTK_WEDGE_CONVERGED: f64 = 1.0e-3;

/// Rust return bundle for VTK `vtkWedge::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WedgeEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 6],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkWedge::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WedgeIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// Rust typed equivalent of VTK `vtkWedge::GetFace` returning `vtkCell*`.
#[derive(Debug)]
pub enum WedgeFace<'a> {
    Triangle(&'a mut Triangle),
    Quad(&'a mut Quad),
}

/// VTK: `vtkWedge`.
#[derive(Debug)]
pub struct Wedge {
    cell_3d: Cell3D,
    line: Line,
    triangle: Triangle,
    quad: Quad,
}

impl Wedge {
    /// VTK: `vtkWedge::NumberOfPoints`.
    pub const NUMBER_OF_POINTS: VtkIdType = 6;
    /// VTK: `vtkWedge::NumberOfEdges`.
    pub const NUMBER_OF_EDGES: VtkIdType = 9;
    /// VTK: `vtkWedge::NumberOfFaces`.
    pub const NUMBER_OF_FACES: VtkIdType = 5;
    /// VTK: `vtkWedge::MaximumFaceSize`.
    pub const MAXIMUM_FACE_SIZE: VtkIdType = 4;
    /// VTK: `vtkWedge::MaximumValence`.
    pub const MAXIMUM_VALENCE: VtkIdType = 3;

    /// VTK: `vtkWedge::New`.
    pub fn new() -> Self {
        let mut wedge = Self {
            cell_3d: Cell3D::with_class_name("vtkWedge"),
            line: Line::new(),
            triangle: Triangle::new(),
            quad: Quad::new(),
        };
        wedge
            .cell_3d
            .cell_mut()
            .get_points_mut()
            .set_number_of_points(Self::NUMBER_OF_POINTS);
        wedge
            .cell_3d
            .cell_mut()
            .get_point_ids_mut()
            .set_number_of_ids(Self::NUMBER_OF_POINTS);
        for i in 0..Self::NUMBER_OF_POINTS {
            wedge
                .cell_3d
                .cell_mut()
                .get_points_mut()
                .set_point(i, [0.0, 0.0, 0.0]);
            wedge.cell_3d.cell_mut().get_point_ids_mut().set_id(i, 0);
        }
        wedge
    }

    /// VTK: `vtkWedge::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nLine:\n{}\nTriangle:\n{}\nQuad:\n{}",
            self.cell_3d.print_self(),
            self.line.print_self(),
            self.triangle.print_self(),
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
            .max(self.triangle.get_m_time())
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

    /// VTK: `vtkCell3D::SetMergeTolerance`.
    pub fn set_merge_tolerance(&mut self, merge_tolerance: f64) {
        self.cell_3d.set_merge_tolerance(merge_tolerance);
    }

    /// VTK: `vtkCell3D::GetMergeTolerance`.
    pub fn get_merge_tolerance(&self) -> f64 {
        self.cell_3d.get_merge_tolerance()
    }

    /// VTK: `vtkWedge::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Wedge as i32
    }

    /// VTK: `vtkWedge::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        self.cell_3d.get_cell_dimension()
    }

    /// VTK: `vtkWedge::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        Self::NUMBER_OF_EDGES as i32
    }

    /// VTK: `vtkWedge::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        Self::NUMBER_OF_FACES as i32
    }

    /// VTK: `vtkWedge::GetCentroid`.
    pub fn get_centroid(&self) -> (bool, [f64; 3]) {
        Self::compute_centroid(self.get_points(), None)
    }

    /// VTK: `vtkWedge::ComputeCentroid`.
    pub fn compute_centroid(points: &Points, point_ids: Option<&[VtkIdType]>) -> (bool, [f64; 3]) {
        let face0 = Self::triangle_face_point_ids(0, point_ids);
        let face1 = Self::triangle_face_point_ids(1, point_ids);
        let (_ok0, mut centroid) = triangle_centroid(points, face0);
        let (_ok1, p) = triangle_centroid(points, face1);
        for i in 0..3 {
            centroid[i] = 0.5 * (centroid[i] + p[i]);
        }
        (true, centroid)
    }

    /// VTK: `vtkWedge::IsInsideOut`.
    pub fn is_inside_out(&self) -> bool {
        let mut a = self.get_points().get_point(0);
        let mut b = self.get_points().get_point(1);
        let c = self.get_points().get_point(2);
        for i in 0..3 {
            b[i] -= a[i];
            a[i] -= c[i];
        }
        let n0 = cross(b, a);

        let mut a = self.get_points().get_point(3);
        let mut b = self.get_points().get_point(4);
        let c = self.get_points().get_point(5);
        for i in 0..3 {
            b[i] -= a[i];
            a[i] -= c[i];
        }
        let n1 = cross(b, a);
        dot(n0, n1) > 0.0
    }

    /// VTK: `vtkWedge::EvaluatePosition`.
    pub fn evaluate_position(&self, x: [f64; 3]) -> WedgeEvaluatePosition {
        let mut longest_edge: f64 = 0.0;
        for edge in EDGES {
            let pt0 = self.get_points().get_point(edge[0]);
            let pt1 = self.get_points().get_point(edge[1]);
            longest_edge = longest_edge.max(distance2_between_points(pt0, pt1));
        }
        let volume_bound = longest_edge * longest_edge.sqrt();
        let determinant_tolerance = if 1.0e-20 < 0.00001 * volume_bound {
            1.0e-20
        } else {
            0.00001 * volume_bound
        };

        let mut params = [0.5, 0.5, 0.5];
        let mut pcoords = params;
        let mut weights = [0.0; 6];
        let mut converged = false;
        for _iteration in 0..VTK_WEDGE_MAX_ITERATION {
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
                    scol[j] += coord[j] * derivs[6 + i];
                    tcol[j] += coord[j] * derivs[12 + i];
                }
            }
            for i in 0..3 {
                fcol[i] -= x[i];
            }
            let determinant = determinant3x3_from_columns(rcol, scol, tcol);
            if determinant.abs() < determinant_tolerance {
                return WedgeEvaluatePosition {
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
            if (pcoords[0] - params[0]).abs() < VTK_WEDGE_CONVERGED
                && (pcoords[1] - params[1]).abs() < VTK_WEDGE_CONVERGED
                && (pcoords[2] - params[2]).abs() < VTK_WEDGE_CONVERGED
            {
                converged = true;
                break;
            }
            if pcoords[0].abs() > VTK_DIVERGED
                || pcoords[1].abs() > VTK_DIVERGED
                || pcoords[2].abs() > VTK_DIVERGED
            {
                return WedgeEvaluatePosition {
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
            return WedgeEvaluatePosition {
                inside: -1,
                sub_id: 0,
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
            && pcoords[2] >= -0.001
            && pcoords[2] <= 1.001
            && pcoords[0] + pcoords[1] <= 1.001
        {
            WedgeEvaluatePosition {
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
            WedgeEvaluatePosition {
                inside: 0,
                sub_id: 0,
                pcoords,
                dist2: distance2_between_points(closest, x),
                weights,
                closest_point: Some(closest),
            }
        }
    }

    /// VTK: `vtkWedge::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 6]) {
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

    /// VTK: `vtkWedge::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let normals = [
            [0.0, 0.83205, -0.5547],
            [-0.639602, -0.639602, -0.426401],
            [0.83205, 0.0, -0.5547],
            [0.0, 0.83205, 0.5547],
            [-0.639602, -0.639602, 0.426401],
            [0.83205, 0.0, 0.5547],
            [-0.707107, 0.707107, 0.0],
            [0.447214, 0.894427, 0.0],
            [0.894427, 0.447214, 0.0],
        ];
        let point = [0.333333, 0.333333, 0.5];
        let mut vals = [0.0; 9];
        for i in 0..9 {
            vals[i] = normals[i][0] * (pcoords[0] - point[0])
                + normals[i][1] * (pcoords[1] - point[1])
                + normals[i][2] * (pcoords[2] - point[2]);
        }

        let face: &[VtkIdType] = if vals[0] >= 0.0 && vals[1] >= 0.0 && vals[2] >= 0.0 {
            &[0, 1, 2]
        } else if vals[3] >= 0.0 && vals[4] >= 0.0 && vals[5] >= 0.0 {
            &[3, 4, 5]
        } else if vals[0] <= 0.0 && vals[3] <= 0.0 && vals[6] <= 0.0 && vals[7] <= 0.0 {
            &[0, 1, 4, 3]
        } else if vals[1] <= 0.0 && vals[4] <= 0.0 && vals[7] >= 0.0 && vals[8] >= 0.0 {
            &[1, 2, 5, 4]
        } else {
            &[2, 0, 3, 5]
        };
        pts.set_number_of_ids(face.len() as VtkIdType);
        for (i, local_id) in face.iter().copied().enumerate() {
            pts.set_id(i as VtkIdType, self.get_point_ids().get_id(local_id));
        }
        pcoords.iter().all(|p| *p >= 0.0 && *p <= 1.0) as i32
    }

    /// VTK: `vtkWedge::GetEdgeToAdjacentFacesArray`.
    pub fn get_edge_to_adjacent_faces_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGE_TO_ADJACENT_FACES[edge_id as usize]
    }

    /// VTK: `vtkWedge::GetFaceToAdjacentFacesArray`.
    pub fn get_face_to_adjacent_faces_array(face_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &FACE_TO_ADJACENT_FACES[face_id as usize]
    }

    /// VTK: `vtkWedge::GetPointToIncidentEdgesArray`.
    pub fn get_point_to_incident_edges_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_INCIDENT_EDGES[point_id as usize]
    }

    /// VTK: `vtkWedge::GetPointToIncidentFacesArray`.
    pub fn get_point_to_incident_faces_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_INCIDENT_FACES[point_id as usize]
    }

    /// VTK: `vtkWedge::GetPointToOneRingPointsArray`.
    pub fn get_point_to_one_ring_points_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_ONE_RING_POINTS[point_id as usize]
    }

    /// VTK: `vtkWedge::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkWedge::GetFaceArray`.
    pub fn get_face_array(face_id: VtkIdType) -> &'static [VtkIdType; 5] {
        &FACES[face_id as usize]
    }

    /// VTK: `vtkWedge::GetEdge`.
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

    /// VTK: `vtkWedge::GetFace`.
    pub fn get_face(&mut self, face_id: i32) -> WedgeFace<'_> {
        let verts = *Self::get_face_array(face_id as VtkIdType);
        if verts[3] != -1 {
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
            for i in 0..4 {
                self.quad
                    .cell_mut()
                    .get_point_ids_mut()
                    .set_id(i as VtkIdType, point_ids[i]);
                self.quad
                    .cell_mut()
                    .get_points_mut()
                    .set_point(i as VtkIdType, points[i]);
            }
            WedgeFace::Quad(&mut self.quad)
        } else {
            let point_ids = [
                self.get_point_ids().get_id(verts[0]),
                self.get_point_ids().get_id(verts[1]),
                self.get_point_ids().get_id(verts[2]),
            ];
            let points = [
                self.get_points().get_point(verts[0]),
                self.get_points().get_point(verts[1]),
                self.get_points().get_point(verts[2]),
            ];
            for i in 0..3 {
                self.triangle
                    .cell_mut()
                    .get_point_ids_mut()
                    .set_id(i as VtkIdType, point_ids[i]);
                self.triangle
                    .cell_mut()
                    .get_points_mut()
                    .set_point(i as VtkIdType, points[i]);
            }
            WedgeFace::Triangle(&mut self.triangle)
        }
    }

    /// VTK: `vtkWedge::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> WedgeIntersectWithLine {
        let mut result = WedgeIntersectWithLine {
            intersection: 0,
            t: f64::MAX,
            x: [0.0; 3],
            pcoords: [0.0; 3],
            sub_id: 0,
        };

        for face_num in 0..2 {
            let verts = FACES[face_num];
            let points = [
                self.get_points().get_point(verts[0]),
                self.get_points().get_point(verts[1]),
                self.get_points().get_point(verts[2]),
            ];
            for i in 0..3 {
                self.triangle
                    .cell_mut()
                    .get_points_mut()
                    .set_point(i as VtkIdType, points[i]);
            }
            let hit = self.triangle.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                result.intersection = 1;
                if hit.t < result.t {
                    result.t = hit.t;
                    result.x = hit.x;
                    match face_num {
                        0 => result.pcoords = [hit.pcoords[0], hit.pcoords[1], 0.0],
                        1 => result.pcoords = [hit.pcoords[0], hit.pcoords[1], 1.0],
                        _ => {}
                    }
                    result.sub_id = hit.sub_id;
                }
            }
        }

        for face_num in 2..Self::NUMBER_OF_FACES as usize {
            let verts = FACES[face_num];
            let points = [
                self.get_points().get_point(verts[0]),
                self.get_points().get_point(verts[1]),
                self.get_points().get_point(verts[2]),
                self.get_points().get_point(verts[3]),
            ];
            for i in 0..4 {
                self.quad
                    .cell_mut()
                    .get_points_mut()
                    .set_point(i as VtkIdType, points[i]);
            }
            let hit = self.quad.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                result.intersection = 1;
                if hit.t < result.t {
                    result.t = hit.t;
                    result.x = hit.x;
                    match face_num {
                        2 => result.pcoords = [hit.pcoords[1], 0.0, hit.pcoords[0]],
                        3 => {
                            result.pcoords = [1.0 - hit.pcoords[1], hit.pcoords[1], hit.pcoords[0]]
                        }
                        4 => result.pcoords = [0.0, hit.pcoords[1], hit.pcoords[0]],
                        _ => {}
                    }
                    result.sub_id = hit.sub_id;
                }
            }
        }
        result
    }

    /// VTK: `vtkWedge::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(12);
        let ids = [0, 1, 2, 3, 1, 4, 5, 3, 1, 3, 5, 2];
        for (i, id) in ids.into_iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, id);
        }
        1
    }

    /// VTK: `vtkWedge::Derivatives`.
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
                let value = values[dim as usize * i + k];
                sum[0] += function_derivs[i] * value;
                sum[1] += function_derivs[6 + i] * value;
                sum[2] += function_derivs[12 + i] * value;
            }
            for j in 0..3 {
                derivs[3 * k + j] = sum[0] * jacobian_inverse[j][0]
                    + sum[1] * jacobian_inverse[j][1]
                    + sum[2] * jacobian_inverse[j][2];
            }
        }
    }

    /// VTK: `vtkWedge::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 6] {
        [
            (1.0 - pcoords[0] - pcoords[1]) * (1.0 - pcoords[2]),
            pcoords[0] * (1.0 - pcoords[2]),
            pcoords[1] * (1.0 - pcoords[2]),
            (1.0 - pcoords[0] - pcoords[1]) * pcoords[2],
            pcoords[0] * pcoords[2],
            pcoords[1] * pcoords[2],
        ]
    }

    /// VTK: `vtkWedge::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        weights[..6].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkWedge::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 18] {
        [
            -1.0 + pcoords[2],
            1.0 - pcoords[2],
            0.0,
            -pcoords[2],
            pcoords[2],
            0.0,
            -1.0 + pcoords[2],
            0.0,
            1.0 - pcoords[2],
            -pcoords[2],
            0.0,
            pcoords[2],
            -1.0 + pcoords[0] + pcoords[1],
            -pcoords[0],
            -pcoords[1],
            1.0 - pcoords[0] - pcoords[1],
            pcoords[0],
            pcoords[1],
        ]
    }

    /// VTK: `vtkWedge::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        derivs[..18].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkWedge::JacobianInverse`.
    pub fn jacobian_inverse(&self, pcoords: [f64; 3]) -> (i32, [[f64; 3]; 3], [f64; 18]) {
        let derivs = Self::interpolation_derivs(pcoords);
        let mut m = [[0.0; 3]; 3];
        for j in 0..Self::NUMBER_OF_POINTS as usize {
            let x = self.get_points().get_point(j as VtkIdType);
            for i in 0..3 {
                m[0][i] += x[i] * derivs[j];
                m[1][i] += x[i] * derivs[6 + j];
                m[2][i] += x[i] * derivs[12 + j];
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

    /// VTK: `vtkWedge::GetPointToOneRingPoints`.
    pub fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_one_ring_points_array(point_id),
        )
    }

    /// VTK: `vtkWedge::GetPointToIncidentFaces`.
    pub fn get_point_to_incident_faces(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_faces_array(point_id),
        )
    }

    /// VTK: `vtkWedge::GetPointToIncidentEdges`.
    pub fn get_point_to_incident_edges(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_edges_array(point_id),
        )
    }

    /// VTK: `vtkWedge::GetFaceToAdjacentFaces`.
    pub fn get_face_to_adjacent_faces(
        &self,
        face_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            NUMBER_OF_POINTS_IN_FACE[face_id as usize],
            Self::get_face_to_adjacent_faces_array(face_id),
        )
    }

    /// VTK: `vtkWedge::GetEdgeToAdjacentFaces`.
    pub fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_to_adjacent_faces_array(edge_id)
    }

    /// VTK: `vtkWedge::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_array(edge_id)
    }

    /// VTK: `vtkWedge::GetFacePoints`.
    pub fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType; 5]) {
        (
            NUMBER_OF_POINTS_IN_FACE[face_id as usize],
            Self::get_face_array(face_id),
        )
    }

    /// VTK: `vtkWedge::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.333333, 0.333333, 0.5])
    }

    /// VTK: `vtkWedge::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 18] {
        &WEDGE_CELL_PCOORDS
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

    fn triangle_face_point_ids(face_id: usize, point_ids: Option<&[VtkIdType]>) -> [VtkIdType; 3] {
        let face = FACES[face_id];
        [
            point_ids.map_or(face[0], |ids| ids[face[0] as usize]),
            point_ids.map_or(face[1], |ids| ids[face[1] as usize]),
            point_ids.map_or(face[2], |ids| ids[face[2] as usize]),
        ]
    }
}

impl Default for Wedge {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for Wedge {
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

impl Cell3DApi for Wedge {
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
        self.is_inside_out()
    }
}

fn triangle_centroid(points: &Points, ids: [VtkIdType; 3]) -> (bool, [f64; 3]) {
    let mut centroid = [0.0; 3];
    for id in ids {
        let point = points.get_point(id);
        for i in 0..3 {
            centroid[i] += point[i] / 3.0;
        }
    }
    (true, centroid)
}

const EDGES: [[VtkIdType; 2]; Wedge::NUMBER_OF_EDGES as usize] = [
    [0, 1],
    [1, 2],
    [2, 0],
    [3, 4],
    [4, 5],
    [5, 3],
    [0, 3],
    [1, 4],
    [2, 5],
];

const FACES: [[VtkIdType; 5]; Wedge::NUMBER_OF_FACES as usize] = [
    [0, 2, 1, -1, -1],
    [3, 4, 5, -1, -1],
    [0, 1, 4, 3, -1],
    [1, 2, 5, 4, -1],
    [2, 0, 3, 5, -1],
];

const EDGE_TO_ADJACENT_FACES: [[VtkIdType; 2]; Wedge::NUMBER_OF_EDGES as usize] = [
    [0, 2],
    [0, 3],
    [0, 3],
    [1, 2],
    [1, 3],
    [1, 4],
    [2, 4],
    [2, 3],
    [3, 4],
];

const FACE_TO_ADJACENT_FACES: [[VtkIdType; 4]; Wedge::NUMBER_OF_FACES as usize] = [
    [4, 3, 2, -1],
    [2, 3, 4, -1],
    [0, 3, 1, 4],
    [0, 4, 1, 2],
    [0, 2, 1, 3],
];

const POINT_TO_INCIDENT_EDGES: [[VtkIdType; 3]; Wedge::NUMBER_OF_POINTS as usize] = [
    [0, 6, 2],
    [0, 1, 7],
    [1, 2, 8],
    [3, 5, 6],
    [3, 7, 4],
    [4, 8, 5],
];

const POINT_TO_INCIDENT_FACES: [[VtkIdType; 3]; Wedge::NUMBER_OF_POINTS as usize] = [
    [2, 4, 0],
    [0, 3, 2],
    [0, 4, 3],
    [1, 4, 2],
    [2, 3, 1],
    [3, 4, 1],
];

const POINT_TO_ONE_RING_POINTS: [[VtkIdType; 3]; Wedge::NUMBER_OF_POINTS as usize] = [
    [1, 3, 2],
    [0, 2, 4],
    [1, 0, 5],
    [4, 5, 0],
    [3, 1, 5],
    [4, 2, 3],
];

const NUMBER_OF_POINTS_IN_FACE: [VtkIdType; Wedge::NUMBER_OF_FACES as usize] = [3, 3, 4, 4, 4];

const WEDGE_CELL_PCOORDS: [f64; 18] = [
    0.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    0.0, 1.0, 0.0, //
    0.0, 0.0, 1.0, //
    1.0, 0.0, 1.0, //
    0.0, 1.0, 1.0, //
];
