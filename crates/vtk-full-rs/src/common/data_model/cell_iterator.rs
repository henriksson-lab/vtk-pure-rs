use crate::common::core::{IdList, IdTypeArray, Object, Points, VtkIdType, VtkMTimeType};

use super::{CellArray, CellType, CellTypeUtilities};

const UNINITIALIZED_FLAG: u8 = 0x0;
const CELL_TYPE_FLAG: u8 = 0x1;
const POINT_IDS_FLAG: u8 = 0x2;
const POINTS_FLAG: u8 = 0x4;
const FACES_FLAG: u8 = 0x8;

/// VTK: `vtkCellIterator`.
#[derive(Debug, Clone)]
pub struct CellIterator {
    object: Object,
    cell_type: i32,
    points: Points,
    point_ids: IdList,
    faces: CellArray,
    legacy_faces_container: IdList,
    cache_flags: u8,
}

impl CellIterator {
    /// VTK: `vtkCellIterator::vtkCellIterator`.
    #[allow(dead_code)]
    pub(crate) fn with_class_name(class_name: &'static str) -> Self {
        Self {
            object: Object::with_class_name(class_name),
            cell_type: CellType::Empty as i32,
            points: Points::new(),
            point_ids: IdList::new(),
            faces: CellArray::new(),
            legacy_faces_container: IdList::new(),
            cache_flags: UNINITIALIZED_FLAG,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn set_cell_type(&mut self, cell_type: i32) {
        self.cell_type = cell_type;
    }

    #[allow(dead_code)]
    pub(crate) fn points_mut(&mut self) -> &mut Points {
        &mut self.points
    }

    #[allow(dead_code)]
    pub(crate) fn point_ids_mut(&mut self) -> &mut IdList {
        &mut self.point_ids
    }

    #[allow(dead_code)]
    pub(crate) fn faces_mut(&mut self) -> &mut CellArray {
        &mut self.faces
    }

    fn reset_cache(&mut self) {
        self.cache_flags = UNINITIALIZED_FLAG;
        self.cell_type = CellType::Empty as i32;
    }

    fn set_cache(&mut self, flags: u8) {
        self.cache_flags |= flags;
    }

    fn check_cache(&self, flags: u8) -> bool {
        (self.cache_flags & flags) == flags
    }

    fn get_point_ids(&self) -> &IdList {
        &self.point_ids
    }

    fn get_points(&self) -> &Points {
        &self.points
    }

    fn get_cell_faces(&self) -> &CellArray {
        &self.faces
    }

    fn get_serialized_cell_faces(&mut self) -> &IdList {
        let mut tmp = IdTypeArray::new();
        self.faces.export_legacy_format(&mut tmp);

        self.legacy_faces_container.initialize();
        self.legacy_faces_container
            .insert_next_id(self.faces.get_number_of_cells());
        for value in tmp.as_slice() {
            self.legacy_faces_container.insert_next_id(*value);
        }

        &self.legacy_faces_container
    }

    /// VTK: `vtkCellIterator::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "CacheFlags: {}\nCellType: {}\nPoints:\n{}\nPointIds: {}\nFaces: number_of_cells={}\n",
            cache_flags_string(self.cache_flags),
            self.cell_type,
            self.points.print_self(),
            self.point_ids.get_number_of_ids(),
            self.faces.get_number_of_cells()
        )
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkCellIterator::IsTypeOf`.
    pub fn is_type_of(name: &str) -> bool {
        name == "vtkCellIterator" || Object::is_type_of(name)
    }

    /// VTK: `vtkCellIterator::IsA`.
    pub fn is_a(&self, name: &str) -> bool {
        Self::is_type_of(name)
    }

    /// VTK: `vtkCellIterator::GetNumberOfGenerationsFromBaseType`.
    pub fn get_number_of_generations_from_base_type(name: &str) -> VtkIdType {
        match name {
            "vtkCellIterator" => 0,
            "vtkObject" => 1,
            "vtkObjectBase" => 2,
            _ => Object::get_number_of_generations_from_base_type(name) as VtkIdType,
        }
    }

    /// VTK: `vtkCellIterator::GetNumberOfGenerationsFromBase`.
    pub fn get_number_of_generations_from_base(&self, name: &str) -> VtkIdType {
        Self::get_number_of_generations_from_base_type(name)
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }
}

/// VTK pure virtual and inline/default API for `vtkCellIterator`.
pub trait CellIteratorApi {
    fn cell_iterator(&self) -> &CellIterator;
    fn cell_iterator_mut(&mut self) -> &mut CellIterator;

    /// VTK: `vtkCellIterator::IsDoneWithTraversal`.
    fn is_done_with_traversal(&self) -> bool;

    /// VTK: `vtkCellIterator::GetCellId`.
    fn get_cell_id(&self) -> VtkIdType;

    /// VTK protected pure virtual: `vtkCellIterator::ResetToFirstCell`.
    fn reset_to_first_cell(&mut self);

    /// VTK protected pure virtual: `vtkCellIterator::IncrementToNextCell`.
    fn increment_to_next_cell(&mut self);

    /// VTK protected pure virtual: `vtkCellIterator::FetchCellType`.
    fn fetch_cell_type(&mut self);

    /// VTK protected pure virtual: `vtkCellIterator::FetchPointIds`.
    fn fetch_point_ids(&mut self);

    /// VTK protected pure virtual: `vtkCellIterator::FetchPoints`.
    fn fetch_points(&mut self);

    /// VTK protected virtual: `vtkCellIterator::FetchFaces`.
    fn fetch_faces(&mut self) {}

    /// VTK: `vtkCellIterator::InitTraversal`.
    fn init_traversal(&mut self) {
        self.reset_to_first_cell();
        self.cell_iterator_mut().reset_cache();
    }

    /// VTK: `vtkCellIterator::GoToNextCell`.
    fn go_to_next_cell(&mut self) {
        self.increment_to_next_cell();
        self.cell_iterator_mut().reset_cache();
    }

    /// VTK: `vtkCellIterator::GetCellType`.
    fn get_cell_type(&mut self) -> i32 {
        if !self.cell_iterator().check_cache(CELL_TYPE_FLAG) {
            self.fetch_cell_type();
            self.cell_iterator_mut().set_cache(CELL_TYPE_FLAG);
        }
        self.cell_iterator().cell_type
    }

    /// VTK: `vtkCellIterator::GetCellDimension`.
    fn get_cell_dimension(&mut self) -> i32 {
        let cell_type = self.get_cell_type();
        if cell_type < 0 || cell_type > u8::MAX as i32 {
            return -1;
        }
        CellTypeUtilities::get_dimension(cell_type as u8)
    }

    /// VTK: `vtkCellIterator::GetPointIds`.
    fn get_point_ids(&mut self) -> &IdList {
        if !self.cell_iterator().check_cache(POINT_IDS_FLAG) {
            self.fetch_point_ids();
            self.cell_iterator_mut().set_cache(POINT_IDS_FLAG);
        }
        self.cell_iterator().get_point_ids()
    }

    /// VTK: `vtkCellIterator::GetPoints`.
    fn get_points(&mut self) -> &Points {
        if !self.cell_iterator().check_cache(POINTS_FLAG) {
            self.fetch_points();
            self.cell_iterator_mut().set_cache(POINTS_FLAG);
        }
        self.cell_iterator().get_points()
    }

    /// VTK: `vtkCellIterator::GetCellFaces`.
    fn get_cell_faces(&mut self) -> &CellArray {
        if !self.cell_iterator().check_cache(FACES_FLAG) {
            self.fetch_faces();
            self.cell_iterator_mut().set_cache(FACES_FLAG);
        }
        self.cell_iterator().get_cell_faces()
    }

    /// VTK: `vtkCellIterator::GetSerializedCellFaces`.
    fn get_serialized_cell_faces(&mut self) -> &IdList {
        if !self.cell_iterator().check_cache(FACES_FLAG) {
            self.fetch_faces();
            self.cell_iterator_mut().set_cache(FACES_FLAG);
        }
        self.cell_iterator_mut().get_serialized_cell_faces()
    }

    /// VTK: `vtkCellIterator::GetNumberOfPoints`.
    fn get_number_of_points(&mut self) -> VtkIdType {
        if !self.cell_iterator().check_cache(POINT_IDS_FLAG) {
            self.fetch_point_ids();
            self.cell_iterator_mut().set_cache(POINT_IDS_FLAG);
        }
        self.cell_iterator().point_ids.get_number_of_ids()
    }

    /// VTK: `vtkCellIterator::GetNumberOfFaces`.
    fn get_number_of_faces(&mut self) -> VtkIdType {
        match self.get_cell_type() {
            x if NO_FACE_CELL_TYPES.contains(&x) => 0,
            x if TETRA_CELL_TYPES.contains(&x) => 4,
            x if FIVE_FACE_CELL_TYPES.contains(&x) => 5,
            x if SIX_FACE_CELL_TYPES.contains(&x) => 6,
            x if x == CellType::PentagonalPrism as i32 => 7,
            x if x == CellType::HexagonalPrism as i32 => 8,
            x if x == CellType::Polyhedron as i32 => self.get_cell_faces().get_number_of_cells(),
            _ => 0,
        }
    }
}

const NO_FACE_CELL_TYPES: &[i32] = &[
    CellType::Empty as i32,
    CellType::Vertex as i32,
    CellType::PolyVertex as i32,
    CellType::Line as i32,
    CellType::PolyLine as i32,
    CellType::Triangle as i32,
    CellType::TriangleStrip as i32,
    CellType::Polygon as i32,
    CellType::Pixel as i32,
    CellType::Quad as i32,
    CellType::QuadraticEdge as i32,
    CellType::QuadraticTriangle as i32,
    CellType::QuadraticQuad as i32,
    CellType::QuadraticPolygon as i32,
    CellType::BiQuadraticQuad as i32,
    CellType::QuadraticLinearQuad as i32,
    CellType::BiQuadraticTriangle as i32,
    CellType::CubicLine as i32,
    CellType::ConvexPointSet as i32,
    CellType::HigherOrderCurve as i32,
    CellType::HigherOrderTriangle as i32,
    CellType::HigherOrderQuadrilateral as i32,
    CellType::LagrangeCurve as i32,
    CellType::LagrangeTriangle as i32,
    CellType::LagrangeQuadrilateral as i32,
    CellType::BezierCurve as i32,
    CellType::BezierTriangle as i32,
    CellType::BezierQuadrilateral as i32,
];

const TETRA_CELL_TYPES: &[i32] = &[
    CellType::Tetra as i32,
    CellType::QuadraticTetra as i32,
    CellType::HigherOrderTetrahedron as i32,
    CellType::LagrangeTetrahedron as i32,
    CellType::BezierTetrahedron as i32,
];

const FIVE_FACE_CELL_TYPES: &[i32] = &[
    CellType::Pyramid as i32,
    CellType::QuadraticPyramid as i32,
    CellType::TriQuadraticPyramid as i32,
    CellType::HigherOrderPyramid as i32,
    CellType::Wedge as i32,
    CellType::QuadraticWedge as i32,
    CellType::QuadraticLinearWedge as i32,
    CellType::BiQuadraticQuadraticWedge as i32,
    CellType::HigherOrderWedge as i32,
    CellType::LagrangeWedge as i32,
    CellType::BezierWedge as i32,
];

const SIX_FACE_CELL_TYPES: &[i32] = &[
    CellType::Voxel as i32,
    CellType::Hexahedron as i32,
    CellType::QuadraticHexahedron as i32,
    CellType::TriQuadraticHexahedron as i32,
    CellType::HigherOrderHexahedron as i32,
    CellType::BiQuadraticQuadraticHexahedron as i32,
    CellType::LagrangeHexahedron as i32,
    CellType::BezierHexahedron as i32,
];

fn cache_flags_string(cache_flags: u8) -> String {
    if cache_flags == UNINITIALIZED_FLAG {
        return "UninitializedFlag".to_string();
    }

    let mut flags = Vec::new();
    if (cache_flags & CELL_TYPE_FLAG) == CELL_TYPE_FLAG {
        flags.push("CellTypeFlag");
    }
    if (cache_flags & POINT_IDS_FLAG) == POINT_IDS_FLAG {
        flags.push("PointIdsFlag");
    }
    if (cache_flags & POINTS_FLAG) == POINTS_FLAG {
        flags.push("PointsFlag");
    }
    if (cache_flags & FACES_FLAG) == FACES_FLAG {
        flags.push("FacesFlag");
    }
    flags.join(" | ")
}
