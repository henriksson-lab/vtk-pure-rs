use std::cell::Cell as InteriorCell;

use crate::common::core::{
    math::{distance2_between_points, norm, normalize},
    IdList, Object, ObjectBaseApi, Points, VtkIdType, VTK_DOUBLE,
};

use super::{Tetra, Triangle};

/// VTK: `VTK_CELL_SIZE`.
pub const VTK_CELL_SIZE: usize = 512;

/// VTK: `VTK_TOL`.
pub const VTK_TOL: f64 = 1.0e-5;

/// Shared base storage for VTK cell implementations.
///
/// VTK origin: selected audited symbols from `VTK/Common/DataModel/vtkCell.h`
/// and `VTK/Common/DataModel/vtkCell.cxx`.
#[derive(Debug)]
pub struct Cell {
    object: Object,
    points: Points,
    point_ids: IdList,
    bounds: InteriorCell<[f64; 6]>,
}

impl Cell {
    /// VTK: protected `vtkCell::vtkCell`.
    pub(crate) fn new() -> Self {
        Self::with_class_name("vtkCell")
    }

    /// VTK: protected `vtkCell::vtkCell` for subclass base construction.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            points: Points::new_with_data_type(VTK_DOUBLE),
            point_ids: IdList::new(),
            bounds: InteriorCell::new([0.0; 6]),
        }
    }

    /// VTK: `vtkCell::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut text = self.object.print_self();
        let num_ids = self.point_ids.get_number_of_ids();
        text.push_str(&format!("\nNumber Of Points: {num_ids}\n"));

        if num_ids > 0 {
            let bounds = self.get_bounds();
            text.push_str("Bounds:\n");
            text.push_str(&format!("  Xmin,Xmax: ({}, {})\n", bounds[0], bounds[1]));
            text.push_str(&format!("  Ymin,Ymax: ({}, {})\n", bounds[2], bounds[3]));
            text.push_str(&format!("  Zmin,Zmax: ({}, {})\n", bounds[4], bounds[5]));
            text.push_str("  Point ids are: ");
            for i in 0..num_ids {
                if i > 0 {
                    text.push_str(", ");
                }
                text.push_str(&self.point_ids.get_id(i).to_string());
            }
            text.push('\n');
        }

        text
    }

    /// VTK: `vtkCell::Initialize(int, const vtkIdType*, vtkPoints*)`.
    pub fn initialize_with_point_ids(&mut self, npts: i32, pts: &[VtkIdType], p: &Points) {
        self.point_ids.reset();
        self.points.reset();

        let npts = npts.max(0) as usize;
        assert!(
            pts.len() >= npts,
            "vtkCell::Initialize point id slice shorter than npts"
        );

        for (i, &point_id) in pts.iter().take(npts).enumerate() {
            self.point_ids.insert_id(i as VtkIdType, point_id);
            self.points
                .insert_point(i as VtkIdType, p.get_point(point_id));
        }
    }

    /// VTK: `vtkCell::Initialize(int, vtkPoints*)`.
    pub fn initialize_from_points(&mut self, npts: i32, p: &Points) {
        self.point_ids.reset();
        self.points.reset();

        for i in 0..npts.max(0) {
            let point_id = i as VtkIdType;
            self.point_ids.insert_id(point_id, point_id);
            self.points.insert_point(point_id, p.get_point(point_id));
        }
    }

    /// VTK: virtual no-argument `vtkCell::Initialize`.
    pub fn initialize(&mut self) {}

    /// VTK: `vtkCell::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.points.shallow_copy(&source.points);
        self.point_ids.shallow_copy(&source.point_ids);
        self.bounds.set(source.bounds.get());
    }

    /// VTK: `vtkCell::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.points.deep_copy(&source.points);
        self.point_ids.deep_copy(&source.point_ids);
        self.bounds.set(source.bounds.get());
    }

    /// VTK: `vtkCell::ComputeBoundingSphere`.
    pub fn compute_bounding_sphere(&self) -> ([f64; 3], f64) {
        match self.points.get_number_of_points() {
            0 => return ([f64::NAN; 3], f64::NAN),
            1 => return (self.points.get_point(0), 0.0),
            2 => {
                let p0 = self.points.get_point(0);
                let p1 = self.points.get_point(1);
                let center = [
                    0.5 * (p0[0] + p1[0]),
                    0.5 * (p0[1] + p1[1]),
                    0.5 * (p0[2] + p1[2]),
                ];
                return (center, distance2_between_points(center, p0));
            }
            3 => {
                let (valid, center) = Triangle::compute_centroid(&self.points, None);
                if valid {
                    return (
                        center,
                        distance2_between_points(center, self.points.get_point(0)),
                    );
                }
            }
            4 => {
                let (valid, center) = Tetra::compute_centroid(&self.points, None);
                if valid {
                    return (
                        center,
                        distance2_between_points(center, self.points.get_point(0)),
                    );
                }
            }
            _ => {}
        }

        let point_count = self.points.get_number_of_points();
        let x = self.points.get_point(0);
        let mut yid = 1;
        let mut zid = 0;

        let mut dist2 = 0.0;
        for id in 1..point_count {
            let tmpdist2 = distance2_between_points(self.points.get_point(id), x);
            if tmpdist2 > dist2 {
                dist2 = tmpdist2;
                yid = id;
            }
        }

        let y = self.points.get_point(yid);
        dist2 = 0.0;
        for id in 0..point_count {
            let tmpdist2 = distance2_between_points(self.points.get_point(id), y);
            if tmpdist2 > dist2 {
                dist2 = tmpdist2;
                zid = id;
            }
        }

        let z = self.points.get_point(zid);
        let mut center = [
            0.5 * (y[0] + z[0]),
            0.5 * (y[1] + z[1]),
            0.5 * (y[2] + z[2]),
        ];
        dist2 = distance2_between_points(y, center);

        loop {
            let mut outside_point_id = point_count;
            for point_id in 0..point_count {
                if distance2_between_points(self.points.get_point(point_id), center) > dist2 {
                    outside_point_id = point_id;
                    break;
                }
            }

            if outside_point_id == point_count {
                return (center, dist2);
            }

            let point = self.points.get_point(outside_point_id);
            let mut v = [
                point[0] - center[0],
                point[1] - center[1],
                point[2] - center[2],
            ];
            let d = 0.5 * (norm(&v) - dist2.sqrt());
            normalize(&mut v);
            center[0] += d * v[0];
            center[1] += d * v[1];
            center[2] += d * v[2];

            let max_center_epsilon =
                f64::EPSILON * center[0].abs().max(center[1].abs()).max(center[2].abs());
            dist2 += (dist2 * f64::EPSILON).max(max_center_epsilon * max_center_epsilon);
            dist2 = dist2.max(distance2_between_points(point, center));
        }
    }

    /// VTK: `vtkCell::IsLinear`.
    pub fn is_linear(&self) -> i32 {
        1
    }

    /// VTK: `vtkCell::RequiresInitialization`.
    pub fn requires_initialization(&self) -> i32 {
        0
    }

    /// VTK: `vtkCell::IsExplicitCell`.
    pub fn is_explicit_cell(&self) -> i32 {
        0
    }

    /// VTK: `vtkCell::RequiresExplicitFaceRepresentation`.
    pub fn requires_explicit_face_representation(&self) -> i32 {
        0
    }

    /// VTK: `vtkCell::GetPoints`.
    pub fn get_points(&self) -> &Points {
        &self.points
    }

    /// Rust representation of mutable access to VTK public field `Points`.
    pub(crate) fn get_points_mut(&mut self) -> &mut Points {
        &mut self.points
    }

    /// VTK: `vtkCell::GetNumberOfPoints`.
    pub fn get_number_of_points(&self) -> VtkIdType {
        self.point_ids.get_number_of_ids()
    }

    /// VTK: `vtkCell::GetPointIds`.
    pub fn get_point_ids(&self) -> &IdList {
        &self.point_ids
    }

    /// Rust representation of mutable access to VTK public field `PointIds`.
    pub(crate) fn get_point_ids_mut(&mut self) -> &mut IdList {
        &mut self.point_ids
    }

    /// VTK: `vtkCell::GetPointId`.
    pub fn get_point_id(&self, pt_id: i32) -> VtkIdType {
        self.point_ids.get_id(pt_id as VtkIdType)
    }

    /// VTK: `vtkCell::GetBounds(double[6])` and `vtkCell::GetBounds()`.
    pub fn get_bounds(&self) -> [f64; 6] {
        let bounds = self.points.get_bounds();
        self.bounds.set(bounds);
        bounds
    }

    /// VTK protected field: `Bounds`.
    pub(crate) fn cached_bounds(&self) -> [f64; 6] {
        self.bounds.get()
    }

    /// VTK: `vtkCell::GetLength2`.
    pub fn get_length2(&self) -> f64 {
        let bounds = self.get_bounds();
        let mut length2 = 0.0;
        for i in 0..3 {
            let diff = bounds[2 * i + 1] - bounds[2 * i];
            length2 += diff * diff;
        }
        length2
    }

    /// VTK: `vtkCell::GetParametricCenter`.
    pub fn get_parametric_center(&self) -> (i32, [f64; 3]) {
        (0, [0.5, 0.5, 0.5])
    }

    /// VTK: `vtkCell::GetParametricDistance`.
    pub fn get_parametric_distance(&self, pcoords: [f64; 3]) -> f64 {
        let mut p_dist_max = 0.0_f64;
        for value in pcoords {
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

    /// VTK: `vtkCell::IsPrimaryCell`.
    pub fn is_primary_cell(&self) -> i32 {
        1
    }

    /// VTK: `vtkCell::GetParametricCoords`.
    pub fn get_parametric_coords(&self) -> Option<&'static [f64]> {
        None
    }

    /// VTK: `vtkCell::InterpolateFunctions`.
    pub fn interpolate_functions(&self, _pcoords: [f64; 3], _weights: &mut [f64]) {}

    /// VTK: `vtkCell::InterpolateDerivs`.
    pub fn interpolate_derivs(&self, _pcoords: [f64; 3], _derivs: &mut [f64]) {}

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> u64 {
        self.object
            .get_m_time()
            .max(self.points.get_m_time())
            .max(self.point_ids.get_m_time())
    }
}

/// VTK virtual surface for concrete `vtkCell` subclasses.
pub trait CellBaseApi {
    /// Access to the embedded `vtkCell` base storage.
    fn cell(&self) -> &Cell;

    /// Mutable access to the embedded `vtkCell` base storage.
    fn cell_mut(&mut self) -> &mut Cell;

    /// VTK: `vtkCell::GetCellType`.
    fn get_cell_type(&self) -> i32;

    /// VTK: `vtkCell::GetCellDimension`.
    fn get_cell_dimension(&self) -> i32;

    /// VTK: `vtkCell::GetNumberOfEdges`.
    fn get_number_of_edges(&self) -> i32;

    /// VTK: `vtkCell::GetNumberOfFaces`.
    fn get_number_of_faces(&self) -> i32;

    /// VTK: `vtkCell::TriangulateLocalIds`.
    fn triangulate_local_ids(&self, index: i32, pt_ids: &mut IdList) -> i32;

    /// VTK: `vtkCell::Triangulate`.
    fn triangulate(&self, index: i32, pt_ids: &mut IdList, pts: &mut Points) -> i32 {
        if self.triangulate_local_ids(index, pt_ids) == 0 {
            return 0;
        }

        pts.set_number_of_points(pt_ids.get_number_of_ids());
        for i in 0..pt_ids.get_number_of_ids() {
            let local_id = pt_ids.get_id(i);
            pts.set_point(i, self.cell().get_points().get_point(local_id));
            pt_ids.set_id(i, self.cell().get_point_ids().get_id(local_id));
        }

        1
    }

    /// VTK: `vtkCell::TriangulateIds`.
    fn triangulate_ids(&self, index: i32, pt_ids: &mut IdList) -> i32 {
        if self.triangulate_local_ids(index, pt_ids) == 0 {
            return 0;
        }

        for i in 0..pt_ids.get_number_of_ids() {
            let local_id = pt_ids.get_id(i);
            pt_ids.set_id(i, self.cell().get_point_ids().get_id(local_id));
        }

        1
    }
}
