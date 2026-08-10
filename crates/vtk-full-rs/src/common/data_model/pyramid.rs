use crate::common::core::{
    math::{
        cross, determinant3x3_from_columns, distance2_between_points, dot, invert_matrix,
        normalize, subtract,
    },
    IdList, Points, VtkIdType,
};

use super::{Cell, Cell3D, Cell3DApi, CellBaseApi, CellType, Line, Quad, Triangle};

const VTK_DIVERGED: f64 = 1.0e6;
const VTK_MAX_ITERATION: usize = 10;
const VTK_CONVERGED: f64 = 1.0e-3;

/// Rust return bundle for VTK `vtkPyramid::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PyramidEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 5],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkPyramid::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PyramidIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// Rust typed equivalent of VTK `vtkPyramid::GetFace` returning `vtkCell*`.
#[derive(Debug)]
pub enum PyramidFace<'a> {
    Triangle(&'a mut Triangle),
    Quad(&'a mut Quad),
}

/// VTK: `vtkPyramid`.
#[derive(Debug)]
pub struct Pyramid {
    cell_3d: Cell3D,
    line: Line,
    triangle: Triangle,
    quad: Quad,
}

impl Pyramid {
    /// VTK: `vtkPyramid::NumberOfPoints`.
    pub const NUMBER_OF_POINTS: VtkIdType = 5;
    /// VTK: `vtkPyramid::NumberOfEdges`.
    pub const NUMBER_OF_EDGES: VtkIdType = 8;
    /// VTK: `vtkPyramid::NumberOfFaces`.
    pub const NUMBER_OF_FACES: VtkIdType = 5;
    /// VTK: `vtkPyramid::MaximumFaceSize`.
    pub const MAXIMUM_FACE_SIZE: VtkIdType = 4;
    /// VTK: `vtkPyramid::MaximumValence`.
    pub const MAXIMUM_VALENCE: VtkIdType = 4;

    /// VTK: `vtkPyramid::New`.
    pub fn new() -> Self {
        let mut pyramid = Self {
            cell_3d: Cell3D::with_class_name("vtkPyramid"),
            line: Line::new(),
            triangle: Triangle::new(),
            quad: Quad::new(),
        };
        pyramid
            .cell_3d
            .cell_mut()
            .get_points_mut()
            .set_number_of_points(Self::NUMBER_OF_POINTS);
        pyramid
            .cell_3d
            .cell_mut()
            .get_point_ids_mut()
            .set_number_of_ids(Self::NUMBER_OF_POINTS);
        for i in 0..Self::NUMBER_OF_POINTS {
            pyramid
                .cell_3d
                .cell_mut()
                .get_points_mut()
                .set_point(i, [0.0, 0.0, 0.0]);
            pyramid.cell_3d.cell_mut().get_point_ids_mut().set_id(i, 0);
        }
        pyramid
    }

    /// VTK: `vtkPyramid::PrintSelf`.
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

    /// VTK: `vtkPyramid::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Pyramid as i32
    }

    /// VTK: `vtkPyramid::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        self.cell_3d.get_cell_dimension()
    }

    /// VTK: `vtkPyramid::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        Self::NUMBER_OF_EDGES as i32
    }

    /// VTK: `vtkPyramid::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        Self::NUMBER_OF_FACES as i32
    }

    /// VTK: `vtkPyramid::GetCentroid`.
    pub fn get_centroid(&self) -> (bool, [f64; 3]) {
        Self::compute_centroid(self.get_points(), None)
    }

    /// VTK: `vtkPyramid::ComputeCentroid`.
    pub fn compute_centroid(points: &Points, point_ids: Option<&[VtkIdType]>) -> (bool, [f64; 3]) {
        let face = Self::face_point_ids(0, point_ids);
        let (ok, mut centroid) =
            polygon_centroid(points, &face[..NUMBER_OF_POINTS_IN_FACE[0] as usize]);
        let apex_id = point_ids.map_or(4, |ids| ids[4]);
        let apex = points.get_point(apex_id);
        for i in 0..3 {
            centroid[i] = 0.75 * centroid[i] + 0.25 * apex[i];
        }
        (ok, centroid)
    }

    /// VTK: `vtkPyramid::IsInsideOut`.
    pub fn is_inside_out(&self) -> bool {
        let normal = polygon_normal(
            self.get_points(),
            &FACES[4][..NUMBER_OF_POINTS_IN_FACE[4] as usize],
        );
        let a = self.get_points().get_point(0);
        let mut b = self.get_points().get_point(4);
        for i in 0..3 {
            b[i] -= a[i];
        }
        dot(normal, b) > 0.0
    }

    /// VTK: `vtkPyramid::EvaluatePosition`.
    pub fn evaluate_position(&self, x: [f64; 3]) -> PyramidEvaluatePosition {
        let apex_point = self.get_points().get_point(4);
        let mut dist2 = distance2_between_points(apex_point, x);
        let mut base_midpoint = self.get_points().get_point(0);
        for i in 1..4 {
            let point = self.get_points().get_point(i);
            for j in 0..3 {
                base_midpoint[j] += point[j];
            }
        }
        for value in &mut base_midpoint {
            *value /= 4.0;
        }
        let length2 = distance2_between_points(apex_point, base_midpoint);
        if dist2 == 0.0 || (length2 != 0.0 && dist2 / length2 < 1.0e-6) {
            let pcoords = [0.0, 0.0, 1.0];
            return PyramidEvaluatePosition {
                inside: 1,
                sub_id: 0,
                pcoords,
                dist2: 0.0,
                weights: Self::interpolation_functions(pcoords),
                closest_point: Some(x),
            };
        }

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

        let mut params = [0.3333333, 0.3333333, 0.3333333];
        let mut pcoords = params;
        let mut weights = [0.0; 5];
        let mut converged = false;
        for _iteration in 0..VTK_MAX_ITERATION {
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
                    scol[j] += coord[j] * derivs[5 + i];
                    tcol[j] += coord[j] * derivs[10 + i];
                }
            }
            for i in 0..3 {
                fcol[i] -= x[i];
            }
            let determinant = determinant3x3_from_columns(rcol, scol, tcol);
            if determinant.abs() < determinant_tolerance {
                return PyramidEvaluatePosition {
                    inside: -1,
                    sub_id: 0,
                    pcoords,
                    dist2,
                    weights,
                    closest_point: None,
                };
            }
            pcoords[0] = params[0] - determinant3x3_from_columns(fcol, scol, tcol) / determinant;
            pcoords[1] = params[1] - determinant3x3_from_columns(rcol, fcol, tcol) / determinant;
            pcoords[2] = params[2] - determinant3x3_from_columns(rcol, scol, fcol) / determinant;
            if (pcoords[0] - params[0]).abs() < VTK_CONVERGED
                && (pcoords[1] - params[1]).abs() < VTK_CONVERGED
                && (pcoords[2] - params[2]).abs() < VTK_CONVERGED
            {
                converged = true;
                break;
            }
            if pcoords[0].abs() > VTK_DIVERGED
                || pcoords[1].abs() > VTK_DIVERGED
                || pcoords[2].abs() > VTK_DIVERGED
            {
                return PyramidEvaluatePosition {
                    inside: -1,
                    sub_id: 0,
                    pcoords,
                    dist2,
                    weights,
                    closest_point: None,
                };
            }
            params = pcoords;
        }
        if !converged {
            return PyramidEvaluatePosition {
                inside: -1,
                sub_id: 0,
                pcoords,
                dist2,
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
        {
            PyramidEvaluatePosition {
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
            dist2 = distance2_between_points(closest, x);
            PyramidEvaluatePosition {
                inside: 0,
                sub_id: 0,
                pcoords,
                dist2,
                weights,
                closest_point: Some(closest),
            }
        }
    }

    /// VTK: `vtkPyramid::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 5]) {
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

    /// VTK: `vtkPyramid::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let normals = [
            [0.0, -0.5547002, 0.8320503],
            [0.5547002, 0.0, 0.8320503],
            [0.0, 0.5547002, 0.8320503],
            [-0.5547002, 0.0, 0.8320503],
            [0.70710670, -0.70710670, 0.0],
            [0.70710670, 0.70710670, 0.0],
        ];
        let point = [0.5, 0.5, 0.3333333];
        let mut vals = [0.0; 6];
        for i in 0..6 {
            vals[i] = normals[i][0] * (pcoords[0] - point[0])
                + normals[i][1] * (pcoords[1] - point[1])
                + normals[i][2] * (pcoords[2] - point[2]);
        }
        let face: &[VtkIdType] = if vals[4] >= 0.0 && vals[5] <= 0.0 && vals[0] >= 0.0 {
            &[0, 1, 4]
        } else if vals[4] >= 0.0 && vals[5] >= 0.0 && vals[1] >= 0.0 {
            &[1, 2, 4]
        } else if vals[4] <= 0.0 && vals[5] >= 0.0 && vals[2] >= 0.0 {
            &[2, 3, 4]
        } else if vals[4] <= 0.0 && vals[5] <= 0.0 && vals[3] >= 0.0 {
            &[3, 0, 4]
        } else {
            &[0, 1, 2, 3]
        };
        pts.set_number_of_ids(face.len() as VtkIdType);
        for (i, local_id) in face.iter().copied().enumerate() {
            pts.set_id(i as VtkIdType, self.get_point_ids().get_id(local_id));
        }
        pcoords.iter().all(|p| *p >= 0.0 && *p <= 1.0) as i32
    }

    /// VTK: `vtkPyramid::GetEdgeToAdjacentFacesArray`.
    pub fn get_edge_to_adjacent_faces_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGE_TO_ADJACENT_FACES[edge_id as usize]
    }

    /// VTK: `vtkPyramid::GetFaceToAdjacentFacesArray`.
    pub fn get_face_to_adjacent_faces_array(face_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &FACE_TO_ADJACENT_FACES[face_id as usize]
    }

    /// VTK: `vtkPyramid::GetPointToIncidentEdgesArray`.
    pub fn get_point_to_incident_edges_array(point_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &POINT_TO_INCIDENT_EDGES[point_id as usize]
    }

    /// VTK: `vtkPyramid::GetPointToIncidentFacesArray`.
    pub fn get_point_to_incident_faces_array(point_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &POINT_TO_INCIDENT_FACES[point_id as usize]
    }

    /// VTK: `vtkPyramid::GetPointToOneRingPointsArray`.
    pub fn get_point_to_one_ring_points_array(point_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &POINT_TO_ONE_RING_POINTS[point_id as usize]
    }

    /// VTK: `vtkPyramid::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkPyramid::GetFaceArray`.
    pub fn get_face_array(face_id: VtkIdType) -> &'static [VtkIdType; 5] {
        &FACES[face_id as usize]
    }

    /// VTK: `vtkPyramid::GetEdge`.
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

    /// VTK: `vtkPyramid::GetFace`.
    pub fn get_face(&mut self, face_id: i32) -> PyramidFace<'_> {
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
            PyramidFace::Quad(&mut self.quad)
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
            PyramidFace::Triangle(&mut self.triangle)
        }
    }

    /// VTK: `vtkPyramid::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> PyramidIntersectWithLine {
        let mut result = PyramidIntersectWithLine {
            intersection: 0,
            t: f64::MAX,
            x: [0.0; 3],
            pcoords: [0.0; 3],
            sub_id: 0,
        };

        for face_num in 1..Self::NUMBER_OF_FACES as usize {
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
                    let eval = self.evaluate_position(hit.x);
                    result.pcoords = eval.pcoords;
                    result.sub_id = eval.sub_id;
                }
            }
        }

        let verts = FACES[0];
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
                result.pcoords = [hit.pcoords[0], hit.pcoords[1], 0.0];
                result.sub_id = hit.sub_id;
            }
        }
        result
    }

    /// VTK: `vtkPyramid::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        let d1 = distance2_between_points(
            self.get_points().get_point(0),
            self.get_points().get_point(2),
        );
        let d2 = distance2_between_points(
            self.get_points().get_point(1),
            self.get_points().get_point(3),
        );
        pt_ids.set_number_of_ids(8);
        let ids = if d1 < d2 {
            [0, 1, 2, 4, 0, 2, 3, 4]
        } else {
            [0, 1, 3, 4, 1, 2, 3, 4]
        };
        for (i, id) in ids.into_iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, id);
        }
        1
    }

    /// VTK: `vtkPyramid::Derivatives`.
    pub fn derivatives(
        &self,
        sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        if pcoords[2] > 0.999 {
            let pcoords1 = [0.5, 0.5, 2.0 * 0.998 - pcoords[2]];
            let mut derivs1 = vec![0.0; 3 * dim as usize];
            self.derivatives(sub_id, pcoords1, values, dim, &mut derivs1);
            let pcoords2 = [0.5, 0.5, 0.998];
            let mut derivs2 = vec![0.0; 3 * dim as usize];
            self.derivatives(sub_id, pcoords2, values, dim, &mut derivs2);
            for i in 0..3 * dim as usize {
                derivs[i] = 2.0 * derivs2[i] - derivs1[i];
            }
            return;
        }

        let (_success, jacobian_inverse, function_derivs) = self.jacobian_inverse(pcoords);
        for k in 0..dim as usize {
            let mut sum = [0.0; 3];
            for i in 0..Self::NUMBER_OF_POINTS as usize {
                let value = values[dim as usize * i + k];
                sum[0] += function_derivs[i] * value;
                sum[1] += function_derivs[5 + i] * value;
                sum[2] += function_derivs[10 + i] * value;
            }
            for j in 0..3 {
                derivs[3 * k + j] = sum[0] * jacobian_inverse[j][0]
                    + sum[1] * jacobian_inverse[j][1]
                    + sum[2] * jacobian_inverse[j][2];
            }
        }
    }

    /// VTK: `vtkPyramid::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 5] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        let tm = 1.0 - pcoords[2];
        [
            rm * sm * tm,
            pcoords[0] * sm * tm,
            pcoords[0] * pcoords[1] * tm,
            rm * pcoords[1] * tm,
            pcoords[2],
        ]
    }

    /// VTK: `vtkPyramid::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        weights[..5].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkPyramid::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 15] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        let tm = 1.0 - pcoords[2];
        [
            -sm * tm,
            sm * tm,
            pcoords[1] * tm,
            -pcoords[1] * tm,
            0.0,
            -rm * tm,
            -pcoords[0] * tm,
            pcoords[0] * tm,
            rm * tm,
            0.0,
            -rm * sm,
            -pcoords[0] * sm,
            -pcoords[0] * pcoords[1],
            -rm * pcoords[1],
            1.0,
        ]
    }

    /// VTK: `vtkPyramid::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        derivs[..15].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkPyramid::JacobianInverse`.
    pub fn jacobian_inverse(&self, pcoords: [f64; 3]) -> (i32, [[f64; 3]; 3], [f64; 15]) {
        let derivs = Self::interpolation_derivs(pcoords);
        let mut m = [[0.0; 3]; 3];
        for j in 0..Self::NUMBER_OF_POINTS as usize {
            let x = self.get_points().get_point(j as VtkIdType);
            for i in 0..3 {
                m[0][i] += x[i] * derivs[j];
                m[1][i] += x[i] * derivs[5 + j];
                m[2][i] += x[i] * derivs[10 + j];
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

    /// VTK: `vtkPyramid::GetPointToOneRingPoints`.
    pub fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            VALENCE_AT_POINT[point_id as usize],
            Self::get_point_to_one_ring_points_array(point_id),
        )
    }

    /// VTK: `vtkPyramid::GetPointToIncidentFaces`.
    pub fn get_point_to_incident_faces(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            VALENCE_AT_POINT[point_id as usize],
            Self::get_point_to_incident_faces_array(point_id),
        )
    }

    /// VTK: `vtkPyramid::GetPointToIncidentEdges`.
    pub fn get_point_to_incident_edges(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            VALENCE_AT_POINT[point_id as usize],
            Self::get_point_to_incident_edges_array(point_id),
        )
    }

    /// VTK: `vtkPyramid::GetFaceToAdjacentFaces`.
    pub fn get_face_to_adjacent_faces(
        &self,
        face_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            NUMBER_OF_POINTS_IN_FACE[face_id as usize],
            Self::get_face_to_adjacent_faces_array(face_id),
        )
    }

    /// VTK: `vtkPyramid::GetEdgeToAdjacentFaces`.
    pub fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_to_adjacent_faces_array(edge_id)
    }

    /// VTK: `vtkPyramid::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_array(edge_id)
    }

    /// VTK: `vtkPyramid::GetFacePoints`.
    pub fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType; 5]) {
        (
            NUMBER_OF_POINTS_IN_FACE[face_id as usize],
            Self::get_face_array(face_id),
        )
    }

    /// VTK: `vtkPyramid::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.4, 0.4, 0.2])
    }

    /// VTK: `vtkPyramid::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 15] {
        &PYRAMID_CELL_PCOORDS
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

    fn face_point_ids(face_id: usize, point_ids: Option<&[VtkIdType]>) -> [VtkIdType; 5] {
        let face = FACES[face_id];
        [
            point_ids.map_or(face[0], |ids| ids[face[0] as usize]),
            point_ids.map_or(face[1], |ids| ids[face[1] as usize]),
            point_ids.map_or(face[2], |ids| ids[face[2] as usize]),
            if face[3] == -1 {
                -1
            } else {
                point_ids.map_or(face[3], |ids| ids[face[3] as usize])
            },
            -1,
        ]
    }
}

impl Default for Pyramid {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for Pyramid {
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

impl Cell3DApi for Pyramid {
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

fn polygon_normal(points: &Points, ids: &[VtkIdType]) -> [f64; 3] {
    let mut normal = [0.0; 3];
    for i in 0..ids.len() {
        let p0 = points.get_point(ids[i]);
        let p1 = points.get_point(ids[(i + 1) % ids.len()]);
        normal[0] += (p0[1] - p1[1]) * (p0[2] + p1[2]);
        normal[1] += (p0[2] - p1[2]) * (p0[0] + p1[0]);
        normal[2] += (p0[0] - p1[0]) * (p0[1] + p1[1]);
    }
    normalize(&mut normal);
    normal
}

fn polygon_centroid(points: &Points, ids: &[VtkIdType]) -> (bool, [f64; 3]) {
    if ids.len() < 2 {
        return (false, [0.0; 3]);
    }
    let normal = polygon_normal(points, ids);
    if normal == [0.0; 3] {
        return (false, [0.0; 3]);
    }
    let wt = 1.0 / ids.len() as f64;
    let mut xx = [0.0; 3];
    for id in ids {
        let point = points.get_point(*id);
        for i in 0..3 {
            xx[i] += wt * point[i];
        }
    }
    let mut total_area = 0.0;
    let mut accum = [0.0; 3];
    let mut pp = points.get_point(ids[ids.len() - 1]);
    for id in ids {
        let qq = points.get_point(*id);
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

const EDGES: [[VtkIdType; 2]; 8] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [0, 4],
    [1, 4],
    [2, 4],
    [3, 4],
];

const FACES: [[VtkIdType; 5]; 5] = [
    [0, 3, 2, 1, -1],
    [0, 1, 4, -1, -1],
    [1, 2, 4, -1, -1],
    [2, 3, 4, -1, -1],
    [3, 0, 4, -1, -1],
];

const EDGE_TO_ADJACENT_FACES: [[VtkIdType; 2]; 8] = [
    [0, 1],
    [0, 2],
    [0, 3],
    [0, 4],
    [1, 4],
    [1, 2],
    [2, 3],
    [3, 4],
];

const FACE_TO_ADJACENT_FACES: [[VtkIdType; 4]; 5] = [
    [4, 3, 2, 1],
    [0, 2, 4, -1],
    [0, 3, 1, -1],
    [0, 4, 2, -1],
    [0, 1, 3, -1],
];

const POINT_TO_INCIDENT_EDGES: [[VtkIdType; 4]; 5] = [
    [0, 4, 3, -1],
    [0, 1, 5, -1],
    [1, 2, 6, -1],
    [2, 3, 7, -1],
    [4, 5, 6, 7],
];

const POINT_TO_INCIDENT_FACES: [[VtkIdType; 4]; 5] = [
    [1, 4, 0, -1],
    [0, 2, 1, -1],
    [0, 3, 2, -1],
    [0, 4, 3, -1],
    [1, 2, 3, 4],
];

const POINT_TO_ONE_RING_POINTS: [[VtkIdType; 4]; 5] = [
    [1, 4, 3, -1],
    [0, 2, 4, -1],
    [1, 3, 4, -1],
    [2, 0, 4, -1],
    [0, 1, 2, 3],
];

const NUMBER_OF_POINTS_IN_FACE: [VtkIdType; 5] = [4, 3, 3, 3, 3];
const VALENCE_AT_POINT: [VtkIdType; 5] = [4, 3, 3, 3, 3];

const PYRAMID_CELL_PCOORDS: [f64; 15] = [
    0.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    1.0, 1.0, 0.0, //
    0.0, 1.0, 0.0, //
    0.0, 0.0, 1.0, //
];
