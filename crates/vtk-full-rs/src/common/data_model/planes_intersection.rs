use super::{Planes, PointsProjectedHull};
use crate::common::core::{
    math::{cross, determinant3x3, invert3x3, multiply3x3},
    AnyArray, FloatArray, Points, VtkIdType, VtkMTimeType, VTK_DOUBLE,
};

const VTK_SMALL_DOUBLE: f64 = 10e-5;
const INSIDE: i32 = 0;
const OUTSIDE: i32 = 1;
const STRADDLE: i32 = 2;
const XDIM: i32 = 0;
const YDIM: i32 = 1;
const ZDIM: i32 = 2;

/// VTK: `vtkPlanesIntersection`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanesIntersection {
    planes_base: Planes,
    planes: Option<Vec<[f64; 4]>>,
    region_pts: Option<PointsProjectedHull>,
}

impl PlanesIntersection {
    /// VTK: `vtkPlanesIntersection::New`.
    pub fn new() -> Self {
        Self {
            planes_base: Planes::new(),
            planes: None,
            region_pts: None,
        }
    }

    /// VTK: `vtkPlanesIntersection::SetRegionVertices(vtkPoints*)`.
    pub fn set_region_vertices(&mut self, v: &Points) {
        let mut region_pts = PointsProjectedHull::new();
        if v.get_data_type() == VTK_DOUBLE {
            region_pts.deep_copy(v);
        } else {
            region_pts.set_data_type_to_double();
            let npts = v.get_number_of_points();
            region_pts.set_number_of_points(npts);
            for i in 0..npts {
                region_pts.set_point(i, v.get_point(i));
            }
        }
        self.region_pts = Some(region_pts);
    }

    /// VTK: `vtkPlanesIntersection::SetRegionVertices(double*, int)`.
    pub fn set_region_vertices_from_slice(&mut self, v: &[[f64; 3]]) {
        let mut region_pts = PointsProjectedHull::new();
        region_pts.set_data_type_to_double();
        region_pts.set_number_of_points(v.len() as VtkIdType);
        for (i, point) in v.iter().copied().enumerate() {
            region_pts.set_point(i as VtkIdType, point);
        }
        self.region_pts = Some(region_pts);
    }

    /// VTK: `vtkPlanesIntersection::GetNumberOfRegionVertices`.
    pub fn get_number_of_region_vertices(&mut self) -> i32 {
        if self.region_pts.is_none() {
            self.compute_region_vertices();
        }
        self.region_pts
            .as_ref()
            .map_or(0, |region_pts| region_pts.get_number_of_points() as i32)
    }

    /// VTK: `vtkPlanesIntersection::GetNumRegionVertices`.
    pub fn get_num_region_vertices(&mut self) -> i32 {
        self.get_number_of_region_vertices()
    }

    /// VTK: `vtkPlanesIntersection::GetRegionVertices`.
    pub fn get_region_vertices(&mut self, v: &mut [[f64; 3]]) -> i32 {
        if self.region_pts.is_none() {
            self.compute_region_vertices();
        }
        let Some(region_pts) = self.region_pts.as_ref() else {
            return 0;
        };

        let npts = (region_pts.get_number_of_points() as usize).min(v.len());
        for (i, point) in v.iter_mut().take(npts).enumerate() {
            *point = region_pts.get_point(i as VtkIdType);
        }
        npts as i32
    }

    /// VTK: `vtkPlanesIntersection::IntersectsRegion`.
    pub fn intersects_region(&mut self, r: &mut Points) -> bool {
        let nplanes = self.get_number_of_planes();
        if nplanes < 4 {
            return false;
        }

        if self.region_pts.is_none() {
            self.compute_region_vertices();
            if self
                .region_pts
                .as_ref()
                .map_or(0, PointsProjectedHull::get_number_of_points)
                < 4
            {
                return false;
            }
        }

        if r.get_number_of_points() < 8 {
            return false;
        }

        let mut intersects = -1;
        let mut all_inside = false;

        if !self.intersects_bounding_box(r) {
            intersects = 0;
        } else if self.encloses_bounding_box(r) {
            intersects = 1;
        } else {
            if self.planes.is_none() {
                self.set_plane_equations();
            }
            all_inside = true;

            for plane in 0..nplanes {
                let where_ = self.evaluate_face_plane(plane, r);
                if all_inside && where_ != INSIDE {
                    all_inside = false;
                }
                if where_ == OUTSIDE {
                    intersects = 0;
                    break;
                }
            }
        }

        if intersects == -1 {
            if all_inside {
                intersects = 1;
            } else if !self.intersects_projection(r, XDIM)
                || !self.intersects_projection(r, YDIM)
                || !self.intersects_projection(r, ZDIM)
            {
                intersects = 0;
            } else {
                intersects = 1;
            }
        }

        intersects == 1
    }

    /// VTK: `vtkPlanesIntersection::PolygonIntersectsBBox`.
    pub fn polygon_intersects_bbox(bounds: [f64; 6], pts: &Points) -> bool {
        let mut pi = Self::new();
        pi.set_region_vertices(pts);

        let mut box_points = box_points(bounds);
        let mut intersects = -1;

        if !pi.intersects_bounding_box(&box_points) {
            intersects = 0;
        } else if pi.encloses_bounding_box(&box_points) {
            intersects = 1;
        }

        if intersects == -1 {
            let mut origin = Points::new();
            origin.set_number_of_points(1);
            origin.set_point(0, pts.get_point(0));

            let mut normal = AnyArray::Float(FloatArray::new());
            normal.set_number_of_components(3);
            normal.set_number_of_tuples(1);

            let npts = pts.get_number_of_points();
            let p0 = pts.get_point(0);
            let p1 = pts.get_point(1);
            let mut nvec = [0.0; 3];
            for p in 2..npts {
                let pp = pts.get_point(p);
                nvec = Self::compute_normal(p0, p1, pp);
                if Self::good_normal(nvec) {
                    break;
                }
            }

            normal
                .insert_numeric_tuple_from_f64_checked(0, &nvec)
                .expect("generated normal array must be numeric");

            pi.set_points(Some(&origin));
            pi.set_normals(Some(&normal));
            pi.set_plane_equations();

            let where_ = pi.evaluate_face_plane(0, &box_points);
            if where_ != STRADDLE {
                intersects = 0;
            }
        }

        if intersects == -1 {
            if !pi.intersects_projection(&mut box_points, XDIM)
                || !pi.intersects_projection(&mut box_points, YDIM)
                || !pi.intersects_projection(&mut box_points, ZDIM)
            {
                intersects = 0;
            } else {
                intersects = 1;
            }
        }

        intersects == 1
    }

    /// VTK: `vtkPlanesIntersection::ComputeNormal`.
    pub fn compute_normal(p1: [f64; 3], p2: [f64; 3], p3: [f64; 3]) -> [f64; 3] {
        let v1 = [p1[0] - p2[0], p1[1] - p2[1], p1[2] - p2[2]];
        let v2 = [p3[0] - p2[0], p3[1] - p2[1], p3[2] - p2[2]];
        cross(v1, v2)
    }

    /// VTK: `vtkPlanesIntersection::GoodNormal`.
    pub fn good_normal(n: [f64; 3]) -> bool {
        (n[0] < VTK_SMALL_DOUBLE)
            || (n[0] > VTK_SMALL_DOUBLE)
            || (n[1] < VTK_SMALL_DOUBLE)
            || (n[1] > VTK_SMALL_DOUBLE)
            || (n[2] < VTK_SMALL_DOUBLE)
            || (n[2] > VTK_SMALL_DOUBLE)
    }

    /// VTK: `vtkPlanesIntersection::EvaluatePlaneEquation`.
    pub fn evaluate_plane_equation(x: [f64; 3], p: [f64; 4]) -> f64 {
        x[0] * p[0] + x[1] * p[1] + x[2] * p[2] + p[3]
    }

    /// VTK: `vtkPlanesIntersection::PlaneEquation`.
    pub fn plane_equation(n: [f64; 3], x: [f64; 3]) -> [f64; 4] {
        [n[0], n[1], n[2], -(n[0] * x[0] + n[1] * x[1] + n[2] * x[2])]
    }

    /// VTK: `vtkPlanesIntersection::Invert3x3`.
    pub fn invert3x3(m: &mut [[f64; 3]; 3]) -> i32 {
        let det = determinant3x3(*m);
        if det > -VTK_SMALL_DOUBLE && det < VTK_SMALL_DOUBLE {
            return -1;
        }
        *m = invert3x3(*m);
        0
    }

    /// VTK: `vtkPlanes::SetPoints`.
    pub fn set_points(&mut self, points: Option<&Points>) {
        self.planes_base.set_points(points);
    }

    /// VTK: `vtkPlanes::GetPoints`.
    pub fn get_points(&self) -> Option<&Points> {
        self.planes_base.get_points()
    }

    /// VTK: `vtkPlanes::SetNormals`.
    pub fn set_normals(&mut self, normals: Option<&AnyArray>) {
        self.planes_base.set_normals(normals);
    }

    /// VTK: `vtkPlanes::GetNormals`.
    pub fn get_normals(&self) -> Option<&AnyArray> {
        self.planes_base.get_normals()
    }

    /// VTK: `vtkPlanes::GetNumberOfPlanes`.
    pub fn get_number_of_planes(&self) -> i32 {
        self.planes_base.get_number_of_planes()
    }

    /// VTK: `vtkPlanes::SetBounds`.
    pub fn set_bounds(&mut self, bounds: [f64; 6]) {
        self.planes_base.set_bounds(bounds);
    }

    /// VTK: `vtkPlanes::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = self.planes_base.print_self();
        result.push_str(&format!(
            "Planes: {}\nRegionPts: {}\n",
            if self.planes.is_some() {
                "defined"
            } else {
                "null"
            },
            if self.region_pts.is_some() {
                "defined"
            } else {
                "null"
            }
        ));

        if let (Some(points), Some(normals)) = (self.get_points(), self.get_normals()) {
            let npts = points.get_number_of_points();
            for i in 0..npts {
                let point = points.get_point(i);
                let normal = normal_at(normals, i);
                result.push_str(&format!(
                    "Origin {} {} {} Normal {} {} {}\n",
                    point[0], point[1], point[2], normal[0], normal[1], normal[2]
                ));
            }
        }

        if let Some(region_pts) = &self.region_pts {
            let npts = region_pts.get_number_of_points();
            for i in 0..npts {
                let point = region_pts.get_point(i);
                result.push_str(&format!("Vertex {} {} {}\n", point[0], point[1], point[2]));
            }
        }
        result
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkPlanesIntersection"
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.planes_base.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.planes_base.get_m_time()
    }

    fn set_plane_equations(&mut self) {
        let nplanes = self.get_number_of_planes();
        let mut equations = Vec::with_capacity(nplanes as usize);
        let Some(points) = self.get_points() else {
            self.planes = Some(equations);
            return;
        };
        let Some(normals) = self.get_normals() else {
            self.planes = Some(equations);
            return;
        };

        for i in 0..nplanes {
            let x = points.get_point(i as VtkIdType);
            let n = normal_at(normals, i as VtkIdType);
            equations.push(Self::plane_equation(n, x));
        }
        self.planes = Some(equations);
    }

    fn compute_region_vertices(&mut self) {
        self.region_pts = Some(PointsProjectedHull::new());
        let nplanes = self.get_number_of_planes();
        if nplanes <= 3 {
            return;
        }
        if self.planes.is_none() {
            self.set_plane_equations();
        }

        let mut nvertices = 0;
        for i in 0..nplanes {
            for j in (i + 1)..nplanes {
                for k in (j + 1)..nplanes {
                    let mut m = self.planes_matrix(i, j, k);
                    if Self::invert3x3(&mut m) != 0 {
                        continue;
                    }
                    let rhs = self.planes_rhs(i, j, k);
                    let testv = multiply3x3(m, rhs);

                    if self.duplicate(testv) || self.outside_region(testv) {
                        continue;
                    }

                    self.region_pts
                        .as_mut()
                        .expect("region points initialized")
                        .insert_point(nvertices, testv);
                    nvertices += 1;
                }
            }
        }
    }

    fn planes_matrix(&self, p1: i32, p2: i32, p3: i32) -> [[f64; 3]; 3] {
        let planes = self.planes.as_ref().expect("plane equations initialized");
        [
            [
                planes[p1 as usize][0],
                planes[p1 as usize][1],
                planes[p1 as usize][2],
            ],
            [
                planes[p2 as usize][0],
                planes[p2 as usize][1],
                planes[p2 as usize][2],
            ],
            [
                planes[p3 as usize][0],
                planes[p3 as usize][1],
                planes[p3 as usize][2],
            ],
        ]
    }

    fn planes_rhs(&self, p1: i32, p2: i32, p3: i32) -> [f64; 3] {
        let planes = self.planes.as_ref().expect("plane equations initialized");
        [
            -planes[p1 as usize][3],
            -planes[p2 as usize][3],
            -planes[p3 as usize][3],
        ]
    }

    fn duplicate(&self, testv: [f64; 3]) -> bool {
        let Some(region_pts) = self.region_pts.as_ref() else {
            return false;
        };
        let npts = region_pts.get_number_of_points();
        for i in 0..npts {
            if region_pts.get_point(i) == testv {
                return true;
            }
        }
        false
    }

    fn outside_region(&self, testv: [f64; 3]) -> bool {
        let Some(planes) = self.planes.as_ref() else {
            return false;
        };
        for plane in planes.iter().take(self.get_number_of_planes() as usize) {
            let fx = Self::evaluate_plane_equation(testv, *plane);
            if fx > VTK_SMALL_DOUBLE {
                return true;
            }
        }
        false
    }

    fn intersects_bounding_box(&self, r: &Points) -> bool {
        let Some(region_pts) = self.region_pts.as_ref() else {
            return false;
        };
        let box_bounds = r.get_bounds();
        let region_bounds = region_pts.get_bounds();
        !((box_bounds[1] < region_bounds[0])
            || (box_bounds[0] > region_bounds[1])
            || (box_bounds[3] < region_bounds[2])
            || (box_bounds[2] > region_bounds[3])
            || (box_bounds[5] < region_bounds[4])
            || (box_bounds[4] > region_bounds[5]))
    }

    fn encloses_bounding_box(&self, r: &Points) -> bool {
        let Some(region_pts) = self.region_pts.as_ref() else {
            return false;
        };
        let box_bounds = r.get_bounds();
        let region_bounds = region_pts.get_bounds();
        !((box_bounds[0] > region_bounds[0])
            || (box_bounds[1] < region_bounds[1])
            || (box_bounds[2] > region_bounds[2])
            || (box_bounds[3] < region_bounds[3])
            || (box_bounds[4] > region_bounds[4])
            || (box_bounds[5] < region_bounds[5]))
    }

    fn evaluate_face_plane(&self, plane: i32, r: &Points) -> i32 {
        let mut n = [0.0; 3];
        let bounds = r.get_bounds();
        let mut with_n = [0.0; 3];
        let mut opposite_n = [0.0; 3];

        if let Some(normals) = self.get_normals() {
            n = normal_at(normals, plane as VtkIdType);
        }

        for i in 0..3 {
            if n[i] < 0.0 {
                with_n[i] = bounds[i * 2];
                opposite_n[i] = bounds[i * 2 + 1];
            } else {
                with_n[i] = bounds[i * 2 + 1];
                opposite_n[i] = bounds[i * 2];
            }
        }

        let plane_equation =
            self.planes.as_ref().expect("plane equations initialized")[plane as usize];

        let neg_val = Self::evaluate_plane_equation(opposite_n, plane_equation);
        if neg_val > 0.0 {
            return OUTSIDE;
        }

        let pos_val = Self::evaluate_plane_equation(with_n, plane_equation);
        if pos_val < 0.0 {
            INSIDE
        } else {
            STRADDLE
        }
    }

    fn intersects_projection(&mut self, r: &mut Points, direction: i32) -> bool {
        let Some(region_pts) = self.region_pts.as_mut() else {
            return false;
        };
        match direction {
            XDIM => region_pts.rectangle_intersection_x(r),
            YDIM => region_pts.rectangle_intersection_y(r),
            ZDIM => region_pts.rectangle_intersection_z(r),
            _ => false,
        }
    }
}

impl Default for PlanesIntersection {
    fn default() -> Self {
        Self::new()
    }
}

fn normal_at(normals: &AnyArray, i: VtkIdType) -> [f64; 3] {
    let tuple = normals
        .numeric_tuple_as_f64_checked(i as usize)
        .expect("vtkPlanesIntersection normals must be a numeric data array");
    [tuple[0], tuple[1], tuple[2]]
}

fn box_points(bounds: [f64; 6]) -> Points {
    let mut box_points = Points::new();
    box_points.set_number_of_points(8);
    box_points.set_point(0, [bounds[0], bounds[2], bounds[4]]);
    box_points.set_point(1, [bounds[1], bounds[2], bounds[4]]);
    box_points.set_point(2, [bounds[1], bounds[3], bounds[4]]);
    box_points.set_point(3, [bounds[0], bounds[3], bounds[4]]);
    box_points.set_point(4, [bounds[0], bounds[2], bounds[5]]);
    box_points.set_point(5, [bounds[1], bounds[2], bounds[5]]);
    box_points.set_point(6, [bounds[1], bounds[3], bounds[5]]);
    box_points.set_point(7, [bounds[0], bounds[3], bounds[5]]);
    box_points
}
