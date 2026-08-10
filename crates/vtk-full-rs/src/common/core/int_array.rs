use super::{
    data_array::{define_typed_array, vtk_id_to_usize, DataArray},
    vtk_type::{VtkDataType, VtkIdType},
};

define_typed_array!(IntArray, i32, super::data_array::NativeVtkType<i32>);
