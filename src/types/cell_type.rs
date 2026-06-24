/// VTK cell type identifiers.
///
/// Matches the constants from VTK's `vtkCellType.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CellType {
    // Linear cells
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

    // Quadratic cells
    QuadraticEdge = 21,
    QuadraticTriangle = 22,
    QuadraticQuad = 23,
    QuadraticTetra = 24,
    QuadraticHexahedron = 25,
    QuadraticWedge = 26,
    QuadraticPyramid = 27,

    // Bi-quadratic cells
    BiQuadraticQuad = 28,
    TriQuadraticHexahedron = 29,
    QuadraticLinearQuad = 30,
    QuadraticLinearWedge = 31,
    BiQuadraticQuadraticWedge = 32,
    BiQuadraticQuadraticHexahedron = 33,
    BiQuadraticTriangle = 34,

    // Special
    ConvexPointSet = 41,
    Polyhedron = 42,

    // Higher-order / Lagrange cells
    LagrangeCurve = 68,
    LagrangeTriangle = 69,
    LagrangeQuadrilateral = 70,
    LagrangeTetrahedron = 71,
    LagrangeHexahedron = 72,
    LagrangeWedge = 73,
    LagrangePyramid = 74,

    // Bezier cells
    BezierCurve = 75,
    BezierTriangle = 76,
    BezierQuadrilateral = 77,
    BezierTetrahedron = 78,
    BezierHexahedron = 79,
    BezierWedge = 80,
    BezierPyramid = 81,
}

impl CellType {
    /// Number of points for fixed-size linear cell types.
    /// Returns `None` for variable-size types (PolyVertex, PolyLine, Polygon, etc).
    pub fn num_points(&self) -> Option<usize> {
        match self {
            CellType::Vertex => Some(1),
            CellType::Line => Some(2),
            CellType::Triangle => Some(3),
            CellType::Pixel | CellType::Quad | CellType::Tetra => Some(4),
            CellType::Hexahedron | CellType::Voxel => Some(8),
            CellType::Wedge => Some(6),
            CellType::Pyramid => Some(5),
            CellType::PentagonalPrism => Some(10),
            CellType::HexagonalPrism => Some(12),
            CellType::QuadraticEdge => Some(3),
            CellType::QuadraticTriangle => Some(6),
            CellType::QuadraticQuad => Some(8),
            CellType::QuadraticTetra => Some(10),
            CellType::QuadraticHexahedron => Some(20),
            CellType::QuadraticWedge => Some(15),
            CellType::QuadraticPyramid => Some(13),
            CellType::BiQuadraticQuad => Some(9),
            CellType::TriQuadraticHexahedron => Some(27),
            CellType::QuadraticLinearQuad => Some(6),
            CellType::QuadraticLinearWedge => Some(12),
            CellType::BiQuadraticQuadraticWedge => Some(18),
            CellType::BiQuadraticQuadraticHexahedron => Some(24),
            CellType::BiQuadraticTriangle => Some(7),
            // Lagrange and Bezier are variable-order
            _ => None,
        }
    }

    /// Topological dimension of this cell type.
    pub fn dimension(&self) -> usize {
        match self {
            CellType::Empty => 0,
            CellType::Vertex | CellType::PolyVertex => 0,
            CellType::Line | CellType::PolyLine | CellType::QuadraticEdge => 1,
            CellType::Triangle
            | CellType::TriangleStrip
            | CellType::Polygon
            | CellType::Pixel
            | CellType::Quad
            | CellType::QuadraticTriangle
            | CellType::QuadraticQuad
            | CellType::BiQuadraticQuad
            | CellType::QuadraticLinearQuad
            | CellType::BiQuadraticTriangle
            | CellType::LagrangeTriangle
            | CellType::LagrangeQuadrilateral
            | CellType::BezierTriangle
            | CellType::BezierQuadrilateral => 2,
            CellType::LagrangeCurve | CellType::BezierCurve => 1,
            _ => 3,
        }
    }

    /// Create from raw u8 value.
    pub fn from_u8(v: u8) -> Option<CellType> {
        match v {
            0 => Some(CellType::Empty),
            1 => Some(CellType::Vertex),
            2 => Some(CellType::PolyVertex),
            3 => Some(CellType::Line),
            4 => Some(CellType::PolyLine),
            5 => Some(CellType::Triangle),
            6 => Some(CellType::TriangleStrip),
            7 => Some(CellType::Polygon),
            8 => Some(CellType::Pixel),
            9 => Some(CellType::Quad),
            10 => Some(CellType::Tetra),
            11 => Some(CellType::Voxel),
            12 => Some(CellType::Hexahedron),
            13 => Some(CellType::Wedge),
            14 => Some(CellType::Pyramid),
            15 => Some(CellType::PentagonalPrism),
            16 => Some(CellType::HexagonalPrism),
            21 => Some(CellType::QuadraticEdge),
            22 => Some(CellType::QuadraticTriangle),
            23 => Some(CellType::QuadraticQuad),
            24 => Some(CellType::QuadraticTetra),
            25 => Some(CellType::QuadraticHexahedron),
            26 => Some(CellType::QuadraticWedge),
            27 => Some(CellType::QuadraticPyramid),
            28 => Some(CellType::BiQuadraticQuad),
            29 => Some(CellType::TriQuadraticHexahedron),
            30 => Some(CellType::QuadraticLinearQuad),
            31 => Some(CellType::QuadraticLinearWedge),
            32 => Some(CellType::BiQuadraticQuadraticWedge),
            33 => Some(CellType::BiQuadraticQuadraticHexahedron),
            34 => Some(CellType::BiQuadraticTriangle),
            41 => Some(CellType::ConvexPointSet),
            42 => Some(CellType::Polyhedron),
            68 => Some(CellType::LagrangeCurve),
            69 => Some(CellType::LagrangeTriangle),
            70 => Some(CellType::LagrangeQuadrilateral),
            71 => Some(CellType::LagrangeTetrahedron),
            72 => Some(CellType::LagrangeHexahedron),
            73 => Some(CellType::LagrangeWedge),
            74 => Some(CellType::LagrangePyramid),
            75 => Some(CellType::BezierCurve),
            76 => Some(CellType::BezierTriangle),
            77 => Some(CellType::BezierQuadrilateral),
            78 => Some(CellType::BezierTetrahedron),
            79 => Some(CellType::BezierHexahedron),
            80 => Some(CellType::BezierWedge),
            81 => Some(CellType::BezierPyramid),
            _ => None,
        }
    }

    /// Whether this is a linear (non-higher-order) cell.
    pub fn is_linear(&self) -> bool {
        (*self as u8) <= 16
    }

    /// Whether this is a quadratic cell.
    pub fn is_quadratic(&self) -> bool {
        let v = *self as u8;
        (21..=34).contains(&v)
    }

    /// Whether this is a Lagrange higher-order cell.
    pub fn is_lagrange(&self) -> bool {
        let v = *self as u8;
        (68..=74).contains(&v)
    }

    /// Whether this is a Bezier higher-order cell.
    pub fn is_bezier(&self) -> bool {
        let v = *self as u8;
        (75..=81).contains(&v)
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            CellType::Empty => "Empty",
            CellType::Vertex => "Vertex",
            CellType::PolyVertex => "PolyVertex",
            CellType::Line => "Line",
            CellType::PolyLine => "PolyLine",
            CellType::Triangle => "Triangle",
            CellType::TriangleStrip => "TriangleStrip",
            CellType::Polygon => "Polygon",
            CellType::Pixel => "Pixel",
            CellType::Quad => "Quad",
            CellType::Tetra => "Tetra",
            CellType::Voxel => "Voxel",
            CellType::Hexahedron => "Hexahedron",
            CellType::Wedge => "Wedge",
            CellType::Pyramid => "Pyramid",
            CellType::PentagonalPrism => "PentagonalPrism",
            CellType::HexagonalPrism => "HexagonalPrism",
            CellType::QuadraticEdge => "QuadraticEdge",
            CellType::QuadraticTriangle => "QuadraticTriangle",
            CellType::QuadraticQuad => "QuadraticQuad",
            CellType::QuadraticTetra => "QuadraticTetra",
            CellType::QuadraticHexahedron => "QuadraticHexahedron",
            CellType::QuadraticWedge => "QuadraticWedge",
            CellType::QuadraticPyramid => "QuadraticPyramid",
            CellType::BiQuadraticQuad => "BiQuadraticQuad",
            CellType::TriQuadraticHexahedron => "TriQuadraticHexahedron",
            CellType::QuadraticLinearQuad => "QuadraticLinearQuad",
            CellType::QuadraticLinearWedge => "QuadraticLinearWedge",
            CellType::BiQuadraticQuadraticWedge => "BiQuadraticQuadraticWedge",
            CellType::BiQuadraticQuadraticHexahedron => "BiQuadraticQuadraticHexahedron",
            CellType::BiQuadraticTriangle => "BiQuadraticTriangle",
            CellType::ConvexPointSet => "ConvexPointSet",
            CellType::Polyhedron => "Polyhedron",
            CellType::LagrangeCurve => "LagrangeCurve",
            CellType::LagrangeTriangle => "LagrangeTriangle",
            CellType::LagrangeQuadrilateral => "LagrangeQuadrilateral",
            CellType::LagrangeTetrahedron => "LagrangeTetrahedron",
            CellType::LagrangeHexahedron => "LagrangeHexahedron",
            CellType::LagrangeWedge => "LagrangeWedge",
            CellType::LagrangePyramid => "LagrangePyramid",
            CellType::BezierCurve => "BezierCurve",
            CellType::BezierTriangle => "BezierTriangle",
            CellType::BezierQuadrilateral => "BezierQuadrilateral",
            CellType::BezierTetrahedron => "BezierTetrahedron",
            CellType::BezierHexahedron => "BezierHexahedron",
            CellType::BezierWedge => "BezierWedge",
            CellType::BezierPyramid => "BezierPyramid",
        };
        write!(f, "{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u8_roundtrip() {
        let types = [
            CellType::Empty,
            CellType::Vertex,
            CellType::Line,
            CellType::Triangle,
            CellType::Quad,
            CellType::Tetra,
            CellType::Hexahedron,
            CellType::Wedge,
            CellType::Pyramid,
            CellType::QuadraticEdge,
            CellType::QuadraticTriangle,
            CellType::LagrangeCurve,
            CellType::BezierTriangle,
        ];
        for ct in &types {
            let v = *ct as u8;
            assert_eq!(CellType::from_u8(v), Some(*ct), "roundtrip failed for {v}");
        }
    }

    #[test]
    fn from_u8_invalid() {
        assert_eq!(CellType::from_u8(255), None);
        assert_eq!(CellType::from_u8(50), None);
    }

    #[test]
    fn num_points_linear() {
        assert_eq!(CellType::Vertex.num_points(), Some(1));
        assert_eq!(CellType::Line.num_points(), Some(2));
        assert_eq!(CellType::Triangle.num_points(), Some(3));
        assert_eq!(CellType::Quad.num_points(), Some(4));
        assert_eq!(CellType::Tetra.num_points(), Some(4));
        assert_eq!(CellType::Hexahedron.num_points(), Some(8));
        assert_eq!(CellType::Wedge.num_points(), Some(6));
        assert_eq!(CellType::Pyramid.num_points(), Some(5));
    }

    #[test]
    fn num_points_quadratic() {
        assert_eq!(CellType::QuadraticEdge.num_points(), Some(3));
        assert_eq!(CellType::QuadraticTriangle.num_points(), Some(6));
        assert_eq!(CellType::QuadraticQuad.num_points(), Some(8));
        assert_eq!(CellType::QuadraticTetra.num_points(), Some(10));
        assert_eq!(CellType::QuadraticHexahedron.num_points(), Some(20));
    }

    #[test]
    fn num_points_variable() {
        assert_eq!(CellType::Polygon.num_points(), None);
        assert_eq!(CellType::PolyLine.num_points(), None);
        assert_eq!(CellType::PolyVertex.num_points(), None);
        assert_eq!(CellType::LagrangeCurve.num_points(), None);
    }

    #[test]
    fn dimension() {
        assert_eq!(CellType::Vertex.dimension(), 0);
        assert_eq!(CellType::Line.dimension(), 1);
        assert_eq!(CellType::Triangle.dimension(), 2);
        assert_eq!(CellType::Quad.dimension(), 2);
        assert_eq!(CellType::Tetra.dimension(), 3);
        assert_eq!(CellType::Hexahedron.dimension(), 3);
        assert_eq!(CellType::LagrangeCurve.dimension(), 1);
        assert_eq!(CellType::LagrangeTriangle.dimension(), 2);
        assert_eq!(CellType::BezierQuadrilateral.dimension(), 2);
    }

    #[test]
    fn classification() {
        assert!(CellType::Triangle.is_linear());
        assert!(!CellType::QuadraticTriangle.is_linear());
        assert!(CellType::QuadraticTriangle.is_quadratic());
        assert!(CellType::LagrangeCurve.is_lagrange());
        assert!(!CellType::LagrangeCurve.is_bezier());
        assert!(CellType::BezierTriangle.is_bezier());
        assert!(!CellType::BezierTriangle.is_lagrange());
    }

    #[test]
    fn display() {
        assert_eq!(format!("{}", CellType::Triangle), "Triangle");
        assert_eq!(format!("{}", CellType::Hexahedron), "Hexahedron");
        assert_eq!(format!("{}", CellType::LagrangeCurve), "LagrangeCurve");
    }
}
