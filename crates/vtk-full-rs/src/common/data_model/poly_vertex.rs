use crate::common::core::{math::distance2_between_points, IdList, Points, VtkIdType};

use super::{Cell, CellBaseApi, CellType, Vertex};

/// Rust return bundle for VTK `vtkPolyVertex::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct PolyVertexEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: Vec<f64>,
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkPolyVertex::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolyVertexIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// VTK: `vtkPolyVertex`.
#[derive(Debug)]
pub struct PolyVertex {
    cell: Cell,
    vertex: Vertex,
}

impl PolyVertex {
    /// VTK: `vtkPolyVertex::New`.
    pub fn new() -> Self {
        Self {
            cell: Cell::with_class_name("vtkPolyVertex"),
            vertex: Vertex::new(),
        }
    }

    /// VTK: `vtkPolyVertex::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell.print_self();
        text.push_str("Vertex:\n");
        text.push_str(&self.vertex.print_self());
        text
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.cell.get_m_time().max(self.vertex.get_m_time())
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

    /// VTK: `vtkPolyVertex::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::PolyVertex as i32
    }

    /// VTK: `vtkPolyVertex::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyVertex::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyVertex::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyVertex::GetEdge`.
    pub fn get_edge(&self, _edge_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkPolyVertex::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkPolyVertex::IsPrimaryCell`.
    pub fn is_primary_cell(&self) -> i32 {
        0
    }

    /// VTK: `vtkPolyVertex::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> PolyVertexEvaluatePosition {
        let num_pts = self.cell.get_points().get_number_of_points();
        let wants_closest_point = closest_point.is_some();
        let mut closest_point = closest_point;
        let mut min_dist2 = f64::MAX;
        let mut sub_id = 0;
        let mut closest = None;

        for i in 0..num_pts {
            let point = self.cell.get_points().get_point(i);
            let dist2 = distance2_between_points(point, x);
            if dist2 < min_dist2 {
                if let Some(out) = closest_point.as_deref_mut() {
                    *out = point;
                }
                closest = Some(point);
                min_dist2 = dist2;
                sub_id = i as i32;
            }
        }

        let mut weights = vec![0.0; num_pts.max(0) as usize];
        if num_pts > 0 {
            weights[sub_id as usize] = 1.0;
        }

        let inside = (min_dist2 == 0.0) as i32;
        PolyVertexEvaluatePosition {
            inside,
            sub_id,
            pcoords: [if inside != 0 { 0.0 } else { -1.0 }, -1.0, -1.0],
            dist2: min_dist2,
            weights,
            closest_point: if wants_closest_point { closest } else { None },
        }
    }

    /// VTK: `vtkPolyVertex::EvaluateLocation`.
    pub fn evaluate_location(&self, sub_id: i32, _pcoords: [f64; 3]) -> ([f64; 3], Vec<f64>) {
        let point = self.cell.get_points().get_point(sub_id as VtkIdType);
        let mut weights = vec![0.0; self.cell.get_number_of_points().max(0) as usize];
        weights[sub_id as usize] = 1.0;
        (point, weights)
    }

    /// VTK: `vtkPolyVertex::CellBoundary`.
    pub fn cell_boundary(&self, sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        pts.set_number_of_ids(1);
        pts.set_id(0, self.cell.get_point_ids().get_id(sub_id as VtkIdType));
        (pcoords[0] == 0.0) as i32
    }

    /// VTK: `vtkPolyVertex::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> PolyVertexIntersectWithLine {
        let num_pts = self.cell.get_points().get_number_of_points();
        let mut last_miss = None;
        for sub_id in 0..num_pts {
            self.vertex
                .cell_mut()
                .get_points_mut()
                .set_point(0, self.cell.get_points().get_point(sub_id));
            let hit = self.vertex.intersect_with_line(p1, p2, tol);
            if hit.intersection != 0 {
                return PolyVertexIntersectWithLine {
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
        PolyVertexIntersectWithLine {
            intersection: 0,
            t,
            x,
            pcoords,
            sub_id: num_pts as i32,
        }
    }

    /// VTK: `vtkPolyVertex::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        let num_pts = self.cell.get_points().get_number_of_points();
        pt_ids.set_number_of_ids(num_pts);
        for i in 0..num_pts {
            pt_ids.set_id(i, i);
        }
        1
    }

    /// VTK: `vtkPolyVertex::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        _pcoords: [f64; 3],
        _values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let dim = dim.max(0) as usize;
        let required = if dim == 0 { 0 } else { (dim - 1) * dim + 3 };
        assert!(
            derivs.len() >= required,
            "vtkPolyVertex::Derivatives derivs slice too short"
        );
        for i in 0..dim {
            let idx = i * dim;
            derivs[idx] = 0.0;
            derivs[idx + 1] = 0.0;
            derivs[idx + 2] = 0.0;
        }
    }

    /// VTK: `vtkPolyVertex::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (
            (self.cell.get_points().get_number_of_points() / 2) as i32,
            [0.5, 0.5, 0.5],
        )
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }
}

impl Default for PolyVertex {
    fn default() -> Self {
        Self::new()
    }
}

impl CellBaseApi for PolyVertex {
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
