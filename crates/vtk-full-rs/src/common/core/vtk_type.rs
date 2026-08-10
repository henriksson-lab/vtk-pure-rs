/// VTK `vtkType.h` data type id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VtkDataType {
    Void,
    Bit,
    Char,
    UnsignedChar,
    Short,
    UnsignedShort,
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    Float,
    Double,
    IdType,
    String,
    Opaque,
    SignedChar,
    LongLong,
    UnsignedLongLong,
    Variant,
    Object,
}

impl VtkDataType {
    pub const VTK_VOID: i32 = 0;
    pub const VTK_BIT: i32 = 1;
    pub const VTK_CHAR: i32 = 2;
    pub const VTK_UNSIGNED_CHAR: i32 = 3;
    pub const VTK_SHORT: i32 = 4;
    pub const VTK_UNSIGNED_SHORT: i32 = 5;
    pub const VTK_INT: i32 = 6;
    pub const VTK_UNSIGNED_INT: i32 = 7;
    pub const VTK_LONG: i32 = 8;
    pub const VTK_UNSIGNED_LONG: i32 = 9;
    pub const VTK_FLOAT: i32 = 10;
    pub const VTK_DOUBLE: i32 = 11;
    pub const VTK_ID_TYPE: i32 = 12;
    pub const VTK_ID_TYPE_IMPL: i32 = Self::VTK_LONG_LONG;
    pub const VTK_STRING: i32 = 13;
    pub const VTK_OPAQUE: i32 = 14;
    pub const VTK_SIGNED_CHAR: i32 = 15;
    pub const VTK_LONG_LONG: i32 = 16;
    pub const VTK_UNSIGNED_LONG_LONG: i32 = 17;
    pub const VTK_VARIANT: i32 = 20;
    pub const VTK_OBJECT: i32 = 21;

    pub(crate) fn id(self) -> i32 {
        match self {
            Self::Void => Self::VTK_VOID,
            Self::Bit => Self::VTK_BIT,
            Self::Char => Self::VTK_CHAR,
            Self::UnsignedChar => Self::VTK_UNSIGNED_CHAR,
            Self::Short => Self::VTK_SHORT,
            Self::UnsignedShort => Self::VTK_UNSIGNED_SHORT,
            Self::Int => Self::VTK_INT,
            Self::UnsignedInt => Self::VTK_UNSIGNED_INT,
            Self::Long => Self::VTK_LONG,
            Self::UnsignedLong => Self::VTK_UNSIGNED_LONG,
            Self::Float => Self::VTK_FLOAT,
            Self::Double => Self::VTK_DOUBLE,
            Self::IdType => Self::VTK_ID_TYPE,
            Self::String => Self::VTK_STRING,
            Self::Opaque => Self::VTK_OPAQUE,
            Self::SignedChar => Self::VTK_SIGNED_CHAR,
            Self::LongLong => Self::VTK_LONG_LONG,
            Self::UnsignedLongLong => Self::VTK_UNSIGNED_LONG_LONG,
            Self::Variant => Self::VTK_VARIANT,
            Self::Object => Self::VTK_OBJECT,
        }
    }

    pub(crate) fn from_id(id: i32) -> Option<Self> {
        Some(match id {
            Self::VTK_VOID => Self::Void,
            Self::VTK_BIT => Self::Bit,
            Self::VTK_CHAR => Self::Char,
            Self::VTK_UNSIGNED_CHAR => Self::UnsignedChar,
            Self::VTK_SHORT => Self::Short,
            Self::VTK_UNSIGNED_SHORT => Self::UnsignedShort,
            Self::VTK_INT => Self::Int,
            Self::VTK_UNSIGNED_INT => Self::UnsignedInt,
            Self::VTK_LONG => Self::Long,
            Self::VTK_UNSIGNED_LONG => Self::UnsignedLong,
            Self::VTK_FLOAT => Self::Float,
            Self::VTK_DOUBLE => Self::Double,
            Self::VTK_ID_TYPE => Self::IdType,
            Self::VTK_STRING => Self::String,
            Self::VTK_OPAQUE => Self::Opaque,
            Self::VTK_SIGNED_CHAR => Self::SignedChar,
            Self::VTK_LONG_LONG => Self::LongLong,
            Self::VTK_UNSIGNED_LONG_LONG => Self::UnsignedLongLong,
            Self::VTK_VARIANT => Self::Variant,
            Self::VTK_OBJECT => Self::Object,
            _ => return None,
        })
    }

    pub fn vtk_name(self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Bit => "bit",
            Self::Char => "char",
            Self::UnsignedChar => "unsigned char",
            Self::Short => "short",
            Self::UnsignedShort => "unsigned short",
            Self::Int => "int",
            Self::UnsignedInt => "unsigned int",
            Self::Long => "long",
            Self::UnsignedLong => "unsigned long",
            Self::Float => "float",
            Self::Double => "double",
            Self::IdType => "idtype",
            Self::String => "string",
            Self::Opaque => "opaque",
            Self::SignedChar => "signed char",
            Self::LongLong => "long long",
            Self::UnsignedLongLong => "unsigned long long",
            Self::Variant => "variant",
            Self::Object => "object",
        }
    }

    pub(crate) fn size(self) -> usize {
        match self {
            Self::Bit | Self::Char | Self::UnsignedChar | Self::SignedChar => 1,
            Self::Short | Self::UnsignedShort => 2,
            Self::Int | Self::UnsignedInt | Self::Float => 4,
            Self::Long
            | Self::UnsignedLong
            | Self::Double
            | Self::IdType
            | Self::LongLong
            | Self::UnsignedLongLong => 8,
            _ => 0,
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::Bit
                | Self::Char
                | Self::UnsignedChar
                | Self::Short
                | Self::UnsignedShort
                | Self::Int
                | Self::UnsignedInt
                | Self::Long
                | Self::UnsignedLong
                | Self::Float
                | Self::Double
                | Self::IdType
                | Self::SignedChar
                | Self::LongLong
                | Self::UnsignedLongLong
        )
    }

    pub fn is_integral(self) -> bool {
        self.is_numeric() && !matches!(self, Self::Float | Self::Double)
    }

    pub(crate) fn range(self) -> Option<(f64, f64)> {
        Some(match self {
            Self::Bit => (0.0, 1.0),
            Self::Char | Self::SignedChar => (i8::MIN as f64, i8::MAX as f64),
            Self::UnsignedChar => (u8::MIN as f64, u8::MAX as f64),
            Self::Short => (i16::MIN as f64, i16::MAX as f64),
            Self::UnsignedShort => (u16::MIN as f64, u16::MAX as f64),
            Self::Int => (i32::MIN as f64, i32::MAX as f64),
            Self::UnsignedInt => (u32::MIN as f64, u32::MAX as f64),
            Self::Long | Self::IdType | Self::LongLong => (i64::MIN as f64, i64::MAX as f64),
            Self::UnsignedLong | Self::UnsignedLongLong => (u64::MIN as f64, u64::MAX as f64),
            Self::Float => (-1.0e38, 1.0e38),
            Self::Double => (-1.0e299, 1.0e299),
            _ => return None,
        })
    }
}

pub type VtkChar = i8;
pub type VtkLong = i64;
pub type VtkUnsignedLong = u64;
pub type VtkIdType = i64;
pub type VtkMTimeType = u64;
pub type VtkTypeInt64 = i64;
pub type VtkTypeUInt32 = u32;
pub type VtkTypeUInt64 = u64;

pub fn vtk_data_types_compare(left: i32, right: i32) -> bool {
    left == right
        || ((left == VtkDataType::VTK_ID_TYPE || left == VtkDataType::VTK_ID_TYPE_IMPL)
            && (right == VtkDataType::VTK_ID_TYPE || right == VtkDataType::VTK_ID_TYPE_IMPL))
}

pub(crate) fn vtk_round_integer(value: f64) -> f64 {
    if value >= 0.0 {
        (value + 0.5).floor()
    } else {
        (value - 0.5).floor()
    }
}

pub(crate) fn vtk_interpolated_component(value: f64, data_type: VtkDataType) -> f64 {
    let (min, max) = data_type.range().unwrap_or((f64::MIN, f64::MAX));
    let mut value = value.max(min).min(max);
    if data_type.is_integral() {
        value = vtk_round_integer(value);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_type_ids_match_vtk_type_h() {
        assert_eq!(VtkDataType::Bit.id(), 1);
        assert_eq!(VtkDataType::Double.id(), 11);
        assert_eq!(VtkDataType::IdType.id(), 12);
        assert_eq!(VtkDataType::Variant.id(), 20);
    }

    #[test]
    fn id_type_compares_with_underlying_long_long() {
        assert!(vtk_data_types_compare(
            VtkDataType::VTK_ID_TYPE,
            VtkDataType::VTK_LONG_LONG
        ));
        assert!(!vtk_data_types_compare(
            VtkDataType::VTK_ID_TYPE,
            VtkDataType::VTK_INT
        ));
    }

    #[test]
    fn integer_interpolation_clamps_and_rounds_like_vtk() {
        assert_eq!(
            vtk_interpolated_component(255.9, VtkDataType::UnsignedChar),
            255.0
        );
        assert_eq!(vtk_interpolated_component(2.5, VtkDataType::Int), 3.0);
        assert_eq!(vtk_interpolated_component(-2.5, VtkDataType::Int), -3.0);
    }
}
