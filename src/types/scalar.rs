use std::fmt;

/// Runtime identifier for scalar types stored in data arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScalarType {
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl ScalarType {
    /// Name used in VTK legacy file format (e.g. "float", "double", "int").
    pub fn vtk_name(&self) -> &'static str {
        match self {
            ScalarType::F32 => "float",
            ScalarType::F64 => "double",
            ScalarType::I8 => "signed_char",
            ScalarType::I16 => "short",
            ScalarType::I32 => "int",
            ScalarType::I64 => "vtktypeint64",
            ScalarType::U8 => "unsigned_char",
            ScalarType::U16 => "unsigned_short",
            ScalarType::U32 => "unsigned_int",
            ScalarType::U64 => "vtktypeuint64",
        }
    }

    /// Parse a VTK legacy type name.
    pub fn from_vtk_name(name: &str) -> Option<ScalarType> {
        match name.to_ascii_lowercase().as_str() {
            "float" => Some(ScalarType::F32),
            "double" => Some(ScalarType::F64),
            "char" | "signed_char" | "int8" | "vtktypeint8" => Some(ScalarType::I8),
            "short" => Some(ScalarType::I16),
            "int" => Some(ScalarType::I32),
            "long" | "long_long" | "vtkidtype" | "vtktypeint64" => Some(ScalarType::I64),
            "unsigned_char" | "uint8" | "vtktypeuint8" => Some(ScalarType::U8),
            "unsigned_short" => Some(ScalarType::U16),
            "unsigned_int" => Some(ScalarType::U32),
            "unsigned_long" | "unsigned_long_long" | "vtktypeuint64" => Some(ScalarType::U64),
            _ => None,
        }
    }

    /// Size of this scalar type in bytes.
    pub fn size(&self) -> usize {
        match self {
            ScalarType::F32 | ScalarType::I32 | ScalarType::U32 => 4,
            ScalarType::F64 | ScalarType::I64 | ScalarType::U64 => 8,
            ScalarType::I16 | ScalarType::U16 => 2,
            ScalarType::I8 | ScalarType::U8 => 1,
        }
    }
}

impl fmt::Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.vtk_name())
    }
}

/// Trait for types that can be stored in a `DataArray`.
///
/// Implemented for all standard numeric types (f32, f64, i8..i64, u8..u64).
pub trait Scalar:
    Copy + Default + Send + Sync + PartialOrd + fmt::Display + fmt::Debug + 'static
{
    const TYPE_ID: ScalarType;

    /// Convert to f64 for type-erased operations.
    fn to_f64(self) -> f64;

    /// Convert from f64 for type-erased operations.
    fn from_f64(v: f64) -> Self;
}

macro_rules! impl_scalar {
    ($ty:ty, $id:ident) => {
        impl Scalar for $ty {
            const TYPE_ID: ScalarType = ScalarType::$id;

            #[inline]
            fn to_f64(self) -> f64 {
                self as f64
            }

            #[inline]
            fn from_f64(v: f64) -> Self {
                v as Self
            }
        }
    };
}

impl_scalar!(f32, F32);
impl_scalar!(f64, F64);
impl_scalar!(i8, I8);
impl_scalar!(i16, I16);
impl_scalar!(i32, I32);
impl_scalar!(i64, I64);
impl_scalar!(u8, U8);
impl_scalar!(u16, U16);
impl_scalar!(u32, U32);
impl_scalar!(u64, U64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vtk_name_roundtrip() {
        for ty in [
            ScalarType::F32,
            ScalarType::F64,
            ScalarType::I8,
            ScalarType::I16,
            ScalarType::I32,
            ScalarType::I64,
            ScalarType::U8,
            ScalarType::U16,
            ScalarType::U32,
            ScalarType::U64,
        ] {
            assert_eq!(ScalarType::from_vtk_name(ty.vtk_name()), Some(ty));
        }
    }

    #[test]
    fn vtk_aliases_match_legacy_names() {
        assert_eq!(
            ScalarType::from_vtk_name("signed_char"),
            Some(ScalarType::I8)
        );
        assert_eq!(
            ScalarType::from_vtk_name("unsigned_char"),
            Some(ScalarType::U8)
        );
        assert_eq!(
            ScalarType::from_vtk_name("long_long"),
            Some(ScalarType::I64)
        );
        assert_eq!(
            ScalarType::from_vtk_name("unsigned_long_long"),
            Some(ScalarType::U64)
        );
        assert_eq!(
            ScalarType::from_vtk_name("vtkIdType"),
            Some(ScalarType::I64)
        );
    }

    #[test]
    fn scalar_f64_conversion() {
        assert_eq!(42i32.to_f64(), 42.0);
        assert_eq!(i32::from_f64(42.7), 42);
    }
}
