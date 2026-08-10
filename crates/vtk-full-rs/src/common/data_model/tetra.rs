use crate::common::core::{
    math::{
        cross, determinant3x3_from_columns, determinant3x3_from_values, dot, invert_matrix,
        normalize, solve_linear_system,
    },
    IdList, Points, VtkIdType,
};

use super::{Cell, Cell3D, Cell3DApi, CellBaseApi, CellType, Line, Triangle};

/// VTK: `vtkTetra`.
#[derive(Debug)]
pub struct Tetra {
    cell_3d: Cell3D,
    line: Line,
    triangle: Triangle,
}

impl Tetra {
    /// VTK: `vtkTetra::NumberOfPoints`.
    pub const NUMBER_OF_POINTS: VtkIdType = 4;
    /// VTK: `vtkTetra::NumberOfEdges`.
    pub const NUMBER_OF_EDGES: VtkIdType = 6;
    /// VTK: `vtkTetra::NumberOfFaces`.
    pub const NUMBER_OF_FACES: VtkIdType = 4;
    /// VTK: `vtkTetra::MaximumFaceSize`.
    pub const MAXIMUM_FACE_SIZE: VtkIdType = 3;
    /// VTK: `vtkTetra::MaximumValence`.
    pub const MAXIMUM_VALENCE: VtkIdType = 3;

    /// VTK: `vtkTetra::New`.
    pub fn new() -> Self {
        let mut tetra = Self {
            cell_3d: Cell3D::with_class_name("vtkTetra"),
            line: Line::new(),
            triangle: Triangle::new(),
        };
        tetra
            .cell_3d
            .cell_mut()
            .get_points_mut()
            .set_number_of_points(4);
        tetra
            .cell_3d
            .cell_mut()
            .get_point_ids_mut()
            .set_number_of_ids(4);
        for i in 0..4 {
            tetra
                .cell_3d
                .cell_mut()
                .get_points_mut()
                .set_point(i, [0.0, 0.0, 0.0]);
            tetra.cell_3d.cell_mut().get_point_ids_mut().set_id(i, 0);
        }
        tetra
    }

    /// VTK: `vtkTetra::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell_3d.print_self();
        text.push_str("\nLine:\n");
        text.push_str(&self.line.print_self());
        text.push_str("\nTriangle:\n");
        text.push_str(&self.triangle.print_self());
        text
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

    /// VTK: `vtkCell::ComputeBoundingSphere`.
    pub fn compute_bounding_sphere(&self) -> ([f64; 3], f64) {
        self.cell_3d.cell().compute_bounding_sphere()
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

    /// VTK: `vtkTetra::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Tetra as i32
    }

    /// VTK: `vtkTetra::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        Self::NUMBER_OF_EDGES as i32
    }

    /// VTK: `vtkTetra::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        Self::NUMBER_OF_FACES as i32
    }

    /// VTK: `vtkTetra::GetCentroid`.
    pub fn get_centroid(&self) -> (bool, [f64; 3]) {
        Self::compute_centroid(self.cell_3d.cell().get_points(), None)
    }

    /// VTK: `vtkTetra::ComputeCentroid`.
    pub fn compute_centroid(points: &Points, point_ids: Option<&[VtkIdType]>) -> (bool, [f64; 3]) {
        let ids = point_ids.unwrap_or(&[0, 1, 2, 3]);
        assert!(
            ids.len() >= Self::NUMBER_OF_POINTS as usize,
            "vtkTetra::ComputeCentroid point id slice too short"
        );

        let mut centroid = [0.0; 3];
        for &id in ids.iter().take(Self::NUMBER_OF_POINTS as usize) {
            let point = points.get_point(id);
            for i in 0..3 {
                centroid[i] += point[i];
            }
        }
        for value in &mut centroid {
            *value /= Self::NUMBER_OF_POINTS as f64;
        }
        (true, centroid)
    }

    /// VTK: `vtkTetra::IsInsideOut`.
    pub fn is_inside_out(&self) -> bool {
        let points = self.cell_3d.cell().get_points();
        let a = points.get_point(0);
        let b = points.get_point(1);
        let c = points.get_point(2);
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let v = cross(d, e);
        let fourth = points.get_point(3);
        let offset = [
            fourth[0] - (a[0] + b[0] + c[0]) / 3.0,
            fourth[1] - (a[1] + b[1] + c[1]) / 3.0,
            fourth[2] - (a[2] + b[2] + c[2]) / 3.0,
        ];
        dot(offset, v) < 0.0
    }

    /// VTK: `vtkTetra::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut Line {
        let verts = Self::get_edge_array(edge_id as VtkIdType);

        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(0, self.cell_3d.cell().get_point_ids().get_id(verts[0]));
        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(1, self.cell_3d.cell().get_point_ids().get_id(verts[1]));

        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(0, self.cell_3d.cell().get_points().get_point(verts[0]));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(1, self.cell_3d.cell().get_points().get_point(verts[1]));

        &mut self.line
    }

    /// VTK: `vtkTetra::GetFace`.
    pub fn get_face(&mut self, face_id: i32) -> &mut Triangle {
        let verts = Self::get_face_array(face_id as VtkIdType);
        for i in 0..3 {
            let vert = verts[i];
            self.triangle.cell_mut().get_point_ids_mut().set_id(
                i as VtkIdType,
                self.cell_3d.cell().get_point_ids().get_id(vert),
            );
            self.triangle.cell_mut().get_points_mut().set_point(
                i as VtkIdType,
                self.cell_3d.cell().get_points().get_point(vert),
            );
        }

        &mut self.triangle
    }

    /// VTK: `vtkTetra::EvaluatePosition`.
    pub fn evaluate_position(
        &mut self,
        x: [f64; 3],
        closest_point_requested: bool,
    ) -> TetraEvaluatePosition {
        let points = self.cell_3d.cell().get_points();
        let pt1 = points.get_point(1);
        let pt2 = points.get_point(2);
        let pt3 = points.get_point(3);
        let pt4 = points.get_point(0);

        let rhs = [x[0] - pt4[0], x[1] - pt4[1], x[2] - pt4[2]];
        let c1 = [pt1[0] - pt4[0], pt1[1] - pt4[1], pt1[2] - pt4[2]];
        let c2 = [pt2[0] - pt4[0], pt2[1] - pt4[1], pt2[2] - pt4[2]];
        let c3 = [pt3[0] - pt4[0], pt3[1] - pt4[1], pt3[2] - pt4[2]];

        let det = determinant3x3_from_columns(c1, c2, c3);
        if det == 0.0 {
            return TetraEvaluatePosition {
                inside: -1,
                sub_id: 0,
                pcoords: [0.0, 0.0, 0.0],
                dist2: 0.0,
                weights: [0.0; 4],
                closest_point: None,
            };
        }

        let pcoords = [
            determinant3x3_from_columns(rhs, c2, c3) / det,
            determinant3x3_from_columns(c1, rhs, c3) / det,
            determinant3x3_from_columns(c1, c2, rhs) / det,
        ];
        let p4 = 1.0 - pcoords[0] - pcoords[1] - pcoords[2];
        let weights = [p4, pcoords[0], pcoords[1], pcoords[2]];

        if pcoords[0] >= -0.001
            && pcoords[0] <= 1.001
            && pcoords[1] >= -0.001
            && pcoords[1] <= 1.001
            && pcoords[2] >= -0.001
            && pcoords[2] <= 1.001
            && p4 >= -0.001
            && p4 <= 1.001
        {
            return TetraEvaluatePosition {
                inside: 1,
                sub_id: 0,
                pcoords,
                dist2: 0.0,
                weights,
                closest_point: closest_point_requested.then_some(x),
            };
        }

        let (dist2, closest_point) = if closest_point_requested {
            let mut min_dist2 = f64::MAX;
            let mut closest = [0.0; 3];
            for face_num in 0..Self::NUMBER_OF_FACES as i32 {
                let face_eval = self.get_face(face_num).evaluate_position(x, true);
                if face_eval.dist2 < min_dist2 {
                    min_dist2 = face_eval.dist2;
                    closest = face_eval.closest_point.unwrap_or([0.0, 0.0, 0.0]);
                }
            }
            (min_dist2, Some(closest))
        } else {
            (0.0, None)
        };

        TetraEvaluatePosition {
            inside: 0,
            sub_id: 0,
            pcoords,
            dist2,
            weights,
            closest_point,
        }
    }

    /// VTK: `vtkTetra::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> TetraIntersectWithLine {
        let mut intersection = 0;
        let mut t = f64::MAX;
        let mut x = [0.0; 3];
        let mut pcoords = [0.0; 3];
        let mut sub_id = 0;

        for face_num in 0..Self::NUMBER_OF_FACES as i32 {
            let face_eval = self.get_face(face_num).intersect_with_line(p1, p2, tol);
            if face_eval.intersection != 0 {
                intersection = 1;
                if face_eval.t < t {
                    t = face_eval.t;
                    x = face_eval.x;
                    sub_id = face_eval.sub_id;
                    match face_num {
                        0 => {
                            pcoords[0] = face_eval.pcoords[0];
                            pcoords[1] = 0.0;
                            pcoords[2] = face_eval.pcoords[1];
                        }
                        1 => {
                            pcoords[0] = 1.0 - face_eval.pcoords[0] - face_eval.pcoords[1];
                            pcoords[1] = face_eval.pcoords[0];
                            pcoords[2] = face_eval.pcoords[1];
                        }
                        2 => {
                            pcoords[0] = 0.0;
                            pcoords[1] = 1.0 - face_eval.pcoords[0] - face_eval.pcoords[1];
                            pcoords[2] = face_eval.pcoords[1];
                        }
                        _ => {
                            pcoords[0] = face_eval.pcoords[0];
                            pcoords[1] = face_eval.pcoords[1];
                            pcoords[2] = face_eval.pcoords[2];
                        }
                    }
                }
            }
        }

        TetraIntersectWithLine {
            intersection,
            t,
            x,
            pcoords,
            sub_id,
        }
    }

    /// VTK: `vtkTetra::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 4]) {
        let weights = Self::interpolation_functions(pcoords);
        let mut x = [0.0; 3];
        for i in 0..Self::NUMBER_OF_POINTS {
            let point = self.cell_3d.cell().get_points().get_point(i);
            for j in 0..3 {
                x[j] += point[j] * weights[i as usize];
            }
        }
        (x, weights)
    }

    /// VTK: `vtkTetra::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let mut min_pcoord = 1.0 - pcoords[0] - pcoords[1] - pcoords[2];
        let mut idx = 3;
        for (i, &pcoord) in pcoords.iter().enumerate() {
            if pcoord < min_pcoord {
                min_pcoord = pcoord;
                idx = i;
            }
        }

        pts.set_number_of_ids(3);
        let point_ids = self.cell_3d.cell().get_point_ids();
        match idx {
            0 => {
                pts.set_id(0, point_ids.get_id(0));
                pts.set_id(1, point_ids.get_id(2));
                pts.set_id(2, point_ids.get_id(3));
            }
            1 => {
                pts.set_id(0, point_ids.get_id(0));
                pts.set_id(1, point_ids.get_id(1));
                pts.set_id(2, point_ids.get_id(3));
            }
            2 => {
                pts.set_id(0, point_ids.get_id(0));
                pts.set_id(1, point_ids.get_id(1));
                pts.set_id(2, point_ids.get_id(2));
            }
            _ => {
                pts.set_id(0, point_ids.get_id(1));
                pts.set_id(1, point_ids.get_id(2));
                pts.set_id(2, point_ids.get_id(3));
            }
        }

        (pcoords[0] >= 0.0
            && pcoords[1] >= 0.0
            && pcoords[2] >= 0.0
            && pcoords[0] <= 1.0
            && pcoords[1] <= 1.0
            && pcoords[2] <= 1.0
            && (1.0 - pcoords[0] - pcoords[1] - pcoords[2]) >= 0.0) as i32
    }

    /// VTK: `vtkTetra::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        assert!(edge_id < Self::NUMBER_OF_EDGES, "edgeId too large");
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkTetra::GetFaceArray`.
    pub fn get_face_array(face_id: VtkIdType) -> &'static [VtkIdType; 4] {
        assert!(face_id < Self::NUMBER_OF_FACES, "faceId too large");
        &FACES[face_id as usize]
    }

    /// VTK: `vtkTetra::GetEdgeToAdjacentFacesArray`.
    pub fn get_edge_to_adjacent_faces_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        assert!(edge_id < Self::NUMBER_OF_EDGES, "edgeId too large");
        &EDGE_TO_ADJACENT_FACES[edge_id as usize]
    }

    /// VTK: `vtkTetra::GetFaceToAdjacentFacesArray`.
    pub fn get_face_to_adjacent_faces_array(face_id: VtkIdType) -> &'static [VtkIdType; 3] {
        assert!(face_id < Self::NUMBER_OF_FACES, "faceId too large");
        &FACE_TO_ADJACENT_FACES[face_id as usize]
    }

    /// VTK: `vtkTetra::GetPointToIncidentEdgesArray`.
    pub fn get_point_to_incident_edges_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        assert!(point_id < Self::NUMBER_OF_POINTS, "pointId too large");
        &POINT_TO_INCIDENT_EDGES[point_id as usize]
    }

    /// VTK: `vtkTetra::GetPointToIncidentFacesArray`.
    pub fn get_point_to_incident_faces_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        assert!(point_id < Self::NUMBER_OF_POINTS, "pointId too large");
        &POINT_TO_INCIDENT_FACES[point_id as usize]
    }

    /// VTK: `vtkTetra::GetPointToOneRingPointsArray`.
    pub fn get_point_to_one_ring_points_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        assert!(point_id < Self::NUMBER_OF_POINTS, "pointId too large");
        &POINT_TO_ONE_RING_POINTS[point_id as usize]
    }

    /// VTK: `vtkTetra::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_array(edge_id)
    }

    /// VTK: `vtkTetra::GetFacePoints`.
    pub fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (Self::MAXIMUM_FACE_SIZE, Self::get_face_array(face_id))
    }

    /// VTK: `vtkTetra::GetEdgeToAdjacentFaces`.
    pub fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_to_adjacent_faces_array(edge_id)
    }

    /// VTK: `vtkTetra::GetFaceToAdjacentFaces`.
    pub fn get_face_to_adjacent_faces(
        &self,
        face_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_FACE_SIZE,
            Self::get_face_to_adjacent_faces_array(face_id),
        )
    }

    /// VTK: `vtkTetra::GetPointToIncidentEdges`.
    pub fn get_point_to_incident_edges(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_edges_array(point_id),
        )
    }

    /// VTK: `vtkTetra::GetPointToIncidentFaces`.
    pub fn get_point_to_incident_faces(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_faces_array(point_id),
        )
    }

    /// VTK: `vtkTetra::GetPointToOneRingPoints`.
    pub fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_one_ring_points_array(point_id),
        )
    }

    /// VTK: `vtkTetra::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.25, 0.25, 0.25])
    }

    /// VTK: `vtkTetra::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(Self::NUMBER_OF_POINTS);
        for id in 0..Self::NUMBER_OF_POINTS {
            pt_ids.set_id(id, id);
        }
        1
    }

    /// VTK: `vtkTetra::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        _pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let dim = dim.max(0) as usize;
        assert!(
            values.len() >= dim * Self::NUMBER_OF_POINTS as usize,
            "vtkTetra::Derivatives values slice too short"
        );
        assert!(
            derivs.len() >= dim * 3,
            "vtkTetra::Derivatives derivs slice too short"
        );

        let (_success, jacobian_inverse, function_derivs) = self.jacobian_inverse();

        for k in 0..dim {
            let mut sum = [0.0; 3];
            for i in 0..Self::NUMBER_OF_POINTS as usize {
                let value = values[dim * i + k];
                sum[0] += function_derivs[i] * value;
                sum[1] += function_derivs[4 + i] * value;
                sum[2] += function_derivs[8 + i] * value;
            }

            for j in 0..3 {
                derivs[3 * k + j] = sum[0] * jacobian_inverse[j][0]
                    + sum[1] * jacobian_inverse[j][1]
                    + sum[2] * jacobian_inverse[j][2];
            }
        }
    }

    /// VTK: `vtkTetra::TetraCenter`.
    pub fn tetra_center(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], p4: [f64; 3]) -> [f64; 3] {
        [
            (p1[0] + p2[0] + p3[0] + p4[0]) / 4.0,
            (p1[1] + p2[1] + p3[1] + p4[1]) / 4.0,
            (p1[2] + p2[2] + p3[2] + p4[2]) / 4.0,
        ]
    }

    /// VTK: `vtkTetra::ComputeVolume`.
    pub fn compute_volume(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], p4: [f64; 3]) -> f64 {
        determinant3x3_from_values(
            p2[0] - p1[0],
            p3[0] - p1[0],
            p4[0] - p1[0],
            p2[1] - p1[1],
            p3[1] - p1[1],
            p4[1] - p1[1],
            p2[2] - p1[2],
            p3[2] - p1[2],
            p4[2] - p1[2],
        ) / 6.0
    }

    /// VTK: `vtkTetra::Circumsphere`.
    pub fn circumsphere(x1: [f64; 3], x2: [f64; 3], x3: [f64; 3], x4: [f64; 3]) -> (f64, [f64; 3]) {
        let mut n12 = [0.0; 3];
        let mut n13 = [0.0; 3];
        let mut n14 = [0.0; 3];
        let mut x12 = [0.0; 3];
        let mut x13 = [0.0; 3];
        let mut x14 = [0.0; 3];
        for i in 0..3 {
            n12[i] = x2[i] - x1[i];
            n13[i] = x3[i] - x1[i];
            n14[i] = x4[i] - x1[i];
            x12[i] = (x2[i] + x1[i]) * 0.5;
            x13[i] = (x3[i] + x1[i]) * 0.5;
            x14[i] = (x4[i] + x1[i]) * 0.5;
        }

        let rhs = vec![dot(n12, x12), dot(n13, x13), dot(n14, x14)];
        let (success, _factored, rhs) =
            solve_linear_system(vec![n12.to_vec(), n13.to_vec(), n14.to_vec()], rhs, 3);
        if !success {
            return (f64::MAX, [0.0, 0.0, 0.0]);
        }

        let center = [rhs[0], rhs[1], rhs[2]];
        let mut sum = 0.0;
        for point in [x1, x2, x3, x4] {
            for i in 0..3 {
                let diff = point[i] - rhs[i];
                sum += diff * diff;
            }
        }

        let radius_squared = sum * 0.25;
        if radius_squared > f64::MAX {
            (f64::MAX, center)
        } else {
            (radius_squared, center)
        }
    }

    /// VTK: `vtkTetra::Insphere`.
    pub fn insphere(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], p4: [f64; 3]) -> (f64, [f64; 3]) {
        let u = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let v = [p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]];
        let w = [p4[0] - p1[0], p4[1] - p1[1], p4[2] - p1[2]];

        let mut p = cross(u, v);
        normalize(&mut p);
        let mut q = cross(v, w);
        normalize(&mut q);
        let mut r = cross(w, u);
        normalize(&mut r);

        let o1 = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
        let o2 = [q[0] - r[0], q[1] - r[1], q[2] - r[2]];
        let y = cross(o1, o2);

        let o1 = [u[0] - w[0], u[1] - w[1], u[2] - w[2]];
        let o2 = [v[0] - w[0], v[1] - w[1], v[2] - w[2]];
        let mut s = cross(o1, o2);
        normalize(&mut s);
        s = [-s[0], -s[1], -s[2]];

        let o1 = [s[0] - p[0], s[1] - p[1], s[2] - p[2]];
        let t = dot(w, s) / dot(y, o1);
        let center = [p1[0] + t * y[0], p1[1] + t * y[1], p1[2] + t * y[2]];

        ((t * dot(y, p)).abs(), center)
    }

    /// VTK: `vtkTetra::BarycentricCoords`.
    pub fn barycentric_coords(
        x: [f64; 3],
        x1: [f64; 3],
        x2: [f64; 3],
        x3: [f64; 3],
        x4: [f64; 3],
    ) -> (i32, [f64; 4]) {
        let a1 = vec![x1[0], x2[0], x3[0], x4[0]];
        let a2 = vec![x1[1], x2[1], x3[1], x4[1]];
        let a3 = vec![x1[2], x2[2], x3[2], x4[2]];
        let a4 = vec![1.0, 1.0, 1.0, 1.0];
        let p = vec![x[0], x[1], x[2], 1.0];

        let (success, _factored, p) = solve_linear_system(vec![a1, a2, a3, a4], p, 4);
        if success {
            (1, [p[0], p[1], p[2], p[3]])
        } else {
            (0, [0.0; 4])
        }
    }

    /// VTK: `vtkTetra::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 4] {
        [
            1.0 - pcoords[0] - pcoords[1] - pcoords[2],
            pcoords[0],
            pcoords[1],
            pcoords[2],
        ]
    }

    /// VTK: `vtkTetra::InterpolationDerivs`.
    pub fn interpolation_derivs(_pcoords: [f64; 3]) -> [f64; 12] {
        [
            -1.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0,
        ]
    }

    /// VTK: `vtkTetra::JacobianInverse`.
    pub fn jacobian_inverse(&self) -> (i32, [[f64; 3]; 3], [f64; 12]) {
        let derivs = Self::interpolation_derivs([0.0, 0.0, 0.0]);
        let mut m = [[0.0; 3]; 3];

        for j in 0..Self::NUMBER_OF_POINTS as usize {
            let x = self.cell_3d.cell().get_points().get_point(j as VtkIdType);
            for i in 0..3 {
                m[0][i] += x[i] * derivs[j];
                m[1][i] += x[i] * derivs[4 + j];
                m[2][i] += x[i] * derivs[8 + j];
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

    /// VTK: `vtkTetra::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        assert!(
            weights.len() >= Self::NUMBER_OF_POINTS as usize,
            "vtkTetra::InterpolateFunctions weights slice too short"
        );
        weights[..Self::NUMBER_OF_POINTS as usize]
            .copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkTetra::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        assert!(
            derivs.len() >= 12,
            "vtkTetra::InterpolateDerivs derivs slice too short"
        );
        derivs[..12].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkTetra::GetTriangleCases`.
    pub fn get_triangle_cases(case_id: i32) -> &'static [i32; 7] {
        assert!(
            (0..TRIANGLE_CASES.len() as i32).contains(&case_id),
            "caseId out of range"
        );
        &TRIANGLE_CASES[case_id as usize]
    }

    /// VTK: `vtkTetra::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 12] {
        &PARAMETRIC_COORDS
    }

    /// VTK: `vtkTetra::GetParametricDistance`.
    pub fn get_parametric_distance(&self, pcoords: [f64; 3]) -> f64 {
        let pc = [
            pcoords[0],
            pcoords[1],
            pcoords[2],
            1.0 - pcoords[0] - pcoords[1] - pcoords[2],
        ];
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
}

/// Rust return bundle for VTK `vtkTetra::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TetraEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 4],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkTetra::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TetraIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

impl Default for Tetra {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for Tetra {
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

impl Cell3DApi for Tetra {
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
        let (count, pts) = self.get_point_to_one_ring_points(point_id);
        (count, pts.as_slice())
    }

    fn get_centroid(&self) -> (bool, [f64; 3]) {
        self.get_centroid()
    }

    fn is_inside_out(&self) -> bool {
        self.is_inside_out()
    }
}

/// VTK: local `edges` table in `vtkTetra.cxx`.
const EDGES: [[VtkIdType; 2]; Tetra::NUMBER_OF_EDGES as usize] =
    [[0, 1], [1, 2], [2, 0], [0, 3], [1, 3], [2, 3]];

/// VTK: local `faces` table in `vtkTetra.cxx`.
const FACES: [[VtkIdType; 4]; Tetra::NUMBER_OF_FACES as usize] =
    [[0, 1, 3, -1], [1, 2, 3, -1], [2, 0, 3, -1], [0, 2, 1, -1]];

/// VTK: local `edgeToAdjacentFaces` table in `vtkTetra.cxx`.
const EDGE_TO_ADJACENT_FACES: [[VtkIdType; 2]; Tetra::NUMBER_OF_EDGES as usize] =
    [[0, 3], [1, 3], [2, 3], [0, 2], [0, 1], [1, 2]];

/// VTK: local `faceToAdjacentFaces` table in `vtkTetra.cxx`.
const FACE_TO_ADJACENT_FACES: [[VtkIdType; 3]; Tetra::NUMBER_OF_FACES as usize] =
    [[3, 1, 2], [3, 2, 0], [3, 0, 1], [2, 1, 0]];

/// VTK: local `pointToIncidentEdges` table in `vtkTetra.cxx`.
const POINT_TO_INCIDENT_EDGES: [[VtkIdType; 3]; Tetra::NUMBER_OF_POINTS as usize] =
    [[0, 3, 2], [0, 1, 4], [1, 2, 5], [3, 4, 5]];

/// VTK: local `pointToIncidentFaces` table in `vtkTetra.cxx`.
const POINT_TO_INCIDENT_FACES: [[VtkIdType; 3]; Tetra::NUMBER_OF_POINTS as usize] =
    [[0, 2, 3], [3, 1, 0], [3, 2, 1], [0, 1, 2]];

/// VTK: local `pointToOneRingPoints` table in `vtkTetra.cxx`.
const POINT_TO_ONE_RING_POINTS: [[VtkIdType; 3]; Tetra::NUMBER_OF_POINTS as usize] =
    [[1, 3, 2], [0, 2, 3], [1, 0, 3], [0, 1, 2]];

/// VTK: local `triCases` table in `vtkTetra.cxx`.
const TRIANGLE_CASES: [[i32; 7]; 16] = [
    [-1, -1, -1, -1, -1, -1, -1],
    [3, 0, 2, -1, -1, -1, -1],
    [1, 0, 4, -1, -1, -1, -1],
    [2, 3, 4, 2, 4, 1, -1],
    [2, 1, 5, -1, -1, -1, -1],
    [5, 3, 1, 1, 3, 0, -1],
    [2, 0, 5, 5, 0, 4, -1],
    [5, 3, 4, -1, -1, -1, -1],
    [4, 3, 5, -1, -1, -1, -1],
    [4, 0, 5, 5, 0, 2, -1],
    [5, 0, 3, 1, 0, 5, -1],
    [2, 5, 1, -1, -1, -1, -1],
    [4, 3, 1, 1, 3, 2, -1],
    [4, 0, 1, -1, -1, -1, -1],
    [2, 0, 3, -1, -1, -1, -1],
    [-1, -1, -1, -1, -1, -1, -1],
];

/// VTK: static `vtkTetraCellPCoords` table in `vtkTetra.cxx`.
const PARAMETRIC_COORDS: [f64; 12] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
