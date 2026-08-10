use crate::common::{
    core::{AnyArray, ArrayError, DoubleArray, VtkDataType, VtkIdType},
    data_model::{
        StructuredData, VTK_STRUCTURED_EMPTY, VTK_STRUCTURED_SINGLE_POINT, VTK_STRUCTURED_XYZ_GRID,
        VTK_STRUCTURED_XY_PLANE, VTK_STRUCTURED_XZ_PLANE, VTK_STRUCTURED_X_LINE,
        VTK_STRUCTURED_YZ_PLANE, VTK_STRUCTURED_Y_LINE, VTK_STRUCTURED_Z_LINE,
    },
};
use std::sync::Arc;

const IDENTITY_MATRIX_3X3: [f64; 9] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

/// Backend for implicit structured point coordinates.
///
/// VTK origin: `VTK/Common/Core/vtkStructuredPointBackend.h` and
/// `VTK/Common/Core/vtkStructuredPointBackend.txx`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredPointBackend {
    storage: Arc<StructuredPointBackendStorage>,
}

#[derive(Debug, Clone, PartialEq)]
struct StructuredPointBackendStorage {
    x_coordinates: AnyArray,
    y_coordinates: AnyArray,
    z_coordinates: AnyArray,
    extent: [i32; 6],
    dimensions: [VtkIdType; 3],
    data_description: i32,
    uses_direction_matrix: bool,
    index_to_physical_matrix: [f64; 16],
}

impl StructuredPointBackend {
    pub fn new(
        x_coordinates: &AnyArray,
        y_coordinates: &AnyArray,
        z_coordinates: &AnyArray,
        extent: [i32; 6],
        data_description: i32,
        direction_matrix: [f64; 9],
    ) -> Self {
        let uses_direction_matrix = direction_matrix != IDENTITY_MATRIX_3X3;
        let dimensions = StructuredData::get_dimensions_from_extent(extent);
        let mut index_to_physical_matrix = [0.0; 16];

        if uses_direction_matrix {
            let origin = [
                numeric_component(x_coordinates, 0, 0),
                numeric_component(y_coordinates, 0, 0),
                numeric_component(z_coordinates, 0, 0),
            ];
            let spacing = [
                numeric_component(x_coordinates, 1, 0) - origin[0],
                numeric_component(y_coordinates, 1, 0) - origin[1],
                numeric_component(z_coordinates, 1, 0) - origin[2],
            ];
            index_to_physical_matrix[0] = direction_matrix[0] * spacing[0];
            index_to_physical_matrix[1] = direction_matrix[1] * spacing[1];
            index_to_physical_matrix[2] = direction_matrix[2] * spacing[2];
            index_to_physical_matrix[3] = origin[0];
            index_to_physical_matrix[4] = direction_matrix[3] * spacing[0];
            index_to_physical_matrix[5] = direction_matrix[4] * spacing[1];
            index_to_physical_matrix[6] = direction_matrix[5] * spacing[2];
            index_to_physical_matrix[7] = origin[1];
            index_to_physical_matrix[8] = direction_matrix[6] * spacing[0];
            index_to_physical_matrix[9] = direction_matrix[7] * spacing[1];
            index_to_physical_matrix[10] = direction_matrix[8] * spacing[2];
            index_to_physical_matrix[11] = origin[2];
            index_to_physical_matrix[15] = 1.0;
        }

        Self {
            storage: Arc::new(StructuredPointBackendStorage {
                x_coordinates: x_coordinates.shallow_clone(),
                y_coordinates: y_coordinates.shallow_clone(),
                z_coordinates: z_coordinates.shallow_clone(),
                extent,
                dimensions: [
                    VtkIdType::from(dimensions[0]),
                    VtkIdType::from(dimensions[1]),
                    VtkIdType::from(dimensions[2]),
                ],
                data_description,
                uses_direction_matrix,
                index_to_physical_matrix,
            }),
        }
    }

    /// VTK: `vtkStructuredPointBackend::mapStructuredXComponent`.
    pub fn map_structured_x_component(&self, i: i32) -> f64 {
        if self.storage.data_description == VTK_STRUCTURED_EMPTY {
            0.0
        } else {
            numeric_component(&self.storage.x_coordinates, i as VtkIdType, 0)
        }
    }

    /// VTK: `vtkStructuredPointBackend::mapStructuredYComponent`.
    pub fn map_structured_y_component(&self, j: i32) -> f64 {
        if self.storage.data_description == VTK_STRUCTURED_EMPTY {
            0.0
        } else {
            numeric_component(&self.storage.y_coordinates, j as VtkIdType, 0)
        }
    }

    /// VTK: `vtkStructuredPointBackend::mapStructuredZComponent`.
    pub fn map_structured_z_component(&self, k: i32) -> f64 {
        if self.storage.data_description == VTK_STRUCTURED_EMPTY {
            0.0
        } else {
            numeric_component(&self.storage.z_coordinates, k as VtkIdType, 0)
        }
    }

    /// VTK: `vtkStructuredPointBackend::mapStructuredTuple`.
    pub fn map_structured_tuple(&self, ijk: [i32; 3]) -> [f64; 3] {
        if self.storage.uses_direction_matrix {
            self.transform_index_to_physical_point([
                ijk[0] + self.storage.extent[0],
                ijk[1] + self.storage.extent[2],
                ijk[2] + self.storage.extent[4],
            ])
        } else if self.storage.data_description == VTK_STRUCTURED_EMPTY {
            [0.0, 0.0, 0.0]
        } else {
            [
                numeric_component(&self.storage.x_coordinates, ijk[0] as VtkIdType, 0),
                numeric_component(&self.storage.y_coordinates, ijk[1] as VtkIdType, 0),
                numeric_component(&self.storage.z_coordinates, ijk[2] as VtkIdType, 0),
            ]
        }
    }

    /// VTK: `vtkStructuredPointBackend::mapTuple`.
    pub fn map_tuple(&self, tuple_id: VtkIdType) -> [f64; 3] {
        let ijk = self.compute_point_structured_coords(tuple_id);
        if self.storage.uses_direction_matrix {
            self.transform_index_to_physical_point([
                ijk[0] + self.storage.extent[0],
                ijk[1] + self.storage.extent[2],
                ijk[2] + self.storage.extent[4],
            ])
        } else if self.storage.data_description == VTK_STRUCTURED_EMPTY {
            [0.0, 0.0, 0.0]
        } else {
            [
                numeric_component(&self.storage.x_coordinates, ijk[0] as VtkIdType, 0),
                numeric_component(&self.storage.y_coordinates, ijk[1] as VtkIdType, 0),
                numeric_component(&self.storage.z_coordinates, ijk[2] as VtkIdType, 0),
            ]
        }
    }

    /// VTK: `vtkStructuredPointBackend::mapComponent`.
    pub fn map_component(&self, tuple_id: VtkIdType, component: i32) -> f64 {
        if self.storage.uses_direction_matrix {
            return self.map_tuple(tuple_id)[component as usize];
        }

        match self.storage.data_description {
            VTK_STRUCTURED_EMPTY => 0.0,
            VTK_STRUCTURED_SINGLE_POINT => match component {
                0 => numeric_component(&self.storage.x_coordinates, 0, 0),
                1 => numeric_component(&self.storage.y_coordinates, 0, 0),
                2 => numeric_component(&self.storage.z_coordinates, 0, 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_X_LINE => match component {
                0 => numeric_component(&self.storage.x_coordinates, tuple_id, 0),
                1 => numeric_component(&self.storage.y_coordinates, 0, 0),
                2 => numeric_component(&self.storage.z_coordinates, 0, 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_Y_LINE => match component {
                0 => numeric_component(&self.storage.x_coordinates, 0, 0),
                1 => numeric_component(&self.storage.y_coordinates, tuple_id, 0),
                2 => numeric_component(&self.storage.z_coordinates, 0, 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_Z_LINE => match component {
                0 => numeric_component(&self.storage.x_coordinates, 0, 0),
                1 => numeric_component(&self.storage.y_coordinates, 0, 0),
                2 => numeric_component(&self.storage.z_coordinates, tuple_id, 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_XY_PLANE => match component {
                0 => numeric_component(&self.storage.x_coordinates, tuple_id % self.dim0(), 0),
                1 => numeric_component(&self.storage.y_coordinates, tuple_id / self.dim0(), 0),
                2 => numeric_component(&self.storage.z_coordinates, 0, 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_YZ_PLANE => match component {
                0 => numeric_component(&self.storage.x_coordinates, 0, 0),
                1 => numeric_component(&self.storage.y_coordinates, tuple_id % self.dim1(), 0),
                2 => numeric_component(&self.storage.z_coordinates, tuple_id / self.dim1(), 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_XZ_PLANE => match component {
                0 => numeric_component(&self.storage.x_coordinates, tuple_id % self.dim0(), 0),
                1 => numeric_component(&self.storage.y_coordinates, 0, 0),
                2 => numeric_component(&self.storage.z_coordinates, tuple_id / self.dim0(), 0),
                _ => 0.0,
            },
            VTK_STRUCTURED_XYZ_GRID => match component {
                0 => numeric_component(&self.storage.x_coordinates, tuple_id % self.dim0(), 0),
                1 => numeric_component(
                    &self.storage.y_coordinates,
                    (tuple_id / self.dim0()) % self.dim1(),
                    0,
                ),
                2 => numeric_component(&self.storage.z_coordinates, tuple_id / self.dim01(), 0),
                _ => 0.0,
            },
            _ => 0.0,
        }
    }

    /// VTK: `vtkStructuredPointBackend::map`.
    pub fn map(&self, value_id: VtkIdType) -> f64 {
        self.map_component(value_id / 3, (value_id % 3) as i32)
    }

    /// VTK: `vtkStructuredPointBackend::GetXCoordinates`.
    pub fn get_x_coordinates(&self) -> &AnyArray {
        &self.storage.x_coordinates
    }

    /// VTK: `vtkStructuredPointBackend::GetYCoordinates`.
    pub fn get_y_coordinates(&self) -> &AnyArray {
        &self.storage.y_coordinates
    }

    /// VTK: `vtkStructuredPointBackend::GetZCoordinates`.
    pub fn get_z_coordinates(&self) -> &AnyArray {
        &self.storage.z_coordinates
    }

    /// VTK: `vtkStructuredPointBackend::GetUsesDirectionMatrix`.
    pub fn get_uses_direction_matrix(&self) -> bool {
        self.storage.uses_direction_matrix
    }

    fn compute_point_structured_coords(&self, point_id: VtkIdType) -> [i32; 3] {
        match self.storage.data_description {
            VTK_STRUCTURED_EMPTY | VTK_STRUCTURED_SINGLE_POINT => [0, 0, 0],
            VTK_STRUCTURED_X_LINE => [point_id as i32, 0, 0],
            VTK_STRUCTURED_Y_LINE => [0, point_id as i32, 0],
            VTK_STRUCTURED_Z_LINE => [0, 0, point_id as i32],
            VTK_STRUCTURED_XY_PLANE => [
                (point_id % self.dim0()) as i32,
                (point_id / self.dim0()) as i32,
                0,
            ],
            VTK_STRUCTURED_YZ_PLANE => [
                0,
                (point_id % self.dim1()) as i32,
                (point_id / self.dim1()) as i32,
            ],
            VTK_STRUCTURED_XZ_PLANE => [
                (point_id % self.dim0()) as i32,
                0,
                (point_id / self.dim0()) as i32,
            ],
            VTK_STRUCTURED_XYZ_GRID => [
                (point_id % self.dim0()) as i32,
                ((point_id / self.dim0()) % self.dim1()) as i32,
                (point_id / self.dim01()) as i32,
            ],
            _ => [0, 0, 0],
        }
    }

    fn transform_index_to_physical_point(&self, ijk: [i32; 3]) -> [f64; 3] {
        let m = self.storage.index_to_physical_matrix;
        let i = f64::from(ijk[0]);
        let j = f64::from(ijk[1]);
        let k = f64::from(ijk[2]);
        [
            m[0] * i + m[1] * j + m[2] * k + m[3],
            m[4] * i + m[5] * j + m[6] * k + m[7],
            m[8] * i + m[9] * j + m[10] * k + m[11],
        ]
    }

    fn dim0(&self) -> VtkIdType {
        self.storage.dimensions[0]
    }

    fn dim1(&self) -> VtkIdType {
        self.storage.dimensions[1]
    }

    fn dim01(&self) -> VtkIdType {
        self.storage.dimensions[0] * self.storage.dimensions[1]
    }
}

/// Double-valued `vtkStructuredPointArray`.
///
/// VTK origin: `VTK/Common/Core/vtkStructuredPointArray.h` and
/// `VTK/Common/Core/vtkStructuredPointArray.txx`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredPointArray {
    storage: Arc<StructuredPointArrayStorage>,
}

#[derive(Debug, Clone, PartialEq)]
struct StructuredPointArrayStorage {
    name: String,
    backend: Option<StructuredPointBackend>,
    number_of_components: i32,
    number_of_tuples: VtkIdType,
    modified_time: u64,
}

impl Default for StructuredPointArray {
    fn default() -> Self {
        Self::new()
    }
}

impl StructuredPointArray {
    /// VTK: `vtkStructuredPointArray<ValueTypeT>::New`.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(StructuredPointArrayStorage {
                name: String::new(),
                backend: None,
                number_of_components: 1,
                number_of_tuples: 0,
                modified_time: 0,
            }),
        }
    }

    fn storage_mut(&mut self) -> &mut StructuredPointArrayStorage {
        Arc::make_mut(&mut self.storage)
    }

    /// VTK: `vtkStructuredPointArray::ConstructBackend`.
    pub fn construct_backend(
        &mut self,
        x_coordinates: &AnyArray,
        y_coordinates: &AnyArray,
        z_coordinates: &AnyArray,
        extent: [i32; 6],
        data_description: i32,
        direction_matrix: [f64; 9],
    ) {
        let backend = StructuredPointBackend::new(
            x_coordinates,
            y_coordinates,
            z_coordinates,
            extent,
            data_description,
            direction_matrix,
        );
        let storage = self.storage_mut();
        storage.backend = Some(backend);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    /// VTK: `vtkStructuredPointArray::ConstructBackend` identity-matrix overload.
    pub fn construct_backend_without_direction_matrix(
        &mut self,
        x_coordinates: &AnyArray,
        y_coordinates: &AnyArray,
        z_coordinates: &AnyArray,
        extent: [i32; 6],
        data_description: i32,
    ) {
        self.construct_backend(
            x_coordinates,
            y_coordinates,
            z_coordinates,
            extent,
            data_description,
            IDENTITY_MATRIX_3X3,
        );
    }

    /// VTK: `vtk::CreateStructuredPointArray<double>`.
    pub fn create_structured_point_array(
        x_coordinates: &AnyArray,
        y_coordinates: &AnyArray,
        z_coordinates: &AnyArray,
        extent: [i32; 6],
        data_description: i32,
        direction_matrix: [f64; 9],
    ) -> Self {
        let mut array = Self::new();
        array.construct_backend(
            x_coordinates,
            y_coordinates,
            z_coordinates,
            extent,
            data_description,
            direction_matrix,
        );
        array.set_number_of_components(3);
        array.set_number_of_tuples(StructuredData::get_number_of_points(extent));
        array
    }

    /// VTK: `vtkStructuredPointArray::GetXCoordinates`.
    pub fn get_x_coordinates(&self) -> Option<&AnyArray> {
        self.storage
            .backend
            .as_ref()
            .map(StructuredPointBackend::get_x_coordinates)
    }

    /// VTK: `vtkStructuredPointArray::GetYCoordinates`.
    pub fn get_y_coordinates(&self) -> Option<&AnyArray> {
        self.storage
            .backend
            .as_ref()
            .map(StructuredPointBackend::get_y_coordinates)
    }

    /// VTK: `vtkStructuredPointArray::GetZCoordinates`.
    pub fn get_z_coordinates(&self) -> Option<&AnyArray> {
        self.storage
            .backend
            .as_ref()
            .map(StructuredPointBackend::get_z_coordinates)
    }

    /// VTK: `vtkStructuredPointArray::GetUsesDirectionMatrix`.
    pub fn get_uses_direction_matrix(&self) -> bool {
        self.storage
            .backend
            .as_ref()
            .is_some_and(StructuredPointBackend::get_uses_direction_matrix)
    }

    pub fn get_backend(&self) -> Option<&StructuredPointBackend> {
        self.storage.backend.as_ref()
    }

    pub fn get_name(&self) -> &str {
        &self.storage.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let storage = self.storage_mut();
        storage.name = name.into();
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_data_type(&self) -> VtkDataType {
        VtkDataType::Double
    }

    pub fn get_number_of_components(&self) -> i32 {
        self.storage.number_of_components
    }

    pub fn set_number_of_components(&mut self, number_of_components: i32) {
        let storage = self.storage_mut();
        storage.number_of_components = number_of_components.max(1);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_number_of_tuples(&self) -> VtkIdType {
        self.storage.number_of_tuples
    }

    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        let storage = self.storage_mut();
        storage.number_of_tuples = number_of_tuples.max(0);
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn get_number_of_values(&self) -> VtkIdType {
        self.get_number_of_tuples()
            .saturating_mul(VtkIdType::from(self.get_number_of_components()))
    }

    pub fn reserve_tuples(&mut self, _number_of_tuples: VtkIdType) -> bool {
        true
    }

    pub fn reserve_values(&mut self, _number_of_values: VtkIdType) -> bool {
        true
    }

    pub fn initialize(&mut self) {
        let storage = self.storage_mut();
        storage.backend = None;
        storage.number_of_tuples = 0;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn reset(&mut self) {
        let storage = self.storage_mut();
        storage.number_of_tuples = 0;
        storage.modified_time = storage.modified_time.saturating_add(1);
    }

    pub fn squeeze(&mut self) {}

    pub fn remove_tuple(&mut self, _tuple_idx: VtkIdType) {}

    pub fn get_actual_memory_size(&self) -> usize {
        0
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        0
    }

    pub fn has_standard_memory_layout(&self) -> bool {
        false
    }

    pub fn get_m_time(&self) -> u64 {
        self.storage.modified_time
    }

    pub(crate) fn deep_clone(&self) -> Self {
        Self {
            storage: Arc::new((*self.storage).clone()),
        }
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        self.clone()
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.storage, &other.storage)
    }

    pub(crate) fn checked_tuple_as_f64(&self, tuple_idx: usize) -> Result<Vec<f64>, ArrayError> {
        Ok(self
            .storage
            .backend
            .as_ref()
            .map(|backend| backend.map_tuple(tuple_idx as VtkIdType).to_vec())
            .unwrap_or_else(|| vec![0.0; self.get_number_of_components() as usize]))
    }

    pub(crate) fn set_typed_tuple_from_f64(&mut self, _tuple_idx: usize, _tuple: &[f64]) {}

    pub(crate) fn insert_typed_tuple_from_f64(&mut self, _tuple_idx: usize, _tuple: &[f64]) {}

    pub fn set_component(&mut self, _tuple_idx: VtkIdType, _component_idx: i32, _value: f64) {}

    pub fn get_component(&self, tuple_idx: VtkIdType, component_idx: i32) -> f64 {
        self.storage
            .backend
            .as_ref()
            .map(|backend| backend.map_component(tuple_idx, component_idx))
            .unwrap_or(0.0)
    }

    pub fn set_component_name(&mut self, _component: VtkIdType, _name: impl Into<String>) {}

    pub fn get_component_name(&self, _component: VtkIdType) -> Option<&str> {
        None
    }

    pub(crate) fn has_a_component_name(&self) -> bool {
        false
    }

    pub(crate) fn compute_scalar_range(&self, ranges: &mut [f64]) -> bool {
        let components = self.get_number_of_components().max(0) as usize;
        if ranges.len() < components * 2 {
            return false;
        }
        for component in 0..components {
            let mut min = f64::INFINITY;
            let mut max = f64::NEG_INFINITY;
            for tuple_id in 0..self.get_number_of_tuples() {
                let value = self.get_component(tuple_id, component as i32);
                min = min.min(value);
                max = max.max(value);
            }
            ranges[component * 2] = min;
            ranges[component * 2 + 1] = max;
        }
        true
    }

    pub(crate) fn compute_finite_scalar_range(&self, ranges: &mut [f64]) -> bool {
        self.compute_scalar_range(ranges)
    }

    pub(crate) fn compute_vector_range(&self, range: &mut [f64]) -> bool {
        if range.len() < 2 {
            return false;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for tuple_id in 0..self.get_number_of_tuples() {
            let tuple = self
                .checked_tuple_as_f64(tuple_id as usize)
                .expect("structured point tuple read must be infallible");
            let norm = tuple.iter().map(|value| value * value).sum::<f64>().sqrt();
            min = min.min(norm);
            max = max.max(norm);
        }
        range[0] = min;
        range[1] = max;
        true
    }

    pub(crate) fn compute_finite_vector_range(&self, range: &mut [f64]) -> bool {
        self.compute_vector_range(range)
    }

    pub(crate) fn to_double_array(&self) -> DoubleArray {
        let components = self.get_number_of_components().max(1) as usize;
        let mut values = Vec::with_capacity((self.get_number_of_values().max(0)) as usize);
        for tuple_id in 0..self.get_number_of_tuples() {
            let tuple = self
                .checked_tuple_as_f64(tuple_id as usize)
                .expect("structured point tuple read must be infallible");
            values.extend(tuple.into_iter().take(components));
        }
        DoubleArray::from_vec(self.get_name(), values, components)
    }
}

fn numeric_component(array: &AnyArray, tuple_id: VtkIdType, component: i32) -> f64 {
    array
        .numeric_tuple_as_f64_checked(tuple_id as usize)
        .ok()
        .and_then(|tuple| tuple.get(component as usize).copied())
        .unwrap_or(0.0)
}
