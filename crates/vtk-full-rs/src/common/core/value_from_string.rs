/// VTK: `vtkValueFromString<T>`.
pub fn vtk_value_from_string<T: VtkValueFromString>(
    begin: &str,
    end: usize,
    output: &mut T,
) -> usize {
    let end = end.min(begin.len());
    let Some(input) = begin.get(..end) else {
        return 0;
    };
    T::parse_value_from_string(input, output)
}

pub trait VtkValueFromString: private::Sealed {
    fn parse_value_from_string(input: &str, output: &mut Self) -> usize;
}

mod private {
    pub trait Sealed {}
}

macro_rules! impl_signed_integer {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl private::Sealed for $ty {}

            impl VtkValueFromString for $ty {
                fn parse_value_from_string(input: &str, output: &mut Self) -> usize {
                    parse_signed_integer(input, output)
                }
            }
        )+
    };
}

macro_rules! impl_unsigned_integer {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl private::Sealed for $ty {}

            impl VtkValueFromString for $ty {
                fn parse_value_from_string(input: &str, output: &mut Self) -> usize {
                    parse_unsigned_integer(input, output)
                }
            }
        )+
    };
}

macro_rules! impl_float {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl private::Sealed for $ty {}

            impl VtkValueFromString for $ty {
                fn parse_value_from_string(input: &str, output: &mut Self) -> usize {
                    parse_float(input, output)
                }
            }
        )+
    };
}

impl_signed_integer!(i8, i16, i32, i64);
impl_unsigned_integer!(u8, u16, u32, u64);
impl_float!(f32, f64);

impl private::Sealed for bool {}

impl VtkValueFromString for bool {
    fn parse_value_from_string(input: &str, output: &mut Self) -> usize {
        parse_bool(input, output)
    }
}

fn detect_base(input: &[u8], mut index: usize) -> (usize, u32) {
    if input[index] == b'0' {
        index += 1;
        if index == input.len() {
            return (index, 0);
        }
        match input[index] {
            b'x' | b'X' => {
                index += 1;
                if index == input.len() {
                    return (index - 1, 0);
                }
                (index, 16)
            }
            b'b' | b'B' => {
                index += 1;
                if index == input.len() {
                    return (index - 1, 0);
                }
                (index, 2)
            }
            b'o' | b'O' => {
                index += 1;
                if index == input.len() {
                    return (index - 1, 0);
                }
                (index, 8)
            }
            _ => (index, 0),
        }
    } else {
        (index, 10)
    }
}

trait SignedInteger: Copy {
    fn zero() -> Self;
    fn from_decimal(value: i128) -> Option<Self>;
    fn from_prefixed_unsigned(value: u128) -> Option<Self>;
}

macro_rules! impl_signed_integer_helpers {
    ($($ty:ty => $unsigned:ty),+ $(,)?) => {
        $(
            impl SignedInteger for $ty {
                fn zero() -> Self {
                    0
                }

                fn from_decimal(value: i128) -> Option<Self> {
                    Self::try_from(value).ok()
                }

                fn from_prefixed_unsigned(value: u128) -> Option<Self> {
                    let value = <$unsigned>::try_from(value).ok()?;
                    Some(value as Self)
                }
            }
        )+
    };
}

impl_signed_integer_helpers!(i8 => u8, i16 => u16, i32 => u32, i64 => u64);

fn parse_signed_integer<T>(input: &str, output: &mut T) -> usize
where
    T: SignedInteger,
{
    let bytes = input.as_bytes();
    if bytes.is_empty() {
        return 0;
    }

    let mut index = 0;
    let mut minus_sign = false;
    if bytes[index] == b'-' {
        minus_sign = true;
        index += 1;
    } else if bytes[index] == b'+' {
        index += 1;
    }
    if index == bytes.len() {
        return 0;
    }

    let (start, base) = detect_base(bytes, index);
    if base == 0 {
        *output = T::zero();
        return start;
    }
    if base != 10 && minus_sign {
        return 0;
    }

    let (digits_end, value) = parse_unsigned_digits(bytes, start, base);
    let Some(value) = value else {
        return 0;
    };

    if base == 10 {
        let signed = if minus_sign {
            let Ok(value) = i128::try_from(value) else {
                return 0;
            };
            -value
        } else {
            let Ok(value) = i128::try_from(value) else {
                return 0;
            };
            value
        };
        if let Some(value) = T::from_decimal(signed) {
            *output = value;
            return digits_end;
        }
        return 0;
    }

    if let Some(value) = T::from_prefixed_unsigned(value) {
        *output = value;
        digits_end
    } else {
        0
    }
}

fn parse_unsigned_integer<T>(input: &str, output: &mut T) -> usize
where
    T: TryFrom<u128> + Copy,
{
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] == b'-' {
        return 0;
    }

    let mut index = 0;
    if bytes[index] == b'+' {
        index += 1;
    }
    if index == bytes.len() {
        return 0;
    }

    let (start, base) = detect_base(bytes, index);
    if base == 0 {
        if let Ok(value) = T::try_from(0_u128) {
            *output = value;
            return start;
        }
        return 0;
    }

    let (digits_end, value) = parse_unsigned_digits(bytes, start, base);
    let Some(value) = value else {
        return 0;
    };
    if let Ok(value) = T::try_from(value) {
        *output = value;
        digits_end
    } else {
        0
    }
}

fn parse_unsigned_digits(input: &[u8], start: usize, base: u32) -> (usize, Option<u128>) {
    let mut index = start;
    let mut value = 0_u128;
    let mut saw_digit = false;

    while index < input.len() {
        let digit = match input[index] {
            b'0'..=b'9' => u32::from(input[index] - b'0'),
            b'a'..=b'f' => 10 + u32::from(input[index] - b'a'),
            b'A'..=b'F' => 10 + u32::from(input[index] - b'A'),
            _ => break,
        };
        if digit >= base {
            break;
        }
        let Some(next) = value
            .checked_mul(u128::from(base))
            .and_then(|value| value.checked_add(u128::from(digit)))
        else {
            return (index, None);
        };
        value = next;
        saw_digit = true;
        index += 1;
    }

    (index, saw_digit.then_some(value))
}

fn parse_float<T>(input: &str, output: &mut T) -> usize
where
    T: FloatFromF64,
{
    if input.is_empty() || input.as_bytes()[0] == b'+' {
        return 0;
    }

    let Some(end) = float_prefix_end(input) else {
        return 0;
    };
    let Ok(value) = input[..end].parse::<f64>() else {
        return 0;
    };
    *output = T::from_f64(value);
    end
}

trait FloatFromF64: Copy {
    fn from_f64(value: f64) -> Self;
}

impl FloatFromF64 for f32 {
    fn from_f64(value: f64) -> Self {
        value as Self
    }
}

impl FloatFromF64 for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }
}

fn float_prefix_end(input: &str) -> Option<usize> {
    let lower = input.to_ascii_lowercase();
    if lower.starts_with("-inf") {
        return Some(4);
    }
    if lower.starts_with("inf") || lower.starts_with("nan") {
        return Some(3);
    }

    let bytes = input.as_bytes();
    let mut index = 0;
    if bytes.get(index) == Some(&b'-') {
        index += 1;
    }

    let mut digits_before = 0;
    while matches!(bytes.get(index), Some(b'0'..=b'9')) {
        digits_before += 1;
        index += 1;
    }

    let mut digits_after = 0;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            digits_after += 1;
            index += 1;
        }
    }

    if digits_before == 0 && digits_after == 0 {
        return None;
    }

    let mantissa_end = index;
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        let exponent_mark = index;
        index += 1;
        if matches!(bytes.get(index), Some(b'-' | b'+')) {
            index += 1;
        }
        let exponent_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
        if exponent_start == index {
            return Some(mantissa_end);
        }
        if input[..index].parse::<f64>().is_ok() {
            return Some(index);
        }
        return Some(exponent_mark);
    }

    Some(mantissa_end)
}

fn parse_bool(input: &str, output: &mut bool) -> usize {
    if input.is_empty() {
        return 0;
    }
    if input.as_bytes()[0] == b'0' {
        *output = false;
        return 1;
    }
    if input.as_bytes()[0] == b'1' {
        *output = true;
        return 1;
    }
    if input.starts_with("true") || input.starts_with("True") {
        *output = true;
        return 4;
    }
    if input.starts_with("false") || input.starts_with("False") {
        *output = false;
        return 5;
    }
    0
}
