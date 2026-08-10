use crate::common::core::{
    math::{
        cross, determinant2x2, determinant3x3, determinant3x3_from_values,
        distance2_between_points, dot, dot2d, invert_matrix, norm, normalize, solve_linear_system,
        squared_norm,
    },
    IdList, Points, VtkIdType, VTK_DOUBLE_MAX,
};

use super::{Cell, CellBaseApi, CellType, Line, Plane, Quadric};

/// VTK: `vtkTriangle`.
#[derive(Debug)]
pub struct Triangle {
    cell: Cell,
    line: Line,
}

impl Triangle {
    /// VTK: `vtkTriangle::New`.
    pub fn new() -> Self {
        let mut triangle = Self {
            cell: Cell::with_class_name("vtkTriangle"),
            line: Line::new(),
        };
        triangle.cell.get_points_mut().set_number_of_points(3);
        triangle.cell.get_point_ids_mut().set_number_of_ids(3);
        for i in 0..3 {
            triangle.cell.get_points_mut().set_point(i, [0.0, 0.0, 0.0]);
            triangle.cell.get_point_ids_mut().set_id(i, 0);
        }
        triangle
    }

    /// VTK: `vtkTriangle::PrintSelf`.
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
        self.cell.shallow_copy(&source.cell);
    }

    /// VTK: `vtkCell::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.cell.deep_copy(&source.cell);
    }

    /// VTK: `vtkTriangle::GetCellType`.
    pub fn get_cell_type(&self) -> i32 {
        CellType::Triangle as i32
    }

    /// VTK: `vtkTriangle::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        2
    }

    /// VTK: `vtkTriangle::GetNumberOfEdges`.
    pub fn get_number_of_edges(&self) -> i32 {
        3
    }

    /// VTK: `vtkTriangle::GetNumberOfFaces`.
    pub fn get_number_of_faces(&self) -> i32 {
        0
    }

    /// VTK: `vtkTriangle::GetFace`.
    pub fn get_face(&self, _face_id: i32) -> Option<&Cell> {
        None
    }

    /// VTK: `vtkTriangle::GetEdgeArray`.
    pub fn get_edge_array(edge_id: VtkIdType) -> &'static [VtkIdType; 2] {
        &EDGES[edge_id as usize]
    }

    /// VTK: `vtkTriangle::GetEdge`.
    pub fn get_edge(&mut self, edge_id: i32) -> &mut Line {
        let edge_id = edge_id as VtkIdType;
        let edge_id_plus_one = if edge_id > 1 { 0 } else { edge_id + 1 };

        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(0, self.cell.get_point_ids().get_id(edge_id));
        self.line
            .cell_mut()
            .get_point_ids_mut()
            .set_id(1, self.cell.get_point_ids().get_id(edge_id_plus_one));

        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(0, self.cell.get_points().get_point(edge_id));
        self.line
            .cell_mut()
            .get_points_mut()
            .set_point(1, self.cell.get_points().get_point(edge_id_plus_one));

        &mut self.line
    }

    /// VTK: `vtkTriangle::IntersectWithLine`.
    pub fn intersect_with_line(
        &mut self,
        p1: [f64; 3],
        p2: [f64; 3],
        tol: f64,
    ) -> TriangleIntersectWithLine {
        let mut pcoords = [0.0, 0.0, 0.0];
        let tol2 = tol * tol;
        let pt1 = self.cell.get_points().get_point(1);
        let pt2 = self.cell.get_points().get_point(2);
        let pt3 = self.cell.get_points().get_point(0);
        let n = Self::compute_normal(pt1, pt2, pt3);

        if n[0] != 0.0 || n[1] != 0.0 || n[2] != 0.0 {
            let (plane_hit, t, x) = Plane::intersect_with_line(p1, p2, n, pt1);
            if plane_hit == 0 {
                if t != VTK_DOUBLE_MAX || (dot(n, pt1) - dot(n, p1)) != 0.0 {
                    return TriangleIntersectWithLine {
                        intersection: 0,
                        t,
                        x,
                        pcoords,
                        sub_id: 0,
                    };
                }

                let p1_eval = self.evaluate_position(p1, true);
                if p1_eval.inside == 1 {
                    return TriangleIntersectWithLine {
                        intersection: 1,
                        t: 0.0,
                        x: p1,
                        pcoords: p1_eval.pcoords,
                        sub_id: p1_eval.sub_id,
                    };
                }

                let mut intersection = false;
                let mut closest_distance = VTK_DOUBLE_MAX;
                let mut closest_x = [0.0; 3];
                let mut closest_pcoords = [0.0; 3];

                for i in 0..self.get_number_of_edges() {
                    let edge_eval = self.get_edge(i).intersect_with_line(p1, p2, tol);
                    if edge_eval.intersection != 0 {
                        intersection = true;
                        if edge_eval.t < closest_distance {
                            closest_distance = edge_eval.t;
                            let eval = self.evaluate_position(edge_eval.x, true);
                            closest_x = edge_eval.x;
                            closest_pcoords = eval.pcoords;
                        }
                    }
                }

                if !intersection {
                    return TriangleIntersectWithLine {
                        intersection: 0,
                        t,
                        x,
                        pcoords,
                        sub_id: 0,
                    };
                }

                return TriangleIntersectWithLine {
                    intersection: 1,
                    t: closest_distance,
                    x: closest_x,
                    pcoords: closest_pcoords,
                    sub_id: 0,
                };
            }

            let eval = self.evaluate_position(x, true);
            if eval.inside >= 0 {
                pcoords = eval.pcoords;
                if eval.dist2 <= tol2 {
                    return TriangleIntersectWithLine {
                        intersection: 1,
                        t,
                        x,
                        pcoords,
                        sub_id: eval.sub_id,
                    };
                }
                return TriangleIntersectWithLine {
                    intersection: eval.inside,
                    t,
                    x,
                    pcoords,
                    sub_id: eval.sub_id,
                };
            }
        }

        let dist2_pt1_pt2 = distance2_between_points(pt1, pt2);
        let dist2_pt2_pt3 = distance2_between_points(pt2, pt3);
        let dist2_pt3_pt1 = distance2_between_points(pt3, pt1);
        if dist2_pt1_pt2 > dist2_pt2_pt3 && dist2_pt1_pt2 > dist2_pt3_pt1 {
            self.line.cell_mut().get_points_mut().set_point(0, pt1);
            self.line.cell_mut().get_points_mut().set_point(1, pt2);
        } else if dist2_pt2_pt3 > dist2_pt3_pt1 && dist2_pt2_pt3 > dist2_pt1_pt2 {
            self.line.cell_mut().get_points_mut().set_point(0, pt2);
            self.line.cell_mut().get_points_mut().set_point(1, pt3);
        } else {
            self.line.cell_mut().get_points_mut().set_point(0, pt3);
            self.line.cell_mut().get_points_mut().set_point(1, pt1);
        }

        let line_eval = self.line.intersect_with_line(p1, p2, tol);
        if line_eval.intersection != 0 {
            let x = line_eval.x;
            let pt3_pt1 = [pt1[0] - pt3[0], pt1[1] - pt3[1], pt1[2] - pt3[2]];
            let pt3_pt2 = [pt2[0] - pt3[0], pt2[1] - pt3[1], pt2[2] - pt3[2]];
            let pt3_x = [x[0] - pt3[0], x[1] - pt3[1], x[2] - pt3[2]];
            return TriangleIntersectWithLine {
                intersection: 1,
                t: line_eval.t,
                x,
                pcoords: [
                    dot(pt3_x, pt3_pt1) / dist2_pt3_pt1,
                    dot(pt3_x, pt3_pt2) / dist2_pt2_pt3,
                    0.0,
                ],
                sub_id: line_eval.sub_id,
            };
        }

        TriangleIntersectWithLine {
            intersection: 0,
            t: line_eval.t,
            x: line_eval.x,
            pcoords,
            sub_id: 0,
        }
    }

    /// VTK: `vtkTriangle::EvaluatePosition`.
    pub fn evaluate_position(
        &self,
        x: [f64; 3],
        closest_point_requested: bool,
    ) -> TriangleEvaluatePosition {
        let pt1 = self.cell.get_points().get_point(1);
        let pt2 = self.cell.get_points().get_point(2);
        let pt3 = self.cell.get_points().get_point(0);
        let n = Self::compute_normal_direction(pt1, pt2, pt3);
        let cp = Plane::generalized_project_point(x, pt1, n);

        let mut idx = 0;
        let mut max_component = 0.0;
        for (i, component) in n.iter().enumerate() {
            let fabsn = if *component < 0.0 {
                -*component
            } else {
                *component
            };
            if fabsn > max_component {
                max_component = fabsn;
                idx = i;
            }
        }

        let mut indices = [0usize; 2];
        let mut j = 0;
        for i in 0..3 {
            if i != idx {
                indices[j] = i;
                j += 1;
            }
        }

        let rhs = [
            cp[indices[0]] - pt3[indices[0]],
            cp[indices[1]] - pt3[indices[1]],
        ];
        let c1 = [
            pt1[indices[0]] - pt3[indices[0]],
            pt1[indices[1]] - pt3[indices[1]],
        ];
        let c2 = [
            pt2[indices[0]] - pt3[indices[0]],
            pt2[indices[1]] - pt3[indices[1]],
        ];
        let det = determinant2x2(c1[0], c2[0], c1[1], c2[1]);
        if det == 0.0 {
            return TriangleEvaluatePosition {
                inside: -1,
                sub_id: 0,
                pcoords: [0.0, 0.0, 0.0],
                dist2: 0.0,
                weights: [0.0; 3],
                closest_point: None,
            };
        }

        let pcoords = [
            determinant2x2(rhs[0], c2[0], rhs[1], c2[1]) / det,
            determinant2x2(c1[0], rhs[0], c1[1], rhs[1]) / det,
            0.0,
        ];
        let weights = [1.0 - (pcoords[0] + pcoords[1]), pcoords[0], pcoords[1]];

        if weights
            .iter()
            .all(|weight| *weight >= 0.0 && *weight <= 1.0)
        {
            return TriangleEvaluatePosition {
                inside: 1,
                sub_id: 0,
                pcoords,
                dist2: if closest_point_requested {
                    distance2_between_points(cp, x)
                } else {
                    0.0
                },
                weights,
                closest_point: closest_point_requested.then_some(cp),
            };
        }

        let (dist2, closest_point) = if closest_point_requested {
            Self::closest_point_outside_triangle(x, pt1, pt2, pt3, weights)
        } else {
            (0.0, None)
        };

        TriangleEvaluatePosition {
            inside: 0,
            sub_id: 0,
            pcoords,
            dist2,
            weights,
            closest_point,
        }
    }

    /// VTK: `vtkTriangle::CellBoundary`.
    pub fn cell_boundary(&self, _sub_id: i32, pcoords: [f64; 3], pts: &mut IdList) -> i32 {
        let t1 = pcoords[0] - pcoords[1];
        let t2 = 0.5 * (1.0 - pcoords[0]) - pcoords[1];
        let t3 = 2.0 * pcoords[0] + pcoords[1] - 1.0;

        pts.set_number_of_ids(2);
        if t1 >= 0.0 && t2 >= 0.0 {
            pts.set_id(0, self.cell.get_point_ids().get_id(0));
            pts.set_id(1, self.cell.get_point_ids().get_id(1));
        } else if t2 < 0.0 && t3 >= 0.0 {
            pts.set_id(0, self.cell.get_point_ids().get_id(1));
            pts.set_id(1, self.cell.get_point_ids().get_id(2));
        } else {
            pts.set_id(0, self.cell.get_point_ids().get_id(2));
            pts.set_id(1, self.cell.get_point_ids().get_id(0));
        }

        (pcoords[0] >= 0.0
            && pcoords[1] >= 0.0
            && pcoords[0] <= 1.0
            && pcoords[1] <= 1.0
            && (1.0 - pcoords[0] - pcoords[1]) >= 0.0) as i32
    }

    /// VTK: `vtkTriangle::EvaluateLocation`.
    pub fn evaluate_location(&self, _sub_id: i32, pcoords: [f64; 3]) -> ([f64; 3], [f64; 3]) {
        let weights = Self::interpolation_functions(pcoords);
        let mut x = [0.0; 3];
        for i in 0..3 {
            let point = self.cell.get_points().get_point(i);
            for j in 0..3 {
                x[j] += point[j] * weights[i as usize];
            }
        }
        (x, weights)
    }

    /// VTK: `vtkTriangle::TriangulateLocalIds`.
    pub fn triangulate_local_ids(&self, _index: i32, pt_ids: &mut IdList) -> i32 {
        pt_ids.set_number_of_ids(3);
        for id in 0..3 {
            pt_ids.set_id(id, id);
        }
        1
    }

    /// VTK: `vtkTriangle::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [1.0 / 3.0, 1.0 / 3.0, 0.0])
    }

    /// VTK: `vtkTriangle::GetParametricDistance`.
    pub fn get_parametric_distance(&self, pcoords: [f64; 3]) -> f64 {
        let pc = [pcoords[0], pcoords[1], 1.0 - pcoords[0] - pcoords[1]];
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

    /// VTK: `vtkTriangle::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> &'static [f64; 9] {
        &TRIANGLE_CELL_PCOORDS
    }

    /// VTK: `vtkTriangle::ComputeArea`.
    pub fn compute_area(&self) -> f64 {
        Self::triangle_area(
            self.cell.get_points().get_point(0),
            self.cell.get_points().get_point(1),
            self.cell.get_points().get_point(2),
        )
    }

    /// VTK: `vtkTriangle::InterpolationFunctions`.
    pub fn interpolation_functions(pcoords: [f64; 3]) -> [f64; 3] {
        [1.0 - pcoords[0] - pcoords[1], pcoords[0], pcoords[1]]
    }

    /// VTK: `vtkTriangle::InterpolationDerivs`.
    pub fn interpolation_derivs(_pcoords: [f64; 3]) -> [f64; 6] {
        [-1.0, 1.0, 0.0, -1.0, 0.0, 1.0]
    }

    /// VTK: `vtkTriangle::InterpolateFunctions`.
    pub fn interpolate_functions(&self, pcoords: [f64; 3], weights: &mut [f64]) {
        assert!(
            weights.len() >= 3,
            "vtkTriangle::InterpolateFunctions weights slice too short"
        );
        weights[..3].copy_from_slice(&Self::interpolation_functions(pcoords));
    }

    /// VTK: `vtkTriangle::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, pcoords: [f64; 3], derivs: &mut [f64]) {
        assert!(
            derivs.len() >= 6,
            "vtkTriangle::InterpolateDerivs derivs slice too short"
        );
        derivs[..6].copy_from_slice(&Self::interpolation_derivs(pcoords));
    }

    /// VTK: `vtkTriangle::Derivatives`.
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
            values.len() >= dim * 3,
            "vtkTriangle::Derivatives values slice too short"
        );
        assert!(
            derivs.len() >= dim * 3,
            "vtkTriangle::Derivatives derivs slice too short"
        );

        let x0 = self.cell.get_points().get_point(0);
        let x1 = self.cell.get_points().get_point(1);
        let x2 = self.cell.get_points().get_point(2);
        let n = Self::compute_normal(x0, x1, x2);

        let mut v10 = [x1[0] - x0[0], x1[1] - x0[1], x1[2] - x0[2]];
        let v = [x2[0] - x0[0], x2[1] - x0[1], x2[2] - x0[2]];
        let mut v20 = cross(n, v10);

        let len_x = normalize(&mut v10);
        if len_x <= 0.0 || normalize(&mut v20) <= 0.0 {
            for j in 0..dim {
                for i in 0..3 {
                    let idx = j * dim + i;
                    if idx < derivs.len() {
                        derivs[idx] = 0.0;
                    }
                }
            }
            return;
        }

        let v1 = [len_x, 0.0];
        let v2 = [dot(v, v10), dot(v, v20)];
        let function_derivs = Self::interpolation_derivs([0.0, 0.0, 0.0]);

        let jacobian = vec![vec![v1[0], v1[1]], vec![v2[0], v2[1]]];
        let (_success, _factored, jacobian_inverse) = invert_matrix(jacobian, 2);

        for j in 0..dim {
            let mut sum = [0.0; 2];
            for i in 0..3 {
                let value = values[dim * i + j];
                sum[0] += function_derivs[i] * value;
                sum[1] += function_derivs[3 + i] * value;
            }

            let d_by_dx = sum[0] * jacobian_inverse[0][0] + sum[1] * jacobian_inverse[0][1];
            let d_by_dy = sum[0] * jacobian_inverse[1][0] + sum[1] * jacobian_inverse[1][1];
            derivs[3 * j] = d_by_dx * v10[0] + d_by_dy * v20[0];
            derivs[3 * j + 1] = d_by_dx * v10[1] + d_by_dy * v20[1];
            derivs[3 * j + 2] = d_by_dx * v10[2] + d_by_dy * v20[2];
        }
    }

    /// VTK: `vtkTriangle::TriangleCenter`.
    pub fn triangle_center(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]) -> [f64; 3] {
        [
            (p1[0] + p2[0] + p3[0]) / 3.0,
            (p1[1] + p2[1] + p3[1]) / 3.0,
            (p1[2] + p2[2] + p3[2]) / 3.0,
        ]
    }

    /// VTK: `vtkTriangle::TriangleArea`.
    pub fn triangle_area(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]) -> f64 {
        let n = Self::compute_normal_direction(p1, p2, p3);
        0.5 * norm(&n)
    }

    /// VTK: `vtkTriangle::ComputeNormalDirection`.
    pub fn compute_normal_direction(v1: [f64; 3], v2: [f64; 3], v3: [f64; 3]) -> [f64; 3] {
        let ax = v3[0] - v2[0];
        let ay = v3[1] - v2[1];
        let az = v3[2] - v2[2];
        let bx = v1[0] - v2[0];
        let by = v1[1] - v2[1];
        let bz = v1[2] - v2[2];

        [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx]
    }

    /// VTK: `vtkTriangle::ComputeNormal(const double[3], const double[3], const double[3], double[3])`.
    pub fn compute_normal(v1: [f64; 3], v2: [f64; 3], v3: [f64; 3]) -> [f64; 3] {
        let mut n = Self::compute_normal_direction(v1, v2, v3);
        normalize(&mut n);
        n
    }

    /// VTK: `vtkTriangle::ComputeNormal(vtkPoints*, int, const vtkIdType*, double[3])`.
    pub fn compute_normal_from_points(points: &Points, point_ids: &[VtkIdType]) -> [f64; 3] {
        assert!(
            point_ids.len() >= 3,
            "vtkTriangle::ComputeNormal point id slice too short"
        );
        Self::compute_normal(
            points.get_point(point_ids[0]),
            points.get_point(point_ids[1]),
            points.get_point(point_ids[2]),
        )
    }

    /// VTK: `vtkTriangle::Circumcircle`.
    pub fn circumcircle(x1: [f64; 2], x2: [f64; 2], x3: [f64; 2]) -> (f64, [f64; 2]) {
        let n12 = [x2[0] - x1[0], x2[1] - x1[1]];
        let n13 = [x3[0] - x1[0], x3[1] - x1[1]];
        let x12 = [(x2[0] + x1[0]) * 0.5, (x2[1] + x1[1]) * 0.5];
        let x13 = [(x3[0] + x1[0]) * 0.5, (x3[1] + x1[1]) * 0.5];
        let rhs = vec![dot2d(n12, x12), dot2d(n13, x13)];

        let (success, _factored, rhs) =
            solve_linear_system(vec![n12.to_vec(), n13.to_vec()], rhs, 2);
        if !success {
            return (VTK_DOUBLE_MAX, [0.0, 0.0]);
        }

        let center = [rhs[0], rhs[1]];
        let mut sum = 0.0;
        for point in [x1, x2, x3] {
            for i in 0..2 {
                let diff = point[i] - center[i];
                sum += diff * diff;
            }
        }

        let radius_squared = sum / 3.0;
        if radius_squared > VTK_DOUBLE_MAX {
            (VTK_DOUBLE_MAX, center)
        } else {
            (radius_squared, center)
        }
    }

    /// VTK: `vtkTriangle::BarycentricCoords`.
    pub fn barycentric_coords(
        x: [f64; 2],
        x1: [f64; 2],
        x2: [f64; 2],
        x3: [f64; 2],
    ) -> (i32, [f64; 3]) {
        let a1 = vec![x1[0], x2[0], x3[0]];
        let a2 = vec![x1[1], x2[1], x3[1]];
        let a3 = vec![1.0, 1.0, 1.0];
        let p = vec![x[0], x[1], 1.0];

        let (success, _factored, p) = solve_linear_system(vec![a1, a2, a3], p, 3);
        if success {
            (1, [p[0], p[1], p[2]])
        } else {
            (0, [0.0; 3])
        }
    }

    /// VTK: `vtkTriangle::ProjectTo2D`.
    pub fn project_to_2d(
        x1: [f64; 3],
        x2: [f64; 3],
        x3: [f64; 3],
    ) -> (i32, [f64; 2], [f64; 2], [f64; 2]) {
        let n = Self::compute_normal(x1, x2, x3);
        let mut v21 = [x2[0] - x1[0], x2[1] - x1[1], x2[2] - x1[2]];
        let v31 = [x3[0] - x1[0], x3[1] - x1[1], x3[2] - x1[2]];

        let x_len = normalize(&mut v21);
        if x_len <= 0.0 {
            return (0, [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]);
        }

        let v = cross(n, v21);
        (1, [0.0, 0.0], [x_len, 0.0], [dot(v31, v21), dot(v31, v)])
    }

    /// VTK: `vtkTriangle::TrianglesIntersect`.
    pub fn triangles_intersect(
        mut p1: [f64; 3],
        mut q1: [f64; 3],
        mut r1: [f64; 3],
        mut p2: [f64; 3],
        mut q2: [f64; 3],
        mut r2: [f64; 3],
    ) -> i32 {
        let det1 = [
            triangle_plane_determinant(p2, q2, r2, p1),
            triangle_plane_determinant(p2, q2, r2, q1),
            triangle_plane_determinant(p2, q2, r2, r1),
        ];

        if det1
            .iter()
            .all(|value| value.abs() < TRIANGLE_INTERSECTION_EPS)
        {
            let v1 = [q1[0] - p1[0], q1[1] - p1[1], q1[2] - p1[2]];
            let v2 = [r1[0] - p1[0], r1[1] - p1[1], r1[2] - p1[2]];
            let normal = cross(v1, v2);
            let mut index = 0;
            for i in 1..3 {
                if normal[index].abs() < normal[i].abs() {
                    index = i;
                }
            }

            return match index {
                0 => coplanar_triangles_intersect(
                    [p1[1], p1[2]],
                    [q1[1], q1[2]],
                    [r1[1], r1[2]],
                    [p2[1], p2[2]],
                    [q2[1], q2[2]],
                    [r2[1], r2[2]],
                ),
                1 => coplanar_triangles_intersect(
                    [p1[0], p1[2]],
                    [q1[0], q1[2]],
                    [r1[0], r1[2]],
                    [p2[0], p2[2]],
                    [q2[0], q2[2]],
                    [r2[0], r2[2]],
                ),
                _ => coplanar_triangles_intersect(
                    [p1[0], p1[1]],
                    [q1[0], q1[1]],
                    [r1[0], r1[1]],
                    [p2[0], p2[1]],
                    [q2[0], q2[1]],
                    [r2[0], r2[1]],
                ),
            };
        }

        let mut degenerate = false;
        for (i, point) in [p1, q1, r1].into_iter().enumerate() {
            if det1[i].abs() < TRIANGLE_INTERSECTION_EPS {
                degenerate = true;
                if Self::point_in_triangle(point, p2, q2, r2, TRIANGLE_INTERSECTION_EPS) != 0 {
                    return 1;
                }
            }
        }
        if degenerate {
            return 0;
        }

        let sum_of_signs = (det1[0] > 0.0) as i32 + (det1[1] > 0.0) as i32 + (det1[2] > 0.0) as i32;
        if sum_of_signs == 0 || sum_of_signs == 3 {
            return 0;
        }

        let det2 = [
            triangle_plane_determinant(p1, q1, r1, p2),
            triangle_plane_determinant(p1, q1, r1, q2),
            triangle_plane_determinant(p1, q1, r1, r2),
        ];
        let sum_of_signs = (det2[0] > 0.0) as i32 + (det2[1] > 0.0) as i32 + (det2[2] > 0.0) as i32;
        if sum_of_signs == 0 || sum_of_signs == 3 {
            return 0;
        }

        let mut index1 = 0;
        for i in 0..3 {
            let sum_of_signs = (det1[(i + 1) % 3] > 0.0) as i32 + (det1[(i + 2) % 3] > 0.0) as i32;
            if sum_of_signs != 1 {
                index1 = i;
                break;
            }
        }
        let t1 = [p1, q1, r1];
        p1 = t1[index1];
        q1 = t1[(index1 + 1) % 3];
        r1 = t1[(index1 + 2) % 3];
        let swap1 = det1[index1] < -TRIANGLE_INTERSECTION_EPS;

        let mut index2 = 0;
        for i in 0..3 {
            let sum_of_signs = (det2[(i + 1) % 3] > 0.0) as i32 + (det2[(i + 2) % 3] > 0.0) as i32;
            if sum_of_signs != 1 {
                index2 = i;
                break;
            }
        }
        let t2 = [p2, q2, r2];
        p2 = t2[index2];
        q2 = t2[(index2 + 1) % 3];
        r2 = t2[(index2 + 2) % 3];
        let swap2 = det2[index2] < -TRIANGLE_INTERSECTION_EPS;

        if swap1 {
            std::mem::swap(&mut q2, &mut r2);
        }
        if swap2 {
            std::mem::swap(&mut q1, &mut r1);
        }

        ((triangle_plane_determinant(p1, q1, p2, q2) <= 0.0)
            && (triangle_plane_determinant(p1, r1, r2, p2) <= 0.0)) as i32
    }

    /// VTK: `vtkTriangle::PointInTriangle`.
    pub fn point_in_triangle(
        x: [f64; 3],
        p1: [f64; 3],
        p2: [f64; 3],
        p3: [f64; 3],
        tol2: f64,
    ) -> i32 {
        let x1 = [x[0] - p1[0], x[1] - p1[1], x[2] - p1[2]];
        let x2 = [x[0] - p2[0], x[1] - p2[1], x[2] - p2[2]];
        let x3 = [x[0] - p3[0], x[1] - p3[1], x[2] - p3[2]];
        let v13 = [p1[0] - p3[0], p1[1] - p3[1], p1[2] - p3[2]];
        let v21 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];
        let v32 = [p3[0] - p2[0], p3[1] - p2[1], p3[2] - p2[2]];

        if squared_norm(x1) <= tol2 || squared_norm(x2) <= tol2 || squared_norm(x3) <= tol2 {
            return 1;
        }

        let n1 = cross(x1, v13);
        let n2 = cross(x2, v21);
        let n3 = cross(x3, v32);
        ((dot(n1, n2) >= 0.0) && (dot(n2, n3) >= 0.0) && (dot(n1, n3) >= 0.0)) as i32
    }

    /// VTK: `vtkTriangle::ComputeQuadric`.
    pub fn compute_quadric(x1: [f64; 3], x2: [f64; 3], x3: [f64; 3]) -> [[f64; 4]; 4] {
        let cross_x1_x2 = cross(x1, x2);
        let cross_x2_x3 = cross(x2, x3);
        let cross_x3_x1 = cross(x3, x1);
        let determinant_abc = determinant3x3([x1, x2, x3]);
        let n = [
            cross_x1_x2[0] + cross_x2_x3[0] + cross_x3_x1[0],
            cross_x1_x2[1] + cross_x2_x3[1] + cross_x3_x1[1],
            cross_x1_x2[2] + cross_x2_x3[2] + cross_x3_x1[2],
            -determinant_abc,
        ];

        let mut quadric = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                quadric[i][j] = n[i] * n[j];
            }
        }
        quadric
    }

    /// VTK: `vtkTriangle::ComputeQuadric`.
    pub fn compute_quadric_to_quadric(
        x1: [f64; 3],
        x2: [f64; 3],
        x3: [f64; 3],
        quadric: &mut Quadric,
    ) {
        let quadric_matrix = Self::compute_quadric(x1, x2, x3);
        quadric.set_coefficients_components(
            quadric_matrix[0][0],
            quadric_matrix[1][1],
            quadric_matrix[2][2],
            2.0 * quadric_matrix[0][1],
            2.0 * quadric_matrix[1][2],
            2.0 * quadric_matrix[0][2],
            2.0 * quadric_matrix[0][3],
            2.0 * quadric_matrix[1][3],
            2.0 * quadric_matrix[2][3],
            quadric_matrix[3][3],
        );
    }

    /// VTK: `vtkTriangle::ComputeCentroid`.
    pub fn compute_centroid(points: &Points, point_ids: Option<&[VtkIdType]>) -> (bool, [f64; 3]) {
        let ids = point_ids.unwrap_or(&[0, 1, 2]);
        assert!(
            ids.len() >= 3,
            "vtkTriangle::ComputeCentroid point id slice too short"
        );
        let mut centroid = [0.0; 3];
        for &id in ids.iter().take(3) {
            let point = points.get_point(id);
            for i in 0..3 {
                centroid[i] += point[i];
            }
        }
        for value in &mut centroid {
            *value /= 3.0;
        }
        (true, centroid)
    }

    fn closest_point_outside_triangle(
        x: [f64; 3],
        pt1: [f64; 3],
        pt2: [f64; 3],
        pt3: [f64; 3],
        weights: [f64; 3],
    ) -> (f64, Option<[f64; 3]>) {
        if weights[1] < 0.0 && weights[2] < 0.0 {
            Self::closest_of_point_and_two_lines(x, pt3, (pt1, pt3), (pt3, pt2))
        } else if weights[2] < 0.0 && weights[0] < 0.0 {
            Self::closest_of_point_and_two_lines(x, pt1, (pt1, pt3), (pt1, pt2))
        } else if weights[1] < 0.0 && weights[0] < 0.0 {
            Self::closest_of_point_and_two_lines(x, pt2, (pt2, pt3), (pt1, pt2))
        } else if weights[0] < 0.0 {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pt1, pt2);
            (dist2, Some(closest))
        } else if weights[1] < 0.0 {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pt2, pt3);
            (dist2, Some(closest))
        } else if weights[2] < 0.0 {
            let (dist2, _t, closest) = Line::distance_to_line_with_closest_point(x, pt1, pt3);
            (dist2, Some(closest))
        } else {
            debug_assert!(
                false,
                "Arrived in vtkTriangle::EvaluatePosition dead branch"
            );
            (0.0, Some([0.0, 0.0, 0.0]))
        }
    }

    fn closest_of_point_and_two_lines(
        x: [f64; 3],
        point: [f64; 3],
        line1: ([f64; 3], [f64; 3]),
        line2: ([f64; 3], [f64; 3]),
    ) -> (f64, Option<[f64; 3]>) {
        let dist2_point = distance2_between_points(x, point);
        let (dist2_line1, _t1, closest_point1) =
            Line::distance_to_line_with_closest_point(x, line1.0, line1.1);
        let (dist2_line2, _t2, closest_point2) =
            Line::distance_to_line_with_closest_point(x, line2.0, line2.1);

        let (mut dist2, mut closest) = if dist2_point < dist2_line1 {
            (dist2_point, point)
        } else {
            (dist2_line1, closest_point1)
        };
        if dist2_line2 < dist2 {
            dist2 = dist2_line2;
            closest = closest_point2;
        }
        (dist2, Some(closest))
    }

    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }
}

/// Rust return bundle for VTK `vtkTriangle::EvaluatePosition` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleEvaluatePosition {
    pub inside: i32,
    pub sub_id: i32,
    pub pcoords: [f64; 3],
    pub dist2: f64,
    pub weights: [f64; 3],
    pub closest_point: Option<[f64; 3]>,
}

/// Rust return bundle for VTK `vtkTriangle::IntersectWithLine` out-parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriangleIntersectWithLine {
    pub intersection: i32,
    pub t: f64,
    pub x: [f64; 3],
    pub pcoords: [f64; 3],
    pub sub_id: i32,
}

impl CellBaseApi for Triangle {
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

static EDGES: [[VtkIdType; 2]; 3] = [[0, 1], [1, 2], [2, 0]];

static TRIANGLE_CELL_PCOORDS: [f64; 9] = [
    0.0, 0.0, 0.0, //
    1.0, 0.0, 0.0, //
    0.0, 1.0, 0.0,
];

const TRIANGLE_INTERSECTION_EPS: f64 = 256.0 * f64::EPSILON;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TriangleOrientation {
    Colinear = 1,
    Clockwise = 2,
    Counterclockwise = 4,
}

fn triangle_plane_determinant(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> f64 {
    determinant3x3_from_values(
        a[0] - d[0],
        a[1] - d[1],
        a[2] - d[2],
        b[0] - d[0],
        b[1] - d[1],
        b[2] - d[2],
        c[0] - d[0],
        c[1] - d[1],
        c[2] - d[2],
    )
}

fn orientation(p1: [f64; 2], p2: [f64; 2], p3: [f64; 2]) -> TriangleOrientation {
    let v1 = [p2[0] - p1[0], p2[1] - p1[1]];
    let v2 = [p3[0] - p1[0], p3[1] - p1[1]];
    let signed_area = v1[0] * v2[1] - v1[1] * v2[0];
    if signed_area.abs() < TRIANGLE_INTERSECTION_EPS {
        TriangleOrientation::Colinear
    } else if signed_area > 0.0 {
        TriangleOrientation::Counterclockwise
    } else {
        TriangleOrientation::Clockwise
    }
}

fn orientation_sum(values: [TriangleOrientation; 3]) -> i32 {
    values[0] as i32 + values[1] as i32 + values[2] as i32
}

fn coplanar_triangles_intersect(
    p1: [f64; 2],
    mut q1: [f64; 2],
    mut r1: [f64; 2],
    mut p2: [f64; 2],
    mut q2: [f64; 2],
    mut r2: [f64; 2],
) -> i32 {
    use TriangleOrientation::{Clockwise, Colinear, Counterclockwise};

    if orientation(p1, q1, r1) == Clockwise {
        std::mem::swap(&mut q1, &mut r1);
    }
    if orientation(p2, q2, r2) == Clockwise {
        std::mem::swap(&mut q2, &mut r2);
    }

    let p1_orientation = [
        orientation(p2, q2, p1),
        orientation(q2, r2, p1),
        orientation(r2, p2, p1),
    ];
    let sum_of_signs = orientation_sum(p1_orientation);

    if sum_of_signs == 3 * Counterclockwise as i32
        || sum_of_signs == 2 * Colinear as i32 + Clockwise as i32
        || sum_of_signs == 2 * Colinear as i32 + Counterclockwise as i32
        || sum_of_signs == Colinear as i32 + 2 * Counterclockwise as i32
    {
        return 1;
    }

    let mut index = 0;
    let mut found_index = false;
    for i in 0..3 {
        if p1_orientation[i] == Counterclockwise {
            if p1_orientation[(i + 1) % 3] == Counterclockwise
                && p1_orientation[(i + 2) % 3] == Clockwise
            {
                index = i;
                found_index = true;
                break;
            }
            if p1_orientation[(i + 1) % 3] == Clockwise && p1_orientation[(i + 2) % 3] == Clockwise
            {
                index = i;
                found_index = true;
                break;
            }
            if p1_orientation[(i + 1) % 3] == Colinear && p1_orientation[(i + 2) % 3] == Clockwise {
                index = i;
                found_index = true;
                break;
            }
            if p1_orientation[(i + 1) % 3] == Clockwise && p1_orientation[(i + 2) % 3] == Colinear {
                index = i;
                found_index = true;
                break;
            }
        }
    }

    if !found_index {
        return 0;
    }

    let t2 = [p2, q2, r2];
    p2 = t2[index];
    q2 = t2[(index + 1) % 3];
    r2 = t2[(index + 2) % 3];

    if p1_orientation[(index + 1) % 3] == Counterclockwise {
        if orientation(r2, p2, q1) != Clockwise {
            if orientation(r2, p1, q1) != Clockwise {
                if orientation(p1, p2, q1) != Clockwise {
                    1
                } else if orientation(p1, p2, r1) != Clockwise {
                    (orientation(q1, r1, p2) != Clockwise) as i32
                } else {
                    0
                }
            } else {
                0
            }
        } else if orientation(r2, p2, r1) != Clockwise {
            if orientation(q1, r1, r2) == Clockwise {
                0
            } else {
                (orientation(p1, p2, r1) != Clockwise) as i32
            }
        } else {
            0
        }
    } else if orientation(r2, p2, q1) != Clockwise {
        if orientation(q2, r2, q1) != Clockwise {
            if orientation(p1, p2, q1) != Clockwise {
                (orientation(p1, q2, q1) != Counterclockwise) as i32
            } else if orientation(p1, p2, r1) == Clockwise {
                0
            } else {
                (orientation(p2, q1, r1) != Clockwise) as i32
            }
        } else if orientation(p1, q2, q1) != Counterclockwise {
            if orientation(q2, r2, r1) == Clockwise {
                0
            } else {
                (orientation(q1, r1, q2) != Clockwise) as i32
            }
        } else {
            0
        }
    } else if orientation(r2, p2, r1) == Clockwise {
        0
    } else if orientation(q1, r1, r2) != Clockwise {
        (orientation(r1, p1, p2) != Clockwise) as i32
    } else if orientation(q1, r1, q2) == Clockwise {
        0
    } else {
        (orientation(q2, r2, r1) != Clockwise) as i32
    }
}
