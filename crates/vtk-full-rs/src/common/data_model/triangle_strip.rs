use crate::common::core::{IdList, Points, VtkIdType, VTK_DOUBLE_MAX};

use super::{Cell, CellArray, CellBaseApi, CellType, Line, Triangle};

const TRIANGLE_STRIP_IDX: [[VtkIdType; 3]; 2] = [[0, 1, 2], [1, 0, 2]];

/// Rust return bundle for VTK `vtkTriangleStrip::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangleStripEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: Vec<f64>,
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkTriangleStrip::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleStripIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkTriangleStrip`.
#[derive(Debug)]
pub struct TriangleStrip {
    cell: Cell,
    line: Line,
    triangle: Triangle,
}

impl TriangleStrip {
    /// VTK: `vtkTriangleStrip::New`.
    pub fn new() -> Self {
        Self {
            cell: Cell::with_class_name("vtkTriangleStrip"),
            line: Line::new(),
            triangle: Triangle::new(),
        }
    }

    /// VTK: `vtkTriangleStrip::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell.print_self();
        text.push_str("Line:\n");
        text.push_str(&self.line.print_self());
        text.push_str("Triangle:\n");
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

    /// VTK: `vtkTriangleStrip::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::TriangleStrip as i32
    }

    /// VTK: `vtkTriangleStrip::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkTriangleStrip::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        self.get_number_of_points() as i32
    }

    /// VTK: `vtkTriangleStrip::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkTriangleStrip::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkTriangleStrip::IsPrimaryCell`.
    pub fn is_primary_cell(&self) -> i32 {
        0
    }

    /// VTK: `vtkTriangleStrip::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut Line {
        let num_points = self.get_number_of_points();
        let edge_id = edge_id as VtkIdType;
        let (id1, id2) = if edge_id == 0 {
            (0, 1)
        } else if edge_id == num_points - 1 {
            (edge_id - 1, edge_id)
        } else {
            (edge_id - 1, edge_id + 1)
        };

        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(0, self.cell.get_point_ids().get_id(id1));
        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(1, self.cell.get_point_ids().get_id(id2));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(0, self.cell.get_points().get_point(id1));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(1, self.cell.get_points().get_point(id2));

        &mut self.line
    }

    /// VTK: `vtkTriangleStrip::EvaluatePosition`.
    pub fn evaluate_position(
        &mut self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> TriangleStripEvaluatePosition {
        let num_points = self.cell.get_points().get_number_of_points();
        let num_tris = num_points.saturating_sub(2);
        let wants_closest_point = closest_point.is_some();
        let mut closest_point = closest_point;
        let mut return_status = 0;
        let mut sub_id = 0;
        let mut pcoords = [0.0, 0.0, 0.0];
        let mut min_dist2 = VTK_DOUBLE_MAX;
        let mut active_weights = [0.0; 3];
        let mut closest = None;

        for i in 0..num_tris {
            self.set_triangle_points(i);
            let status = self.triangle.evaluate_position(x, true);
            if status.inside != -1
                && (status.dist2 < min_dist2 || (status.dist2 == min_dist2 && return_status == 0))
            {
                return_status = status.inside;
                sub_id = i as i32;
                pcoords[0] = status.pcoords[0];
                pcoords[1] = status.pcoords[1];
                min_dist2 = status.dist2;
                active_weights = status.weights;
                if let Some(point) = status.closest_point {
                    if let Some(out) = closest_point.as_deref_mut() {
                        *out = point;
                    }
                    closest = Some(point);
                }
            }
        }

        let mut weights = vec![0.0; num_points.max(0) as usize];
        if num_tris > 0 {
            let sub_id = sub_id as usize;
            weights[sub_id] = active_weights[0];
            weights[sub_id + 1] = active_weights[1];
            weights[sub_id + 2] = active_weights[2];
        }

        TriangleStripEvaluatePosition {
            inside: return_status,
            sub_id,
            pcoords,
            dist2: min_dist2,
            weights,
            closest_point: if wants_closest_point { closest } else { None },
        }
    }

    /// VTK: `vtkTriangleStrip::EvaluateLocation`.
    pub fn evaluate_location(&self, sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], Vec<f64>) {
        let order = (sub_id % 2) as usize;
        let sub_id = sub_id as VtkIdType;
        let u3 = 1.0 - pcoords[0] - pcoords[1];
        let mut weights = vec![0.0; self.cell.get_points().get_number_of_points().max(0) as usize];
        weights[sub_id as usize] = u3;
        weights[sub_id as usize + 1] = pcoords[0];
        weights[sub_id as usize + 2] = pcoords[1];

        let mut x = [0.0; 3];
        for j in 0..3 {
            let point = self
                .cell
                .get_points()
                .get_point(sub_id + TRIANGLE_STRIP_IDX[order][j]);
            let weight = weights[sub_id as usize + j];
            for i in 0..3 {
                x[i] += point[i] * weight;
            }
        }
        (x, weights)
    }

    /// VTK: `vtkTriangleStrip::CellBoundary`.
    pub fn cell_boundary(&mut self, sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let order = (sub_id % 2) as usize;
        let sub_id = sub_id as VtkIdType;
        for i in 0..3 {
            self.triangle.cell_mut().get_point_ids_mut().set_id(
                i as VtkIdType,
                self.cell
                    .get_point_ids()
                    .get_id(sub_id + TRIANGLE_STRIP_IDX[order][i]),
            );
        }
        self.triangle.cell_boundary(0, pcoords, pts)
    }

    /// VTK: `vtkTriangleStrip::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> TriangleStripIntersectWithLine {
        let num_tris = self
            .cell
            .get_points()
            .get_number_of_points()
            .saturating_sub(2);
        let mut last_miss = None;
        for sub_id in 0..num_tris {
            self.set_triangle_points(sub_id);
            let hit = self.triangle.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                return TriangleStripIntersectWithLine {
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
        TriangleStripIntersectWithLine {
            intersection: 0,
            t,
            x,
            pcoords,
            sub_id: num_tris as i32,
        }
    }

    /// VTK: `vtkTriangleStrip::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        let num_tris = self
            .cell
            .get_points()
            .get_number_of_points()
            .saturating_sub(2);
        pt_ids.set_number_of_ids(3 * num_tris);
        for sub_id in 0..num_tris {
            let order = (sub_id % 2) as usize;
            for i in 0..3 {
                pt_ids.set_id(
                    sub_id * 3 + i as VtkIdType,
                    sub_id + TRIANGLE_STRIP_IDX[order][i],
                );
            }
        }
        1
    }

    /// VTK: `vtkTriangleStrip::Derivatives`.
    pub fn derivatives(
        &mut self,
        sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let dim_usize = dim.max(0) as usize;
        let sub_id_usize = (sub_id.max(0) as usize) * dim_usize;
        assert!(
            values.len() >= sub_id_usize + 3 * dim_usize,
            "vtkTriangleStrip::Derivatives values slice too short"
        );
        self.set_triangle_points(sub_id as VtkIdType);
        self.triangle
            .derivatives(0, pcoords, &values[sub_id_usize..], dim, derivs);
    }

    /// VTK: `vtkTriangleStrip::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        let num_points = self.cell.get_points().get_number_of_points();
        let sub_id = if num_points > 1 {
            ((num_points - 2) / 2) as i32
        } else {
            0
        };
        (sub_id, [0.333333, 0.333333, 0.0])
    }

    /// VTK: `vtkTriangleStrip::DecomposeStrip`.
    pub fn decompose_strip(npts: i32, pts: &[VtkIdType], tris: &mut CellArray) {
        if npts < 2 {
            return;
        }
        let npts = npts as usize;
        assert!(
            pts.len() >= npts,
            "vtkTriangleStrip::DecomposeStrip point id slice shorter than npts"
        );

        let mut p1 = pts[0];
        let mut p2 = pts[1];
        for i in 0..npts.saturating_sub(2) {
            let p3 = pts[i + 2];
            tris.insert_next_cell_empty(3);
            if i % 2 != 0 {
                tris.insert_cell_point(p2);
                tris.insert_cell_point(p1);
                tris.insert_cell_point(p3);
            } else {
                tris.insert_cell_point(p1);
                tris.insert_cell_point(p2);
                tris.insert_cell_point(p3);
            }
            p1 = p2;
            p2 = p3;
        }
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }

    fn set_triangle_points(&mut self, sub_id: VtkIdType) {
        for i in 0..3 {
            self.triangle
                .cell_mut()
                .get_points_mut()
                .set_point(i, self.cell.get_points().get_point(sub_id + i));
        }
    }
}

impl Default for TriangleStrip {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for TriangleStrip {
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
