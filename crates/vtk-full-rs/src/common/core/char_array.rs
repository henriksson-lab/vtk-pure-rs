use super::{
    data_array::{define_typed_array, vtk_id_to_usize, DataArray},
    vtk_type::{VtkDataType, VtkIdType},
};

define_typed_array!(
    CharArray,
    super::vtk_type::VtkChar,
    super::data_array::CharKind
);
