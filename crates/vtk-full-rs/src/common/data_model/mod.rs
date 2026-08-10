//! VTK/Common/DataModel translation targets.
pub mod abstract_cell_array;
pub mod abstract_cell_links;
pub mod abstract_cell_locator;
pub mod abstract_electronic_data;
pub mod abstract_point_locator;
pub mod adjacent_vertex_iterator;
pub mod atom;
pub mod attributes_error_metric;
pub mod bond;
pub mod bounding_box;
pub mod cartesian_grid;
pub mod cell;
pub mod cell_3d;
pub mod cell_array;
pub mod cell_array_iterator;
pub mod cell_attribute_calculator;
pub mod cell_data;
pub mod cell_grid_bounds_query;
pub mod cell_grid_query;
pub mod cell_grid_range_query;
pub mod cell_grid_sides_cache;
pub mod cell_grid_sides_query;
pub mod cell_iterator;
pub mod cell_locator_strategy;
pub mod cell_metadata;
pub mod cell_type_utilities;
pub mod cell_types;
pub mod composite_data_iterator;
pub mod cone;
pub mod coordinate_frame;
pub mod cylinder;
pub mod data_assembly;
pub mod data_object;
pub mod data_object_collection;
pub mod data_object_types;
pub mod data_set;
pub mod data_set_attributes;
pub mod data_set_attributes_field_list;
pub mod data_set_cell_iterator;
pub mod data_set_collection;
pub mod directed_acyclic_graph;
pub mod edge_list_iterator;
pub mod empty_cell;
pub mod field_data;
pub mod find_cell_strategy;
pub mod frustum;
pub mod generic_attribute;
pub mod generic_cell_iterator;
pub mod generic_cell_tessellator;
pub mod generic_edge_table;
pub mod generic_point_iterator;
pub mod generic_subdivision_error_metric;
pub mod geometric_error_metric;
pub mod graph;
pub mod graph_edge;
pub mod hexahedron;
pub mod hyper_tree_grid_locator;
pub mod image_data;
pub mod implicit_boolean;
pub mod implicit_function;
pub mod implicit_function_collection;
pub mod implicit_halo;
pub mod implicit_sum;
pub mod implicit_window_function;
pub mod in_edge_iterator;
pub mod incremental_point_locator;
pub mod information_quadrature_scheme_definition_vector_key;
pub mod line;
pub mod locator;
pub mod marching_cubes_polygon_cases;
pub mod marching_cubes_triangle_cases;
pub mod marching_squares_line_cases;
pub mod merge_points;
pub mod molecule;
pub mod multi_block_data_set;
pub mod multi_piece_data_set;
pub mod mutable_directed_graph;
pub mod mutable_undirected_graph;
pub mod non_merging_point_locator;
pub mod out_edge_iterator;
pub mod partitioned_data_set;
pub mod partitioned_data_set_collection;
pub mod perlin_noise;
pub mod pixel;
pub mod pixel_extent;
pub mod pixel_transfer;
pub mod plane;
pub mod plane_collection;
pub mod planes;
pub mod planes_intersection;
pub mod point_data;
pub mod point_locator;
pub mod point_set;
pub mod point_set_cell_iterator;
pub mod points_projected_hull;
pub mod poly_data_collection;
pub mod poly_data_material;
pub mod poly_line;
pub mod poly_vertex;
pub mod pyramid;
pub mod quad;
pub mod quadratic_edge;
pub mod quadratic_triangle;
pub mod quadrature_scheme_definition;
pub mod quadric;
pub mod rectilinear_grid;
pub mod reeb_graph_simplification_metric;
pub mod simple_cell_tessellator;
pub mod smooth_error_metric;
pub mod sort_field_data;
pub mod sphere;
pub mod spheres;
pub mod spline;
pub mod static_cell_links;
pub mod static_cell_links_template;
pub mod structured_cell_array;
pub mod structured_data;
pub mod structured_extent;
pub mod structured_grid;
pub mod structured_points;
pub mod structured_points_collection;
pub mod superquadric;
pub mod table;
pub mod tetra;
pub mod tree;
pub mod tree_bfs_iterator;
pub mod tree_dfs_iterator;
pub mod tree_iterator;
pub mod triangle;
pub mod triangle_strip;
pub mod uniform_grid;
pub mod vertex;
pub mod vertex_list_iterator;
pub mod voxel;
pub mod wedge;
pub mod xml_data_element;

pub use crate::common::core::Variant;
pub use abstract_cell_array::{AbstractCellArray, AbstractCellArrayApi, AbstractCellArrayHandle};
pub use abstract_cell_links::{
    AbstractCellLinks, AbstractCellLinksApi, AbstractCellLinksHandle, CellLinksTypes,
};
pub use abstract_cell_locator::{AbstractCellLocator, AbstractCellLocatorApi};
pub use abstract_electronic_data::{AbstractElectronicData, AbstractElectronicDataApi};
pub use abstract_point_locator::{AbstractPointLocator, AbstractPointLocatorApi};
pub use adjacent_vertex_iterator::{AdjacentVertexIterator, AdjacentVertexIteratorGraphHandle};
pub use atom::Atom;
pub use attributes_error_metric::AttributesErrorMetric;
pub use bond::Bond;
pub use bounding_box::BoundingBox;
pub use cartesian_grid::{CartesianGrid, VTK_3D_EXTENT};
pub use cell::{Cell, CellBaseApi, VTK_CELL_SIZE, VTK_TOL};
pub use cell_3d::{Cell3D, Cell3DApi};
pub use cell_array::{CellArray, CellArrayStorageType};
pub use cell_array_iterator::CellArrayIterator;
pub use cell_attribute_calculator::CellAttributeCalculator;
pub use cell_data::CellData;
pub use cell_grid_bounds_query::CellGridBoundsQuery;
pub use cell_grid_query::CellGridQuery;
pub use cell_grid_range_query::{
    CellAttributeHandle, CellAttributeRangeApi, CellGridHandle, CellGridRangeQuery, ComponentRange,
};
pub use cell_grid_sides_cache::{
    CellGridSide, CellGridSideConnectivityValue, CellGridSidesCache, CellGridSidesCacheEntry,
};
pub use cell_grid_sides_query::{
    CellGridSidesQuery, PassWork, SelectionMode, SideFlags, SideIdSet, SideSetArray, SidesByCellId,
    SidesByCellType, SidesByShape, SummaryStrategy,
};
pub use cell_iterator::{CellIterator, CellIteratorApi};
pub use cell_locator_strategy::{AbstractCellLocatorHandle, CellLocatorStrategy};
pub use cell_metadata::{
    CellGridMetadataHandle, CellGridResponderApi, CellGridResponders, CellMetadata, CellTypeId,
    DofType, MetadataConstructor,
};
pub use cell_type_utilities::{CellType, CellTypeUtilities};
pub use cell_types::CellTypes;
pub use composite_data_iterator::{
    CompositeDataIterator, CompositeDataIteratorApi, CompositeDataSetHandle,
    CompositeIteratorDataObjectHandle, CompositeIteratorInformationHandle,
};
pub use cone::Cone;
pub use coordinate_frame::CoordinateFrame;
pub use cylinder::Cylinder;
pub use data_assembly::{DataAssembly, TraversalOrder};
pub(crate) use data_object::DataObjectType;
pub use data_object::{
    DataObject, DataObjectHandle, CELL, EDGE, FIELD, NUMBER_OF_ATTRIBUTE_TYPES, POINT,
    POINT_THEN_CELL, ROW, VERTEX,
};
pub use data_object_collection::DataObjectCollection;
pub use data_object_types::{DataObjectTypes, VTK_STRUCTURED_POINTS, VTK_UNIFORM_GRID};
pub use data_set::{DataSet, DataSetApi};
pub(crate) use data_set_attributes::{DataSetAttribute, DataSetAttributeCopyOperation};
pub use data_set_attributes::{
    DataSetAttributes, DataSetAttributesError, ALLCOPY, COPYTUPLE, DUPLICATECELL, DUPLICATEPOINT,
    EDGEFLAG, EXTERIORCELL, GLOBALIDS, HIDDENCELL, HIDDENPOINT, HIGHCONNECTIVITYCELL,
    HIGHERORDERDEGREES, INTERPOLATE, LOWCONNECTIVITYCELL, NORMALS, NUM_ATTRIBUTES, PASSDATA,
    PEDIGREEIDS, PROCESSIDS, RATIONALWEIGHTS, REFINEDCELL, SCALARS, TANGENTS, TCOORDS, TENSORS,
    VECTORS,
};
pub use data_set_attributes_field_list::DataSetAttributesFieldList;
pub use data_set_cell_iterator::DataSetCellIterator;
pub use data_set_collection::DataSetCollection;
pub use directed_acyclic_graph::DirectedAcyclicGraph;
pub use edge_list_iterator::{EdgeListIterator, EdgeListIteratorGraphHandle};
pub use empty_cell::{EmptyCell, EmptyCellEvaluatePosition, EmptyCellIntersectWithLine};
pub use field_data::FieldData;
pub(crate) use field_data::FieldDataArray;
pub use find_cell_strategy::{
    CellApi, CellHandle, FindCellStrategy, FindCellStrategyApi, GenericCellHandle,
};
pub use frustum::Frustum;
pub use generic_attribute::{
    GenericAdaptorCellHandle as GenericAttributeAdaptorCellHandle, GenericAttribute,
    GenericAttributeApi, GenericCellIteratorHandle, GenericPointIteratorHandle,
    VTK_BOUNDARY_CENTERED, VTK_CELL_CENTERED, VTK_POINT_CENTERED,
};
pub use generic_cell_iterator::{GenericCellIterator, GenericCellIteratorApi};
pub use generic_cell_tessellator::{GenericCellTessellator, GenericSubdivisionErrorMetricHandle};
pub use generic_edge_table::GenericEdgeTable;
pub use generic_point_iterator::{GenericPointIterator, GenericPointIteratorApi};
pub use generic_subdivision_error_metric::{
    GenericAdaptorCellApi, GenericAdaptorCellHandle, GenericAttributeCollectionApi,
    GenericAttributeCollectionHandle, GenericAttributeHandle, GenericDataSetApi,
    GenericDataSetHandle, GenericSubdivisionErrorMetric, GenericSubdivisionErrorMetricApi,
};
pub use geometric_error_metric::GeometricErrorMetric;
pub use graph::{DirectedGraph, Edge, GraphError, InEdge, OutEdge, UndirectedGraph};
pub use graph_edge::GraphEdge;
pub use hexahedron::{Hexahedron, HexahedronEvaluatePosition, HexahedronIntersectWithLine};
pub use hyper_tree_grid_locator::{
    HyperTreeGridHandle, HyperTreeGridLocator, HyperTreeGridLocatorApi,
};
pub use image_data::{ImageCell, ImageData};
pub use implicit_boolean::{
    ImplicitBoolean, VTK_DIFFERENCE, VTK_INTERSECTION, VTK_UNION, VTK_UNION_OF_MAGNITUDES,
};
pub use implicit_function::{ImplicitFunctionApi, ImplicitFunctionHandle};
pub use implicit_function_collection::ImplicitFunctionCollection;
pub use implicit_halo::ImplicitHalo;
pub use implicit_sum::ImplicitSum;
pub use implicit_window_function::ImplicitWindowFunction;
pub use in_edge_iterator::{InEdgeIterator, InEdgeIteratorGraphHandle};
pub use incremental_point_locator::{IncrementalPointLocator, IncrementalPointLocatorApi};
pub use information_quadrature_scheme_definition_vector_key::InformationQuadratureSchemeDefinitionVectorKey;
pub use line::{
    Line, LineDistanceBetween, LineEvaluatePosition, LineIntersectWithLine, LineIntersectionType,
    LineToleranceType,
};
pub use locator::{Locator, LocatorApi, PolyDataHandle};
pub use marching_cubes_polygon_cases::MarchingCubesPolygonCases;
pub use marching_cubes_triangle_cases::MarchingCubesTriangleCases;
pub use marching_squares_line_cases::MarchingSquaresLineCases;
pub use merge_points::MergePoints;
pub use molecule::Molecule;
pub use multi_block_data_set::MultiBlockDataSet;
pub use multi_piece_data_set::{MultiPieceDataSet, VTK_MULTIPIECE_DATA_SET};
pub use mutable_directed_graph::MutableDirectedGraph;
pub use mutable_undirected_graph::MutableUndirectedGraph;
pub use non_merging_point_locator::NonMergingPointLocator;
pub use out_edge_iterator::{OutEdgeIterator, OutEdgeIteratorGraphHandle};
pub use partitioned_data_set::PartitionedDataSet;
pub use partitioned_data_set_collection::PartitionedDataSetCollection;
pub use perlin_noise::PerlinNoise;
pub use pixel::{Pixel, PixelEvaluatePosition};
pub use pixel_extent::PixelExtent;
pub use pixel_transfer::{PixelScalar, PixelTransfer};
pub use plane::Plane;
pub use plane_collection::PlaneCollection;
pub use planes::Planes;
pub use planes_intersection::PlanesIntersection;
pub use point_data::PointData;
pub use point_locator::PointLocator;
pub use point_set::PointSet;
pub use point_set_cell_iterator::PointSetCellIterator;
pub use points_projected_hull::PointsProjectedHull;
pub use poly_data_collection::PolyDataCollection;
pub use poly_data_material::PolyDataMaterial;
pub use poly_line::{PolyLine, PolyLineEvaluatePosition, PolyLineIntersectWithLine};
pub use poly_vertex::{PolyVertex, PolyVertexEvaluatePosition, PolyVertexIntersectWithLine};
pub use pyramid::{Pyramid, PyramidEvaluatePosition, PyramidFace, PyramidIntersectWithLine};
pub use quad::{Quad, QuadEvaluatePosition, QuadIntersectWithLine};
pub use quadratic_edge::{
    QuadraticEdge, QuadraticEdgeEvaluatePosition, QuadraticEdgeIntersectWithLine,
};
pub use quadratic_triangle::{
    QuadraticTriangle, QuadraticTriangleEvaluatePosition, QuadraticTriangleIntersectWithLine,
};
pub use quadrature_scheme_definition::{
    QuadratureSchemeDefinition, QuadratureSchemeDefinitionHandle, VTK_EMPTY_CELL_ID,
    VTK_NUMBER_OF_CELL_TYPES,
};
pub use quadric::Quadric;
pub use rectilinear_grid::{RectilinearCell, RectilinearGrid};
pub use reeb_graph_simplification_metric::{
    AbstractArrayHandle, DataArrayHandle, ReebGraphSimplificationMetric,
};
pub use simple_cell_tessellator::{
    SimpleCellTessellator, SimpleCellTessellatorCellArrayApi, SimpleCellTessellatorCellArrayHandle,
    SimpleCellTessellatorEdgeTableApi, SimpleCellTessellatorEdgeTableHandle,
    SimpleCellTessellatorPointDataApi, SimpleCellTessellatorPointDataHandle,
    SimpleCellTessellatorPointsApi, SimpleCellTessellatorPointsHandle,
    SimpleCellTessellatorResetApi, SimpleCellTessellatorResetHandle,
};
pub use smooth_error_metric::SmoothErrorMetric;
pub use sort_field_data::SortFieldData;
pub use sphere::Sphere;
pub use spheres::Spheres;
pub use spline::{Spline, SplineApi};
pub use static_cell_links::StaticCellLinks;
pub use static_cell_links_template::{StaticCellLinkId, StaticCellLinksTemplate};
pub use structured_cell_array::StructuredCellArray;
pub use structured_data::{
    StructuredData, StructuredDataType, VTK_STRUCTURED_EMPTY, VTK_STRUCTURED_INVALID,
    VTK_STRUCTURED_SINGLE_POINT, VTK_STRUCTURED_UNCHANGED, VTK_STRUCTURED_XYZ_GRID,
    VTK_STRUCTURED_XY_PLANE, VTK_STRUCTURED_XZ_PLANE, VTK_STRUCTURED_X_LINE,
    VTK_STRUCTURED_YZ_PLANE, VTK_STRUCTURED_Y_LINE, VTK_STRUCTURED_Z_LINE,
};
pub use structured_extent::StructuredExtent;
pub use structured_grid::{StructuredCell, StructuredGrid};
pub use structured_points::StructuredPoints;
pub use structured_points_collection::StructuredPointsCollection;
pub use superquadric::{Superquadric, VTK_MIN_SUPERQUADRIC_THICKNESS};
pub use table::Table;
pub use tetra::{Tetra, TetraEvaluatePosition, TetraIntersectWithLine};
pub use tree::Tree;
pub use tree_bfs_iterator::TreeBFSIterator;
pub use tree_dfs_iterator::{TreeDFSIterator, DISCOVER, FINISH};
pub use tree_iterator::{TreeIterator, TreeIteratorApi};
pub use triangle::{Triangle, TriangleEvaluatePosition, TriangleIntersectWithLine};
pub use triangle_strip::{
    TriangleStrip, TriangleStripEvaluatePosition, TriangleStripIntersectWithLine,
};
pub use uniform_grid::UniformGrid;
pub use vertex::{Vertex, VertexEvaluatePosition, VertexIntersectWithLine};
pub use vertex_list_iterator::{VertexListIterator, VertexListIteratorGraphHandle};
pub use voxel::{Voxel, VoxelEvaluatePosition};
pub use wedge::{Wedge, WedgeEvaluatePosition, WedgeFace, WedgeIntersectWithLine};
pub use xml_data_element::{
    XMLDataElement, XMLDataElementHandle, VTK_ENCODING_NONE, VTK_ENCODING_UNKNOWN,
    VTK_ENCODING_UTF_8,
};
