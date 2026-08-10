use crate::common::core::{Points, TimeStamp, VtkIdType};
use std::cmp::Ordering;

const XDIM: usize = 0;
const YDIM: usize = 1;
const ZDIM: usize = 2;
const XMIN: usize = 0;
const XMAX: usize = 1;
const YMIN: usize = 2;
const YMAX: usize = 3;

/// VTK: `vtkPointsProjectedHull`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointsProjectedHull {
    points: Points,
    pts: Vec<[f64; 3]>,
    npts: VtkIdType,
    pts_time: u64,
    ccw_hull: [Vec<[f64; 2]>; 3],
    hull_bbox: [[f32; 4]; 3],
    hull_time: [TimeStamp; 3],
}

impl PointsProjectedHull {
    /// VTK: `vtkPointsProjectedHull::New`.
    pub fn new() -> Self {
        Self {
            points: Points::new(),
            pts: Vec::new(),
            npts: 0,
            pts_time: 0,
            ccw_hull: [Vec::new(), Vec::new(), Vec::new()],
            hull_bbox: [[0.0; 4]; 3],
            hull_time: [TimeStamp::new(), TimeStamp::new(), TimeStamp::new()],
        }
    }

    /// VTK: `vtkPointsProjectedHull::Initialize`.
    pub fn initialize(&mut self) {
        self.clear_allocations();
        self.init_flags();
        self.points.initialize();
    }

    /// VTK: `vtkPointsProjectedHull::Reset`.
    pub fn reset(&mut self) {
        self.initialize();
    }

    /// VTK: `vtkPointsProjectedHull::Update`.
    pub fn update(&mut self) {
        self.clear_allocations();
        self.init_flags();
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionX(vtkPoints*)`.
    pub fn rectangle_intersection_x(&mut self, r: &mut Points) -> bool {
        r.modified();
        let bounds = r.get_bounds();
        self.rectangle_intersection_x_components(
            bounds[((XDIM * 2 + 2) % 6) as usize],
            bounds[((XDIM * 2 + 2) % 6 + 1) as usize],
            bounds[((XDIM * 2 + 4) % 6) as usize],
            bounds[((XDIM * 2 + 4) % 6 + 1) as usize],
        )
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionX`.
    pub fn rectangle_intersection_x_components(
        &mut self,
        ymin: f64,
        ymax: f64,
        zmin: f64,
        zmax: f64,
    ) -> bool {
        self.ensure_hull(XDIM);
        self.rectangle_intersection(ymin, ymax, zmin, zmax, XDIM)
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionX`.
    pub fn rectangle_intersection_x_components_f32(
        &mut self,
        ymin: f32,
        ymax: f32,
        zmin: f32,
        zmax: f32,
    ) -> bool {
        self.rectangle_intersection_x_components(ymin as f64, ymax as f64, zmin as f64, zmax as f64)
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionY(vtkPoints*)`.
    pub fn rectangle_intersection_y(&mut self, r: &mut Points) -> bool {
        r.modified();
        let bounds = r.get_bounds();
        self.rectangle_intersection_y_components(
            bounds[((YDIM * 2 + 2) % 6) as usize],
            bounds[((YDIM * 2 + 2) % 6 + 1) as usize],
            bounds[((YDIM * 2 + 4) % 6) as usize],
            bounds[((YDIM * 2 + 4) % 6 + 1) as usize],
        )
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionY`.
    pub fn rectangle_intersection_y_components(
        &mut self,
        zmin: f64,
        zmax: f64,
        xmin: f64,
        xmax: f64,
    ) -> bool {
        self.ensure_hull(YDIM);
        self.rectangle_intersection(zmin, zmax, xmin, xmax, YDIM)
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionY`.
    pub fn rectangle_intersection_y_components_f32(
        &mut self,
        zmin: f32,
        zmax: f32,
        xmin: f32,
        xmax: f32,
    ) -> bool {
        self.rectangle_intersection_y_components(zmin as f64, zmax as f64, xmin as f64, xmax as f64)
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionZ(vtkPoints*)`.
    pub fn rectangle_intersection_z(&mut self, r: &mut Points) -> bool {
        r.modified();
        let bounds = r.get_bounds();
        self.rectangle_intersection_z_components(
            bounds[((ZDIM * 2 + 2) % 6) as usize],
            bounds[((ZDIM * 2 + 2) % 6 + 1) as usize],
            bounds[((ZDIM * 2 + 4) % 6) as usize],
            bounds[((ZDIM * 2 + 4) % 6 + 1) as usize],
        )
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionZ`.
    pub fn rectangle_intersection_z_components(
        &mut self,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
    ) -> bool {
        self.ensure_hull(ZDIM);
        self.rectangle_intersection(xmin, xmax, ymin, ymax, ZDIM)
    }

    /// VTK: `vtkPointsProjectedHull::RectangleIntersectionZ`.
    pub fn rectangle_intersection_z_components_f32(
        &mut self,
        xmin: f32,
        xmax: f32,
        ymin: f32,
        ymax: f32,
    ) -> bool {
        self.rectangle_intersection_z_components(xmin as f64, xmax as f64, ymin as f64, ymax as f64)
    }

    /// VTK: `vtkPointsProjectedHull::GetCCWHullX`.
    pub fn get_ccw_hull_x(&mut self, pts: &mut [f64]) -> i32 {
        self.get_ccw_hull(pts, XDIM)
    }

    /// VTK: `vtkPointsProjectedHull::GetCCWHullX`.
    pub fn get_ccw_hull_x_f32(&mut self, pts: &mut [f32]) -> i32 {
        self.get_ccw_hull_f32(pts, XDIM)
    }

    /// VTK: `vtkPointsProjectedHull::GetCCWHullY`.
    pub fn get_ccw_hull_y(&mut self, pts: &mut [f64]) -> i32 {
        self.get_ccw_hull(pts, YDIM)
    }

    /// VTK: `vtkPointsProjectedHull::GetCCWHullY`.
    pub fn get_ccw_hull_y_f32(&mut self, pts: &mut [f32]) -> i32 {
        self.get_ccw_hull_f32(pts, YDIM)
    }

    /// VTK: `vtkPointsProjectedHull::GetCCWHullZ`.
    pub fn get_ccw_hull_z(&mut self, pts: &mut [f64]) -> i32 {
        self.get_ccw_hull(pts, ZDIM)
    }

    /// VTK: `vtkPointsProjectedHull::GetCCWHullZ`.
    pub fn get_ccw_hull_z_f32(&mut self, pts: &mut [f32]) -> i32 {
        self.get_ccw_hull_f32(pts, ZDIM)
    }

    /// VTK: `vtkPointsProjectedHull::GetSizeCCWHullX`.
    pub fn get_size_ccw_hull_x(&mut self) -> i32 {
        self.ensure_hull(XDIM);
        self.ccw_hull[XDIM].len() as i32
    }

    /// VTK: `vtkPointsProjectedHull::GetSizeCCWHullY`.
    pub fn get_size_ccw_hull_y(&mut self) -> i32 {
        self.ensure_hull(YDIM);
        self.ccw_hull[YDIM].len() as i32
    }

    /// VTK: `vtkPointsProjectedHull::GetSizeCCWHullZ`.
    pub fn get_size_ccw_hull_z(&mut self) -> i32 {
        self.ensure_hull(ZDIM);
        self.ccw_hull[ZDIM].len() as i32
    }

    /// VTK: `vtkPoints::InsertNextPoint`.
    pub fn insert_next_point(&mut self, point: [f64; 3]) -> VtkIdType {
        let id = self.points.insert_next_point(point);
        self.update();
        id
    }

    /// VTK: `vtkPoints::GetPoint`.
    pub fn get_point(&self, idx: VtkIdType) -> [f64; 3] {
        self.points.get_point(idx)
    }

    /// VTK: `vtkPoints::SetPoint`.
    pub fn set_point(&mut self, idx: VtkIdType, point: [f64; 3]) {
        self.points.set_point(idx, point);
        self.update();
    }

    /// VTK: `vtkPoints::InsertPoint`.
    pub fn insert_point(&mut self, idx: VtkIdType, point: [f64; 3]) {
        self.points.insert_point(idx, point);
        self.update();
    }

    /// VTK: `vtkPoints::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.points.get_number_of_points()
    }

    /// VTK: `vtkPoints::SetNumberOfPoints`.
    pub fn set_number_of_points(&mut self, num_points: VtkIdType) {
        self.points.set_number_of_points(num_points);
        self.update();
    }

    /// VTK: `vtkPoints::DeepCopy`.
    pub fn deep_copy(&mut self, other: &Points) {
        self.points.deep_copy(other);
        self.update();
    }

    /// VTK: `vtkPoints::SetDataTypeToDouble`.
    pub fn set_data_type_to_double(&mut self) {
        self.points.set_data_type_to_double();
        self.update();
    }

    /// VTK: `vtkPoints::GetDataType`.
    pub fn get_data_type(&self) -> i32 {
        self.points.get_data_type()
    }

    /// VTK: `vtkPoints::GetBounds`.
    pub fn get_bounds(&self) -> [f64; 6] {
        self.points.get_bounds()
    }

    /// VTK: `vtkPoints::Modified`.
    pub fn modified(&mut self) {
        self.points.modified();
    }

    /// VTK: `vtkPoints::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.points.get_m_time()
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        "vtkPointsProjectedHull"
    }

    /// VTK: `vtkPointsProjectedHull::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nPts: {}\nNpts: {}\nPtsTime: {}\nCCWHull X: {}\nHullBBox X: [{}, {}] [{}, {}] HullSize X: {}\nHullTime X: {}\nCCWHull Y: {}\nHullBBox Y: [{}, {}] [{}, {}] HullSize Y: {}\nHullTime Y: {}\nCCWHull Z: {}\nHullBBox Z: [{}, {}] [{}, {}] HullSize Z: {}\nHullTime Z: {}\n",
            self.points.print_self(),
            if self.pts.is_empty() { "null" } else { "defined" },
            self.npts,
            self.pts_time,
            if self.ccw_hull[0].is_empty() { "null" } else { "defined" },
            self.hull_bbox[0][0],
            self.hull_bbox[0][1],
            self.hull_bbox[0][2],
            self.hull_bbox[0][3],
            self.ccw_hull[0].len(),
            self.hull_time[0].get_m_time(),
            if self.ccw_hull[1].is_empty() { "null" } else { "defined" },
            self.hull_bbox[1][0],
            self.hull_bbox[1][1],
            self.hull_bbox[1][2],
            self.hull_bbox[1][3],
            self.ccw_hull[1].len(),
            self.hull_time[1].get_m_time(),
            if self.ccw_hull[2].is_empty() { "null" } else { "defined" },
            self.hull_bbox[2][0],
            self.hull_bbox[2][1],
            self.hull_bbox[2][2],
            self.hull_bbox[2][3],
            self.ccw_hull[2].len(),
            self.hull_time[2].get_m_time()
        )
    }

    fn init_flags(&mut self) {
        self.pts.clear();
        self.npts = 0;
        for dim in 0..3 {
            self.ccw_hull[dim].clear();
            self.hull_bbox[dim] = [0.0; 4];
        }
    }

    fn clear_allocations(&mut self) {
        for dim in 0..3 {
            self.ccw_hull[dim].clear();
        }
        self.pts.clear();
    }

    fn ensure_hull(&mut self, dim: usize) {
        if self.ccw_hull[dim].is_empty() || self.get_m_time() > self.hull_time[dim].get_m_time() {
            self.graham_scan_algorithm(dim);
        }
    }

    fn get_ccw_hull(&mut self, pts: &mut [f64], dim: usize) -> i32 {
        self.ensure_hull(dim);
        let copylen = self.ccw_hull[dim].len().min(pts.len() / 2);
        if copylen == 0 {
            return 0;
        }
        for (i, point) in self.ccw_hull[dim].iter().take(copylen).enumerate() {
            pts[i * 2] = point[0];
            pts[i * 2 + 1] = point[1];
        }
        copylen as i32
    }

    fn get_ccw_hull_f32(&mut self, pts: &mut [f32], dim: usize) -> i32 {
        self.ensure_hull(dim);
        let copylen = self.ccw_hull[dim].len().min(pts.len() / 2);
        if copylen == 0 {
            return 0;
        }
        for (i, point) in self.ccw_hull[dim].iter().take(copylen).enumerate() {
            pts[i * 2] = point[0] as f32;
            pts[i * 2 + 1] = point[1] as f32;
        }
        copylen as i32
    }

    fn rectangle_intersection(
        &self,
        hmin: f64,
        hmax: f64,
        vmin: f64,
        vmax: f64,
        dim: usize,
    ) -> bool {
        if !self.rectangle_bounding_box_intersection(hmin, hmax, vmin, vmax, dim) {
            return false;
        }
        if self.rectangle_outside(hmin, hmax, vmin, vmax, dim) {
            return false;
        }
        true
    }

    fn graham_scan_algorithm(&mut self, dir: usize) -> i32 {
        if self.npts == 0 || self.get_m_time() > self.pts_time {
            self.get_points();
        }
        if self.npts == 0 {
            return 0;
        }

        let (horiz_axis, vert_axis) = match dir {
            XDIM => (YDIM, ZDIM),
            YDIM => (ZDIM, XDIM),
            ZDIM => (XDIM, YDIM),
            _ => unreachable!("vtkPointsProjectedHull has three projection directions"),
        };

        let mut hull_pts: Vec<[f64; 2]> = self
            .pts
            .iter()
            .map(|point| [point[horiz_axis], point[vert_axis]])
            .collect();
        hull_pts.sort_by(|a, b| partial_cmp_f64(a[1], b[1]));

        let mut first_id = 0usize;
        for i in 1..hull_pts.len() {
            if hull_pts[i][1] != hull_pts[0][1] {
                break;
            }
            if hull_pts[i][0] > hull_pts[first_id][0] {
                first_id = i;
            }
        }

        let first_pt = hull_pts[first_id];
        if first_id != 0 {
            hull_pts.swap(0, first_id);
        }

        let mut compacted = Vec::with_capacity(hull_pts.len());
        compacted.push(hull_pts[0]);
        let mut dups = 0usize;
        for point in hull_pts.iter().copied().skip(1) {
            if point[1] == first_pt[1] && point[0] == first_pt[0] {
                dups += 1;
            } else {
                compacted.push(point);
            }
        }
        hull_pts = compacted;
        let mut n_hull_pts = self.npts as usize - dups;
        if n_hull_pts == 0 {
            return 0;
        }
        hull_pts.truncate(n_hull_pts);

        if hull_pts.len() > 2 {
            hull_pts[1..].sort_by(|a, b| ccw_compare(first_pt, *a, *b));
        }

        n_hull_pts = remove_extras(&mut hull_pts, n_hull_pts);
        hull_pts.truncate(n_hull_pts);

        if n_hull_pts > 2 {
            let mut top = 1usize;
            for i in 2..n_hull_pts {
                let newpos = position_in_hull(&hull_pts, top, hull_pts[i]);
                hull_pts[newpos] = hull_pts[i];
                top = newpos;
            }
            n_hull_pts = top + 1;
            hull_pts.truncate(n_hull_pts);
        }

        if n_hull_pts > 0 {
            let mut x0 = hull_pts[0][0];
            let mut x1 = hull_pts[0][0];
            let mut y0 = hull_pts[0][1];
            let mut y1 = hull_pts[0][1];
            for point in hull_pts.iter().take(n_hull_pts).skip(1) {
                if point[0] < x0 {
                    x0 = point[0];
                } else if point[0] > x1 {
                    x1 = point[0];
                }
                if point[1] < y0 {
                    y0 = point[1];
                } else if point[1] > y1 {
                    y1 = point[1];
                }
            }
            self.hull_bbox[dir][XMIN] = x0 as f32;
            self.hull_bbox[dir][XMAX] = x1 as f32;
            self.hull_bbox[dir][YMIN] = y0 as f32;
            self.hull_bbox[dir][YMAX] = y1 as f32;
        }

        self.ccw_hull[dir] = hull_pts;
        self.hull_time[dir].modified();
        0
    }

    fn get_points(&mut self) {
        self.npts = self.points.get_number_of_points();
        self.pts.clear();
        self.pts.reserve(self.npts as usize);
        for i in 0..self.npts {
            self.pts.push(self.points.get_point(i));
        }
        self.pts_time = self.get_m_time();
    }

    fn rectangle_bounding_box_intersection(
        &self,
        hmin: f64,
        hmax: f64,
        vmin: f64,
        vmax: f64,
        dim: usize,
    ) -> bool {
        let r2_bounds = self.hull_bbox[dim];
        !((hmin > r2_bounds[XMAX] as f64)
            || (hmax < r2_bounds[XMIN] as f64)
            || (vmin > r2_bounds[YMAX] as f64)
            || (vmax < r2_bounds[YMIN] as f64))
    }

    fn rectangle_outside(&self, hmin: f64, hmax: f64, vmin: f64, vmax: f64, dir: usize) -> bool {
        let npts = self.ccw_hull[dir].len();
        if npts < 2 {
            return true;
        }
        if npts == 2 {
            return rectangle_outside_1d_polygon(hmin, hmax, vmin, vmax, &self.ccw_hull[dir]);
        }

        let hull = &self.ccw_hull[dir];
        let mut inside_pt = [hull[0][0] + hull[2][0], hull[0][1] + hull[2][1]];
        if npts == 3 {
            inside_pt[0] += hull[1][0];
            inside_pt[1] += hull[1][1];
            inside_pt[0] /= 3.0;
            inside_pt[1] /= 3.0;
        } else {
            inside_pt[0] /= 2.0;
            inside_pt[1] /= 2.0;
        }

        for i in 0..(npts - 1) {
            if outside_line(hmin, hmax, vmin, vmax, hull[i], hull[i + 1], inside_pt) {
                return true;
            }
        }
        false
    }
}

impl Default for PointsProjectedHull {
    fn default() -> Self {
        Self::new()
    }
}

fn vtk_is_left(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2]) -> f64 {
    (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p2[0] - p0[0]) * (p1[1] - p0[1])
}

fn partial_cmp_f64(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

fn ccw_compare(first_pt: [f64; 2], a: [f64; 2], b: [f64; 2]) -> Ordering {
    let val = vtk_is_left(first_pt, a, b);
    if val < 0.0 {
        Ordering::Greater
    } else if val == 0.0 {
        Ordering::Equal
    } else {
        Ordering::Less
    }
}

fn distance(p1: [f64; 2], p2: [f64; 2]) -> f64 {
    (p1[0] - p2[0]) * (p1[0] - p2[0]) + (p1[1] - p2[1]) * (p1[1] - p2[1])
}

fn remove_extras(pts: &mut [[f64; 2]], n: usize) -> usize {
    let mut prev = 0usize;
    for i in 1..n {
        let mut skip_me = false;
        if pts[i] == pts[prev] {
            skip_me = true;
        } else if prev >= 1 {
            let where_ = vtk_is_left(pts[0], pts[prev], pts[i]);
            if where_ == 0.0 {
                let d1 = distance(pts[0], pts[prev]);
                let d2 = distance(pts[0], pts[i]);
                if d2 > d1 {
                    pts[prev] = pts[i];
                }
                skip_me = true;
            }
        }

        if !skip_me {
            prev += 1;
            if prev < i {
                pts[prev] = pts[i];
            }
        }
    }
    prev + 1
}

fn position_in_hull(pts: &[[f64; 2]], top: usize, pt: [f64; 2]) -> usize {
    let mut p2 = top;
    let mut p1 = p2.saturating_sub(1);
    while p2 > 0 {
        let where_ = vtk_is_left(pts[p1], pts[p2], pt);
        if where_ > 0.0 {
            break;
        }
        p2 -= 1;
        p1 = p1.saturating_sub(1);
    }
    p2 + 1
}

fn outside_horizontal_line(vmin: f64, vmax: f64, p0: [f64; 2], inside_pt: [f64; 2]) -> bool {
    if inside_pt[1] > p0[1] {
        vmax <= p0[1]
    } else {
        vmin >= p0[1]
    }
}

fn outside_vertical_line(hmin: f64, hmax: f64, p0: [f64; 2], inside_pt: [f64; 2]) -> bool {
    if inside_pt[0] > p0[0] {
        hmax <= p0[0]
    } else {
        hmin >= p0[0]
    }
}

fn outside_line(
    hmin: f64,
    hmax: f64,
    vmin: f64,
    vmax: f64,
    p0: [f64; 2],
    p1: [f64; 2],
    inside_pt: [f64; 2],
) -> bool {
    if (p1[1] - p0[1]) == 0.0 {
        return outside_horizontal_line(vmin, vmax, p0, inside_pt);
    }
    if (p1[0] - p0[0]) == 0.0 {
        return outside_vertical_line(hmin, hmax, p0, inside_pt);
    }

    let ip = vtk_is_left(p0, p1, inside_pt);
    let pts = [[hmin, vmin], [hmin, vmax], [hmax, vmax], [hmax, vmin]];
    for point in pts {
        let rp = vtk_is_left(p0, p1, point);
        if ((rp < 0.0) && (ip < 0.0)) || ((rp > 0.0) && (ip > 0.0)) {
            return false;
        }
    }
    true
}

fn rectangle_outside_1d_polygon(
    hmin: f64,
    hmax: f64,
    vmin: f64,
    vmax: f64,
    hull: &[[f64; 2]],
) -> bool {
    let p0 = hull[0];
    let p1 = hull[1];
    let pts = [[hmin, vmin], [hmin, vmax], [hmax, vmax], [hmax, vmin]];
    let mut reference = 0.0;
    for point in pts {
        let side = vtk_is_left(p0, p1, point);
        if reference != 0.0 {
            if side != reference {
                return false;
            }
        } else if side != 0.0 {
            reference = side;
        }
    }
    true
}
