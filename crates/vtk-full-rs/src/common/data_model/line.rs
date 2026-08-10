use crate::common::core::{
    math::{determinant2x2, distance2_between_points, dot, normalize, squared_norm},
    IdList, Points, VtkIdType, VTK_DOUBLE_MAX,
};

use super::{Cell, CellBaseApi, CellType, VTK_TOL};

/// VTK: `vtkLine::IntersectionType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LineIntersectionType {
    NoIntersect = 0,
    Intersect = 2,
    OnLine = 3,
}

/// VTK: `vtkLine::ToleranceType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LineToleranceType {
    Relative = 0,
    Absolute = 1,
    RelativeFuzzy = 2,
    AbsoluteFuzzy = 3,
}

/// VTK: `vtkLine`.
#[derive(Debug)]
pub struct Line {
    cell: Cell,
}

impl Line {
    /// VTK: `vtkLine::New`.
    pub fn new() -> Self {
        let mut line = Self {
            cell: Cell::with_class_name("vtkLine"),
        };
        line.cell.get_points_mut().set_number_of_points(2);
        line.cell.get_point_ids_mut().set_number_of_ids(2);
        for i in 0..2 {
            line.cell.get_points_mut().set_point(i, [0.0, 0.0, 0.0]);
            line.cell.get_point_ids_mut().set_id(i, 0);
        }
        line
    }

    /// VTK: `vtkLine::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.cell.print_self()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.cell.get_m_time()
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

    /// VTK: `vtkLine::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Line as i32
    }

    /// VTK: `vtkLine::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        1
    }

    /// VTK: `vtkLine::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        0
    }

    /// VTK: `vtkLine::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkLine::GetEdge`.
    pub fn get_edge(&self, _edge_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkLine::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkLine::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.5, 0.0, 0.0])
    }

    /// VTK: `vtkLine::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        pts.set_number_of_ids(1);
        if pcoords[0] >= 0.5 {
            pts.set_id(0, self.cell.get_point_ids().get_id(1));
            (pcoords[0] <= 1.0) as i32
        } else {
            pts.set_id(0, self.cell.get_point_ids().get_id(0));
            (pcoords[0] >= 0.0) as i32
        }
    }

    /// VTK: `vtkLine::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        x: [f64; 3],
        closest_point: Option<&mut [f64; 3]>,
    ) -> LineEvaluatePosition {
        let a1 = self.cell.get_points().get_point(0);
        let a2 = self.cell.get_points().get_point(1);
        let (dist2, t, closest) = Self::distance_to_line_with_closest_point(x, a1, a2);
        if let Some(closest_point) = closest_point {
            *closest_point = closest;
        }
        LineEvaluatePosition {
            inside: (t >= 0.0 && t <= 1.0) as i32,
            sub_id: 0,
            pcoords: [t, 0.0, 0.0],
            dist2,
            weights: [1.0 - t, t],
        }
    }

    /// VTK: `vtkLine::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 2]) {
        let a1 = self.cell.get_points().get_point(0);
        let a2 = self.cell.get_points().get_point(1);
        let mut x = [0.0; 3];
        for i in 0..3 {
            x[i] = a1[i] + pcoords[0] * (a2[i] - a1[i]);
        }
        (x, [1.0 - pcoords[0], pcoords[0]])
    }

    /// VTK: `vtkLine::IntersectWithLine`.
    pub fn intersect_with_line(
        &self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> LineIntersectWithLine {
        let a1 = self.cell.get_points().get_point(0);
        let a2 = self.cell.get_points().get_point(1);
        let (intersection, mut t, mut pcoord0) =
            Self::intersection(p1, p2, a1, a2, f64::INFINITY, LineToleranceType::Relative);

        if intersection == LineIntersectionType::Intersect {
            let mut x = [0.0; 3];
            let mut proj_xyz = [0.0; 3];
            for i in 0..3 {
                x[i] = a1[i] + pcoord0 * (a2[i] - a1[i]);
                proj_xyz[i] = p1[i] + t * (p2[i] - p1[i]);
            }
            return LineIntersectWithLine {
                intersection: (distance2_between_points(x, proj_xyz) <= tol * tol) as i32,
                t,
                x,
                pcoords: [pcoord0, 0.0, 0.0],
                sub_id: 0,
            };
        }

        if t < 0.0 {
            t = 0.0;
            let (dist2, pcoord, x) = Self::distance_to_line_with_closest_point(p1, a1, a2);
            pcoord0 = pcoord;
            return LineIntersectWithLine {
                intersection: (dist2 <= tol * tol) as i32,
                t,
                x,
                pcoords: [pcoord0, 0.0, 0.0],
                sub_id: 0,
            };
        }
        if t > 1.0 {
            t = 1.0;
            let (dist2, pcoord, x) = Self::distance_to_line_with_closest_point(p2, a1, a2);
            pcoord0 = pcoord;
            return LineIntersectWithLine {
                intersection: (dist2 <= tol * tol) as i32,
                t,
                x,
                pcoords: [pcoord0, 0.0, 0.0],
                sub_id: 0,
            };
        }
        if pcoord0 < 0.0 {
            pcoord0 = 0.0;
            let (dist2, t_on_input, x) = Self::distance_to_line_with_closest_point(a1, p1, p2);
            t = t_on_input;
            return LineIntersectWithLine {
                intersection: (dist2 <= tol * tol) as i32,
                t,
                x,
                pcoords: [pcoord0, 0.0, 0.0],
                sub_id: 0,
            };
        }
        if pcoord0 > 1.0 {
            pcoord0 = 1.0;
            let (dist2, t_on_input, x) = Self::distance_to_line_with_closest_point(a2, p1, p2);
            t = t_on_input;
            return LineIntersectWithLine {
                intersection: (dist2 <= tol * tol) as i32,
                t,
                x,
                pcoords: [pcoord0, 0.0, 0.0],
                sub_id: 0,
            };
        }

        LineIntersectWithLine {
            intersection: 0,
            t,
            x: [0.0; 3],
            pcoords: [pcoord0, 0.0, 0.0],
            sub_id: 0,
        }
    }

    /// VTK: `vtkLine::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(2);
        pt_ids.set_id(0, 0);
        pt_ids.set_id(1, 1);
        1
    }

    /// VTK: `vtkLine::Inflate`.
    pub fn inflate(&mut self, dist: f64) -> i32 {
        let mut p0 = self.cell.get_points().get_point(0);
        let mut p1 = self.cell.get_points().get_point(1);
        if nearly_equal(p0[0], p1[0]) && nearly_equal(p0[1], p1[1]) && nearly_equal(p0[2], p1[2]) {
            return 0;
        }

        let mut v = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        normalize(&mut v);
        for i in 0..3 {
            p0[i] -= v[i] * dist;
            p1[i] += v[i] * dist;
        }
        self.cell.get_points_mut().set_point(0, p0);
        self.cell.get_points_mut().set_point(1, p1);
        1
    }

    /// VTK: `vtkLine::Derivatives`.
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
            values.len() >= dim * 2,
            "vtkLine::Derivatives values slice too short"
        );
        assert!(
            derivs.len() >= dim * 3,
            "vtkLine::Derivatives derivs slice too short"
        );

        let x0 = self.cell.get_points().get_point(0);
        let x1 = self.cell.get_points().get_point(1);
        let delta = [x1[0] - x0[0], x1[1] - x0[1], x1[2] - x0[2]];
        for i in 0..dim {
            for j in 0..3 {
                derivs[3 * i + j] = if delta[j] != 0.0 {
                    (values[i + dim] - values[i]) / delta[j]
                } else {
                    0.0
                };
            }
        }
    }

    /// VTK: `vtkLine::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 2] {
        [1.0 - pcoords[0], pcoords[0]]
    }

    /// VTK: `vtkLine::InterpolationDerivs`.
    pub fn interpolation_derivs(_pcoords: [f64; 3]) -> [f64; 2] {
        [-1.0, 1.0]
    }

    /// VTK: `vtkLine::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        assert!(
            weights.len() >= 2,
            "vtkLine::InterpolateFunctions weights slice too short"
        );
        weights[..2].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkLine::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        assert!(
            derivs.len() >= 2,
            "vtkLine::InterpolateDerivs derivs slice too short"
        );
        derivs[..2].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkLine::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 6] {
        &LINE_CELL_PCOORDS
    }

    /// VTK: `vtkLine::Evaluate`.
    pub fn evaluate(n: [f64; 2], o: [f64; 2], x: [f64; 2]) -> f64 {
        (x[0] - o[0]) * n[0] + (x[1] - o[1]) * n[1]
    }

    /// VTK: `vtkLine::Intersection`.
    pub fn intersection(
        a1: [f64; 3],
        a2: [f64; 3],
        b1: [f64; 3],
        b2: [f64; 3],
        tolerance: f64,
        tolerance_type: LineToleranceType,
    ) -> (LineIntersectionType, f64, f64) {
        let a21 = [a2[0] - a1[0], a2[1] - a1[1], a2[2] - a1[2]];
        let b21 = [b2[0] - b1[0], b2[1] - b1[1], b2[2] - b1[2]];
        let b1a1 = [b1[0] - a1[0], b1[1] - a1[1], b1[2] - a1[2]];

        let row1 = [dot(a21, a21), -dot(a21, b21)];
        let row2 = [row1[1], dot(b21, b21)];
        let c = [dot(a21, b1a1), -dot(b21, b1a1)];
        let det = determinant2x2(row1[0], row1[1], row2[0], row2[1]);

        if det == 0.0 {
            let candidates = [
                (a1, b1, b2, 0usize),
                (a2, b1, b2, 1usize),
                (b1, a1, a2, 2usize),
                (b2, a1, a2, 3usize),
            ];
            let mut min_dist = VTK_DOUBLE_MAX;
            let mut u = 0.0;
            let mut v = 0.0;
            for (p, l1, l2, i) in candidates {
                let (dist, t, _closest) = Self::distance_to_line_with_closest_point(p, l1, l2);
                if dist < min_dist {
                    min_dist = dist;
                    match i {
                        0 => {
                            v = t;
                            u = 0.0;
                        }
                        1 => {
                            v = t;
                            u = 1.0;
                        }
                        2 => {
                            u = t;
                            v = 0.0;
                        }
                        _ => {
                            u = t;
                            v = 1.0;
                        }
                    }
                }
            }
            return (LineIntersectionType::OnLine, u, v);
        }

        let u = determinant2x2(c[0], row1[1], c[1], row2[1]) / det;
        let v = determinant2x2(row1[0], c[0], row2[0], c[1]) / det;

        let ptu = [a1[0] + u * a21[0], a1[1] + u * a21[1], a1[2] + u * a21[2]];
        let ptv = [b1[0] + v * b21[0], b1[1] + v * b21[1], b1[2] + v * b21[2]];
        let diff2 = squared_norm([ptu[0] - ptv[0], ptu[1] - ptv[1], ptu[2] - ptv[2]]);

        let mut tol2 = 0.0;
        if tolerance.is_finite() {
            tol2 = match tolerance_type {
                LineToleranceType::Absolute | LineToleranceType::AbsoluteFuzzy => {
                    tolerance * tolerance
                }
                LineToleranceType::Relative | LineToleranceType::RelativeFuzzy => {
                    tolerance * tolerance * squared_norm(ptv).max(squared_norm(ptu))
                }
            };
            if diff2 > tol2 {
                return (LineIntersectionType::NoIntersect, u, v);
            }
        }

        if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
            return (LineIntersectionType::Intersect, u, v);
        }
        if matches!(
            tolerance_type,
            LineToleranceType::RelativeFuzzy | LineToleranceType::AbsoluteFuzzy
        ) && tol2 > 0.0
        {
            let u_tol = (tol2 / squared_norm(a21)).sqrt();
            let v_tol = (tol2 / squared_norm(b21)).sqrt();
            if (-u_tol <= u) && (u <= 1.0 + u_tol) && (-v_tol <= v) && (v <= 1.0 + v_tol) {
                return (LineIntersectionType::Intersect, u, v);
            }
        }

        (LineIntersectionType::NoIntersect, u, v)
    }

    /// VTK: `vtkLine::DistanceBetweenLines`.
    pub fn distance_between_lines(
        l0: [f64; 3],
        l1: [f64; 3],
        m0: [f64; 3],
        m1: [f64; 3],
    ) -> LineDistanceBetween {
        let u = [l1[0] - l0[0], l1[1] - l0[1], l1[2] - l0[2]];
        let v = [m1[0] - m0[0], m1[1] - m0[1], m1[2] - m0[2]];
        let w = [l0[0] - m0[0], l0[1] - m0[1], l0[2] - m0[2]];
        let a = dot(u, u);
        let b = dot(u, v);
        let c = dot(v, v);
        let d = dot(u, w);
        let e = dot(v, w);
        let determinant = a * c - b * b;

        let (t1, t2) = if determinant < 1e-6 {
            (0.0, if b > c { d / b } else { e / c })
        } else {
            ((b * e - c * d) / determinant, (a * e - b * d) / determinant)
        };

        let closest_pt1 = [l0[0] + t1 * u[0], l0[1] + t1 * u[1], l0[2] + t1 * u[2]];
        let closest_pt2 = [m0[0] + t2 * v[0], m0[1] + t2 * v[1], m0[2] + t2 * v[2]];
        LineDistanceBetween {
            distance2: distance2_between_points(closest_pt1, closest_pt2),
            closest_pt1,
            closest_pt2,
            t1,
            t2,
        }
    }

    /// VTK: `vtkLine::DistanceBetweenLineSegments`.
    pub fn distance_between_line_segments(
        l0: [f64; 3],
        l1: [f64; 3],
        m0: [f64; 3],
        m1: [f64; 3],
    ) -> LineDistanceBetween {
        let u = [l1[0] - l0[0], l1[1] - l0[1], l1[2] - l0[2]];
        let v = [m1[0] - m0[0], m1[1] - m0[1], m1[2] - m0[2]];
        let w = [l0[0] - m0[0], l0[1] - m0[1], l0[2] - m0[2]];
        let a = dot(u, u);
        let b = dot(u, v);
        let c = dot(v, v);
        let d = dot(u, w);
        let e = dot(v, w);
        let determinant = a * c - b * b;
        let mut s_n;
        let mut s_d = determinant;
        let mut t_n;
        let mut t_d = determinant;

        if determinant < 1e-6 {
            let candidates = [
                (l0, m0, m1, 0usize),
                (l1, m0, m1, 1usize),
                (m0, l0, l1, 2usize),
                (m1, l0, l1, 3usize),
            ];
            let mut min_dist = VTK_DOUBLE_MAX;
            let mut result = LineDistanceBetween {
                distance2: VTK_DOUBLE_MAX,
                closest_pt1: [0.0; 3],
                closest_pt2: [0.0; 3],
                t1: 0.0,
                t2: 0.0,
            };

            for (p, a1, a2, i) in candidates {
                let (dist, t, pn) = Self::distance_to_line_with_closest_point(p, a1, a2);
                if dist < min_dist {
                    min_dist = dist;
                    let mut t = t;
                    if t < 0.0 {
                        t = 0.0;
                    }
                    if t > 1.0 {
                        t = 1.0;
                    }
                    match i {
                        0 => {
                            result.t2 = t;
                            result.t1 = 0.0;
                            result.closest_pt2 = pn;
                            result.closest_pt1 = p;
                        }
                        1 => {
                            result.t2 = t;
                            result.t1 = 1.0;
                            result.closest_pt2 = pn;
                            result.closest_pt1 = p;
                        }
                        2 => {
                            result.t1 = t;
                            result.t2 = 0.0;
                            result.closest_pt1 = pn;
                            result.closest_pt2 = p;
                        }
                        _ => {
                            result.t1 = t;
                            result.t2 = 1.0;
                            result.closest_pt1 = pn;
                            result.closest_pt2 = p;
                        }
                    }
                    result.distance2 = min_dist;
                }
            }
            return result;
        } else {
            s_n = b * e - c * d;
            t_n = a * e - b * d;
            if s_n < 0.0 {
                s_n = 0.0;
                t_n = e;
                t_d = c;
            } else if s_n > s_d {
                s_n = s_d;
                t_n = e + b;
                t_d = c;
            }
        }

        if t_n < 0.0 {
            t_n = 0.0;
            if -d < 0.0 {
                s_n = 0.0;
            } else if -d > a {
                s_n = s_d;
            } else {
                s_n = -d;
                s_d = a;
            }
        } else if t_n > t_d {
            t_n = t_d;
            if -d + b < 0.0 {
                s_n = 0.0;
            } else if -d + b > a {
                s_n = s_d;
            } else {
                s_n = -d + b;
                s_d = a;
            }
        }

        let t1 = if s_n.abs() < 1e-6 { 0.0 } else { s_n / s_d };
        let t2 = if t_n.abs() < 1e-6 { 0.0 } else { t_n / t_d };
        let closest_pt1 = [l0[0] + t1 * u[0], l0[1] + t1 * u[1], l0[2] + t1 * u[2]];
        let closest_pt2 = [m0[0] + t2 * v[0], m0[1] + t2 * v[1], m0[2] + t2 * v[2]];

        LineDistanceBetween {
            distance2: distance2_between_points(closest_pt1, closest_pt2),
            closest_pt1,
            closest_pt2,
            t1,
            t2,
        }
    }

    /// VTK: `vtkLine::DistanceToLine(const double[3], const double[3], const double[3], double&, double[3])`.
    pub fn distance_to_line_with_closest_point(
        x: [f64; 3],
        p1: [f64; 3],
        p2: [f64; 3],
    ) -> (f64, f64, [f64; 3]) {
        let mut p21 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let num = p21[0] * (x[0] - p1[0]) + p21[1] * (x[1] - p1[1]) + p21[2] * (x[2] - p1[2]);

        let (t, closest) = if num == 0.0 {
            (0.0, p1)
        } else {
            let denom = dot(p21, p21);
            let tolerance = (VTK_TOL * num).abs();
            if denom < tolerance {
                if num > 0.0 {
                    (f64::MAX, p2)
                } else {
                    (f64::MIN_POSITIVE, p1)
                }
            } else {
                let t = num / denom;
                if t < 0.0 {
                    (t, p1)
                } else if t > 1.0 {
                    (t, p2)
                } else {
                    p21[0] = p1[0] + t * p21[0];
                    p21[1] = p1[1] + t * p21[1];
                    p21[2] = p1[2] + t * p21[2];
                    (t, p21)
                }
            }
        };

        (distance2_between_points(closest, x), t, closest)
    }

    /// VTK: `vtkLine::DistanceToLine(const double[3], const double[3], const double[3])`.
    pub fn distance_to_line(x: [f64; 3], p1: [f64; 3], p2: [f64; 3]) -> f64 {
        let np1 = [x[0] - p1[0], x[1] - p1[1], x[2] - p1[2]];
        let mut p1p2 = [p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2]];
        let den = normalize(&mut p1p2);
        if den == 0.0 {
            return dot(np1, np1);
        }
        let proj = dot(np1, p1p2);
        squared_norm([
            np1[0] - proj * p1p2[0],
            np1[1] - proj * p1p2[1],
            np1[2] - proj * p1p2[2],
        ])
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }
}

/// Rust return bundle for VTK `vtkLine::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 2],
}

/// Rust return bundle for VTK `vtkLine::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

/// Rust return bundle for VTK line distance out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineDistanceBetween {
    pub distance2: f64,
    pub closest_pt1: [f64; 3],
    pub closest_pt2: [f64; 3],
    pub t1: f64,
    pub t2: f64,
}

impl CellBaseApi for Line {
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

static LINE_CELL_PCOORDS: [f64; 6] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0];

fn nearly_equal(a: f64, b: f64) -> bool {
    let absdiff = (a - b).abs();
    let d1 = safe_division(absdiff, a.abs());
    let d2 = safe_division(absdiff, b.abs());
    d1 <= f64::EPSILON || d2 <= f64::EPSILON
}

fn safe_division(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return f64::MAX;
    }
    if a == 0.0 || (b > 1.0 && a < b * f64::MIN_POSITIVE) {
        return 0.0;
    }
    a / b
}
