use super::{any_array::AnyArray, double_array::DoubleArray, vtk_type::VtkIdType};

trait VtkAbsolute {
    fn vtk_absolute(self) -> Self;
}

macro_rules! impl_vtk_absolute_signed {
    ($($ty:ty),* $(,)?) => {
        $(
            impl VtkAbsolute for $ty {
                fn vtk_absolute(self) -> Self {
                    self.wrapping_abs()
                }
            }
        )*
    };
}

macro_rules! impl_vtk_absolute_unsigned {
    ($($ty:ty),* $(,)?) => {
        $(
            impl VtkAbsolute for $ty {
                fn vtk_absolute(self) -> Self {
                    self
                }
            }
        )*
    };
}

impl_vtk_absolute_signed!(i8, i16, i32, i64);
impl_vtk_absolute_unsigned!(u8, u16, u32, u64);

impl VtkAbsolute for f32 {
    fn vtk_absolute(self) -> Self {
        self.abs()
    }
}

impl VtkAbsolute for f64 {
    fn vtk_absolute(self) -> Self {
        self.abs()
    }
}

/// VTK: `vtkArrayComponents`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum VtkArrayComponents {
    /// VTK: `vtkArrayComponents::L1Norm`.
    L1Norm = -1,
    /// VTK: `vtkArrayComponents::L2Norm`.
    L2Norm = -2,
    /// VTK: `vtkArrayComponents::LInfNorm`.
    LInfNorm = -99,
    /// VTK: `vtkArrayComponents::AllComponents`.
    AllComponents = -100,
    /// VTK: `vtkArrayComponents::Requested`.
    Requested = -101,
}

impl VtkArrayComponents {
    fn from_i32(value: i32) -> Option<Self> {
        match value {
            -1 => Some(Self::L1Norm),
            -2 => Some(Self::L2Norm),
            -99 => Some(Self::LInfNorm),
            -100 => Some(Self::AllComponents),
            -101 => Some(Self::Requested),
            _ => None,
        }
    }
}

/// VTK: `vtk::ArrayComponents`.
pub fn array_components(enumerant_str: &str) -> i32 {
    match enumerant_str {
        "vtkArrayComponents::L1Norm" | "L1Norm" | "L₁norm" | "L₁ norm" | "||·||₁" => {
            VtkArrayComponents::L1Norm as i32
        }
        "vtkArrayComponents::L2Norm" | "L2Norm" | "L₂norm" | "L₂ norm" | "||·||₂" => {
            VtkArrayComponents::L2Norm as i32
        }
        "vtkArrayComponents::LInfNorm" | "LInfNorm" | "L∞norm" | "L∞ norm" | "||·||∞" => {
            VtkArrayComponents::LInfNorm as i32
        }
        "vtkArrayComponents::AllComponents" | "AllComponents" | "all components" => {
            VtkArrayComponents::AllComponents as i32
        }
        "vtkArrayComponents::Requested" | "Requested" | "requested" | "requested components" => {
            VtkArrayComponents::Requested as i32
        }
        _ => enumerant_str
            .parse::<i32>()
            .expect("VTK ArrayComponents fallback expects an integer string"),
    }
}

/// VTK: `vtk::to_string(vtkArrayComponents)`.
pub fn to_string(enumerant: i32) -> String {
    match VtkArrayComponents::from_i32(enumerant) {
        Some(VtkArrayComponents::AllComponents) => "all components".to_string(),
        Some(VtkArrayComponents::Requested) => "requested".to_string(),
        Some(VtkArrayComponents::L1Norm) => "L₁ norm".to_string(),
        Some(VtkArrayComponents::L2Norm) => "L₂ norm".to_string(),
        Some(VtkArrayComponents::LInfNorm) => "L∞ norm".to_string(),
        None => enumerant.to_string(),
    }
}

/// VTK: `vtk::to_enumerant<vtkArrayComponents>`.
pub fn to_enumerant(enumerant_str: &str) -> i32 {
    array_components(enumerant_str)
}

/// VTK: `vtk::ComponentOrNormAsArray`.
pub fn component_or_norm_as_array(array: Option<&AnyArray>, comp_or_norm: i32) -> Option<AnyArray> {
    let array = array?;
    if comp_or_norm == VtkArrayComponents::AllComponents as i32
        || (array.get_number_of_components() == 1 && comp_or_norm == 0)
    {
        return Some(array.shallow_clone());
    }

    let mut result = if array.is_data_array() {
        match VtkArrayComponents::from_i32(comp_or_norm) {
            Some(VtkArrayComponents::L1Norm) => Some(compute_l1_norm(array)),
            Some(VtkArrayComponents::L2Norm) => Some(compute_l2_norm(array)),
            Some(VtkArrayComponents::LInfNorm) => compute_l_inf_norm(array),
            Some(VtkArrayComponents::AllComponents) => unreachable!("handled above"),
            Some(VtkArrayComponents::Requested) | None => {
                array.copy_component_as_single_component(comp_or_norm)
            }
        }
    } else {
        array.copy_component_as_single_component(comp_or_norm)
    }?;

    let name = format!(
        "{}_{}",
        array_name(array),
        component_name(array, comp_or_norm)
    );
    result.set_name(name);
    Some(result)
}

/// VTK: `vtk::ComponentOrNormAsDataArray`.
pub fn component_or_norm_as_data_array(
    array: Option<&AnyArray>,
    comp_or_norm: i32,
) -> Option<AnyArray> {
    let result = component_or_norm_as_array(array, comp_or_norm)?;
    result.is_data_array().then_some(result)
}

fn array_name(array: &AnyArray) -> &str {
    let name = array.get_name();
    if name.is_empty() {
        "unnamed"
    } else {
        name
    }
}

fn component_name(array: &AnyArray, component: i32) -> String {
    match VtkArrayComponents::from_i32(component) {
        Some(VtkArrayComponents::L1Norm) => return "L₁".to_string(),
        Some(VtkArrayComponents::L2Norm) => return "L₂".to_string(),
        Some(VtkArrayComponents::LInfNorm) => return "L∞".to_string(),
        _ => {}
    }

    if component >= 0 {
        let component = component as VtkIdType;
        if array.has_a_component_name() {
            if let Some(name) = array.get_component_name(component) {
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
        return component.to_string();
    }

    String::new()
}

fn compute_l1_norm(array: &AnyArray) -> AnyArray {
    let mut norm = DoubleArray::new();
    norm.set_number_of_tuples(array.get_number_of_tuples());
    for tuple_idx in 0..array.get_number_of_tuples() {
        let tuple = array
            .component_tuple_values_as_f64(tuple_idx as usize)
            .expect("L1 norm requires a data array");
        let sum = tuple.iter().map(|value| value.abs()).sum::<f64>();
        norm.set_component(tuple_idx, 0, sum);
    }
    AnyArray::Double(norm)
}

fn compute_l2_norm(array: &AnyArray) -> AnyArray {
    let mut norm = DoubleArray::new();
    norm.set_number_of_tuples(array.get_number_of_tuples());
    for tuple_idx in 0..array.get_number_of_tuples() {
        let tuple = array
            .component_tuple_values_as_f64(tuple_idx as usize)
            .expect("L2 norm requires a data array");
        let norm2 = tuple.iter().map(|value| value * value).sum::<f64>();
        norm.set_component(tuple_idx, 0, norm2.sqrt());
    }
    AnyArray::Double(norm)
}

fn compute_l_inf_norm(array: &AnyArray) -> Option<AnyArray> {
    let mut norm = array.new_instance();
    norm.set_number_of_components(1);
    norm.set_number_of_tuples(array.get_number_of_tuples());

    macro_rules! compute_typed_l_inf_norm {
        ($dst:expr, $src:expr) => {{
            for tuple_idx in 0..$src.get_number_of_tuples() {
                let tuple = $src.get_typed_tuple(tuple_idx);
                let mut value = tuple[0].vtk_absolute();
                for component in &tuple[1..] {
                    let candidate = component.vtk_absolute();
                    if candidate > value {
                        value = candidate;
                    }
                }
                $dst.set_typed_tuple(tuple_idx, &[value]);
            }
        }};
    }

    match (&mut norm, array) {
        (AnyArray::Bit(dst), AnyArray::Bit(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::Char(dst), AnyArray::Char(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::SignedChar(dst), AnyArray::SignedChar(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::UnsignedChar(dst), AnyArray::UnsignedChar(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::Short(dst), AnyArray::Short(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::UnsignedShort(dst), AnyArray::UnsignedShort(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::Int(dst), AnyArray::Int(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::UnsignedInt(dst), AnyArray::UnsignedInt(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::Long(dst), AnyArray::Long(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::UnsignedLong(dst), AnyArray::UnsignedLong(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::Float(dst), AnyArray::Float(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::Double(dst), AnyArray::Double(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::IdType(dst), AnyArray::IdType(src)) => compute_typed_l_inf_norm!(dst, src),
        (AnyArray::LongLong(dst), AnyArray::LongLong(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::UnsignedLongLong(dst), AnyArray::UnsignedLongLong(src)) => {
            compute_typed_l_inf_norm!(dst, src)
        }
        (AnyArray::String(_), AnyArray::String(_))
        | (AnyArray::Variant(_), AnyArray::Variant(_)) => {
            return None;
        }
        _ => unreachable!("new_instance returns the same concrete array type"),
    }
    Some(norm)
}
