use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::common::core::{VtkIdType, VtkMTimeType};

use super::{
    GenericAdaptorCellHandle, GenericAttributeCollectionHandle, GenericCellTessellator,
    GenericDataSetHandle, GenericEdgeTable, GenericSubdivisionErrorMetricHandle,
};

const PARAMETRIC_OFFSET: usize = 3;
const ATTRIBUTES_OFFSET: usize = 6;
const TRIANGLE_EDGES_TABLE: [[usize; 2]; 3] = [[0, 1], [1, 2], [2, 0]];
const TRIANGLE_VERTEX_STATE: [u8; 3] = [5, 3, 6];
const TETRA_EDGES_TABLE: [[usize; 2]; 6] = [[0, 1], [1, 2], [2, 0], [0, 3], [1, 3], [2, 3]];
const VERTEX_EDGES: [[usize; 3]; 4] = [[0, 2, 3], [0, 1, 4], [1, 2, 5], [3, 4, 5]];
const VERTEX_FACES: [[usize; 3]; 4] = [[0, 2, 3], [0, 1, 3], [1, 2, 3], [0, 1, 2]];
const TETRA_VERTEX_STATE: [u16; 4] = [0x34d, 0x2d3, 0x3a6, 0x1f8];
const VTK_HIGHER_ORDER_TRIANGLE: i32 = 61;

const NO_TRIAN: [i8; 3] = [-1, -1, -1];
const VTK_TESSELLATOR_TRIANGLE_CASES: [[[i8; 3]; 4]; 9] = [
    [NO_TRIAN, NO_TRIAN, NO_TRIAN, NO_TRIAN],
    [[0, 3, 2], [1, 2, 3], NO_TRIAN, NO_TRIAN],
    [[0, 1, 4], [0, 4, 2], NO_TRIAN, NO_TRIAN],
    [[0, 3, 2], [1, 4, 3], [3, 4, 2], NO_TRIAN],
    [[0, 1, 5], [1, 2, 5], NO_TRIAN, NO_TRIAN],
    [[0, 3, 5], [1, 5, 3], [1, 2, 5], NO_TRIAN],
    [[0, 4, 5], [0, 1, 4], [2, 5, 4], NO_TRIAN],
    [[0, 3, 5], [3, 4, 5], [1, 4, 3], [2, 5, 4]],
    [NO_TRIAN, NO_TRIAN, NO_TRIAN, NO_TRIAN],
];

/// Minimal boundary for `vtkGenericEdgeTable` operations used by the translated
/// `vtkSimpleCellTessellator` lifecycle slice.
pub trait SimpleCellTessellatorEdgeTableApi {
    /// VTK: `vtkGenericEdgeTable::Initialize`.
    fn initialize(&mut self, start: VtkIdType);

    /// VTK: `vtkGenericEdgeTable::SetNumberOfComponents`.
    fn set_number_of_components(&mut self, count: i32);

    /// VTK: `vtkGenericEdgeTable::CheckPoint`.
    fn check_point(&self, point_id: VtkIdType) -> bool;

    /// VTK: `vtkGenericEdgeTable::CheckPoint(vtkIdType, double*, double*)`.
    fn check_point_values(
        &self,
        point_id: VtkIdType,
        point: &mut [f64; 3],
        scalars: &mut [f64],
    ) -> bool;

    /// VTK: `vtkGenericEdgeTable::InsertPointAndScalar`.
    fn insert_point_and_scalar(&mut self, point_id: VtkIdType, point: [f64; 3], scalars: &[f64]);

    /// VTK: `vtkGenericEdgeTable::IncrementPointReferenceCount`.
    fn increment_point_reference_count(&mut self, point_id: VtkIdType);

    /// VTK: `vtkGenericEdgeTable::RemovePoint`.
    fn remove_point(&mut self, point_id: VtkIdType);

    /// VTK: `vtkGenericEdgeTable::CheckEdge`.
    fn check_edge(&self, left_id: VtkIdType, right_id: VtkIdType, point_id: &mut VtkIdType) -> i32;

    /// VTK: `vtkGenericEdgeTable::InsertEdge`, split overload.
    fn insert_edge_with_point(
        &mut self,
        left_id: VtkIdType,
        right_id: VtkIdType,
        cell_id: VtkIdType,
        reference_count: i32,
        point_id: &mut VtkIdType,
    );

    /// VTK: `vtkGenericEdgeTable::InsertEdge`, non-split overload.
    fn insert_edge(
        &mut self,
        left_id: VtkIdType,
        right_id: VtkIdType,
        cell_id: VtkIdType,
        reference_count: i32,
    );

    /// VTK: `vtkGenericEdgeTable::IncrementEdgeReferenceCount`.
    fn increment_edge_reference_count(
        &mut self,
        left_id: VtkIdType,
        right_id: VtkIdType,
        cell_id: VtkIdType,
    );

    /// VTK: `vtkGenericEdgeTable::RemoveEdge`.
    fn remove_edge(&mut self, left_id: VtkIdType, right_id: VtkIdType);
}

/// VTK: `vtkGenericEdgeTable*`.
pub type SimpleCellTessellatorEdgeTableHandle = Rc<RefCell<dyn SimpleCellTessellatorEdgeTableApi>>;

/// Minimal boundary for `vtkDoubleArray`/`vtkCellArray` reset behavior used by
/// `vtkSimpleCellTessellator::Reset`.
pub trait SimpleCellTessellatorResetApi {
    /// VTK: `Reset`.
    fn reset(&mut self);
}

/// VTK: `vtkDoubleArray*` or `vtkCellArray*` reset boundary.
pub type SimpleCellTessellatorResetHandle = Rc<RefCell<dyn SimpleCellTessellatorResetApi>>;

/// Minimal boundary for `vtkDoubleArray` output methods used by direct triangle
/// tessellation.
pub trait SimpleCellTessellatorPointsApi: SimpleCellTessellatorResetApi {
    /// VTK: `vtkDoubleArray::InsertNextTuple`.
    fn insert_next_tuple(&mut self, tuple: [f64; 3]);
}

/// VTK: `vtkDoubleArray*` points output boundary.
pub type SimpleCellTessellatorPointsHandle = Rc<RefCell<dyn SimpleCellTessellatorPointsApi>>;

/// Minimal boundary for `vtkCellArray` output methods used by direct triangle
/// tessellation.
pub trait SimpleCellTessellatorCellArrayApi: SimpleCellTessellatorResetApi {
    /// VTK: `vtkCellArray::InsertNextCell`.
    fn insert_next_cell(&mut self, point_ids: &[VtkIdType]);
}

/// VTK: `vtkCellArray*` output boundary.
pub type SimpleCellTessellatorCellArrayHandle = Rc<RefCell<dyn SimpleCellTessellatorCellArrayApi>>;

/// Minimal boundary for `vtkPointData` methods used by this bottom-up slice.
pub trait SimpleCellTessellatorPointDataApi {
    /// VTK: `vtkPointData::GetNumberOfComponents`.
    fn get_number_of_components(&self) -> i32;

    /// VTK: `vtkPointData::GetNumberOfArrays`.
    fn get_number_of_arrays(&self) -> i32;

    /// VTK: `vtkDataArray::GetNumberOfComponents` through
    /// `vtkPointData::GetArray(i)`.
    fn get_array_number_of_components(&self, index: i32) -> i32;

    /// VTK: `vtkPointData::GetArray(i)->InsertNextTuple`.
    fn insert_next_tuple_into_array(&mut self, index: i32, tuple: &[f64]);
}

/// VTK: `vtkPointData*`.
pub type SimpleCellTessellatorPointDataHandle = Rc<RefCell<dyn SimpleCellTessellatorPointDataApi>>;

/// VTK: `vtkSimpleCellTessellator`.
pub struct SimpleCellTessellator {
    tessellator: GenericCellTessellator,
    generic_cell: Option<GenericAdaptorCellHandle>,
    tessellate_points: Option<SimpleCellTessellatorPointsHandle>,
    tessellate_cell_array: Option<SimpleCellTessellatorCellArrayHandle>,
    tessellate_point_data: Option<SimpleCellTessellatorPointDataHandle>,
    edge_table: SimpleCellTessellatorEdgeTableHandle,
    attribute_collection: Option<GenericAttributeCollectionHandle>,
    scalars: Vec<f64>,
    scalars_capacity: i32,
    point_offset: i32,
    number_of_points: VtkIdType,
    fixed_subdivisions: i32,
    max_subdivision_level: i32,
    current_subdivision_level: i32,
    point_ids: Vec<VtkIdType>,
    point_ids_capacity: i32,
    edge_ids: [VtkIdType; 3],
}

impl SimpleCellTessellator {
    /// VTK: `vtkSimpleCellTessellator::New`.
    pub fn new() -> Self {
        Self {
            tessellator: GenericCellTessellator::with_class_name("vtkSimpleCellTessellator"),
            generic_cell: None,
            tessellate_points: None,
            tessellate_cell_array: None,
            tessellate_point_data: None,
            edge_table: Rc::new(RefCell::new(GenericEdgeTable::new())),
            attribute_collection: None,
            scalars: Vec::new(),
            scalars_capacity: 0,
            point_offset: 0,
            number_of_points: 0,
            fixed_subdivisions: 0,
            max_subdivision_level: 0,
            current_subdivision_level: 0,
            point_ids: Vec::new(),
            point_ids_capacity: 0,
            edge_ids: [-1; 3],
        }
    }

    /// VTK: `vtkSimpleCellTessellator::PrintSelf`.
    pub fn print_self(&self) -> String {
        let mut result = self.tessellator.print_self();
        result.push_str(&format!(
            "GenericCell: {:?}\n",
            self.generic_cell.as_ref().map(Rc::as_ptr)
        ));
        result.push_str(&format!(
            "TessellatePointData: {:?}\n",
            self.tessellate_point_data.as_ref().map(Rc::as_ptr)
        ));
        result.push_str(&format!(
            "TessellateCellArray: {:?}\n",
            self.tessellate_cell_array.as_ref().map(Rc::as_ptr)
        ));
        result.push_str(&format!(
            "TessellatePoints: {:?}\n",
            self.tessellate_points.as_ref().map(Rc::as_ptr)
        ));
        result
    }

    /// VTK: `vtkSimpleCellTessellator::GetGenericCell`.
    pub fn get_generic_cell(&self) -> Option<GenericAdaptorCellHandle> {
        self.generic_cell.clone()
    }

    /// VTK: `vtkSimpleCellTessellator::Reset`.
    pub fn reset(&mut self) {
        self.tessellate_points
            .as_ref()
            .expect("pre: tessellate_points_exists")
            .borrow_mut()
            .reset();
        self.tessellate_cell_array
            .as_ref()
            .expect("pre: tessellate_cell_array_exists")
            .borrow_mut()
            .reset();
    }

    /// VTK: `vtkSimpleCellTessellator::Triangulate`.
    pub fn triangulate(
        &mut self,
        cell: GenericAdaptorCellHandle,
        att: GenericAttributeCollectionHandle,
        points: SimpleCellTessellatorPointsHandle,
        cell_array: SimpleCellTessellatorCellArrayHandle,
        internal_pd: SimpleCellTessellatorPointDataHandle,
    ) {
        assert!(cell.borrow().get_dimension() == 2, "pre: valid_dimension");
        assert!(
            cell.borrow().get_type() == VTK_HIGHER_ORDER_TRIANGLE,
            "pre: direct_triangle_slice"
        );

        let pts = [0, 1, 2];
        let edge_ids = [0, 1, 2];
        let number_of_boundaries = cell.borrow().get_number_of_boundaries(0);
        self.allocate_point_ids(number_of_boundaries);
        cell.borrow().get_point_ids(&mut self.point_ids);
        let ids = [self.point_ids[0], self.point_ids[1], self.point_ids[2]];
        self.triangulate_triangle(
            cell,
            pts,
            ids,
            edge_ids,
            att,
            points,
            cell_array,
            internal_pd,
        );
    }

    /// VTK: `vtkSimpleCellTessellator::TriangulateTriangle`.
    pub(crate) fn triangulate_triangle(
        &mut self,
        cell: GenericAdaptorCellHandle,
        local_ids: [VtkIdType; 3],
        ids: [VtkIdType; 3],
        edge_ids: [VtkIdType; 3],
        att: GenericAttributeCollectionHandle,
        points: SimpleCellTessellatorPointsHandle,
        cell_array: SimpleCellTessellatorCellArrayHandle,
        internal_pd: SimpleCellTessellatorPointDataHandle,
    ) {
        self.generic_cell = Some(cell.clone());
        self.tessellate_points = Some(points);
        self.tessellate_cell_array = Some(cell_array);
        self.tessellate_point_data = Some(internal_pd.clone());
        self.attribute_collection = Some(att);
        self.edge_ids = edge_ids;
        self.tessellator.set_generic_cell(cell.clone());

        let mut root = TriangleTile::default();
        let parametric_coords = cell.borrow().get_parametric_coords().to_vec();
        root.set_point_ids(ids);
        for i in 0..3 {
            let point = parametric_tuple(&parametric_coords, local_ids[i]);
            root.set_vertex(i, point);
        }
        root.set_original();

        let number_of_components = internal_pd.borrow().get_number_of_components();
        self.edge_table
            .borrow_mut()
            .set_number_of_components(number_of_components);

        self.point_offset = number_of_components + ATTRIBUTES_OFFSET as i32;
        self.allocate_scalars(self.point_offset * 3);

        self.insert_points_into_edge_table(&root);
        self.insert_edges_into_edge_table(&mut root);

        let mut work = VecDeque::new();
        work.push_back(root.clone());

        while let Some(curr) = work.pop_front() {
            let pieces = self.refine_triangle_tile(&curr);
            for piece in pieces {
                work.push_back(piece);
            }
            self.remove_edges_from_edge_table(&curr);
        }

        for i in 0..3 {
            self.edge_table
                .borrow_mut()
                .remove_point(root.get_point_id(i));
        }
    }

    /// VTK: `vtkSimpleCellTessellator::Initialize`.
    pub fn initialize(&mut self, data_set: GenericDataSetHandle) {
        self.tessellator.initialize_data_set(data_set.clone());
        self.number_of_points = data_set.borrow().get_number_of_points();
        self.edge_table
            .borrow_mut()
            .initialize(self.number_of_points);
    }

    /// VTK: `vtkSimpleCellTessellator::GetFixedSubdivisions`.
    pub fn get_fixed_subdivisions(&self) -> i32 {
        assert!(
            self.fixed_subdivisions >= 0 && self.fixed_subdivisions <= self.max_subdivision_level,
            "post: positive_result"
        );
        self.fixed_subdivisions
    }

    /// VTK: `vtkSimpleCellTessellator::GetMaxSubdivisionLevel`.
    pub fn get_max_subdivision_level(&self) -> i32 {
        assert!(
            self.max_subdivision_level >= self.fixed_subdivisions,
            "post: positive_result"
        );
        self.max_subdivision_level
    }

    /// VTK: `vtkSimpleCellTessellator::GetMaxAdaptiveSubdivisions`.
    pub fn get_max_adaptive_subdivisions(&self) -> i32 {
        self.max_subdivision_level - self.fixed_subdivisions
    }

    /// VTK: `vtkSimpleCellTessellator::SetFixedSubdivisions`.
    pub fn set_fixed_subdivisions(&mut self, level: i32) {
        assert!(
            level >= 0 && level <= self.get_max_subdivision_level(),
            "pre: positive_level"
        );
        self.fixed_subdivisions = level;
    }

    /// VTK: `vtkSimpleCellTessellator::SetMaxSubdivisionLevel`.
    pub fn set_max_subdivision_level(&mut self, level: i32) {
        assert!(
            level >= self.get_fixed_subdivisions(),
            "pre: positive_level"
        );
        self.max_subdivision_level = level;
    }

    /// VTK: `vtkSimpleCellTessellator::SetSubdivisionLevels`.
    pub fn set_subdivision_levels(&mut self, fixed: i32, max_level: i32) {
        assert!(fixed >= 0, "pre: positive_fixed");
        assert!(fixed <= max_level, "pre: valid_range");
        self.fixed_subdivisions = fixed;
        self.max_subdivision_level = max_level;
    }

    /// VTK: `vtkSimpleCellTessellator::AllocateScalars`.
    pub(crate) fn allocate_scalars(&mut self, size: i32) {
        assert!(size > 0, "pre: positive_size");
        if self.scalars_capacity < size {
            self.scalars.resize(size as usize, 0.0);
            self.scalars_capacity = size;
        }
    }

    /// VTK: `vtkSimpleCellTessellator::AllocatePointIds`.
    pub(crate) fn allocate_point_ids(&mut self, size: i32) {
        assert!(size > 0, "pre: positive_size");
        if self.point_ids_capacity < size {
            self.point_ids.resize(size as usize, 0);
            self.point_ids_capacity = size;
        }
    }

    /// VTK: `vtkSimpleCellTessellator::FacesAreEqual`.
    pub(crate) fn faces_are_equal(original_face: &[VtkIdType], face: [VtkIdType; 3]) -> i32 {
        assert!(!original_face.is_empty(), "pre: originalFace_exists");
        assert!(original_face.len() >= 3, "pre: originalFace_size");

        let mut result = 0;
        let mut i = 0;
        let mut j = 1;
        let mut k = 2;
        while result == 0 && i < 3 {
            result = (original_face[0] == face[i]
                && original_face[1] == face[j]
                && original_face[2] == face[k]) as i32;
            if result == 0 {
                result = (original_face[0] == face[i]
                    && original_face[2] == face[j]
                    && original_face[1] == face[k]) as i32;
            }
            i += 1;
            j += 1;
            k += 1;
            if j > 2 {
                j = 0;
            } else if k > 2 {
                k = 0;
            }
        }
        result
    }

    /// VTK: `vtkSimpleCellTessellator::CopyPoint`.
    pub(crate) fn copy_point(&mut self, point_id: VtkIdType) {
        let mut point = [0.0; 3];
        let mut scalars = vec![0.0; self.point_data_components()];
        assert!(
            self.edge_table
                .borrow()
                .check_point_values(point_id, &mut point, &mut scalars),
            "pre: point_exists"
        );

        self.tessellate_points
            .as_ref()
            .expect("pre: tessellate_points_exists")
            .borrow_mut()
            .insert_next_tuple(point);

        let point_data = self
            .tessellate_point_data
            .as_ref()
            .expect("pre: tessellate_point_data_exists");
        let number_of_arrays = point_data.borrow().get_number_of_arrays();
        let mut offset = 0;
        for i in 0..number_of_arrays {
            let component_count =
                point_data.borrow().get_array_number_of_components(i).max(0) as usize;
            let next_offset = offset + component_count;
            assert!(next_offset <= scalars.len(), "pre: array_tuple_components");
            point_data
                .borrow_mut()
                .insert_next_tuple_into_array(i, &scalars[offset..next_offset]);
            offset = next_offset;
        }
    }

    /// VTK: `vtkSimpleCellTessellator::InsertPointsIntoEdgeTable(vtkTriangleTile&)`.
    fn insert_points_into_edge_table(&mut self, tri: &TriangleTile) {
        for j in 0..3 {
            if !self.edge_table.borrow().check_point(tri.get_point_id(j)) {
                let global = self
                    .generic_cell()
                    .borrow()
                    .evaluate_location(0, tri.get_vertex(j));
                let mut scalars = vec![0.0; self.point_data_components()];
                self.generic_cell().borrow().interpolate_tuple_collection(
                    self.attribute_collection(),
                    tri.get_vertex(j),
                    &mut scalars,
                );
                self.edge_table.borrow_mut().insert_point_and_scalar(
                    tri.get_point_id(j),
                    global,
                    &scalars,
                );
            }
        }
    }

    /// VTK: `vtkSimpleCellTessellator::InsertEdgesIntoEdgeTable(vtkTriangleTile&)`.
    fn insert_edges_into_edge_table(&mut self, tri: &mut TriangleTile) {
        let cell_id = self.generic_cell().borrow().get_id();
        const ALPHA: f64 = 0.5;
        assert!(ALPHA > 0.0 && ALPHA < 1.0, "check: normalized alpha");

        for i in 0..3 {
            self.edge_table
                .borrow_mut()
                .increment_point_reference_count(tri.get_point_id(i));
        }

        for j in 0..3 {
            let mut l = TRIANGLE_EDGES_TABLE[j][0];
            let mut r = TRIANGLE_EDGES_TABLE[j][1];
            let mut left_id = tri.get_point_id(l);
            let mut right_id = tri.get_point_id(r);

            if left_id > right_id {
                std::mem::swap(&mut left_id, &mut right_id);
                std::mem::swap(&mut l, &mut r);
            }

            let left = tri.get_vertex(l);
            let right = tri.get_vertex(r);
            let mut left_point = self.point_workspace();
            let mut mid_point = self.point_workspace();
            let mut right_point = self.point_workspace();
            left_point[PARAMETRIC_OFFSET..PARAMETRIC_OFFSET + 3].copy_from_slice(&left);
            right_point[PARAMETRIC_OFFSET..PARAMETRIC_OFFSET + 3].copy_from_slice(&right);

            let mut point_id = -1;
            let to_split = self
                .edge_table
                .borrow()
                .check_edge(left_id, right_id, &mut point_id);
            let mut do_subdivision;

            if to_split == -1 {
                let parent_edge = tri.find_edge_parent(l, r);
                let ref_count = if parent_edge == -1 {
                    1
                } else {
                    self.get_number_of_cells_using_edge(parent_edge as i32)
                };

                do_subdivision = tri.get_subdivision_level() < self.get_max_subdivision_level();

                if !do_subdivision
                    && self.get_max_subdivision_level() == self.get_fixed_subdivisions()
                {
                    if self.get_measurement() != 0 {
                        self.fill_point_values(left_id, &mut left_point);
                        self.fill_point_values(right_id, &mut right_point);
                        fill_midpoint(left, right, &mut mid_point, ALPHA);
                        self.evaluate_point_values(&mut mid_point);
                        self.tessellator.update_max_error(
                            &mut left_point,
                            &mut mid_point,
                            &mut right_point,
                            ALPHA,
                        );
                    }
                }

                if do_subdivision {
                    self.fill_point_values(left_id, &mut left_point);
                    self.fill_point_values(right_id, &mut right_point);
                    fill_midpoint(left, right, &mut mid_point, ALPHA);
                    do_subdivision = ALPHA != 0.0 && ALPHA != 1.0;

                    if do_subdivision {
                        self.evaluate_point_values(&mut mid_point);
                        do_subdivision =
                            tri.get_subdivision_level() < self.get_fixed_subdivisions();
                        if !do_subdivision {
                            do_subdivision = self.tessellator.requires_edge_subdivision(
                                &mut left_point,
                                &mut mid_point,
                                &mut right_point,
                                ALPHA,
                            ) != 0;
                        }
                    }
                }

                if do_subdivision {
                    self.edge_table.borrow_mut().insert_edge_with_point(
                        left_id,
                        right_id,
                        cell_id,
                        ref_count,
                        &mut point_id,
                    );
                    assert!(point_id != -1, "check: id exists");

                    let local = parametric_from_workspace(&mid_point);
                    tri.set_vertex(j + 3, local);
                    tri.set_point_id(j + 3, point_id);
                    tri.set_edge_parent(j + 3, l, r);

                    self.edge_table.borrow_mut().insert_point_and_scalar(
                        point_id,
                        global_from_workspace(&mid_point),
                        &mid_point[ATTRIBUTES_OFFSET..],
                    );
                } else {
                    self.edge_table
                        .borrow_mut()
                        .insert_edge(left_id, right_id, cell_id, ref_count);
                }
            } else {
                self.edge_table
                    .borrow_mut()
                    .increment_edge_reference_count(left_id, right_id, cell_id);

                if to_split == 1 {
                    tri.set_point_id(j + 3, point_id);
                    let pcoords = midpoint(left, right, ALPHA);
                    tri.set_vertex(j + 3, pcoords);
                    tri.set_edge_parent(j + 3, l, r);
                }
            }
        }
    }

    /// VTK: `vtkSimpleCellTessellator::RemoveEdgesFromEdgeTable(vtkTriangleTile&)`.
    fn remove_edges_from_edge_table(&mut self, tri: &TriangleTile) {
        for i in 0..3 {
            self.edge_table
                .borrow_mut()
                .remove_point(tri.get_point_id(i));
        }

        for edge in TRIANGLE_EDGES_TABLE {
            self.edge_table
                .borrow_mut()
                .remove_edge(tri.get_point_id(edge[0]), tri.get_point_id(edge[1]));
        }
    }

    /// VTK: `vtkTriangleTile::Refine`.
    fn refine_triangle_tile(&mut self, tile: &TriangleTile) -> Vec<TriangleTile> {
        let mut result = Vec::new();

        if tile.get_subdivision_level() < self.get_max_subdivision_level() {
            let mut index = 0;
            for (i, edge) in TRIANGLE_EDGES_TABLE.iter().enumerate() {
                let mut point_id = 0;
                let edge_split = self.edge_table.borrow().check_edge(
                    tile.get_point_id(edge[0]),
                    tile.get_point_id(edge[1]),
                    &mut point_id,
                );
                assert!(edge_split != -1, "check: edge table prepared");
                if edge_split != 0 {
                    index |= 1 << i;
                }
            }

            if index != 0 {
                for case in VTK_TESSELLATOR_TRIANGLE_CASES[index] {
                    if case[0] <= -1 {
                        break;
                    }
                    let mut piece = TriangleTile::default();
                    for j in 0..3 {
                        piece.copy_point(j, tile, case[j] as usize);
                    }
                    piece.set_subdivision_level(tile.get_subdivision_level() + 1);
                    self.insert_edges_into_edge_table(&mut piece);
                    result.push(piece);
                }
            }
        }

        if result.is_empty() {
            self.tessellate_cell_array
                .as_ref()
                .expect("pre: tessellate_cell_array_exists")
                .borrow_mut()
                .insert_next_cell(tile.point_ids());
            for j in 0..3 {
                self.copy_point(tile.get_point_id(j));
            }
        }

        result
    }

    fn fill_point_values(&self, point_id: VtkIdType, point: &mut [f64]) {
        let mut global = [0.0; 3];
        let mut scalars = vec![0.0; point.len().saturating_sub(ATTRIBUTES_OFFSET)];
        assert!(
            self.edge_table
                .borrow()
                .check_point_values(point_id, &mut global, &mut scalars),
            "pre: edge_point_exists"
        );
        point[..3].copy_from_slice(&global);
        point[ATTRIBUTES_OFFSET..ATTRIBUTES_OFFSET + scalars.len()].copy_from_slice(&scalars);
    }

    fn evaluate_point_values(&self, point: &mut [f64]) {
        let pcoords = parametric_from_workspace(point);
        let global = self.generic_cell().borrow().evaluate_location(0, pcoords);
        point[..3].copy_from_slice(&global);
        let mut scalars = vec![0.0; point.len().saturating_sub(ATTRIBUTES_OFFSET)];
        self.generic_cell().borrow().interpolate_tuple_collection(
            self.attribute_collection(),
            pcoords,
            &mut scalars,
        );
        point[ATTRIBUTES_OFFSET..ATTRIBUTES_OFFSET + scalars.len()].copy_from_slice(&scalars);
    }

    fn point_workspace(&self) -> Vec<f64> {
        vec![0.0; self.point_offset as usize]
    }

    fn point_data_components(&self) -> usize {
        self.tessellate_point_data
            .as_ref()
            .map(|pd| pd.borrow().get_number_of_components().max(0) as usize)
            .unwrap_or(0)
    }

    fn generic_cell(&self) -> GenericAdaptorCellHandle {
        self.generic_cell
            .as_ref()
            .expect("pre: generic_cell_exists")
            .clone()
    }

    fn attribute_collection(&self) -> GenericAttributeCollectionHandle {
        self.attribute_collection
            .as_ref()
            .expect("pre: attribute_collection_exists")
            .clone()
    }

    fn get_number_of_cells_using_edge(&self, edge_id: i32) -> i32 {
        assert!(edge_id >= 0, "pre: valid_range");
        let mut edge_sharing = [0; 18];
        self.generic_cell()
            .borrow()
            .count_edge_neighbors(&mut edge_sharing);
        edge_sharing[edge_id as usize] + 1
    }

    /// VTK: `vtkGenericCellTessellator::SetErrorMetrics`.
    pub fn set_error_metrics(&mut self, error_metrics: Vec<GenericSubdivisionErrorMetricHandle>) {
        self.tessellator.set_error_metrics(error_metrics);
    }

    /// VTK: `vtkGenericCellTessellator::GetMeasurement`.
    pub fn get_measurement(&self) -> i32 {
        self.tessellator.get_measurement()
    }

    /// VTK: `vtkGenericCellTessellator::SetMeasurement`.
    pub fn set_measurement(&mut self, measurement: i32) {
        self.tessellator.set_measurement(measurement);
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.tessellator.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.tessellator.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.tessellator.get_m_time()
    }
}

impl Default for SimpleCellTessellator {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SimpleCellTessellator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCellTessellator")
            .field("tessellator", &self.tessellator)
            .field("generic_cell", &self.generic_cell.as_ref().map(Rc::as_ptr))
            .field("scalars_capacity", &self.scalars_capacity)
            .field("point_offset", &self.point_offset)
            .field("number_of_points", &self.number_of_points)
            .field("fixed_subdivisions", &self.fixed_subdivisions)
            .field("max_subdivision_level", &self.max_subdivision_level)
            .field("current_subdivision_level", &self.current_subdivision_level)
            .field("point_ids_capacity", &self.point_ids_capacity)
            .field("edge_ids", &self.edge_ids)
            .field(
                "attribute_collection",
                &self.attribute_collection.as_ref().map(Rc::as_ptr),
            )
            .finish()
    }
}

#[derive(Clone, Debug)]
struct TriangleTile {
    vertices: [[f64; 3]; 6],
    point_ids: [VtkIdType; 6],
    subdivision_level: i32,
    classification_state: [u8; 6],
}

impl Default for TriangleTile {
    fn default() -> Self {
        Self {
            vertices: [[-100.0; 3]; 6],
            point_ids: [-1; 6],
            subdivision_level: 0,
            classification_state: [0; 6],
        }
    }
}

impl TriangleTile {
    fn set_subdivision_level(&mut self, level: i32) {
        assert!(level >= 0, "pre: positive_level");
        self.subdivision_level = level;
    }

    fn get_subdivision_level(&self) -> i32 {
        self.subdivision_level
    }

    fn set_vertex(&mut self, i: usize, vertex: [f64; 3]) {
        self.vertices[i] = vertex;
    }

    fn get_vertex(&self, i: usize) -> [f64; 3] {
        self.vertices[i]
    }

    fn set_point_id(&mut self, i: usize, id: VtkIdType) {
        self.point_ids[i] = id;
    }

    fn set_point_ids(&mut self, ids: [VtkIdType; 3]) {
        self.point_ids[..3].copy_from_slice(&ids);
    }

    fn get_point_id(&self, i: usize) -> VtkIdType {
        self.point_ids[i]
    }

    fn point_ids(&self) -> &[VtkIdType] {
        &self.point_ids[..3]
    }

    fn set_original(&mut self) {
        self.classification_state[0] = TRIANGLE_VERTEX_STATE[0];
        self.classification_state[1] = TRIANGLE_VERTEX_STATE[1];
        self.classification_state[2] = TRIANGLE_VERTEX_STATE[2];
    }

    fn is_an_edge(&self, e1: VtkIdType, e2: VtkIdType) -> bool {
        let mut sum = 0;
        for point_id in &self.point_ids[..3] {
            if e1 == *point_id || e2 == *point_id {
                sum += 1;
            }
        }
        sum == 2
    }

    fn find_edge_parent(&self, p1: usize, p2: usize) -> i8 {
        assert!(p1 <= 2 && p2 <= 2, "pre: primary point");
        let mid_point_state = self.classification_state[p1] & self.classification_state[p2];
        if mid_point_state == 0 {
            -1
        } else if (mid_point_state & 1) != 0 {
            0
        } else if (mid_point_state & 2) != 0 {
            1
        } else {
            2
        }
    }

    fn set_edge_parent(&mut self, mid: usize, p1: usize, p2: usize) {
        assert!((3..=5).contains(&mid), "pre: mid-point");
        assert!(p1 <= 2 && p2 <= 2, "pre: primary point");
        self.classification_state[mid] =
            self.classification_state[p1] & self.classification_state[p2];
    }

    fn copy_point(&mut self, i: usize, source: &TriangleTile, j: usize) {
        assert!(i <= 2, "pre: primary_i");
        assert!(j <= 5, "pre: valid_j");
        self.point_ids[i] = source.point_ids[j];
        self.vertices[i] = source.vertices[j];
        self.classification_state[i] = source.classification_state[j];
    }
}

#[derive(Clone, Debug)]
struct TetraTile {
    vertices: [[f64; 3]; 10],
    point_ids: [VtkIdType; 10],
    subdivision_level: i32,
    classification_state: [u16; 10],
    edge_ids: [i32; 6],
    face_ids: [i32; 4],
}

impl Default for TetraTile {
    /// VTK: `vtkTetraTile::vtkTetraTile`.
    fn default() -> Self {
        Self {
            vertices: [[-100.0; 3]; 10],
            point_ids: [-1; 10],
            subdivision_level: 0,
            classification_state: [0; 10],
            edge_ids: [-1; 6],
            face_ids: [-1; 4],
        }
    }
}

impl TetraTile {
    /// VTK: `vtkTetraTile::SetSubdivisionLevel`.
    fn set_subdivision_level(&mut self, level: i32) {
        assert!(level >= 0, "pre: positive_level");
        self.subdivision_level = level;
    }

    /// VTK: `vtkTetraTile::GetSubdivisionLevel`.
    fn get_subdivision_level(&self) -> i32 {
        self.subdivision_level
    }

    /// VTK: `vtkTetraTile::SetVertex`.
    fn set_vertex(&mut self, i: usize, vertex: [f64; 3]) {
        self.vertices[i] = vertex;
    }

    /// VTK: `vtkTetraTile::GetVertex`.
    fn get_vertex(&self, i: usize) -> [f64; 3] {
        self.vertices[i]
    }

    /// VTK: `vtkTetraTile::SetPointId`.
    fn set_point_id(&mut self, i: usize, id: VtkIdType) {
        self.point_ids[i] = id;
    }

    /// VTK: `vtkTetraTile::SetPointIds`.
    fn set_point_ids(&mut self, ids: [VtkIdType; 4]) {
        self.point_ids[..4].copy_from_slice(&ids);
    }

    /// VTK: `vtkTetraTile::GetPointId`.
    fn get_point_id(&self, i: usize) -> VtkIdType {
        self.point_ids[i]
    }

    /// VTK: `vtkTetraTile::IsAnEdge`.
    fn is_an_edge(&self, e1: VtkIdType, e2: VtkIdType) -> bool {
        let mut sum = 0;
        for point_id in &self.point_ids[..4] {
            if e1 == *point_id || e2 == *point_id {
                sum += 1;
            }
        }
        sum == 2
    }

    /// VTK: `vtkTetraTile::CopyPoint`.
    fn copy_point(&mut self, i: usize, source: &TetraTile, j: usize) {
        assert!(i <= 3, "pre: primary_i");
        assert!(j <= 9, "pre: valid_j");
        self.point_ids[i] = source.point_ids[j];
        self.vertices[i] = source.vertices[j];
        self.classification_state[i] = source.classification_state[j];
    }

    /// VTK: `vtkTetraTile::CopyEdgeAndFaceIds`.
    fn copy_edge_and_face_ids(&mut self, source: &TetraTile) {
        self.edge_ids = source.edge_ids;
        self.face_ids = source.face_ids;
    }

    /// VTK: `vtkTetraTile::GetEdgeIds`.
    fn get_edge_ids(&self, idx: usize) -> i32 {
        assert!(idx < self.edge_ids.len(), "pre: valid_edge_id");
        self.edge_ids[idx]
    }

    /// VTK: `vtkTetraTile::GetFaceIds`.
    fn get_face_ids(&self, idx: usize) -> i32 {
        assert!(idx < self.face_ids.len(), "pre: valid_face_id");
        self.face_ids[idx]
    }

    /// VTK: `vtkTetraTile::SetOriginal`.
    fn set_original(&mut self, order: [usize; 4], edge_ids: [i32; 6], face_ids: [i32; 4]) {
        self.edge_ids = edge_ids;
        self.face_ids = face_ids;

        for (i, original_vertex) in order.into_iter().enumerate() {
            self.classification_state[i] = TETRA_VERTEX_STATE[original_vertex];
            for n in 0..3 {
                let edge = VERTEX_EDGES[original_vertex][n];
                if self.edge_ids[edge] == -1 {
                    self.classification_state[i] &= !(1_u16 << edge);
                }
                let face = VERTEX_FACES[original_vertex][n];
                if self.face_ids[face] == -1 {
                    self.classification_state[i] &= !(1_u16 << (face + 6));
                }
            }
        }
    }

    /// VTK: `vtkTetraTile::FindEdgeParent`.
    fn find_edge_parent(&self, p1: usize, p2: usize) -> (i32, i8) {
        assert!(p1 <= 3 && p2 <= 3, "pre: primary point");
        let mid_point_state = self.classification_state[p1] & self.classification_state[p2];
        if mid_point_state == 0 {
            (3, -1)
        } else if (mid_point_state & 0x3f) != 0 {
            let mut parent_id = 0;
            let mut mask = 1_u16;
            while parent_id < 6 && (mid_point_state & mask) == 0 {
                mask <<= 1;
                parent_id += 1;
            }
            (1, parent_id)
        } else {
            let mut parent_id = 0;
            let mut mask = 0x40_u16;
            while parent_id < 4 && (mid_point_state & mask) == 0 {
                mask <<= 1;
                parent_id += 1;
            }
            (2, parent_id)
        }
    }

    /// VTK: `vtkTetraTile::SetParent`.
    fn set_parent(&mut self, mid: usize, p1: usize, p2: usize) {
        assert!((4..=9).contains(&mid), "pre: mid-point");
        assert!(p1 <= 3 && p2 <= 3, "pre: primary point");
        self.classification_state[mid] =
            self.classification_state[p1] & self.classification_state[p2];
    }
}

/// VTK: static `Reorder` helper in `vtkSimpleCellTessellator.cxx`.
fn reorder(input: [VtkIdType; 4]) -> [usize; 4] {
    let mut min1 = input[0];
    let mut min2 = input[1];
    let mut idx1 = 0;
    let mut idx2 = 1;
    for (i, value) in input.iter().enumerate().skip(1) {
        if min1 > *value {
            min2 = min1;
            idx2 = idx1;
            min1 = *value;
            idx1 = i;
        } else if min2 > *value {
            min2 = *value;
            idx2 = i;
        }
    }

    let mut order = [idx1, idx2, 0, 0];
    if idx1 == 0 {
        if idx2 == 1 {
            order[2] = 2;
            order[3] = 3;
        } else if idx2 == 2 {
            order[2] = 3;
            order[3] = 1;
        } else if idx2 == 3 {
            order[2] = 1;
            order[3] = 2;
        }
    } else if idx1 == 1 {
        if idx2 == 0 {
            order[2] = 3;
            order[3] = 2;
        } else if idx2 == 2 {
            order[2] = 0;
            order[3] = 3;
        } else if idx2 == 3 {
            order[2] = 2;
            order[3] = 0;
        }
    } else if idx1 == 2 {
        if idx2 == 0 {
            order[2] = 1;
            order[3] = 3;
        } else if idx2 == 1 {
            order[2] = 3;
            order[3] = 0;
        } else if idx2 == 3 {
            order[2] = 0;
            order[3] = 1;
        }
    } else if idx1 == 3 {
        if idx2 == 0 {
            order[2] = 2;
            order[3] = 1;
        } else if idx2 == 1 {
            order[2] = 0;
            order[3] = 2;
        } else if idx2 == 2 {
            order[2] = 1;
            order[3] = 0;
        }
    }
    order
}

fn parametric_tuple(coords: &[f64], point_id: VtkIdType) -> [f64; 3] {
    let start = (point_id as usize) * 3;
    [coords[start], coords[start + 1], coords[start + 2]]
}

fn midpoint(left: [f64; 3], right: [f64; 3], alpha: f64) -> [f64; 3] {
    [
        left[0] + alpha * (right[0] - left[0]),
        left[1] + alpha * (right[1] - left[1]),
        left[2] + alpha * (right[2] - left[2]),
    ]
}

fn fill_midpoint(left: [f64; 3], right: [f64; 3], point: &mut [f64], alpha: f64) {
    point[PARAMETRIC_OFFSET..PARAMETRIC_OFFSET + 3].copy_from_slice(&midpoint(left, right, alpha));
}

fn global_from_workspace(point: &[f64]) -> [f64; 3] {
    [point[0], point[1], point[2]]
}

fn parametric_from_workspace(point: &[f64]) -> [f64; 3] {
    [
        point[PARAMETRIC_OFFSET],
        point[PARAMETRIC_OFFSET + 1],
        point[PARAMETRIC_OFFSET + 2],
    ]
}
