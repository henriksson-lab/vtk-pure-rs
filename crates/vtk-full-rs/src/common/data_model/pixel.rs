use crate::common::core::{
    math::{cross, distance2_between_points, dot, norm, normalize},
    IdList, Points, VtkIdType,
};

use super::{Cell, CellBaseApi, CellType, Line, Plane, Triangle};

/// Rust return bundle for VTK `vtkPixel::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 4],
}

/// VTK: `vtkPixel`.
#[derive(Debug)]
pub struct Pixel {
    cell: Cell,
    line: Line,
}

impl Pixel {
    /// VTK: `vtkPixel::New`.
    pub fn new() -> Self {
        let mut pixel = Self {
            cell: Cell::with_class_name("vtkPixel"),
            line: Line::new(),
        };
        pixel.cell.get_points_mut().set_number_of_points(4);
        pixel.cell.get_point_ids_mut().set_number_of_ids(4);
        for i in 0..4 {
            pixel.cell.get_points_mut().set_point(i, [0.0, 0.0, 0.0]);
            pixel.cell.get_point_ids_mut().set_id(i, 0);
        }
        pixel
    }

    /// VTK: `vtkPixel::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.cell.print_self();
        text.push_str("\nLine:\n");
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

    /// VTK: `vtkPixel::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Pixel as i32
    }

    /// VTK: `vtkPixel::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkPixel::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        4
    }

    /// VTK: `vtkPixel::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkPixel::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut Line {
        let verts = EDGES[edge_id as usize];
        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(0, self.cell.get_point_ids().get_id(verts[0]));
        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(1, self.cell.get_point_ids().get_id(verts[1]));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(0, self.cell.get_points().get_point(verts[0]));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(1, self.cell.get_points().get_point(verts[1]));
        &mut self.line
    }

    /// VTK: `vtkPixel::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkPixel::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.5, 0.5, 0.0])
    }

    /// VTK: `vtkPixel::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> PixelEvaluatePosition {
        let pt1 = self.cell.get_points().get_point(0);
        let pt2 = self.cell.get_points().get_point(1);
        let pt3 = self.cell.get_points().get_point(2);
        let n = Triangle::compute_normal(pt1, pt2, pt3);
        let cp = Plane::project_point(x, pt1, n);
        let p21 = [pt2[0] - pt1[0], pt2[1] - pt1[1], pt2[2] - pt1[2]];
        let p31 = [pt3[0] - pt1[0], pt3[1] - pt1[1], pt3[2] - pt1[2]];
        let p = [x[0] - pt1[0], x[1] - pt1[1], x[2] - pt1[2]];
        let mut l21 = norm(&p21);
        let mut l31 = norm(&p31);
        if l21 == 0.0 {
            l21 = 1.0;
        }
        if l31 == 0.0 {
            l31 = 1.0;
        }
        let pcoords = [dot(p21, p) / (l21 * l21), dot(p31, p) / (l31 * l31), 0.0];
        let weights = Self::interpolation_functions(pcoords);
        if pcoords[0] >= 0.0 && pcoords[0] <= 1.0 && pcoords[1] >= 0.0 && pcoords[1] <= 1.0 {
            let dist2 = distance2_between_points(cp, x);
            if let Some(closest_point) = closest_point {
                *closest_point = cp;
            }
            PixelEvaluatePosition {
                inside: 1,
                sub_id: 0,
                pcoords,
                dist2,
                weights,
            }
        } else {
            let pc = [pcoords[0].clamp(0.0, 1.0), pcoords[1].clamp(0.0, 1.0), 0.0];
            let (closest, _) = self.evaluate_location(0, pc);
            let dist2 = distance2_between_points(closest, x);
            if let Some(closest_point) = closest_point {
                *closest_point = closest;
            }
            PixelEvaluatePosition {
                inside: 0,
                sub_id: 0,
                pcoords,
                dist2,
                weights,
            }
        }
    }

    /// VTK: `vtkPixel::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 4]) {
        let pt1 = self.cell.get_points().get_point(0);
        let pt2 = self.cell.get_points().get_point(1);
        let pt3 = self.cell.get_points().get_point(2);
        let mut x = [0.0; 3];
        for i in 0..3 {
            x[i] = pt1[i] + pcoords[0] * (pt2[i] - pt1[i]) + pcoords[1] * (pt3[i] - pt1[i]);
        }
        (x, Self::interpolation_functions(pcoords))
    }

    /// VTK: `vtkPixel::ComputeNormal`.
    pub fn compute_normal(&self) -> (i32, [f64; 3]) {
        let p0 = self.cell.get_points().get_point(0);
        let mut p1 = self.cell.get_points().get_point(1);
        let mut p2 = self.cell.get_points().get_point(2);
        for i in 0..3 {
            p1[i] -= p0[i];
            p2[i] -= p0[i];
        }
        let mut n = cross(p1, p2);
        if n.iter().all(|v| v.abs() < f64::EPSILON) {
            return (-1, n);
        }
        normalize(&mut n);
        (
            ((n[1].abs() > 0.5) as i32) + ((n[2].abs() > 0.5) as i32) * 2,
            n,
        )
    }

    /// VTK: `vtkPixel::Inflate`.
    pub fn inflate(&mut self, dist: f64) -> i32 {
        let p0 = self.cell.get_points().get_point(0);
        let p3 = self.cell.get_points().get_point(3);
        let normal_direction = ((p3[0] - p0[0]).abs() < f64::EPSILON) as i32
            | (((p3[1] - p0[1]).abs() < f64::EPSILON) as i32) << 1
            | (((p3[2] - p0[2]).abs() < f64::EPSILON) as i32) << 2;
        if normal_direction == 0x7 {
            return 0;
        }
        let degenerate_pixel_direction = if (normal_direction - 1) & normal_direction != 0 {
            match !normal_direction & 0x7 {
                1 => 0,
                2 => 1,
                4 => 2,
                _ => -1,
            }
        } else {
            -1
        };

        for index in 0..4 {
            let mut point = self.cell.get_points().get_point(index);
            match normal_direction {
                1 => {
                    point[1] += dist * if index % 2 != 0 { 1.0 } else { -1.0 };
                    point[2] += dist * if index / 2 != 0 { 1.0 } else { -1.0 };
                }
                2 => {
                    point[0] += dist * if index % 2 != 0 { 1.0 } else { -1.0 };
                    point[2] += dist * if index / 2 != 0 { 1.0 } else { -1.0 };
                }
                4 => {
                    point[0] += dist * if index % 2 != 0 { 1.0 } else { -1.0 };
                    point[1] += dist * if index / 2 != 0 { 1.0 } else { -1.0 };
                }
                _ => {
                    point[degenerate_pixel_direction as usize] +=
                        dist * if index % 2 != 0 { 1.0 } else { -1.0 };
                }
            }
            self.cell.get_points_mut().set_point(index, point);
        }
        1
    }

    /// VTK: `vtkPixel::ComputeBoundingSphere`.
    pub fn compute_bounding_sphere(&self) -> ([f64; 3], f64) {
        let p0 = self.cell.get_points().get_point(0);
        let p3 = self.cell.get_points().get_point(3);
        let center = [
            0.5 * (p0[0] + p3[0]),
            0.5 * (p0[1] + p3[1]),
            0.5 * (p0[2] + p3[2]),
        ];
        (center, distance2_between_points(center, p0))
    }

    /// VTK: `vtkPixel::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let t1 = pcoords[0] - pcoords[1];
        let t2 = 1.0 - pcoords[0] - pcoords[1];
        pts.set_number_of_ids(2);
        let edge = if t1 >= 0.0 && t2 >= 0.0 {
            [0, 1]
        } else if t1 >= 0.0 && t2 < 0.0 {
            [1, 3]
        } else if t1 < 0.0 && t2 < 0.0 {
            [3, 2]
        } else {
            [2, 0]
        };
        pts.set_id(0, self.cell.get_point_ids().get_id(edge[0]));
        pts.set_id(1, self.cell.get_point_ids().get_id(edge[1]));
        (pcoords[0] >= 0.0 && pcoords[0] <= 1.0 && pcoords[1] >= 0.0 && pcoords[1] <= 1.0) as i32
    }

    /// VTK: `vtkPixel::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(6);
        let ids = if index % 2 != 0 {
            [0, 1, 2, 1, 3, 2]
        } else {
            [0, 1, 3, 0, 3, 2]
        };
        for (i, id) in ids.into_iter().enumerate() {
            pt_ids.set_id(i as VtkIdType, id);
        }
        1
    }

    /// VTK: `vtkPixel::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 4] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        [
            rm * sm,
            pcoords[0] * sm,
            rm * pcoords[1],
            pcoords[0] * pcoords[1],
        ]
    }

    /// VTK: `vtkPixel::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        weights[..4].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkPixel::InterpolationDerivs`.
    pub fn interpolation_derivs(pcoords: [f64; 3]) -> [f64; 8] {
        let rm = 1.0 - pcoords[0];
        let sm = 1.0 - pcoords[1];
        [
            -sm,
            sm,
            -pcoords[1],
            pcoords[1],
            -rm,
            -pcoords[0],
            rm,
            pcoords[0],
        ]
    }

    /// VTK: `vtkPixel::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        derivs[..8].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkPixel::Derivatives`.
    pub fn derivatives(
        &self,
        _sub_id: i32,
        pcoords: [f64; 3],
        values: &[f64],
        dim: i32,
        derivs: &mut [f64],
    ) {
        let function_derivs = Self::interpolation_derivs(pcoords);
        let x0 = self.cell.get_points().get_point(0);
        let x3 = self.cell.get_points().get_point(3);
        let spacing = [x3[0] - x0[0], x3[1] - x0[1], x3[2] - x0[2]];
        let (plane, idx) = if spacing[0] > spacing[2] && spacing[1] > spacing[2] {
            (2, [0, 1])
        } else if spacing[0] > spacing[1] && spacing[2] > spacing[1] {
            (1, [0, 2])
        } else {
            (0, [1, 2])
        };
        for k in 0..dim as usize {
            let mut jj = 0;
            for j in 0..3 {
                let sum = if j == plane {
                    0.0
                } else {
                    let mut sum = 0.0;
                    for i in 0..4 {
                        sum += function_derivs[4 * jj + i] * values[dim as usize * i + k];
                    }
                    let scaled = sum / spacing[idx[jj]];
                    jj += 1;
                    scaled
                };
                derivs[3 * k + j] = sum;
            }
        }
    }

    /// VTK: `vtkPixel::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 12] {
        &PIXEL_CELL_PCOORDS
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }
}

impl CellBaseApi for Pixel {
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

static EDGES: [[VtkIdType; 2]; 4] = [[0, 1], [1, 3], [2, 3], [0, 2]];

static PIXEL_CELL_PCOORDS: [f64; 12] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0];
