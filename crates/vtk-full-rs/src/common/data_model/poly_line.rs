use crate::common::core::{IdList, Points, VtkIdType, VTK_DOUBLE_MAX};

use super::{Cell, CellBaseApi, CellType, Line};

/// Rust return bundle for VTK `vtkPolyLine::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct PolyLineEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: Vec<f64>,
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkPolyLine::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolyLineIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkPolyLine`.
#[derive(Debug)]
pub struct PolyLine {
    cell: Cell,
    line: Line,
}

impl PolyLine {
    /// VTK: `vtkPolyLine::New`.
    pub fn new() -> Self {
        Self {
            cell: Cell::with_class_name("vtkPolyLine"),
            line: Line::new(),
        }
    }

    /// VTK: `vtkPolyLine::PrintSelf`.
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

    /// VTK: `vtkPolyLine::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::PolyLine as i32
    }

    /// VTK: `vtkPolyLine::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        1
    }

    /// VTK: `vtkPolyLine::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyLine::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyLine::GetEdge`.
    pub fn get_edge(&self, _edge_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkPolyLine::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkPolyLine::IsPrimaryCell`.
    pub fn is_primary_cell(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyLine::EvaluatePosition`.
    pub fn evaluate_position(
        &mut self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> PolyLineEvaluatePosition {
        let num_pts = self.cell.get_points().get_number_of_points();
        let wants_closest_point = closest_point.is_some();
        let mut closest_point = closest_point;
        let mut return_status = 0;
        let mut sub_id = -1;
        let mut pcoords = [0.0, 0.0, 0.0];
        let mut min_dist2 = VTK_DOUBLE_MAX;
        let mut closest_weights = [0.0, 0.0];
        let mut closest = None;

        for i in 0..num_pts.saturating_sub(1) {
            self.set_line_segment(i);
            let mut line_closest = [0.0; 3];
            let status = self.line.evaluate_position(x, Some(&mut line_closest));
            if status.inside != -1
                && (status.dist2 < min_dist2 || (status.dist2 == min_dist2 && return_status == 0))
            {
                return_status = status.inside;
                min_dist2 = status.dist2;
                sub_id = i as i32;
                pcoords[0] = status.pcoords[0];
                closest_weights = status.weights;
                if let Some(out) = closest_point.as_deref_mut() {
                    *out = line_closest;
                }
                closest = Some(line_closest);
            }
        }

        let mut weights = vec![0.0; num_pts.max(0) as usize];
        if sub_id >= 0 {
            let sub_id = sub_id as usize;
            weights[sub_id] = closest_weights[0];
            weights[sub_id + 1] = closest_weights[1];
        }

        PolyLineEvaluatePosition {
            inside: return_status,
            sub_id,
            pcoords,
            dist2: min_dist2,
            weights,
            closest_point: if wants_closest_point { closest } else { None },
        }
    }

    /// VTK: `vtkPolyLine::EvaluateLocation`.
    pub fn evaluate_location(&self, sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], Vec<f64>) {
        let sub_id = sub_id as VtkIdType;
        let a1 = self.cell.get_points().get_point(sub_id);
        let a2 = self.cell.get_points().get_point(sub_id + 1);
        let mut x = [0.0; 3];
        for i in 0..3 {
            x[i] = a1[i] + pcoords[0] * (a2[i] - a1[i]);
        }

        let mut weights = vec![0.0; self.cell.get_points().get_number_of_points().max(0) as usize];
        if sub_id >= 0 {
            weights[sub_id as usize] = 1.0 - pcoords[0];
            weights[sub_id as usize + 1] = pcoords[0];
        }
        (x, weights)
    }

    /// VTK: `vtkPolyLine::CellBoundary`.
    pub fn cell_boundary(&self, sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        pts.set_number_of_ids(1);
        let sub_id = sub_id as VtkIdType;
        if pcoords[0] >= 0.5 {
            pts.set_id(0, self.cell.get_point_ids().get_id(sub_id + 1));
            (pcoords[0] <= 1.0) as i32
        } else {
            pts.set_id(0, self.cell.get_point_ids().get_id(sub_id));
            (pcoords[0] >= 0.0) as i32
        }
    }

    /// VTK: `vtkPolyLine::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> PolyLineIntersectWithLine {
        let num_lines = self
            .cell
            .get_points()
            .get_number_of_points()
            .saturating_sub(1);
        let mut last_miss = None;
        for sub_id in 0..num_lines {
            self.set_line_segment(sub_id);
            let hit = self.line.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                return PolyLineIntersectWithLine {
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
        PolyLineIntersectWithLine {
            intersection: 0,
            t,
            x,
            pcoords,
            sub_id: num_lines as i32,
        }
    }

    /// VTK: `vtkPolyLine::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        let num_lines = self
            .cell
            .get_points()
            .get_number_of_points()
            .saturating_sub(1);
        pt_ids.set_number_of_ids(2 * num_lines);
        for sub_id in 0..num_lines {
            pt_ids.set_id(sub_id * 2, sub_id);
            pt_ids.set_id(sub_id * 2 + 1, sub_id + 1);
        }
        1
    }

    /// VTK: `vtkPolyLine::Derivatives`.
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
            values.len() >= sub_id_usize + 2 * dim_usize,
            "vtkPolyLine::Derivatives values slice too short"
        );
        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_number_of_ids(2);
        self.set_line_segment(sub_id as VtkIdType);
        self.line
            .derivatives(0, pcoords, &values[sub_id_usize..], dim, derivs);
    }

    /// VTK: `vtkPolyLine::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        let num_points = self.cell.get_points().get_number_of_points();
        let sub_id = if num_points > 0 {
            ((num_points - 1) / 2) as i32
        } else {
            0
        };
        (sub_id, [0.5, 0.0, 0.0])
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }

    fn set_line_segment(&mut self, sub_id: VtkIdType) {
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(0, self.cell.get_points().get_point(sub_id));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(1, self.cell.get_points().get_point(sub_id + 1));
    }
}

impl Default for PolyLine {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for PolyLine {
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
