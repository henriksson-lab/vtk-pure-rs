use super::{
    bit_array::BitArray,
    char_array::CharArray,
    double_array::DoubleArray,
    float_array::FloatArray,
    id_type_array::IdTypeArray,
    int_array::IntArray,
    long_array::LongArray,
    long_long_array::LongLongArray,
    short_array::ShortArray,
    signed_char_array::SignedCharArray,
    string_array::StringArray,
    structured_point_array::StructuredPointArray,
    unsigned_char_array::UnsignedCharArray,
    unsigned_int_array::UnsignedIntArray,
    unsigned_long_array::UnsignedLongArray,
    unsigned_long_long_array::UnsignedLongLongArray,
    unsigned_short_array::UnsignedShortArray,
    variant_array::VariantArray,
    vtk_type::{vtk_data_types_compare, vtk_interpolated_component, VtkDataType, VtkIdType},
};

fn vtk_id_to_usize(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}

fn id_count_to_usize(count: VtkIdType) -> usize {
    usize::try_from(count.max(0)).expect("vtkIdType count must fit usize")
}

fn int_index_to_usize(index: i32) -> usize {
    usize::try_from(index).expect("VTK int index must be non-negative and fit usize")
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum ArrayError {
    #[error("unsupported VTK data type {0:?}")]
    UnsupportedDataType(VtkDataType),
    #[error("array type mismatch: destination {destination:?}, source {source_type:?}")]
    TypeMismatch {
        destination: VtkDataType,
        source_type: VtkDataType,
    },
    #[error("tuple component mismatch: destination {destination}, source {source_components}")]
    TupleComponentMismatch {
        destination: usize,
        source_components: usize,
    },
    #[error("tuple id {tuple} out of range for array with {number_of_tuples} tuples")]
    TupleOutOfRange {
        tuple: usize,
        number_of_tuples: usize,
    },
}

/// Runtime VTK `vtkAbstractArray*` equivalent over concrete Rust array types.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyArray {
    Bit(BitArray),
    Char(CharArray),
    SignedChar(SignedCharArray),
    UnsignedChar(UnsignedCharArray),
    Short(ShortArray),
    UnsignedShort(UnsignedShortArray),
    Int(IntArray),
    UnsignedInt(UnsignedIntArray),
    Long(LongArray),
    UnsignedLong(UnsignedLongArray),
    Float(FloatArray),
    Double(DoubleArray),
    IdType(IdTypeArray),
    LongLong(LongLongArray),
    UnsignedLongLong(UnsignedLongLongArray),
    StructuredPoint(StructuredPointArray),
    String(StringArray),
    Variant(VariantArray),
}

macro_rules! dispatch_any {
    ($self:expr, $array:ident => $body:expr) => {
        match $self {
            Self::Bit($array) => $body,
            Self::Char($array) => $body,
            Self::SignedChar($array) => $body,
            Self::UnsignedChar($array) => $body,
            Self::Short($array) => $body,
            Self::UnsignedShort($array) => $body,
            Self::Int($array) => $body,
            Self::UnsignedInt($array) => $body,
            Self::Long($array) => $body,
            Self::UnsignedLong($array) => $body,
            Self::Float($array) => $body,
            Self::Double($array) => $body,
            Self::IdType($array) => $body,
            Self::LongLong($array) => $body,
            Self::UnsignedLongLong($array) => $body,
            Self::StructuredPoint($array) => $body,
            Self::String($array) => $body,
            Self::Variant($array) => $body,
        }
    };
}

macro_rules! dispatch_any_mut {
    ($self:expr, $array:ident => $body:expr) => {
        match $self {
            Self::Bit($array) => $body,
            Self::Char($array) => $body,
            Self::SignedChar($array) => $body,
            Self::UnsignedChar($array) => $body,
            Self::Short($array) => $body,
            Self::UnsignedShort($array) => $body,
            Self::Int($array) => $body,
            Self::UnsignedInt($array) => $body,
            Self::Long($array) => $body,
            Self::UnsignedLong($array) => $body,
            Self::Float($array) => $body,
            Self::Double($array) => $body,
            Self::IdType($array) => $body,
            Self::LongLong($array) => $body,
            Self::UnsignedLongLong($array) => $body,
            Self::StructuredPoint($array) => $body,
            Self::String($array) => $body,
            Self::Variant($array) => $body,
        }
    };
}

fn component_count_to_usize(number_of_components: i32) -> usize {
    usize::try_from(number_of_components.max(1)).expect("component count must fit usize")
}

macro_rules! dispatch_numeric {
    ($self:expr, $array:ident => $body:expr) => {
        match $self {
            Self::Bit($array) => Some($body),
            Self::Char($array) => Some($body),
            Self::SignedChar($array) => Some($body),
            Self::UnsignedChar($array) => Some($body),
            Self::Short($array) => Some($body),
            Self::UnsignedShort($array) => Some($body),
            Self::Int($array) => Some($body),
            Self::UnsignedInt($array) => Some($body),
            Self::Long($array) => Some($body),
            Self::UnsignedLong($array) => Some($body),
            Self::Float($array) => Some($body),
            Self::Double($array) => Some($body),
            Self::IdType($array) => Some($body),
            Self::LongLong($array) => Some($body),
            Self::UnsignedLongLong($array) => Some($body),
            Self::StructuredPoint($array) => Some($body),
            Self::String(_) | Self::Variant(_) => None,
        }
    };
}

macro_rules! dispatch_numeric_mut {
    ($self:expr, $array:ident => $body:expr) => {
        match $self {
            Self::Bit($array) => Some($body),
            Self::Char($array) => Some($body),
            Self::SignedChar($array) => Some($body),
            Self::UnsignedChar($array) => Some($body),
            Self::Short($array) => Some($body),
            Self::UnsignedShort($array) => Some($body),
            Self::Int($array) => Some($body),
            Self::UnsignedInt($array) => Some($body),
            Self::Long($array) => Some($body),
            Self::UnsignedLong($array) => Some($body),
            Self::Float($array) => Some($body),
            Self::Double($array) => Some($body),
            Self::IdType($array) => Some($body),
            Self::LongLong($array) => Some($body),
            Self::UnsignedLongLong($array) => Some($body),
            Self::StructuredPoint($array) => Some($body),
            Self::String(_) | Self::Variant(_) => None,
        }
    };
}

macro_rules! dispatch_non_bit_numeric {
    ($self:expr, $array:ident => $body:expr) => {
        match $self {
            Self::Char($array) => Some($body),
            Self::SignedChar($array) => Some($body),
            Self::UnsignedChar($array) => Some($body),
            Self::Short($array) => Some($body),
            Self::UnsignedShort($array) => Some($body),
            Self::Int($array) => Some($body),
            Self::UnsignedInt($array) => Some($body),
            Self::Long($array) => Some($body),
            Self::UnsignedLong($array) => Some($body),
            Self::Float($array) => Some($body),
            Self::Double($array) => Some($body),
            Self::IdType($array) => Some($body),
            Self::LongLong($array) => Some($body),
            Self::UnsignedLongLong($array) => Some($body),
            Self::StructuredPoint($array) => Some($body),
            Self::Bit(_) | Self::String(_) | Self::Variant(_) => None,
        }
    };
}

impl AnyArray {
    /// VTK: `vtkAbstractArray::CreateArray`.
    pub fn create_array(data_type: VtkDataType) -> Option<Self> {
        Some(match data_type {
            VtkDataType::Bit => Self::Bit(BitArray::new()),
            VtkDataType::Char => Self::Char(CharArray::new()),
            VtkDataType::SignedChar => Self::SignedChar(SignedCharArray::new()),
            VtkDataType::UnsignedChar => Self::UnsignedChar(UnsignedCharArray::new()),
            VtkDataType::Short => Self::Short(ShortArray::new()),
            VtkDataType::UnsignedShort => Self::UnsignedShort(UnsignedShortArray::new()),
            VtkDataType::Int => Self::Int(IntArray::new()),
            VtkDataType::UnsignedInt => Self::UnsignedInt(UnsignedIntArray::new()),
            VtkDataType::Long => Self::Long(LongArray::new()),
            VtkDataType::UnsignedLong => Self::UnsignedLong(UnsignedLongArray::new()),
            VtkDataType::Float => Self::Float(FloatArray::new()),
            VtkDataType::Double => Self::Double(DoubleArray::new()),
            VtkDataType::IdType => Self::IdType(IdTypeArray::new()),
            VtkDataType::LongLong => Self::LongLong(LongLongArray::new()),
            VtkDataType::UnsignedLongLong => Self::UnsignedLongLong(UnsignedLongLongArray::new()),
            VtkDataType::String => Self::String(StringArray::new()),
            VtkDataType::Variant => Self::Variant(VariantArray::new()),
            VtkDataType::Void | VtkDataType::Opaque | VtkDataType::Object => {
                return None;
            }
        })
    }

    pub(crate) fn is_data_array(&self) -> bool {
        !matches!(self, Self::String(_) | Self::Variant(_))
    }

    /// VTK: `vtkDataArray::CreateDataArray`.
    pub fn create_data_array(data_type: VtkDataType) -> Option<Self> {
        let array = Self::create_array(data_type)?;
        array.is_data_array().then_some(array)
    }

    pub fn get_data_type(&self) -> VtkDataType {
        match self {
            Self::Bit(_) => VtkDataType::Bit,
            Self::Char(_) => VtkDataType::Char,
            Self::SignedChar(_) => VtkDataType::SignedChar,
            Self::UnsignedChar(_) => VtkDataType::UnsignedChar,
            Self::Short(_) => VtkDataType::Short,
            Self::UnsignedShort(_) => VtkDataType::UnsignedShort,
            Self::Int(_) => VtkDataType::Int,
            Self::UnsignedInt(_) => VtkDataType::UnsignedInt,
            Self::Long(_) => VtkDataType::Long,
            Self::UnsignedLong(_) => VtkDataType::UnsignedLong,
            Self::Float(_) => VtkDataType::Float,
            Self::Double(_) => VtkDataType::Double,
            Self::IdType(_) => VtkDataType::IdType,
            Self::LongLong(_) => VtkDataType::LongLong,
            Self::UnsignedLongLong(_) => VtkDataType::UnsignedLongLong,
            Self::StructuredPoint(array) => array.get_data_type(),
            Self::String(_) => VtkDataType::String,
            Self::Variant(_) => VtkDataType::Variant,
        }
    }

    #[cfg(test)]
    pub(crate) fn get_data_type_name(&self) -> &'static str {
        self.get_data_type().vtk_name()
    }

    pub(crate) fn is_numeric(&self) -> bool {
        self.get_data_type().is_numeric()
    }

    #[cfg(test)]
    pub(crate) fn is_integral(&self) -> bool {
        self.get_data_type().is_integral()
    }

    pub fn get_name(&self) -> &str {
        dispatch_any!(self, array => array.get_name())
    }

    pub(crate) fn as_unsigned_char_array(&self) -> Option<&UnsignedCharArray> {
        match self {
            Self::UnsignedChar(array) => Some(array),
            _ => None,
        }
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        dispatch_any_mut!(self, array => array.set_name(name));
    }

    pub fn get_number_of_components(&self) -> i32 {
        match self {
            Self::String(array) => array.get_number_of_components(),
            Self::Variant(array) => array.get_number_of_components(),
            _ => dispatch_numeric!(self, array => array.get_number_of_components())
                .expect("numeric array"),
        }
    }

    pub fn set_number_of_components(&mut self, number_of_components: i32) {
        match self {
            Self::String(array) => {
                array.set_number_of_components(number_of_components);
            }
            Self::Variant(array) => {
                array.set_number_of_components(number_of_components);
            }
            _ => {
                dispatch_numeric_mut!(
                    self,
                    array => array.set_number_of_components(number_of_components)
                );
            }
        }
    }

    pub fn get_number_of_tuples(&self) -> VtkIdType {
        match self {
            Self::String(array) => array.get_number_of_tuples(),
            Self::Variant(array) => array.get_number_of_tuples(),
            _ => dispatch_numeric!(self, array => array.get_number_of_tuples())
                .expect("numeric array"),
        }
    }

    pub fn set_number_of_tuples(&mut self, number_of_tuples: VtkIdType) {
        match self {
            Self::String(array) => array.set_number_of_tuples(number_of_tuples),
            Self::Variant(array) => array.set_number_of_tuples(number_of_tuples),
            _ => {
                dispatch_numeric_mut!(self, array => array.set_number_of_tuples(number_of_tuples));
            }
        }
    }

    pub fn get_number_of_values(&self) -> VtkIdType {
        match self {
            Self::String(array) => array.get_number_of_values(),
            Self::Variant(array) => array.get_number_of_values(),
            _ => dispatch_numeric!(self, array => array.get_number_of_values())
                .expect("numeric array"),
        }
    }

    pub fn reserve_values(&mut self, number_of_values: VtkIdType) -> bool {
        match self {
            Self::String(array) => array.reserve_values(number_of_values),
            Self::Variant(array) => array.reserve_values(number_of_values),
            _ => {
                return dispatch_numeric_mut!(
                    self,
                    array => array.reserve_values(number_of_values)
                )
                .expect("numeric array");
            }
        };
        true
    }

    pub fn reserve_tuples(&mut self, number_of_tuples: VtkIdType) -> bool {
        match self {
            Self::String(array) => array.reserve_tuples(number_of_tuples),
            Self::Variant(array) => array.reserve_tuples(number_of_tuples),
            _ => {
                return dispatch_numeric_mut!(
                    self,
                    array => array.reserve_tuples(number_of_tuples)
                )
                .expect("numeric array");
            }
        };
        true
    }

    pub fn initialize(&mut self) {
        dispatch_any_mut!(self, array => array.initialize());
    }

    pub fn reset(&mut self) {
        dispatch_any_mut!(self, array => array.reset());
    }

    pub fn squeeze(&mut self) {
        dispatch_any_mut!(self, array => array.squeeze());
    }

    /// VTK: `vtkDataArray::RemoveTuple`.
    pub fn remove_tuple(&mut self, tuple_idx: VtkIdType) {
        match self {
            Self::Bit(array) => array.remove_tuple(tuple_idx),
            Self::String(_) | Self::Variant(_) => {}
            _ => {
                dispatch_numeric_mut!(self, array => array.remove_tuple(tuple_idx));
            }
        }
    }

    pub fn get_actual_memory_size(&self) -> usize {
        dispatch_any!(self, array => array.get_actual_memory_size())
    }

    /// VTK: `vtkDataArray::GetRange`.
    pub fn get_range(&self) -> Option<[f64; 2]> {
        let mut range = [0.0, 0.0];
        self.compute_range(&mut range, 0).then_some(range)
    }

    /// VTK: `vtkDataArray::ComputeScalarRange` /
    /// `vtkDataArray::ComputeVectorRange`.
    pub(crate) fn compute_range(&self, range: &mut [f64], component: i32) -> bool {
        if let Self::Bit(array) = self {
            if let Some(bit_range) = array.get_range_with_component(component) {
                if range.len() >= 2 {
                    range[0] = bit_range[0];
                    range[1] = bit_range[1];
                }
                return true;
            }
            return false;
        }
        dispatch_non_bit_numeric!(self, array => {
            let components = array.get_number_of_components();
            let component = if component == -1 && components == 1 {
                0
            } else {
                component
            };
            if component < 0 {
                array.compute_vector_range(range)
            } else {
                if component < 0 || component >= components {
                    false
                } else {
                    let offset = component as usize * 2;
                    let mut ranges = vec![0.0; components as usize * 2];
                    let ok = array.compute_scalar_range(&mut ranges);
                    if ok && range.len() >= 2 {
                        range[0] = ranges[offset];
                        range[1] = ranges[offset + 1];
                    }
                    ok
                }
            }
        })
        .unwrap_or(false)
    }

    /// VTK: `vtkDataArray::ComputeFiniteScalarRange` /
    /// `vtkDataArray::ComputeFiniteVectorRange`.
    pub(crate) fn compute_finite_range(&self, range: &mut [f64], component: i32) -> bool {
        if let Self::Bit(array) = self {
            if let Some(bit_range) = array.get_finite_range_with_component(component) {
                if range.len() >= 2 {
                    range[0] = bit_range[0];
                    range[1] = bit_range[1];
                }
                return true;
            }
            return false;
        }
        dispatch_non_bit_numeric!(self, array => {
            let components = array.get_number_of_components();
            let component = if component == -1 && components == 1 {
                0
            } else {
                component
            };
            if component < 0 {
                array.compute_finite_vector_range(range)
            } else {
                if component < 0 || component >= components {
                    false
                } else {
                    let offset = component as usize * 2;
                    let mut ranges = vec![0.0; components as usize * 2];
                    let ok = array.compute_finite_scalar_range(&mut ranges);
                    if ok && range.len() >= 2 {
                        range[0] = ranges[offset];
                        range[1] = ranges[offset + 1];
                    }
                    ok
                }
            }
        })
        .unwrap_or(false)
    }

    /// VTK: `vtkAbstractArray::HasStandardMemoryLayout` / `vtkDataArray::ToAOSDataArray`.
    pub fn has_standard_memory_layout(&self) -> bool {
        match self {
            Self::StructuredPoint(array) => array.has_standard_memory_layout(),
            _ => self.is_data_array(),
        }
    }

    /// VTK: `vtkDataArray::ToAOSDataArray`.
    pub fn to_aos_data_array(&self) -> Option<Self> {
        if !self.is_data_array() {
            return None;
        }
        if self.has_standard_memory_layout() {
            return Some(self.shallow_clone());
        }
        if let Self::StructuredPoint(array) = self {
            return Some(Self::Double(array.to_double_array()));
        }
        let mut aos = Self::create_data_array(self.get_data_type())?;
        aos.deep_copy(self);
        Some(aos)
    }

    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        dispatch_any!(self, array => array.capacity())
    }

    pub fn get_m_time(&self) -> u64 {
        dispatch_any!(self, array => array.get_m_time())
    }

    pub fn new_instance(&self) -> Self {
        let mut array = Self::create_array(self.get_data_type()).expect("supported self data type");
        array.set_number_of_components(self.get_number_of_components());
        array
    }

    pub(crate) fn deep_clone(&self) -> Self {
        match self {
            Self::Bit(array) => Self::Bit(array.deep_clone()),
            Self::Char(array) => Self::Char(array.deep_clone()),
            Self::SignedChar(array) => Self::SignedChar(array.deep_clone()),
            Self::UnsignedChar(array) => Self::UnsignedChar(array.deep_clone()),
            Self::Short(array) => Self::Short(array.deep_clone()),
            Self::UnsignedShort(array) => Self::UnsignedShort(array.deep_clone()),
            Self::Int(array) => Self::Int(array.deep_clone()),
            Self::UnsignedInt(array) => Self::UnsignedInt(array.deep_clone()),
            Self::Long(array) => Self::Long(array.deep_clone()),
            Self::UnsignedLong(array) => Self::UnsignedLong(array.deep_clone()),
            Self::Float(array) => Self::Float(array.deep_clone()),
            Self::Double(array) => Self::Double(array.deep_clone()),
            Self::IdType(array) => Self::IdType(array.deep_clone()),
            Self::LongLong(array) => Self::LongLong(array.deep_clone()),
            Self::UnsignedLongLong(array) => Self::UnsignedLongLong(array.deep_clone()),
            Self::StructuredPoint(array) => Self::StructuredPoint(array.deep_clone()),
            Self::String(array) => Self::String(array.deep_clone()),
            Self::Variant(array) => Self::Variant(array.deep_clone()),
        }
    }

    pub(crate) fn shallow_clone(&self) -> Self {
        match self {
            Self::Bit(array) => Self::Bit(array.shallow_clone()),
            Self::Char(array) => Self::Char(array.shallow_clone()),
            Self::SignedChar(array) => Self::SignedChar(array.shallow_clone()),
            Self::UnsignedChar(array) => Self::UnsignedChar(array.shallow_clone()),
            Self::Short(array) => Self::Short(array.shallow_clone()),
            Self::UnsignedShort(array) => Self::UnsignedShort(array.shallow_clone()),
            Self::Int(array) => Self::Int(array.shallow_clone()),
            Self::UnsignedInt(array) => Self::UnsignedInt(array.shallow_clone()),
            Self::Long(array) => Self::Long(array.shallow_clone()),
            Self::UnsignedLong(array) => Self::UnsignedLong(array.shallow_clone()),
            Self::Float(array) => Self::Float(array.shallow_clone()),
            Self::Double(array) => Self::Double(array.shallow_clone()),
            Self::IdType(array) => Self::IdType(array.shallow_clone()),
            Self::LongLong(array) => Self::LongLong(array.shallow_clone()),
            Self::UnsignedLongLong(array) => Self::UnsignedLongLong(array.shallow_clone()),
            Self::StructuredPoint(array) => Self::StructuredPoint(array.shallow_clone()),
            Self::String(array) => Self::String(array.shallow_clone()),
            Self::Variant(array) => Self::Variant(array.shallow_clone()),
        }
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bit(left), Self::Bit(right)) => left.shares_storage_with(right),
            (Self::Char(left), Self::Char(right)) => left.shares_storage_with(right),
            (Self::SignedChar(left), Self::SignedChar(right)) => left.shares_storage_with(right),
            (Self::UnsignedChar(left), Self::UnsignedChar(right)) => {
                left.shares_storage_with(right)
            }
            (Self::Short(left), Self::Short(right)) => left.shares_storage_with(right),
            (Self::UnsignedShort(left), Self::UnsignedShort(right)) => {
                left.shares_storage_with(right)
            }
            (Self::Int(left), Self::Int(right)) => left.shares_storage_with(right),
            (Self::UnsignedInt(left), Self::UnsignedInt(right)) => left.shares_storage_with(right),
            (Self::Long(left), Self::Long(right)) => left.shares_storage_with(right),
            (Self::UnsignedLong(left), Self::UnsignedLong(right)) => {
                left.shares_storage_with(right)
            }
            (Self::Float(left), Self::Float(right)) => left.shares_storage_with(right),
            (Self::Double(left), Self::Double(right)) => left.shares_storage_with(right),
            (Self::IdType(left), Self::IdType(right)) => left.shares_storage_with(right),
            (Self::LongLong(left), Self::LongLong(right)) => left.shares_storage_with(right),
            (Self::UnsignedLongLong(left), Self::UnsignedLongLong(right)) => {
                left.shares_storage_with(right)
            }
            (Self::StructuredPoint(left), Self::StructuredPoint(right)) => {
                left.shares_storage_with(right)
            }
            (Self::String(left), Self::String(right)) => left.shares_storage_with(right),
            (Self::Variant(left), Self::Variant(right)) => left.shares_storage_with(right),
            _ => false,
        }
    }

    pub fn deep_copy(&mut self, source: &Self) {
        if self.is_numeric()
            && source.is_numeric()
            && (self.get_data_type() != source.get_data_type()
                || matches!(source, Self::StructuredPoint(_))
                || matches!(self, Self::StructuredPoint(_)))
            && !matches!(
                (&*self, source),
                (Self::StructuredPoint(_), Self::StructuredPoint(_))
            )
        {
            self.deep_copy_numeric_from(source);
            return;
        }

        if self.ensure_type_compatible(source).is_err() {
            return;
        }

        match (&mut *self, source) {
            (Self::Bit(dst), Self::Bit(src)) => dst.deep_copy(src),
            (Self::Char(dst), Self::Char(src)) => dst.deep_copy(src),
            (Self::SignedChar(dst), Self::SignedChar(src)) => dst.deep_copy(src),
            (Self::UnsignedChar(dst), Self::UnsignedChar(src)) => dst.deep_copy(src),
            (Self::Short(dst), Self::Short(src)) => dst.deep_copy(src),
            (Self::UnsignedShort(dst), Self::UnsignedShort(src)) => dst.deep_copy(src),
            (Self::Int(dst), Self::Int(src)) => dst.deep_copy(src),
            (Self::UnsignedInt(dst), Self::UnsignedInt(src)) => dst.deep_copy(src),
            (Self::Long(dst), Self::Long(src)) => dst.deep_copy(src),
            (Self::UnsignedLong(dst), Self::UnsignedLong(src)) => dst.deep_copy(src),
            (Self::Float(dst), Self::Float(src)) => dst.deep_copy(src),
            (Self::Double(dst), Self::Double(src)) => dst.deep_copy(src),
            (Self::IdType(dst), Self::IdType(src)) => dst.deep_copy(src),
            (Self::LongLong(dst), Self::LongLong(src)) => dst.deep_copy(src),
            (Self::UnsignedLongLong(dst), Self::UnsignedLongLong(src)) => dst.deep_copy(src),
            (Self::StructuredPoint(dst), Self::StructuredPoint(src)) => *dst = src.deep_clone(),
            (Self::String(dst), Self::String(src)) => dst.deep_copy(src),
            (Self::Variant(dst), Self::Variant(src)) => dst.deep_copy(src),
            _ => unreachable!("type compatibility checked"),
        }

        if self.is_numeric() {
            self.squeeze();
        }
    }

    pub fn shallow_copy(&mut self, source: &Self) {
        if self.ensure_type_compatible(source).is_err() {
            return;
        }
        match (self, source) {
            (Self::Bit(dst), Self::Bit(src)) => dst.shallow_copy(src),
            (Self::Char(dst), Self::Char(src)) => dst.shallow_copy(src),
            (Self::SignedChar(dst), Self::SignedChar(src)) => dst.shallow_copy(src),
            (Self::UnsignedChar(dst), Self::UnsignedChar(src)) => dst.shallow_copy(src),
            (Self::Short(dst), Self::Short(src)) => dst.shallow_copy(src),
            (Self::UnsignedShort(dst), Self::UnsignedShort(src)) => dst.shallow_copy(src),
            (Self::Int(dst), Self::Int(src)) => dst.shallow_copy(src),
            (Self::UnsignedInt(dst), Self::UnsignedInt(src)) => dst.shallow_copy(src),
            (Self::Long(dst), Self::Long(src)) => dst.shallow_copy(src),
            (Self::UnsignedLong(dst), Self::UnsignedLong(src)) => dst.shallow_copy(src),
            (Self::Float(dst), Self::Float(src)) => dst.shallow_copy(src),
            (Self::Double(dst), Self::Double(src)) => dst.shallow_copy(src),
            (Self::IdType(dst), Self::IdType(src)) => dst.shallow_copy(src),
            (Self::LongLong(dst), Self::LongLong(src)) => dst.shallow_copy(src),
            (Self::UnsignedLongLong(dst), Self::UnsignedLongLong(src)) => dst.shallow_copy(src),
            (Self::StructuredPoint(dst), Self::StructuredPoint(src)) => *dst = src.shallow_clone(),
            (Self::String(dst), Self::String(src)) => dst.shallow_copy(src),
            (Self::Variant(dst), Self::Variant(src)) => dst.shallow_copy(src),
            _ => unreachable!("type compatibility checked"),
        }
    }

    /// VTK: `vtkDataArray::SetTuple(dstTupleIdx, srcTupleIdx, source)`.
    pub fn set_tuple(&mut self, dst_tuple_idx: VtkIdType, src_tuple_idx: VtkIdType, source: &Self) {
        let Ok(dst_tuple_idx) = usize::try_from(dst_tuple_idx) else {
            return;
        };
        let Ok(src_tuple_idx) = usize::try_from(src_tuple_idx) else {
            return;
        };
        let _ = self.set_tuple_from(source, src_tuple_idx, dst_tuple_idx);
    }

    /// VTK: `vtkDataArray::CopyComponent(int, vtkAbstractArray*, int)` /
    /// `vtkDataArray::CopyComponent(int, vtkDataArray*, int)`.
    pub fn copy_component(
        &mut self,
        dst_component: i32,
        source: &Self,
        src_component: i32,
    ) -> bool {
        if !self.is_numeric() || !source.is_numeric() {
            return false;
        }
        if self.get_number_of_tuples() != source.get_number_of_tuples() {
            return false;
        }
        if dst_component < 0 || dst_component >= self.get_number_of_components() {
            return false;
        }
        if src_component < 0 || src_component >= source.get_number_of_components() {
            return false;
        }

        let dst_component = int_index_to_usize(dst_component);
        let src_component = int_index_to_usize(src_component);
        let tuple_count = id_count_to_usize(self.get_number_of_tuples());

        for tuple_idx in 0..tuple_count {
            let Ok(tuple) = source.numeric_tuple_as_f64(tuple_idx) else {
                return false;
            };
            if self
                .set_numeric_component_from_f64(tuple_idx, dst_component, tuple[src_component])
                .is_err()
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn set_tuple_from(
        &mut self,
        source: &Self,
        from_tuple: usize,
        to_tuple: usize,
    ) -> Result<(), ArrayError> {
        self.ensure_tuple_source(source)?;
        let destination_type = self.get_data_type();
        let source_type = source.get_data_type();
        match (&mut *self, source) {
            (Self::String(dst), Self::String(src)) => {
                dst.set_tuple(to_tuple as VtkIdType, from_tuple as VtkIdType, src)
            }
            (Self::Variant(dst), Self::Variant(src)) => {
                dst.set_tuple(to_tuple as VtkIdType, from_tuple as VtkIdType, src)
            }
            (dst, src) if dst.is_numeric() && src.is_numeric() => {
                let tuple = src.numeric_tuple_as_f64(from_tuple)?;
                dst.set_numeric_tuple_from_f64(to_tuple, &tuple)?;
            }
            _ => {
                return Err(ArrayError::TypeMismatch {
                    destination: destination_type,
                    source_type,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn copy_tuple_from(
        &mut self,
        source: &Self,
        from_tuple: usize,
        to_tuple: usize,
    ) -> Result<(), ArrayError> {
        self.ensure_tuple_source(source)?;
        let destination_type = self.get_data_type();
        let source_type = source.get_data_type();
        match (&mut *self, source) {
            (Self::String(dst), Self::String(src)) => {
                dst.insert_tuple(to_tuple as VtkIdType, from_tuple as VtkIdType, src)
            }
            (Self::Variant(dst), Self::Variant(src)) => {
                dst.insert_tuple(to_tuple as VtkIdType, from_tuple as VtkIdType, src)
            }
            (dst, src) if dst.is_numeric() && src.is_numeric() => {
                let tuple = src.numeric_tuple_as_f64(from_tuple)?;
                dst.copy_numeric_tuple_from_f64(to_tuple, &tuple)?;
            }
            _ => {
                return Err(ArrayError::TypeMismatch {
                    destination: destination_type,
                    source_type,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn interpolate_tuple_from(
        &mut self,
        source: &Self,
        source_tuples: &[usize],
        weights: &[f64],
        to_tuple: usize,
    ) -> bool {
        if self.ensure_tuple_source(source).is_err() {
            return false;
        }
        if source_tuples.is_empty() {
            return false;
        }
        if source_tuples.len() != weights.len() {
            return false;
        }
        match (&mut *self, source) {
            (Self::String(dst), Self::String(src)) => {
                let source_tuples: Vec<_> =
                    source_tuples.iter().map(|&id| id as VtkIdType).collect();
                dst.interpolate_tuple_from(src, &source_tuples, weights, to_tuple as VtkIdType);
            }
            (Self::Variant(dst), Self::Variant(src)) => {
                let source_tuples: Vec<_> =
                    source_tuples.iter().map(|&id| id as VtkIdType).collect();
                dst.interpolate_tuple_from(src, &source_tuples, weights, to_tuple as VtkIdType);
            }
            (dst, src) if dst.is_numeric() && src.is_numeric() => {
                let components = component_count_to_usize(dst.get_number_of_components());
                let mut tuple = vec![0.0; components];
                for (&source_tuple, &weight) in source_tuples.iter().zip(weights) {
                    let Ok(source_values) = src.numeric_tuple_as_f64(source_tuple) else {
                        return false;
                    };
                    for component in 0..components {
                        tuple[component] += weight * source_values[component];
                    }
                }
                if dst.insert_numeric_tuple_from_f64(to_tuple, &tuple).is_err() {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }
        true
    }

    /// VTK: `vtkDataArray::InterpolateTuple(vtkIdType, vtkIdList*, vtkAbstractArray*, double*)`.
    pub fn interpolate_tuple(
        &mut self,
        dst_tuple_idx: VtkIdType,
        tuple_ids: &[VtkIdType],
        source: &Self,
        weights: &[f64],
    ) -> bool {
        if !self.is_numeric() || !source.is_numeric() {
            return false;
        }
        if dst_tuple_idx < 0 {
            return false;
        }
        if !vtk_data_types_compare(self.get_data_type().id(), source.get_data_type().id()) {
            return false;
        }
        if self.get_number_of_components() != source.get_number_of_components() {
            return false;
        }
        if tuple_ids.is_empty() || tuple_ids.len() != weights.len() {
            return false;
        }

        let source_tuple_count = source.get_number_of_tuples();
        let mut source_tuples = Vec::with_capacity(tuple_ids.len());
        for &tuple_id in tuple_ids {
            if tuple_id < 0 || tuple_id >= source_tuple_count {
                return false;
            }
            source_tuples.push(vtk_id_to_usize(tuple_id));
        }

        self.interpolate_tuple_from(
            source,
            &source_tuples,
            weights,
            vtk_id_to_usize(dst_tuple_idx),
        )
    }

    /// VTK: `vtkDataArray::InterpolateTuple(vtkIdType, vtkIdType, vtkAbstractArray*, vtkIdType, vtkAbstractArray*, double)`.
    pub fn interpolate_tuple_between(
        &mut self,
        dst_tuple_idx: VtkIdType,
        src_tuple_idx1: VtkIdType,
        source1: &Self,
        src_tuple_idx2: VtkIdType,
        source2: &Self,
        t: f64,
    ) -> bool {
        if !self.is_numeric() || !source1.is_numeric() || !source2.is_numeric() {
            return false;
        }
        if dst_tuple_idx < 0 {
            return false;
        }
        if !vtk_data_types_compare(self.get_data_type().id(), source1.get_data_type().id())
            || !vtk_data_types_compare(self.get_data_type().id(), source2.get_data_type().id())
        {
            return false;
        }
        if self.get_number_of_components() != source1.get_number_of_components()
            || self.get_number_of_components() != source2.get_number_of_components()
        {
            return false;
        }
        if src_tuple_idx1 < 0
            || src_tuple_idx1 >= source1.get_number_of_tuples()
            || src_tuple_idx2 < 0
            || src_tuple_idx2 >= source2.get_number_of_tuples()
        {
            return false;
        }

        let id1 = vtk_id_to_usize(src_tuple_idx1);
        let id2 = vtk_id_to_usize(src_tuple_idx2);
        let to_tuple = vtk_id_to_usize(dst_tuple_idx);

        if self.ensure_tuple_source(source1).is_err() || self.ensure_tuple_source(source2).is_err()
        {
            return false;
        }
        match (&mut *self, source1, source2) {
            (dst, src1, src2) if dst.is_numeric() && src1.is_numeric() && src2.is_numeric() => {
                let Ok(tuple1) = src1.numeric_tuple_as_f64(id1) else {
                    return false;
                };
                let Ok(tuple2) = src2.numeric_tuple_as_f64(id2) else {
                    return false;
                };
                let tuple: Vec<_> = tuple1
                    .iter()
                    .zip(tuple2.iter())
                    .map(|(&left, &right)| left + t * (right - left))
                    .collect();
                if dst.insert_numeric_tuple_from_f64(to_tuple, &tuple).is_err() {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }
        true
    }

    pub fn set_component_name(&mut self, component: VtkIdType, name: impl Into<String>) {
        let name = name.into();
        match self {
            Self::String(array) => array.set_component_name(component, name),
            Self::Variant(array) => array.set_component_name(component, name),
            _ => dispatch_numeric_mut!(self, array => array.set_component_name(component, name))
                .expect("numeric array"),
        }
    }

    pub fn get_component_name(&self, component: VtkIdType) -> Option<&str> {
        match self {
            Self::String(array) => array.get_component_name(component),
            Self::Variant(array) => array.get_component_name(component),
            _ => dispatch_numeric!(self, array => array.get_component_name(component))
                .expect("numeric array"),
        }
    }

    pub(crate) fn has_a_component_name(&self) -> bool {
        match self {
            Self::String(array) => array.has_a_component_name(),
            Self::Variant(array) => array.has_a_component_name(),
            _ => dispatch_numeric!(self, array => array.has_a_component_name())
                .expect("numeric array"),
        }
    }

    pub(crate) fn component_tuple_values_as_f64(&self, tuple_idx: usize) -> Option<Vec<f64>> {
        self.numeric_tuple_as_f64(tuple_idx).ok()
    }

    pub(crate) fn copy_component_as_single_component(&self, component: i32) -> Option<Self> {
        if component < 0 || component >= self.get_number_of_components() {
            return None;
        }

        let mut output = self.new_instance();
        output.set_number_of_components(1);
        output.set_number_of_tuples(self.get_number_of_tuples());
        let component = vtk_id_to_usize(component as VtkIdType);
        let tuple_count = vtk_id_to_usize(self.get_number_of_tuples());

        match (&mut output, self) {
            (Self::String(dst), Self::String(src)) => {
                for tuple_idx in 0..tuple_count {
                    let value = src
                        .get_typed_component(tuple_idx as VtkIdType, component as i32)
                        .to_string();
                    dst.set_value(tuple_idx as VtkIdType, value);
                }
            }
            (Self::Variant(dst), Self::Variant(src)) => {
                for tuple_idx in 0..tuple_count {
                    let value = src
                        .get_typed_component(tuple_idx as VtkIdType, component as i32)
                        .clone();
                    dst.set_value(tuple_idx as VtkIdType, value);
                }
            }
            (dst, src) if dst.is_numeric() && src.is_numeric() => {
                for tuple_idx in 0..tuple_count {
                    let tuple = src.numeric_tuple_as_f64(tuple_idx).ok()?;
                    dst.copy_numeric_tuple_from_f64(tuple_idx, &[tuple[component]])
                        .ok()?;
                }
            }
            _ => return None,
        }

        Some(output)
    }

    pub(crate) fn numeric_tuple_as_f64_checked(
        &self,
        tuple_idx: usize,
    ) -> Result<Vec<f64>, ArrayError> {
        self.numeric_tuple_as_f64(tuple_idx)
    }

    pub(crate) fn insert_numeric_tuple_from_f64_checked(
        &mut self,
        tuple_idx: usize,
        tuple: &[f64],
    ) -> Result<(), ArrayError> {
        self.copy_numeric_tuple_from_f64(tuple_idx, tuple)
    }

    fn ensure_type_compatible(&self, source: &Self) -> Result<(), ArrayError> {
        if vtk_data_types_compare(self.get_data_type().id(), source.get_data_type().id()) {
            Ok(())
        } else {
            Err(ArrayError::TypeMismatch {
                destination: self.get_data_type(),
                source_type: source.get_data_type(),
            })
        }
    }

    fn ensure_tuple_source(&self, source: &Self) -> Result<(), ArrayError> {
        if self.get_number_of_components() != source.get_number_of_components() {
            return Err(ArrayError::TupleComponentMismatch {
                destination: component_count_to_usize(self.get_number_of_components()),
                source_components: component_count_to_usize(source.get_number_of_components()),
            });
        }
        if self.is_numeric() && source.is_numeric() {
            return Ok(());
        }
        self.ensure_type_compatible(source)
    }

    fn numeric_tuple_as_f64(&self, tuple_idx: usize) -> Result<Vec<f64>, ArrayError> {
        dispatch_numeric!(self, array => array.checked_tuple_as_f64(tuple_idx)).ok_or(
            ArrayError::TypeMismatch {
                destination: self.get_data_type(),
                source_type: self.get_data_type(),
            },
        )?
    }

    fn set_numeric_tuple_from_f64(
        &mut self,
        tuple_idx: usize,
        tuple: &[f64],
    ) -> Result<(), ArrayError> {
        dispatch_numeric_mut!(self, array => array.set_typed_tuple_from_f64(tuple_idx, tuple))
            .ok_or(ArrayError::TypeMismatch {
                destination: self.get_data_type(),
                source_type: self.get_data_type(),
            })?;
        Ok(())
    }

    fn set_numeric_component_from_f64(
        &mut self,
        tuple_idx: usize,
        component_idx: usize,
        value: f64,
    ) -> Result<(), ArrayError> {
        let tuple_idx =
            VtkIdType::try_from(tuple_idx).map_err(|_| ArrayError::TupleOutOfRange {
                tuple: tuple_idx,
                number_of_tuples: id_count_to_usize(self.get_number_of_tuples()),
            })?;
        let component_idx =
            i32::try_from(component_idx).map_err(|_| ArrayError::TupleComponentMismatch {
                destination: component_count_to_usize(self.get_number_of_components()),
                source_components: component_idx,
            })?;

        dispatch_numeric_mut!(self, array => array.set_component(tuple_idx, component_idx, value))
            .ok_or(ArrayError::TypeMismatch {
                destination: self.get_data_type(),
                source_type: self.get_data_type(),
            })?;
        Ok(())
    }

    pub(crate) fn numeric_component_as_f64_checked(
        &self,
        tuple_idx: usize,
        component_idx: usize,
    ) -> Result<f64, ArrayError> {
        let tuple = self.numeric_tuple_as_f64(tuple_idx)?;
        tuple
            .get(component_idx)
            .copied()
            .ok_or(ArrayError::TupleComponentMismatch {
                destination: tuple.len(),
                source_components: component_idx + 1,
            })
    }

    pub(crate) fn set_numeric_component_from_f64_checked(
        &mut self,
        tuple_idx: usize,
        component_idx: usize,
        value: f64,
    ) -> Result<(), ArrayError> {
        self.set_numeric_component_from_f64(tuple_idx, component_idx, value)
    }

    fn deep_copy_numeric_from(&mut self, source: &Self) {
        let number_of_components = source.get_number_of_components();
        let number_of_tuples = source.get_number_of_tuples();

        self.set_number_of_components(number_of_components);
        self.set_number_of_tuples(number_of_tuples);

        let tuple_count = id_count_to_usize(number_of_tuples);
        for tuple_idx in 0..tuple_count {
            let Ok(tuple) = source.numeric_tuple_as_f64(tuple_idx) else {
                return;
            };
            if self.set_numeric_tuple_from_f64(tuple_idx, &tuple).is_err() {
                return;
            }
        }

        self.squeeze();
    }

    fn insert_numeric_tuple_from_f64(
        &mut self,
        tuple_idx: usize,
        tuple: &[f64],
    ) -> Result<(), ArrayError> {
        let data_type = self.get_data_type();
        let converted: Vec<_> = tuple
            .iter()
            .map(|&value| vtk_interpolated_component(value, data_type))
            .collect();
        dispatch_numeric_mut!(self, array => array.insert_typed_tuple_from_f64(tuple_idx, &converted))
            .ok_or(ArrayError::TypeMismatch {
                destination: self.get_data_type(),
                source_type: self.get_data_type(),
            })?;
        Ok(())
    }

    fn copy_numeric_tuple_from_f64(
        &mut self,
        tuple_idx: usize,
        tuple: &[f64],
    ) -> Result<(), ArrayError> {
        dispatch_numeric_mut!(self, array => array.insert_typed_tuple_from_f64(tuple_idx, tuple))
            .ok_or(ArrayError::TypeMismatch {
            destination: self.get_data_type(),
            source_type: self.get_data_type(),
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_and_signed_char_have_distinct_runtime_type_ids() {
        let char_array =
            AnyArray::Char(CharArray::with_name_and_number_of_components("letters", 1));
        let signed_char_array = AnyArray::SignedChar(
            SignedCharArray::with_name_and_number_of_components("letters", 1),
        );

        assert_eq!(char_array.get_data_type(), VtkDataType::Char);
        assert_eq!(signed_char_array.get_data_type(), VtkDataType::SignedChar);
        assert_ne!(
            char_array.get_data_type().id(),
            signed_char_array.get_data_type().id()
        );
    }

    #[test]
    fn numeric_copy_converts_between_vtk_data_arrays_without_interpolation_rounding() {
        let source = AnyArray::Double(DoubleArray::from_vec("values", vec![2.7, -2.7], 1));
        let mut target = AnyArray::Int(IntArray::with_name_and_number_of_components("values", 1));

        target.copy_tuple_from(&source, 0, 0).expect("copy tuple");
        target.copy_tuple_from(&source, 1, 1).expect("copy tuple");

        let AnyArray::Int(target) = target else {
            panic!("expected int array");
        };
        assert_eq!(target.as_slice(), &[2, -2]);
    }

    #[test]
    fn numeric_interpolation_uses_vtk_integer_rounding() {
        let source = AnyArray::Double(DoubleArray::from_vec("values", vec![2.7, -2.7], 1));
        let mut target = AnyArray::Int(IntArray::with_name_and_number_of_components("values", 1));

        assert!(target.interpolate_tuple_from(&source, &[0], &[1.0], 0));
        assert!(target.interpolate_tuple_from(&source, &[1], &[1.0], 1));

        let AnyArray::Int(target) = target else {
            panic!("expected int array");
        };
        assert_eq!(target.as_slice(), &[3, -4]);
    }

    #[test]
    fn string_interpolation_uses_largest_weight_tuple() {
        let source = AnyArray::String(StringArray::from_vec(
            "labels",
            vec!["low".to_string(), "high".to_string()],
            1,
        ));
        let mut target =
            AnyArray::String(StringArray::with_name_and_number_of_components("labels", 1));

        assert!(target.interpolate_tuple_from(&source, &[0, 1], &[0.25, 0.75], 0));

        let AnyArray::String(target) = target else {
            panic!("expected string array");
        };
        assert_eq!(target.as_slice(), &["high".to_string()]);
    }
}
