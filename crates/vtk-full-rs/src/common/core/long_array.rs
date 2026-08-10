use super::{
    data_array::{define_typed_array, vtk_id_to_usize, DataArray},
    vtk_type::{VtkDataType, VtkIdType},
};

define_typed_array!(
    LongArray,
    super::vtk_type::VtkLong,
    super::data_array::LongKind
);
