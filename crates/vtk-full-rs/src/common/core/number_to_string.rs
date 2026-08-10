use std::fmt;

/// VTK: `vtkNumberToString::Notation`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Mixed = 0,
    Scientific = 1,
    Fixed = 2,
}

impl Notation {
    fn from_i32(value: i32) -> Self {
        match value {
            1 => Self::Scientific,
            2 => Self::Fixed,
            _ => Self::Mixed,
        }
    }
}

/// VTK: `vtkNumberToString`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberToString {
    low_exponent: i32,
    high_exponent: i32,
    notation: i32,
    precision: i32,
}

impl NumberToString {
    pub fn new() -> Self {
        Self {
            low_exponent: -6,
            high_exponent: 20,
            notation: Notation::Mixed as i32,
            precision: 2,
        }
    }

    /// VTK: `vtkNumberToString::SetLowExponent`.
    pub fn set_low_exponent(&mut self, low_exponent: i32) {
        self.low_exponent = low_exponent;
    }

    /// VTK: `vtkNumberToString::GetLowExponent`.
    pub fn get_low_exponent(&self) -> i32 {
        self.low_exponent
    }

    /// VTK: `vtkNumberToString::SetHighExponent`.
    pub fn set_high_exponent(&mut self, high_exponent: i32) {
        self.high_exponent = high_exponent;
    }

    /// VTK: `vtkNumberToString::GetHighExponent`.
    pub fn get_high_exponent(&self) -> i32 {
        self.high_exponent
    }

    /// VTK: `vtkNumberToString::SetNotation`.
    pub fn set_notation(&mut self, notation: i32) {
        self.notation = notation;
    }

    /// VTK: `vtkNumberToString::GetNotation`.
    pub fn get_notation(&self) -> i32 {
        self.notation
    }

    /// VTK: `vtkNumberToString::SetPrecision`.
    pub fn set_precision(&mut self, precision: i32) {
        self.precision = precision;
    }

    /// VTK: `vtkNumberToString::GetPrecision`.
    pub fn get_precision(&self) -> i32 {
        self.precision
    }

    /// VTK: `vtkNumberToString::Convert`.
    pub fn convert<T: NumberToStringValue>(&self, value: T) -> String {
        value.convert_with(self)
    }

    /// VTK: `vtkNumberToString::operator()`.
    pub fn call<T>(&self, value: T) -> T {
        value
    }

    fn convert_f64(&self, value: f64) -> String {
        convert_float(
            Notation::from_i32(self.notation),
            self.precision.max(0) as usize,
            self.low_exponent,
            self.high_exponent,
            value,
            17,
        )
    }

    fn convert_f32(&self, value: f32) -> String {
        convert_float(
            Notation::from_i32(self.notation),
            self.precision.max(0) as usize,
            self.low_exponent,
            self.high_exponent,
            f64::from(value),
            9,
        )
    }
}

impl Default for NumberToString {
    fn default() -> Self {
        Self::new()
    }
}

/// Rust dispatch for VTK `vtkNumberToString::Convert`.
pub trait NumberToStringValue {
    fn convert_with(self, converter: &NumberToString) -> String;
}

impl NumberToStringValue for f64 {
    fn convert_with(self, converter: &NumberToString) -> String {
        converter.convert_f64(self)
    }
}

impl NumberToStringValue for f32 {
    fn convert_with(self, converter: &NumberToString) -> String {
        converter.convert_f32(self)
    }
}

macro_rules! impl_to_string_value {
    ($($value:ty),* $(,)?) => {
        $(
            impl NumberToStringValue for $value {
                fn convert_with(self, _converter: &NumberToString) -> String {
                    self.to_string()
                }
            }
        )*
    };
}

impl_to_string_value!(
    bool, char, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, String, &str
);

/// VTK: `vtkNumberToString::TagDouble`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TagDouble {
    pub value: f64,
}

impl TagDouble {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl fmt::Display for TagDouble {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&NumberToString::new().convert(self.value))
    }
}

/// VTK: `vtkNumberToString::TagFloat`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TagFloat {
    pub value: f32,
}

impl TagFloat {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl fmt::Display for TagFloat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&NumberToString::new().convert(self.value))
    }
}

fn convert_float(
    notation: Notation,
    precision: usize,
    low_exponent: i32,
    high_exponent: i32,
    value: f64,
    max_digits: usize,
) -> String {
    if value.is_infinite() {
        return if value.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if value.is_nan() {
        return "NaN".to_string();
    }

    match notation {
        Notation::Scientific => format!("{value:.precision$e}"),
        Notation::Fixed => format!("{value:.precision$}"),
        Notation::Mixed => {
            let exponent = if value != 0.0 {
                value.abs().log10().floor() as i32
            } else {
                0
            };
            if exponent <= low_exponent || exponent >= high_exponent {
                remove_trailing_zeros_scientific(&format!("{value:.max_digits$e}"))
            } else {
                value.to_string()
            }
        }
    }
}

fn remove_trailing_zeros_scientific(value: &str) -> String {
    let Some(e_pos) = value.find('e') else {
        return value.to_string();
    };
    let Some(dot_pos) = value.find('.') else {
        return value.to_string();
    };
    if dot_pos >= e_pos {
        return value.to_string();
    }

    let mut last_non_zero = e_pos;
    while last_non_zero > dot_pos && value.as_bytes()[last_non_zero - 1] == b'0' {
        last_non_zero -= 1;
    }
    if last_non_zero == dot_pos + 1 {
        last_non_zero -= 1;
    }

    let mut result = String::with_capacity(value.len());
    result.push_str(&value[..last_non_zero]);
    result.push_str(&value[e_pos..]);
    result
}
