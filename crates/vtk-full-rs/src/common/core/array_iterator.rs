use std::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use super::{
    char_array::CharArray,
    double_array::DoubleArray,
    float_array::FloatArray,
    id_type_array::IdTypeArray,
    int_array::IntArray,
    long_array::LongArray,
    long_long_array::LongLongArray,
    object::Object,
    short_array::ShortArray,
    signed_char_array::SignedCharArray,
    string_array::StringArray,
    unsigned_char_array::UnsignedCharArray,
    unsigned_int_array::UnsignedIntArray,
    unsigned_long_array::UnsignedLongArray,
    unsigned_long_long_array::UnsignedLongLongArray,
    unsigned_short_array::UnsignedShortArray,
    variant_array::Variant,
    variant_array::VariantArray,
    vtk_type::{VtkChar, VtkDataType, VtkIdType, VtkLong, VtkUnsignedLong},
};

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}

/// VTK: `vtkArrayIterator`.
///
/// VTK exposes this as an abstract `vtkObject` subclass. Rust keeps the
/// abstract API as a trait; concrete iterator templates own the object state.
pub trait ArrayIterator {
    type Array;

    /// VTK: `vtkArrayIterator::Initialize`.
    ///
    /// # Safety
    ///
    /// `array` must be null or point to a live array object that remains valid
    /// while this iterator references it. Mutating the array outside the
    /// iterator has the same invalidation caveat as VTK's raw pointer API.
    unsafe fn initialize(&mut self, array: *mut Self::Array);

    /// VTK: `vtkArrayIterator::GetDataType`.
    fn get_data_type(&self) -> i32;
}

pub trait ArrayIteratorArray<T> {
    fn get_number_of_tuples(&self) -> VtkIdType;
    fn get_number_of_values(&self) -> VtkIdType;
    fn get_number_of_components(&self) -> i32;
    fn get_data_type(&self) -> VtkDataType;
    fn get_data_type_size(&self) -> i32;
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
}

macro_rules! impl_array_iterator_array {
    ($array:ty, $value:ty) => {
        impl ArrayIteratorArray<$value> for $array {
            fn get_number_of_tuples(&self) -> VtkIdType {
                self.get_number_of_tuples()
            }

            fn get_number_of_values(&self) -> VtkIdType {
                self.get_number_of_values()
            }

            fn get_number_of_components(&self) -> i32 {
                self.get_number_of_components()
            }

            fn get_data_type(&self) -> VtkDataType {
                self.get_data_type()
            }

            fn get_data_type_size(&self) -> i32 {
                self.get_data_type_size()
            }

            fn as_slice(&self) -> &[$value] {
                self.as_slice()
            }

            fn as_mut_slice(&mut self) -> &mut [$value] {
                self.as_mut_slice()
            }
        }
    };
}

impl_array_iterator_array!(CharArray, VtkChar);
impl_array_iterator_array!(SignedCharArray, i8);
impl_array_iterator_array!(UnsignedCharArray, u8);
impl_array_iterator_array!(ShortArray, i16);
impl_array_iterator_array!(UnsignedShortArray, u16);
impl_array_iterator_array!(IntArray, i32);
impl_array_iterator_array!(UnsignedIntArray, u32);
impl_array_iterator_array!(LongArray, VtkLong);
impl_array_iterator_array!(UnsignedLongArray, VtkUnsignedLong);
impl_array_iterator_array!(FloatArray, f32);
impl_array_iterator_array!(DoubleArray, f64);
impl_array_iterator_array!(IdTypeArray, VtkIdType);
impl_array_iterator_array!(LongLongArray, i64);
impl_array_iterator_array!(UnsignedLongLongArray, u64);
impl_array_iterator_array!(VariantArray, Variant);

impl ArrayIteratorArray<String> for StringArray {
    fn get_number_of_tuples(&self) -> VtkIdType {
        self.get_number_of_tuples()
    }

    fn get_number_of_values(&self) -> VtkIdType {
        self.get_number_of_values()
    }

    fn get_number_of_components(&self) -> i32 {
        self.get_number_of_components()
    }

    fn get_data_type(&self) -> VtkDataType {
        VtkDataType::String
    }

    fn get_data_type_size(&self) -> i32 {
        self.get_data_type_size()
    }

    fn as_slice(&self) -> &[String] {
        self.as_slice()
    }

    fn as_mut_slice(&mut self) -> &mut [String] {
        self.as_mut_slice()
    }
}

/// VTK: `vtkArrayIteratorTemplate<T>`.
#[derive(Debug)]
pub struct ArrayIteratorTemplate<T, A>
where
    A: ArrayIteratorArray<T>,
{
    object: Object,
    array: Option<NonNull<A>>,
    value_type: PhantomData<T>,
}

impl<T, A> ArrayIteratorTemplate<T, A>
where
    A: ArrayIteratorArray<T>,
{
    /// VTK: `vtkArrayIteratorTemplate<T>::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkArrayIteratorTemplate"),
            array: None,
            value_type: PhantomData,
        }
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetArray`.
    pub fn get_array(&self) -> *mut A {
        self.array.map_or(ptr::null_mut(), |array| array.as_ptr())
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetTuple`.
    pub fn get_tuple(&self, id: VtkIdType) -> &[T] {
        let array = self.array_ref();
        let number_of_components = vtk_id_to_usize(array.get_number_of_components() as VtkIdType);
        let start = vtk_id_to_usize(id) * number_of_components;
        &array.as_slice()[start..start + number_of_components]
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetValue`.
    pub fn get_value(&self, id: VtkIdType) -> &T {
        &self.array_ref().as_slice()[vtk_id_to_usize(id)]
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::SetValue`.
    pub fn set_value(&mut self, id: VtkIdType, value: T) {
        let array = self.array_mut();
        array.as_mut_slice()[vtk_id_to_usize(id)] = value;
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetNumberOfTuples`.
    pub fn get_number_of_tuples(&self) -> VtkIdType {
        self.array_ref_opt()
            .map_or(0, ArrayIteratorArray::get_number_of_tuples)
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetNumberOfValues`.
    pub fn get_number_of_values(&self) -> VtkIdType {
        self.array_ref_opt()
            .map_or(0, ArrayIteratorArray::get_number_of_values)
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.array_ref_opt()
            .map_or(0, ArrayIteratorArray::get_number_of_components)
    }

    /// VTK: `vtkArrayIteratorTemplate<T>::GetDataTypeSize`.
    pub fn get_data_type_size(&self) -> i32 {
        self.array_ref_opt()
            .map_or(0, ArrayIteratorArray::get_data_type_size)
    }

    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    fn array_ref_opt(&self) -> Option<&A> {
        self.array.map(|array| unsafe { array.as_ref() })
    }

    fn array_ref(&self) -> &A {
        self.array_ref_opt()
            .expect("vtkArrayIteratorTemplate must be initialized")
    }

    fn array_mut(&mut self) -> &mut A {
        self.array
            .map(|mut array| unsafe { array.as_mut() })
            .expect("vtkArrayIteratorTemplate must be initialized")
    }
}

impl<T, A> ArrayIterator for ArrayIteratorTemplate<T, A>
where
    A: ArrayIteratorArray<T>,
{
    type Array = A;

    unsafe fn initialize(&mut self, array: *mut Self::Array) {
        self.array = NonNull::new(array);
    }

    fn get_data_type(&self) -> i32 {
        self.array_ref_opt()
            .map_or(VtkDataType::VTK_VOID, |array| array.get_data_type().id())
    }
}

impl<T, A> Default for ArrayIteratorTemplate<T, A>
where
    A: ArrayIteratorArray<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A> Drop for ArrayIteratorTemplate<T, A>
where
    A: ArrayIteratorArray<T>,
{
    fn drop(&mut self) {
        self.array = None;
    }
}
