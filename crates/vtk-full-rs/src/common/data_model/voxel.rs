use crate::common::core::{math::distance2_between_points, IdList, Points, VtkIdType};

use super::{Cell, Cell3D, Cell3DApi, CellBaseApi, CellType, Line, Pixel};

/// Rust return bundle for VTK `vtkVoxel::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoxelEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 8],
}

/// VTK: `vtkVoxel`.
#[derive(Debug)]
pub struct Voxel {
    cell_3d: Cell3D,
    line: Option<Line>,
    pixel: Option<Pixel>,
}

impl Voxel {
    /// VTK: `vtkVoxel::NumberOfPoints`.
    pub const NUMBER_OF_POINTS: VtkIdType = 8;
    /// VTK: `vtkVoxel::NumberOfEdges`.
    pub const NUMBER_OF_EDGES: VtkIdType = 12;
    /// VTK: `vtkVoxel::NumberOfFaces`.
    pub const NUMBER_OF_FACES: VtkIdType = 6;
    /// VTK: `vtkVoxel::MaximumFaceSize`.
    pub const MAXIMUM_FACE_SIZE: VtkIdType = 4;
    /// VTK: `vtkVoxel::MaximumValence`.
    pub const MAXIMUM_VALENCE: VtkIdType = 3;

    /// VTK: `vtkVoxel::New`.
    pub fn new() -> Self {
        let mut voxel = Self {
            cell_3d: Cell3D::with_class_name("vtkVoxel"),
            line: None,
            pixel: None,
        };
        voxel
            .cell_3d
            .cell_mut()
            .get_points_mut()
            .set_number_of_points(Self::NUMBER_OF_POINTS);
        voxel
            .cell_3d
            .cell_mut()
            .get_point_ids_mut()
            .set_number_of_ids(Self::NUMBER_OF_POINTS);
        for i in 0..Self::NUMBER_OF_POINTS {
            voxel
                .cell_3d
                .cell_mut()
                .get_points_mut()
                .set_point(i, [0.0, 0.0, 0.0]);
            voxel.cell_3d.cell_mut().get_point_ids_mut().set_id(i, 0);
        }
        voxel
    }

    /// VTK: `vtkVoxel::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell_3d.print_self();
        text.push_str("\nLine:\n");
        text.push_str(
            self.line
                .as_ref()
                .map(Line::print_self)
                .as_deref()
                .unwrap_or("None"),
        );
        text.push_str("\nPixel:\n");
        text.push_str(
            self.pixel
                .as_ref()
                .map(Pixel::print_self)
                .as_deref()
                .unwrap_or("None"),
        );
        text
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell_3d.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        let mut mtime = self.cell_3d.get_m_time();
        if let Some(line) = &self.line {
            mtime = mtime.max(line.get_m_time());
        }
        if let Some(pixel) = &self.pixel {
            mtime = mtime.max(pixel.get_m_time());
        }
        mtime
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

    /// VTK: `vtkVoxel::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Voxel as i32
    }

    /// VTK: `vtkVoxel::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        Self::NUMBER_OF_EDGES as i32
    }

    /// VTK: `vtkVoxel::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        Self::NUMBER_OF_FACES as i32
    }

    /// VTK: `vtkVoxel::GetCentroid`.
    pub fn get_centroid(&self) -> (bool, [f64; 3]) {
        Self::compute_centroid(self.get_points(), None)
    }

    /// VTK: `vtkVoxel::ComputeCentroid`.
    pub fn compute_centroid(points: &Points, point_ids: Option<&[VtkIdType]>) -> (bool, [f64; 3]) {
        let p0_id = point_ids.map_or(0, |ids| ids[0]);
        let p7_id = point_ids.map_or(7, |ids| ids[7]);
        let mut centroid = points.get_point(p0_id);
        let p = points.get_point(p7_id);
        for i in 0..3 {
            centroid[i] = 0.5 * (centroid[i] + p[i]);
        }
        (true, centroid)
    }

    /// VTK: `vtkVoxel::IsInsideOut`.
    pub fn is_inside_out(&self) -> bool {
        let pt1 = self.get_points().get_point(0);
        let pt2 = self.get_points().get_point(7);
        (pt2[0] - pt1[0]) * (pt2[1] - pt1[1]) * (pt2[2] - pt1[2]) < 0.0
    }

    /// VTK: `vtkVoxel::ComputeBoundingSphere`.
    pub fn compute_bounding_sphere(&self) -> ([f64; 3], f64) {
        let p0 = self.get_points().get_point(0);
        let p7 = self.get_points().get_point(7);
        let center = [
            0.5 * (p0[0] + p7[0]),
            0.5 * (p0[1] + p7[1]),
            0.5 * (p0[2] + p7[2]),
        ];
        (center, distance2_between_points(center, p0))
    }

    /// VTK: `vtkVoxel::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> VoxelEvaluatePosition {
        let pt1 = self.get_points().get_point(0);
        let pt2 = self.get_points().get_point(1);
        let pt3 = self.get_points().get_point(2);
        let pt4 = self.get_points().get_point(4);
        let pcoords = [
            (x[0] - pt1[0]) / (pt2[0] - pt1[0]),
            (x[1] - pt1[1]) / (pt3[1] - pt1[1]),
            (x[2] - pt1[2]) / (pt4[2] - pt1[2]),
        ];
        if pcoords.iter().all(|p| *p >= 0.0 && *p <= 1.0) {
            if let Some(closest_point) = closest_point {
                *closest_point = x;
            }
            VoxelEvaluatePosition {
                inside: 1,
                sub_id: 0,
                pcoords,
                dist2: 0.0,
                weights: Self::interpolation_functions(pcoords),
            }
        } else {
            let pc = [
                pcoords[0].clamp(0.0, 1.0),
                pcoords[1].clamp(0.0, 1.0),
                pcoords[2].clamp(0.0, 1.0),
            ];
            let (closest, _) = self.evaluate_location(0, pc);
            let dist2 = distance2_between_points(closest, x);
            if let Some(closest_point) = closest_point {
                *closest_point = closest;
            }
            VoxelEvaluatePosition {
                inside: 0,
                sub_id: 0,
                pcoords,
                dist2,
                weights: Self::interpolation_functions(pcoords),
            }
        }
    }

    /// VTK: `vtkVoxel::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 8]) {
        let pt1 = self.get_points().get_point(0);
        let pt2 = self.get_points().get_point(1);
        let pt3 = self.get_points().get_point(2);
        let pt4 = self.get_points().get_point(4);
        let mut x = [0.0; 3];
        for i in 0..3 {
            x[i] = pt1[i]
                + pcoords[0] * (pt2[i] - pt1[i])
                + pcoords[1] * (pt3[i] - pt1[i])
                + pcoords[2] * (pt4[i] - pt1[i]);
        }
        (x, Self::interpolation_functions(pcoords))
    }

    /// VTK: `vtkVoxel::Inflate`.
    pub fn inflate(&mut self, dist: f64) -> i32 {
        for index in 0..Self::NUMBER_OF_POINTS {
            let mut point = self.get_points().get_point(index);
            point[0] += dist * if index % 2 != 0 { 1.0 } else { -1.0 };
            point[1] += dist * if (index / 2) % 2 != 0 { 1.0 } else { -1.0 };
            point[2] += dist * if index / 4 != 0 { 1.0 } else { -1.0 };
            self.cell_3d
                .cell_mut()
                .get_points_mut()
                .set_point(index, point);
        }
        1
    }

    /// VTK: `vtkVoxel::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 8] {
        let r = pcoords[0];
        let s = pcoords[1];
        let t = pcoords[2];
        let rm = 1.0 - r;
        let sm = 1.0 - s;
        let tm = 1.0 - t;
        [
            rm * sm * tm,
            r * sm * tm,
            rm * s * tm,
            r * s * tm,
            rm * sm * t,
            r * sm * t,
            rm * s * t,
            r * s * t,
        ]
    }

    /// VTK: `vtkVoxel::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        weights[..8].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkVoxel::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 24] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        let tm = 1.0 - pcoords[2];
        [
            -sm * tm,
            sm * tm,
            -pcoords[1] * tm,
            pcoords[1] * tm,
            -sm * pcoords[2],
            sm * pcoords[2],
            -pcoords[1] * pcoords[2],
            pcoords[1] * pcoords[2],
            -rm * tm,
            -pcoords[0] * tm,
            rm * tm,
            pcoords[0] * tm,
            -rm * pcoords[2],
            -pcoords[0] * pcoords[2],
            rm * pcoords[2],
            pcoords[0] * pcoords[2],
            -rm * sm,
            -pcoords[0] * sm,
            -rm * pcoords[1],
            -pcoords[0] * pcoords[1],
            rm * sm,
            pcoords[0] * sm,
            rm * pcoords[1],
            pcoords[0] * pcoords[1],
        ]
    }

    /// VTK: `vtkVoxel::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        derivs[..24].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkVoxel::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let t1 = pcoords[0] - pcoords[1];
        let t2 = 1.0 - pcoords[0] - pcoords[1];
        let t3 = pcoords[1] - pcoords[2];
        let t4 = 1.0 - pcoords[1] - pcoords[2];
        let t5 = pcoords[2] - pcoords[0];
        let t6 = 1.0 - pcoords[2] - pcoords[0];
        pts.set_number_of_ids(Self::MAXIMUM_FACE_SIZE);
        let face = if t3 >= 0.0 && t4 >= 0.0 && t5 < 0.0 && t6 >= 0.0 {
            [0, 1, 3, 2]
        } else if t1 >= 0.0 && t2 < 0.0 && t5 < 0.0 && t6 < 0.0 {
            [1, 3, 7, 5]
        } else if t1 >= 0.0 && t2 >= 0.0 && t3 < 0.0 && t4 >= 0.0 {
            [0, 1, 5, 4]
        } else if t3 < 0.0 && t4 < 0.0 && t5 >= 0.0 && t6 < 0.0 {
            [4, 5, 7, 6]
        } else if t1 < 0.0 && t2 >= 0.0 && t5 >= 0.0 && t6 >= 0.0 {
            [0, 4, 6, 2]
        } else {
            [3, 2, 6, 7]
        };
        for (i, local_id) in face.into_iter().enumerate() {
            pts.set_id(i as VtkIdType, self.get_point_ids().get_id(local_id));
        }
        pcoords.iter().all(|p| *p >= 0.0 && *p <= 1.0) as i32
    }

    /// VTK: `vtkVoxel::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkVoxel::GetFaceArray`.
    pub fn get_face_array(face_id: VtkIdType) -> &'static [VtkIdType; 5] {
        &FACES[face_id as usize]
    }

    /// VTK: `vtkVoxel::GetEdge`.
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
        let line = self.line.get_or_insert_with(Line::new);
        for i in 0..2 {
            line.cell_mut()
                .get_point_ids_mut()
                .set_id(i as VtkIdType, point_ids[i]);
            line.cell_mut()
                .get_points_mut()
                .set_point(i as VtkIdType, points[i]);
        }
        line
    }

    /// VTK: `vtkVoxel::GetFace`.
    pub fn get_face(&mut self, face_id: i32) -> &mut Pixel {
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
        let pixel = self.pixel.get_or_insert_with(Pixel::new);
        for i in 0..Self::MAXIMUM_FACE_SIZE as usize {
            pixel
                .cell_mut()
                .get_point_ids_mut()
                .set_id(i as VtkIdType, point_ids[i]);
            pixel
                .cell_mut()
                .get_points_mut()
                .set_point(i as VtkIdType, points[i]);
        }
        pixel
    }

    /// VTK: `vtkVoxel::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(5 * 4);
        let ids = if index % 2 != 0 {
            [0, 1, 2, 4, 1, 4, 5, 7, 1, 4, 7, 2, 1, 2, 7, 3, 2, 7, 6, 4]
        } else {
            [3, 1, 5, 0, 0, 3, 2, 6, 3, 5, 7, 6, 0, 6, 4, 5, 0, 3, 6, 5]
        };
        for (i, id) in ids.into_iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, id);
        }
        1
    }

    /// VTK: `vtkVoxel::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let function_derivs = Self::interpolation_derivs(pcoords);
        let x0 = self.get_points().get_point(0);
        let x1 = self.get_points().get_point(1);
        let x2 = self.get_points().get_point(2);
        let x4 = self.get_points().get_point(4);
        let spacing = [x1[0] - x0[0], x2[1] - x0[1], x4[2] - x0[2]];
        for k in 0..dim as usize {
            for j in 0..3 {
                let mut sum = 0.0;
                for i in 0..Self::NUMBER_OF_POINTS as usize {
                    sum += function_derivs[8 * j + i] * values[dim as usize * i + k];
                }
                derivs[3 * k + j] = sum / spacing[j];
            }
        }
    }

    /// VTK: `vtkVoxel::GetPointToOneRingPoints`.
    pub fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_one_ring_points_array(point_id),
        )
    }

    /// VTK: `vtkVoxel::GetPointToIncidentFaces`.
    pub fn get_point_to_incident_faces(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_faces_array(point_id),
        )
    }

    /// VTK: `vtkVoxel::GetPointToIncidentEdges`.
    pub fn get_point_to_incident_edges(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 3]) {
        (
            Self::MAXIMUM_VALENCE,
            Self::get_point_to_incident_edges_array(point_id),
        )
    }

    /// VTK: `vtkVoxel::GetFaceToAdjacentFaces`.
    pub fn get_face_to_adjacent_faces(
        &self,
        face_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType; 4]) {
        (
            Self::MAXIMUM_FACE_SIZE,
            Self::get_face_to_adjacent_faces_array(face_id),
        )
    }

    /// VTK: `vtkVoxel::GetEdgeToAdjacentFaces`.
    pub fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_to_adjacent_faces_array(edge_id)
    }

    /// VTK: `vtkVoxel::GetEdgeToAdjacentFacesArray`.
    pub fn get_edge_to_adjacent_faces_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGE_TO_ADJACENT_FACES[edge_id as usize]
    }

    /// VTK: `vtkVoxel::GetFaceToAdjacentFacesArray`.
    pub fn get_face_to_adjacent_faces_array(face_id: VtkIdType) -> &'static [VtkIdType; 4] {
        &FACE_TO_ADJACENT_FACES[face_id as usize]
    }

    /// VTK: `vtkVoxel::GetPointToIncidentEdgesArray`.
    pub fn get_point_to_incident_edges_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_INCIDENT_EDGES[point_id as usize]
    }

    /// VTK: `vtkVoxel::GetPointToIncidentFacesArray`.
    pub fn get_point_to_incident_faces_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_INCIDENT_FACES[point_id as usize]
    }

    /// VTK: `vtkVoxel::GetPointToOneRingPointsArray`.
    pub fn get_point_to_one_ring_points_array(point_id: VtkIdType) -> &'static [VtkIdType; 3] {
        &POINT_TO_ONE_RING_POINTS[point_id as usize]
    }

    /// VTK: `vtkVoxel::GetEdgePoints`.
    pub fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        Self::get_edge_array(edge_id)
    }

    /// VTK: `vtkVoxel::GetFacePoints`.
    pub fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType; 5]) {
        (Self::MAXIMUM_FACE_SIZE, Self::get_face_array(face_id))
    }

    /// VTK: `vtkVoxel::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 24] {
        &VOXEL_CELL_PCOORDS
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

impl CellBaseApi for Voxel {
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

impl Cell3DApi for Voxel {
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

const EDGES: [[VtkIdType; 2]; Voxel::NUMBER_OF_EDGES as usize] = [
    [0, 1],
    [1, 3],
    [2, 3],
    [0, 2],
    [4, 5],
    [5, 7],
    [6, 7],
    [4, 6],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

const FACES: [[VtkIdType; 5]; Voxel::NUMBER_OF_FACES as usize] = [
    [2, 0, 6, 4, -1],
    [1, 3, 5, 7, -1],
    [0, 1, 4, 5, -1],
    [3, 2, 7, 6, -1],
    [1, 0, 3, 2, -1],
    [4, 5, 6, 7, -1],
];

const EDGE_TO_ADJACENT_FACES: [[VtkIdType; 2]; Voxel::NUMBER_OF_EDGES as usize] = [
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

const FACE_TO_ADJACENT_FACES: [[VtkIdType; 4]; Voxel::NUMBER_OF_FACES as usize] = [
    [5, 3, 4, 2],
    [4, 3, 5, 2],
    [4, 1, 5, 0],
    [4, 0, 5, 1],
    [2, 0, 3, 1],
    [2, 1, 3, 0],
];

const POINT_TO_INCIDENT_EDGES: [[VtkIdType; 3]; Voxel::NUMBER_OF_POINTS as usize] = [
    [0, 8, 3],
    [0, 1, 9],
    [2, 3, 10],
    [1, 2, 11],
    [4, 7, 8],
    [4, 9, 5],
    [6, 10, 7],
    [5, 11, 6],
];

const POINT_TO_INCIDENT_FACES: [[VtkIdType; 3]; Voxel::NUMBER_OF_POINTS as usize] = [
    [2, 0, 4],
    [4, 1, 2],
    [4, 0, 3],
    [4, 3, 1],
    [5, 0, 2],
    [2, 1, 5],
    [3, 0, 5],
    [1, 3, 5],
];

const POINT_TO_ONE_RING_POINTS: [[VtkIdType; 3]; Voxel::NUMBER_OF_POINTS as usize] = [
    [1, 4, 2],
    [0, 3, 5],
    [3, 0, 6],
    [1, 2, 7],
    [5, 6, 0],
    [4, 1, 7],
    [7, 2, 4],
    [5, 3, 6],
];

const VOXEL_CELL_PCOORDS: [f64; 24] = [
    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0,
    1.0, 1.0, 1.0, 1.0, 1.0,
];
