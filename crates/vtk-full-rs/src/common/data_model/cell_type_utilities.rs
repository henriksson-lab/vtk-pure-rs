/// Number of cell type ids currently reserved by VTK.
///
/// VTK origin: `VTK/Common/DataModel/vtkCellType.h` (`VTK_NUMBER_OF_CELL_TYPES`).
pub const NUMBER_OF_CELL_TYPES: i32 = 82;

/// Stateless wrapper for VTK's `vtkCellTypeUtilities` object.
///
/// VTK origin: `vtkCellTypeUtilities`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellTypeUtilities;

impl CellTypeUtilities {
    /// VTK: `vtkCellTypeUtilities::New`.
    pub fn new() -> Self {
        Self
    }

    /// VTK: `vtkCellTypeUtilities::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!("vtkCellTypeUtilities {{ number_of_cell_types: {NUMBER_OF_CELL_TYPES} }}")
    }

    /// VTK: `vtkCellTypeUtilities::GetDimension`.
    pub fn get_dimension(type_id: u8) -> i32 {
        get_dimension(type_id)
    }

    /// VTK: `vtkCellTypeUtilities::IsLinear`.
    pub fn is_linear(type_id: u8) -> bool {
        is_linear(type_id)
    }

    /// VTK: `vtkCellTypeUtilities::GetTypeAsString`.
    pub fn get_type_as_string(type_id: i32) -> &'static str {
        get_type_as_string(type_id)
    }

    /// VTK: `vtkCellTypeUtilities::GetClassNameFromTypeId`.
    pub fn get_class_name_from_type_id(type_id: i32) -> &'static str {
        get_class_name_from_type_id(type_id)
    }

    /// VTK: `vtkCellTypeUtilities::GetTypeIdFromName`.
    pub fn get_type_id_from_name(name: &str) -> i32 {
        get_type_id_from_name(name)
    }

    /// VTK: `vtkCellTypeUtilities::GetTypeIdFromClassName`.
    pub fn get_type_id_from_class_name(classname: Option<&str>) -> i32 {
        get_type_id_from_class_name(classname)
    }
}

/// VTK cell type identifiers needed by `vtkCellTypeUtilities`.
///
/// VTK origin: `VTK/Common/DataModel/vtkCellType.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CellType {
    Empty = 0,
    Vertex = 1,
    PolyVertex = 2,
    Line = 3,
    PolyLine = 4,
    Triangle = 5,
    TriangleStrip = 6,
    Polygon = 7,
    Pixel = 8,
    Quad = 9,
    Tetra = 10,
    Voxel = 11,
    Hexahedron = 12,
    Wedge = 13,
    Pyramid = 14,
    PentagonalPrism = 15,
    HexagonalPrism = 16,
    QuadraticEdge = 21,
    QuadraticTriangle = 22,
    QuadraticQuad = 23,
    QuadraticTetra = 24,
    QuadraticHexahedron = 25,
    QuadraticWedge = 26,
    QuadraticPyramid = 27,
    BiQuadraticQuad = 28,
    TriQuadraticHexahedron = 29,
    QuadraticLinearQuad = 30,
    QuadraticLinearWedge = 31,
    BiQuadraticQuadraticWedge = 32,
    BiQuadraticQuadraticHexahedron = 33,
    BiQuadraticTriangle = 34,
    CubicLine = 35,
    QuadraticPolygon = 36,
    TriQuadraticPyramid = 37,
    ConvexPointSet = 41,
    Polyhedron = 42,
    HigherOrderCurve = 60,
    HigherOrderTriangle = 61,
    HigherOrderQuadrilateral = 62,
    HigherOrderPolygon = 63,
    HigherOrderTetrahedron = 64,
    HigherOrderWedge = 65,
    HigherOrderPyramid = 66,
    HigherOrderHexahedron = 67,
    LagrangeCurve = 68,
    LagrangeTriangle = 69,
    LagrangeQuadrilateral = 70,
    LagrangeTetrahedron = 71,
    LagrangeHexahedron = 72,
    LagrangeWedge = 73,
    LagrangePyramid = 74,
    BezierCurve = 75,
    BezierTriangle = 76,
    BezierQuadrilateral = 77,
    BezierTetrahedron = 78,
    BezierHexahedron = 79,
    BezierWedge = 80,
    BezierPyramid = 81,
}

impl CellType {
    pub(crate) fn from_id(type_id: u8) -> Option<Self> {
        match type_id {
            0 => Some(Self::Empty),
            1 => Some(Self::Vertex),
            2 => Some(Self::PolyVertex),
            3 => Some(Self::Line),
            4 => Some(Self::PolyLine),
            5 => Some(Self::Triangle),
            6 => Some(Self::TriangleStrip),
            7 => Some(Self::Polygon),
            8 => Some(Self::Pixel),
            9 => Some(Self::Quad),
            10 => Some(Self::Tetra),
            11 => Some(Self::Voxel),
            12 => Some(Self::Hexahedron),
            13 => Some(Self::Wedge),
            14 => Some(Self::Pyramid),
            15 => Some(Self::PentagonalPrism),
            16 => Some(Self::HexagonalPrism),
            21 => Some(Self::QuadraticEdge),
            22 => Some(Self::QuadraticTriangle),
            23 => Some(Self::QuadraticQuad),
            24 => Some(Self::QuadraticTetra),
            25 => Some(Self::QuadraticHexahedron),
            26 => Some(Self::QuadraticWedge),
            27 => Some(Self::QuadraticPyramid),
            28 => Some(Self::BiQuadraticQuad),
            29 => Some(Self::TriQuadraticHexahedron),
            30 => Some(Self::QuadraticLinearQuad),
            31 => Some(Self::QuadraticLinearWedge),
            32 => Some(Self::BiQuadraticQuadraticWedge),
            33 => Some(Self::BiQuadraticQuadraticHexahedron),
            34 => Some(Self::BiQuadraticTriangle),
            35 => Some(Self::CubicLine),
            36 => Some(Self::QuadraticPolygon),
            37 => Some(Self::TriQuadraticPyramid),
            41 => Some(Self::ConvexPointSet),
            42 => Some(Self::Polyhedron),
            60 => Some(Self::HigherOrderCurve),
            61 => Some(Self::HigherOrderTriangle),
            62 => Some(Self::HigherOrderQuadrilateral),
            63 => Some(Self::HigherOrderPolygon),
            64 => Some(Self::HigherOrderTetrahedron),
            65 => Some(Self::HigherOrderWedge),
            66 => Some(Self::HigherOrderPyramid),
            67 => Some(Self::HigherOrderHexahedron),
            68 => Some(Self::LagrangeCurve),
            69 => Some(Self::LagrangeTriangle),
            70 => Some(Self::LagrangeQuadrilateral),
            71 => Some(Self::LagrangeTetrahedron),
            72 => Some(Self::LagrangeHexahedron),
            73 => Some(Self::LagrangeWedge),
            74 => Some(Self::LagrangePyramid),
            75 => Some(Self::BezierCurve),
            76 => Some(Self::BezierTriangle),
            77 => Some(Self::BezierQuadrilateral),
            78 => Some(Self::BezierTetrahedron),
            79 => Some(Self::BezierHexahedron),
            80 => Some(Self::BezierWedge),
            81 => Some(Self::BezierPyramid),
            _ => None,
        }
    }

    /// VTK: `vtkCellTypeUtilities::GetDimension`.
    pub(crate) fn dimension(self) -> i32 {
        match self {
            Self::Empty | Self::Vertex | Self::PolyVertex => 0,
            Self::Line
            | Self::PolyLine
            | Self::QuadraticEdge
            | Self::CubicLine
            | Self::LagrangeCurve
            | Self::BezierCurve => 1,
            Self::Triangle
            | Self::TriangleStrip
            | Self::Polygon
            | Self::Pixel
            | Self::Quad
            | Self::QuadraticTriangle
            | Self::QuadraticQuad
            | Self::QuadraticPolygon
            | Self::QuadraticLinearQuad
            | Self::BiQuadraticQuad
            | Self::BiQuadraticTriangle
            | Self::HigherOrderTriangle
            | Self::HigherOrderQuadrilateral
            | Self::LagrangeTriangle
            | Self::LagrangeQuadrilateral
            | Self::BezierTriangle
            | Self::BezierQuadrilateral => 2,
            Self::Tetra
            | Self::Voxel
            | Self::Hexahedron
            | Self::Wedge
            | Self::Pyramid
            | Self::PentagonalPrism
            | Self::HexagonalPrism
            | Self::QuadraticTetra
            | Self::QuadraticHexahedron
            | Self::QuadraticWedge
            | Self::QuadraticPyramid
            | Self::QuadraticLinearWedge
            | Self::BiQuadraticQuadraticHexahedron
            | Self::BiQuadraticQuadraticWedge
            | Self::TriQuadraticHexahedron
            | Self::TriQuadraticPyramid
            | Self::ConvexPointSet
            | Self::Polyhedron
            | Self::HigherOrderCurve
            | Self::HigherOrderPolygon
            | Self::HigherOrderTetrahedron
            | Self::HigherOrderWedge
            | Self::HigherOrderPyramid
            | Self::HigherOrderHexahedron
            | Self::LagrangeTetrahedron
            | Self::LagrangeHexahedron
            | Self::LagrangeWedge
            | Self::LagrangePyramid
            | Self::BezierTetrahedron
            | Self::BezierHexahedron
            | Self::BezierWedge
            | Self::BezierPyramid => 3,
        }
    }

    /// VTK: `vtkCellTypeUtilities::IsLinear`.
    #[cfg(test)]
    pub(crate) fn is_linear(self) -> bool {
        is_linear(self as u8)
    }

    /// VTK: `vtkCellTypeUtilities::GetTypeAsString`.
    #[cfg(test)]
    pub(crate) fn type_as_string(self) -> &'static str {
        get_type_as_string(self as i32)
    }

    /// VTK: `vtkCellTypeUtilities::GetClassNameFromTypeId`.
    /// VTK: `vtkCellTypes::GetClassNameFromTypeId`.
    #[cfg(test)]
    pub(crate) fn get_class_name(self) -> &'static str {
        get_class_name_from_type_id(self as i32)
    }
}

/// VTK: `vtkCellTypeUtilities::GetDimension`.
fn get_dimension(type_id: u8) -> i32 {
    match CellType::from_id(type_id) {
        Some(cell_type) => cell_type.dimension(),
        None if i32::from(type_id) < NUMBER_OF_CELL_TYPES => 3,
        None => 0,
    }
}

/// VTK: `vtkCellTypeUtilities::IsLinear`.
fn is_linear(type_id: u8) -> bool {
    type_id <= CellType::HexagonalPrism as u8
        || type_id == CellType::ConvexPointSet as u8
        || type_id == CellType::Polyhedron as u8
}

/// VTK: `vtkCellTypeUtilities::GetTypeAsString`.
fn get_type_as_string(type_id: i32) -> &'static str {
    match type_id {
        0 => "Empty Cell",
        1 => "Vertex",
        2 => "Polyvertex",
        3 => "Line",
        4 => "Polyline",
        5 => "Triangle",
        6 => "Triangle Strip",
        7 => "Polygon",
        8 => "Pixel",
        9 => "Quadrilateral",
        10 => "Tetrahedron",
        11 => "Voxel",
        12 => "Hexahedron",
        13 => "Wedge",
        14 => "Pyramid",
        15 => "Pentagonal Prism",
        16 => "Hexagonal Prism",
        21 => "Quadratic Edge",
        22 => "Quadratic Triangle",
        23 => "Quadratic Quadrilateral",
        24 => "Quadratic Tetrahedron",
        25 => "Quadratic Hexahedron",
        26 => "Quadratic Wedge",
        27 => "Quadratic Pyramid",
        28 => "Bi-Quadratic Quadrilateral",
        29 => "Tri-Quadratic Hexahedron",
        30 => "Quadratic Linear Quadrilateral",
        31 => "Quadratic Linear Wedge",
        32 => "Bi-Quadratic Quadratic Wedge",
        33 => "Bi-Quadratic Quadratic Hexahedron",
        34 => "Bi-Quadratic Triangle",
        35 => "Cubic Line",
        36 => "Quadratic Polygon",
        37 => "Tri-Quadratic Pyramid",
        41 => "Convex Pointset",
        42 => "Polyhedron",
        60 => "Higher Order Curve",
        61 => "Higher Order Triangle",
        62 => "Higher Order Quadrilateral",
        64 => "Higher Order Tetrahedron",
        65 => "Higher Order Wedge",
        67 => "Higher Order Hexahedron",
        68 => "Lagrange Curve",
        69 => "Lagrange Triangle",
        70 => "Lagrange Quadrilateral",
        71 => "Lagrange Tetrahedron",
        72 => "Lagrange Hexahedron",
        73 => "Lagrange Wedge",
        75 => "Bezier Curve",
        76 => "Bezier Triangle",
        77 => "Bezier Quadrilateral",
        78 => "Bezier Tetrahedron",
        79 => "Bezier Hexahedron",
        80 => "Bezier Wedge",
        _ => "",
    }
}

/// VTK: `vtkCellTypeUtilities::GetClassNameFromTypeId`.
fn get_class_name_from_type_id(type_id: i32) -> &'static str {
    match type_id {
        0 => "vtkEmptyCell",
        1 => "vtkVertex",
        2 => "vtkPolyVertex",
        3 => "vtkLine",
        4 => "vtkPolyLine",
        5 => "vtkTriangle",
        6 => "vtkTriangleStrip",
        7 => "vtkPolygon",
        8 => "vtkPixel",
        9 => "vtkQuad",
        10 => "vtkTetra",
        11 => "vtkVoxel",
        12 => "vtkHexahedron",
        13 => "vtkWedge",
        14 => "vtkPyramid",
        15 => "vtkPentagonalPrism",
        16 => "vtkHexagonalPrism",
        21 => "vtkQuadraticEdge",
        22 => "vtkQuadraticTriangle",
        23 => "vtkQuadraticQuad",
        24 => "vtkQuadraticTetra",
        25 => "vtkQuadraticHexahedron",
        26 => "vtkQuadraticWedge",
        27 => "vtkQuadraticPyramid",
        28 => "vtkBiQuadraticQuad",
        29 => "vtkTriQuadraticHexahedron",
        30 => "vtkQuadraticLinearQuad",
        31 => "vtkQuadraticLinearWedge",
        32 => "vtkBiQuadraticQuadraticWedge",
        33 => "vtkBiQuadraticQuadraticHexahedron",
        34 => "vtkBiQuadraticTriangle",
        35 => "vtkCubicLine",
        36 => "vtkQuadraticPolygon",
        37 => "vtkTriQuadraticPyramid",
        41 => "vtkConvexPointSet",
        42 => "vtkPolyhedron",
        60 => "vtkHigherOrderCurve",
        61 => "vtkHigherOrderTriangle",
        62 => "vtkHigherOrderQuadrilateral",
        64 => "vtkHigherOrderTetra",
        65 => "vtkHigherOrderWedge",
        67 => "vtkHigherOrderHexahedron",
        68 => "vtkLagrangeCurve",
        69 => "vtkLagrangeTriangle",
        70 => "vtkLagrangeQuadrilateral",
        71 => "vtkLagrangeTetra",
        72 => "vtkLagrangeHexahedron",
        73 => "vtkLagrangeWedge",
        75 => "vtkBezierCurve",
        76 => "vtkBezierTriangle",
        77 => "vtkBezierQuadrilateral",
        78 => "vtkBezierTetra",
        79 => "vtkBezierHexahedron",
        80 => "vtkBezierWedge",
        _ => "UnknownClass",
    }
}

/// VTK: `vtkCellTypeUtilities::GetTypeIdFromName`.
fn get_type_id_from_name(name: &str) -> i32 {
    match name {
        "Empty Cell" => 0,
        "Vertex" => 1,
        "Polyvertex" => 2,
        "Line" => 3,
        "Polyline" => 4,
        "Triangle" => 5,
        "Triangle Strip" => 6,
        "Polygon" => 7,
        "Pixel" => 8,
        "Quadrilateral" => 9,
        "Tetrahedron" => 10,
        "Voxel" => 11,
        "Hexahedron" => 12,
        "Wedge" => 13,
        "Pyramid" => 14,
        "Pentagonal Prism" => 15,
        "Hexagonal Prism" => 16,
        "Quadratic Edge" => 21,
        "Quadratic Triangle" => 22,
        "Quadratic Quadrilateral" => 23,
        "Quadratic Tetrahedron" => 24,
        "Quadratic Hexahedron" => 25,
        "Quadratic Wedge" => 26,
        "Quadratic Pyramid" => 27,
        "Bi-Quadratic Quadrilateral" => 28,
        "Tri-Quadratic Hexahedron" => 29,
        "Quadratic Linear Quadrilateral" => 30,
        "Quadratic Linear Wedge" => 31,
        "Bi-Quadratic Quadratic Wedge" => 32,
        "Bi-Quadratic Quadratic Hexahedron" => 33,
        "Bi-Quadratic Triangle" => 34,
        "Cubic Line" => 35,
        "Quadratic Polygon" => 36,
        "Tri-Quadratic Pyramid" => 37,
        "Convex Pointset" => 41,
        "Polyhedron" => 42,
        "Higher Order Curve" => 60,
        "Higher Order Triangle" => 61,
        "Higher Order Quadrilateral" => 62,
        "Higher Order Tetrahedron" => 64,
        "Higher Order Wedge" => 65,
        "Higher Order Hexahedron" => 67,
        "Lagrange Curve" => 68,
        "Lagrange Triangle" => 69,
        "Lagrange Quadrilateral" => 70,
        "Lagrange Tetrahedron" => 71,
        "Lagrange Hexahedron" => 72,
        "Lagrange Wedge" => 73,
        "Bezier Curve" => 75,
        "Bezier Triangle" => 76,
        "Bezier Quadrilateral" => 77,
        "Bezier Tetrahedron" => 78,
        "Bezier Hexahedron" => 79,
        "Bezier Wedge" => 80,
        _ => CellType::Empty as i32,
    }
}

/// VTK: `vtkCellTypeUtilities::GetTypeIdFromClassName`.
fn get_type_id_from_class_name(classname: Option<&str>) -> i32 {
    let Some(classname) = classname else {
        return -1;
    };

    match classname {
        "vtkEmptyCell" => 0,
        "vtkVertex" => 1,
        "vtkPolyVertex" => 2,
        "vtkLine" => 3,
        "vtkPolyLine" => 4,
        "vtkTriangle" => 5,
        "vtkTriangleStrip" => 6,
        "vtkPolygon" => 7,
        "vtkPixel" => 8,
        "vtkQuad" => 9,
        "vtkTetra" => 10,
        "vtkVoxel" => 11,
        "vtkHexahedron" => 12,
        "vtkWedge" => 13,
        "vtkPyramid" => 14,
        "vtkPentagonalPrism" => 15,
        "vtkHexagonalPrism" => 16,
        "vtkQuadraticEdge" => 21,
        "vtkQuadraticTriangle" => 22,
        "vtkQuadraticQuad" => 23,
        "vtkQuadraticTetra" => 24,
        "vtkQuadraticHexahedron" => 25,
        "vtkQuadraticWedge" => 26,
        "vtkQuadraticPyramid" => 27,
        "vtkBiQuadraticQuad" => 28,
        "vtkTriQuadraticHexahedron" => 29,
        "vtkQuadraticLinearQuad" => 30,
        "vtkQuadraticLinearWedge" => 31,
        "vtkBiQuadraticQuadraticWedge" => 32,
        "vtkBiQuadraticQuadraticHexahedron" => 33,
        "vtkBiQuadraticTriangle" => 34,
        "vtkCubicLine" => 35,
        "vtkQuadraticPolygon" => 36,
        "vtkTriQuadraticPyramid" => 37,
        "vtkConvexPointSet" => 41,
        "vtkPolyhedron" => 42,
        "vtkHigherOrderCurve" => 60,
        "vtkHigherOrderTriangle" => 61,
        "vtkHigherOrderQuadrilateral" => 62,
        "vtkHigherOrderTetra" => 64,
        "vtkHigherOrderWedge" => 65,
        "vtkHigherOrderHexahedron" => 67,
        "vtkLagrangeCurve" => 68,
        "vtkLagrangeTriangle" => 69,
        "vtkLagrangeQuadrilateral" => 70,
        "vtkLagrangeTetra" => 71,
        "vtkLagrangeHexahedron" => 72,
        "vtkLagrangeWedge" => 73,
        "vtkBezierCurve" => 75,
        "vtkBezierTriangle" => 76,
        "vtkBezierQuadrilateral" => 77,
        "vtkBezierTetra" => 78,
        "vtkBezierHexahedron" => 79,
        "vtkBezierWedge" => 80,
        _ => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        get_class_name_from_type_id, get_dimension, get_type_as_string,
        get_type_id_from_class_name, get_type_id_from_name, is_linear, CellType, CellTypeUtilities,
    };

    #[test]
    fn wrapper_constructs_prints_and_delegates_to_free_functions() {
        let utilities = CellTypeUtilities::new();

        assert_eq!(format!("{utilities:?}"), "CellTypeUtilities");
        assert_eq!(
            utilities.print_self(),
            "vtkCellTypeUtilities { number_of_cell_types: 82 }"
        );
        assert_eq!(
            CellTypeUtilities::get_dimension(CellType::Triangle as u8),
            get_dimension(CellType::Triangle as u8)
        );
        assert_eq!(
            CellTypeUtilities::is_linear(CellType::QuadraticEdge as u8),
            is_linear(CellType::QuadraticEdge as u8)
        );
        assert_eq!(
            CellTypeUtilities::get_type_as_string(CellType::Quad as i32),
            get_type_as_string(CellType::Quad as i32)
        );
        assert_eq!(
            CellTypeUtilities::get_class_name_from_type_id(CellType::Quad as i32),
            get_class_name_from_type_id(CellType::Quad as i32)
        );
        assert_eq!(
            CellTypeUtilities::get_type_id_from_name("Quadrilateral"),
            get_type_id_from_name("Quadrilateral")
        );
        assert_eq!(
            CellTypeUtilities::get_type_id_from_class_name(Some("vtkQuad")),
            get_type_id_from_class_name(Some("vtkQuad"))
        );
    }

    #[test]
    fn dimensions_match_vtk_cell_type_utilities_cases() {
        assert_eq!(CellType::Empty.dimension(), 0);
        assert_eq!(CellType::PolyLine.dimension(), 1);
        assert_eq!(CellType::LagrangeCurve.dimension(), 1);
        assert_eq!(CellType::HigherOrderCurve.dimension(), 3);
        assert_eq!(CellType::QuadraticPolygon.dimension(), 2);
        assert_eq!(CellType::BezierQuadrilateral.dimension(), 2);
        assert_eq!(CellType::Hexahedron.dimension(), 3);
        assert_eq!(CellType::Polyhedron.dimension(), 3);
        assert_eq!(get_dimension(17), 3);
        assert_eq!(get_dimension(82), 0);
    }

    #[test]
    fn linearity_matches_vtk_cell_type_utilities_cases() {
        assert!(is_linear(CellType::HexagonalPrism as u8));
        assert!(is_linear(CellType::ConvexPointSet as u8));
        assert!(is_linear(CellType::Polyhedron as u8));
        assert!(!is_linear(CellType::QuadraticEdge as u8));
        assert!(!is_linear(CellType::LagrangeCurve as u8));
    }

    #[test]
    fn display_names_match_vtk_cell_type_utilities_table() {
        assert_eq!(get_type_as_string(CellType::Empty as i32), "Empty Cell");
        assert_eq!(
            get_type_as_string(CellType::PolyVertex as i32),
            "Polyvertex"
        );
        assert_eq!(get_type_as_string(CellType::Quad as i32), "Quadrilateral");
        assert_eq!(
            get_type_as_string(CellType::BiQuadraticQuadraticHexahedron as i32),
            "Bi-Quadratic Quadratic Hexahedron"
        );
        assert_eq!(
            get_type_as_string(CellType::HigherOrderTetrahedron as i32),
            "Higher Order Tetrahedron"
        );
        assert_eq!(
            get_type_as_string(CellType::BezierWedge as i32),
            "Bezier Wedge"
        );
        assert_eq!(CellType::Triangle.type_as_string(), "Triangle");
        assert_eq!(get_type_as_string(17), "Unknown Cell");
        assert_eq!(
            get_type_as_string(CellType::HigherOrderPyramid as i32),
            "Unknown Cell"
        );
    }

    #[test]
    fn class_names_match_vtk_cell_type_utilities_table() {
        assert_eq!(
            get_class_name_from_type_id(CellType::Empty as i32),
            "vtkEmptyCell"
        );
        assert_eq!(
            get_class_name_from_type_id(CellType::PolyVertex as i32),
            "vtkPolyVertex"
        );
        assert_eq!(
            get_class_name_from_type_id(CellType::HigherOrderTetrahedron as i32),
            "vtkHigherOrderTetra"
        );
        assert_eq!(
            get_class_name_from_type_id(CellType::LagrangeTetrahedron as i32),
            "vtkLagrangeTetra"
        );
        assert_eq!(
            get_class_name_from_type_id(CellType::BezierTetrahedron as i32),
            "vtkBezierTetra"
        );
        assert_eq!(CellType::Quad.get_class_name(), "vtkQuad");
        assert_eq!(get_class_name_from_type_id(17), "UnknownClass");
        assert_eq!(
            get_class_name_from_type_id(CellType::BezierPyramid as i32),
            "UnknownClass"
        );
    }

    #[test]
    fn display_names_map_back_to_type_ids_like_vtk() {
        assert_eq!(get_type_id_from_name("Empty Cell"), CellType::Empty as i32);
        assert_eq!(get_type_id_from_name("Polyline"), CellType::PolyLine as i32);
        assert_eq!(
            get_type_id_from_name("Quadratic Quadrilateral"),
            CellType::QuadraticQuad as i32
        );
        assert_eq!(
            get_type_id_from_name("Higher Order Hexahedron"),
            CellType::HigherOrderHexahedron as i32
        );
        assert_eq!(
            get_type_id_from_name("Bezier Tetrahedron"),
            CellType::BezierTetrahedron as i32
        );
        assert_eq!(
            get_type_id_from_name("Unknown Cell"),
            CellType::Empty as i32
        );
        assert_eq!(
            get_type_id_from_name("Bezier Pyramid"),
            CellType::Empty as i32
        );
    }

    #[test]
    fn class_names_map_back_to_type_ids_like_vtk() {
        assert_eq!(
            get_type_id_from_class_name(Some("vtkEmptyCell")),
            CellType::Empty as i32
        );
        assert_eq!(
            get_type_id_from_class_name(Some("vtkTriangleStrip")),
            CellType::TriangleStrip as i32
        );
        assert_eq!(
            get_type_id_from_class_name(Some("vtkHigherOrderTetra")),
            CellType::HigherOrderTetrahedron as i32
        );
        assert_eq!(
            get_type_id_from_class_name(Some("vtkLagrangeTetra")),
            CellType::LagrangeTetrahedron as i32
        );
        assert_eq!(
            get_type_id_from_class_name(Some("vtkBezierTetra")),
            CellType::BezierTetrahedron as i32
        );
        assert_eq!(get_type_id_from_class_name(Some("UnknownClass")), -1);
        assert_eq!(get_type_id_from_class_name(Some("vtkBezierPyramid")), -1);
        assert_eq!(get_type_id_from_class_name(None), -1);
    }
}
