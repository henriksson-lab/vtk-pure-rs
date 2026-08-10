use std::ffi::c_void;

use crate::common::core::{VtkIdType, VtkMTimeType};

use super::{Cell, CellBaseApi};

/// VTK: `vtkOrderedTriangulator*`.
pub(crate) type OrderedTriangulatorHandle = *mut c_void;

/// VTK: `vtkTetra*` used by `vtkCell3D::Clip`.
pub(crate) type Cell3DClipTetraHandle = *mut c_void;

/// VTK: `vtkDoubleArray*` used by `vtkCell3D::Clip`.
pub(crate) type Cell3DClipScalarsHandle = *mut c_void;

/// Shared base storage for VTK 3D cell implementations.
///
/// VTK origin: selected audited symbols from `VTK/Common/DataModel/vtkCell3D.h`
/// and `VTK/Common/DataModel/vtkCell3D.cxx`.
#[derive(Debug)]
pub struct Cell3D {
    cell: Cell,
    triangulator: OrderedTriangulatorHandle,
    merge_tolerance: f64,
    clip_tetra: Cell3DClipTetraHandle,
    clip_scalars: Cell3DClipScalarsHandle,
}

impl Cell3D {
    /// VTK: protected `vtkCell3D::vtkCell3D`.
    pub(crate) fn new() -> Self {
        Self::with_class_name("vtkCell3D")
    }

    /// VTK: protected `vtkCell3D::vtkCell3D` for subclass base construction.
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            cell: Cell::with_class_name(class_name),
            triangulator: std::ptr::null_mut(),
            merge_tolerance: 0.01,
            clip_tetra: std::ptr::null_mut(),
            clip_scalars: std::ptr::null_mut(),
        }
    }

    /// VTK: `vtkCell3D::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "{}\nMergeTolerance: {}",
            self.cell.print_self(),
            self.merge_tolerance
        )
    }

    /// VTK: `vtkCell3D::GetCellDimension`.
    pub fn get_cell_dimension(&self) -> i32 {
        3
    }

    /// VTK: `vtkCell3D::SetMergeTolerance`.
    pub fn set_merge_tolerance(&mut self, merge_tolerance: f64) {
        self.merge_tolerance = merge_tolerance.clamp(0.0001, 0.25);
    }

    /// VTK: `vtkCell3D::GetMergeTolerance`.
    pub fn get_merge_tolerance(&self) -> f64 {
        self.merge_tolerance
    }

    /// Access to the embedded `vtkCell` base storage.
    pub(crate) fn cell(&self) -> &Cell {
        &self.cell
    }

    /// Mutable access to the embedded `vtkCell` base storage.
    pub(crate) fn cell_mut(&mut self) -> &mut Cell {
        &mut self.cell
    }

    /// VTK protected field: `Triangulator`.
    pub(crate) fn get_triangulator(&self) -> OrderedTriangulatorHandle {
        self.triangulator
    }

    /// VTK protected field: `ClipTetra`.
    pub(crate) fn get_clip_tetra(&self) -> Cell3DClipTetraHandle {
        self.clip_tetra
    }

    /// VTK protected field: `ClipScalars`.
    pub(crate) fn get_clip_scalars(&self) -> Cell3DClipScalarsHandle {
        self.clip_scalars
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.cell.get_class_name()
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.cell.get_m_time()
    }
}

/// VTK virtual surface added by `vtkCell3D`.
pub trait Cell3DApi: CellBaseApi {
    /// Access to the embedded `vtkCell3D` base storage.
    fn cell_3d(&self) -> &Cell3D;

    /// Mutable access to the embedded `vtkCell3D` base storage.
    fn cell_3d_mut(&mut self) -> &mut Cell3D;

    /// VTK: `vtkCell3D::GetEdgePoints`.
    fn get_edge_points(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2];

    /// VTK: `vtkCell3D::GetFacePoints`.
    fn get_face_points(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType]);

    /// VTK: `vtkCell3D::GetEdgeToAdjacentFaces`.
    fn get_edge_to_adjacent_faces(&self, edge_id: VtkIdType) -> &'static [VtkIdType; 2];

    /// VTK: `vtkCell3D::GetFaceToAdjacentFaces`.
    fn get_face_to_adjacent_faces(&self, face_id: VtkIdType) -> (VtkIdType, &'static [VtkIdType]);

    /// VTK: `vtkCell3D::GetPointToIncidentEdges`.
    fn get_point_to_incident_edges(&self, point_id: VtkIdType)
        -> (VtkIdType, &'static [VtkIdType]);

    /// VTK: `vtkCell3D::GetPointToIncidentFaces`.
    fn get_point_to_incident_faces(&self, point_id: VtkIdType)
        -> (VtkIdType, &'static [VtkIdType]);

    /// VTK: `vtkCell3D::GetPointToOneRingPoints`.
    fn get_point_to_one_ring_points(
        &self,
        point_id: VtkIdType,
    ) -> (VtkIdType, &'static [VtkIdType]);

    /// VTK: `vtkCell3D::GetCentroid`.
    fn get_centroid(&self) -> (bool, [f64; 3]);

    /// VTK: `vtkCell3D::IsInsideOut`.
    fn is_inside_out(&self) -> bool;
}
