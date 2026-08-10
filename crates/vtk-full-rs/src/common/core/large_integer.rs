use std::{
    fmt,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
    str::FromStr,
};

const BIT_INCREMENT: usize = 32;

/// VTK: `vtkLargeInteger`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LargeInteger {
    number: Vec<u8>,
    negative: bool,
    sig: usize,
}

impl LargeInteger {
    /// VTK: `vtkLargeInteger::vtkLargeInteger`.
    pub fn new() -> Self {
        Self {
            number: vec![0; BIT_INCREMENT],
            negative: false,
            sig: 0,
        }
    }

    /// VTK: `vtkLargeInteger::vtkLargeInteger(long)`.
    pub fn from_long(n: i64) -> Self {
        Self::from_signed(n)
    }

    /// VTK: `vtkLargeInteger::vtkLargeInteger(unsigned long)`.
    pub fn from_unsigned_long(n: u64) -> Self {
        Self::from_unsigned(n)
    }

    /// VTK: `vtkLargeInteger::vtkLargeInteger(int)`.
    pub fn from_int(n: i32) -> Self {
        Self::from_signed(i64::from(n))
    }

    /// VTK: `vtkLargeInteger::vtkLargeInteger(unsigned int)`.
    pub fn from_unsigned_int(n: u32) -> Self {
        Self::from_unsigned(u64::from(n))
    }

    /// VTK: `vtkLargeInteger::vtkLargeInteger(long long)`.
    pub fn from_long_long(n: i64) -> Self {
        Self::from_signed(n)
    }

    /// VTK: `vtkLargeInteger::vtkLargeInteger(unsigned long long)`.
    pub fn from_unsigned_long_long(n: u64) -> Self {
        Self::from_unsigned(n)
    }

    fn from_signed(n: i64) -> Self {
        let negative = n < 0;
        let magnitude = n.unsigned_abs();
        let mut value = Self::from_unsigned(magnitude);
        value.negative = negative && !value.is_zero_bool();
        value
    }

    fn from_unsigned(mut n: u64) -> Self {
        let mut value = Self {
            number: vec![0; BIT_INCREMENT],
            negative: false,
            sig: BIT_INCREMENT - 1,
        };
        for bit in &mut value.number {
            *bit = (n & 1) as u8;
            n >>= 1;
        }
        value.contract();
        value
    }

    /// VTK: `vtkLargeInteger::CastToChar`.
    pub fn cast_to_char(&self) -> i8 {
        self.cast_to_long() as i8
    }

    /// VTK: `vtkLargeInteger::CastToShort`.
    pub fn cast_to_short(&self) -> i16 {
        self.cast_to_long() as i16
    }

    /// VTK: `vtkLargeInteger::CastToInt`.
    pub fn cast_to_int(&self) -> i32 {
        self.cast_to_long() as i32
    }

    /// VTK: `vtkLargeInteger::CastToLong`.
    pub fn cast_to_long(&self) -> i64 {
        let mut n = 0_i64;
        for i in (0..=self.sig).rev() {
            n = n.wrapping_shl(1);
            n |= i64::from(self.number[i]);
        }
        if self.negative {
            -n
        } else {
            n
        }
    }

    /// VTK: `vtkLargeInteger::CastToUnsignedLong`.
    pub fn cast_to_unsigned_long(&self) -> u64 {
        if self.sig >= u64::BITS as usize {
            u64::MAX
        } else {
            let mut n = 0_u64;
            for i in (0..=self.sig).rev() {
                n <<= 1;
                n |= u64::from(self.number[i]);
            }
            n
        }
    }

    /// VTK: `vtkLargeInteger::IsEven`.
    pub fn is_even(&self) -> i32 {
        (self.number[0] == 0) as i32
    }

    /// VTK: `vtkLargeInteger::IsOdd`.
    pub fn is_odd(&self) -> i32 {
        (self.number[0] == 1) as i32
    }

    /// VTK: `vtkLargeInteger::GetLength`.
    pub fn get_length(&self) -> i32 {
        (self.sig + 1) as i32
    }

    /// VTK: `vtkLargeInteger::GetBit`.
    pub fn get_bit(&self, p: u32) -> i32 {
        let p = p as usize;
        if p <= self.sig {
            i32::from(self.number[p])
        } else {
            0
        }
    }

    /// VTK: `vtkLargeInteger::IsZero`.
    pub fn is_zero(&self) -> i32 {
        self.is_zero_bool() as i32
    }

    /// VTK: `vtkLargeInteger::GetSign`.
    pub fn get_sign(&self) -> i32 {
        self.negative as i32
    }

    /// VTK: `vtkLargeInteger::Truncate`.
    pub fn truncate(&mut self, n: u32) {
        if n < 1 {
            self.sig = 0;
            self.number[0] = 0;
            self.negative = false;
        } else if self.sig > n as usize - 1 {
            self.sig = n as usize - 1;
            self.contract();
        }
    }

    /// VTK: `vtkLargeInteger::Complement`.
    pub fn complement(&mut self) {
        if !self.is_zero_bool() {
            self.negative = !self.negative;
        }
    }

    /// VTK: `vtkLargeInteger::IsSmaller`.
    fn is_smaller(&self, n: &Self) -> bool {
        if self.sig < n.sig {
            return true;
        }
        if self.sig > n.sig {
            return false;
        }
        for i in (0..=self.sig).rev() {
            if self.number[i] < n.number[i] {
                return true;
            }
            if self.number[i] > n.number[i] {
                return false;
            }
        }
        false
    }

    /// VTK: `vtkLargeInteger::IsGreater`.
    fn is_greater(&self, n: &Self) -> bool {
        if self.sig > n.sig {
            return true;
        }
        if self.sig < n.sig {
            return false;
        }
        for i in (0..=self.sig).rev() {
            if self.number[i] > n.number[i] {
                return true;
            }
            if self.number[i] < n.number[i] {
                return false;
            }
        }
        false
    }

    /// VTK: `vtkLargeInteger::Expand`.
    fn expand(&mut self, n: usize) {
        if n < self.sig {
            return;
        }
        if self.number.len() <= n {
            self.number.resize(n + 1, 0);
        }
        for i in self.sig + 1..self.number.len() {
            self.number[i] = 0;
        }
        self.sig = n;
    }

    /// VTK: `vtkLargeInteger::Contract`.
    fn contract(&mut self) {
        while self.number[self.sig] == 0 && self.sig > 0 {
            self.sig -= 1;
        }
    }

    /// VTK: `vtkLargeInteger::Plus`.
    fn plus(&mut self, n: &Self) {
        let m = usize::max(self.sig + 1, n.sig + 1);
        self.expand(m);
        let mut i = 0;
        let mut carry = 0_i32;
        while i <= n.sig {
            carry += i32::from(self.number[i]) + i32::from(n.number[i]);
            self.number[i] = (carry & 1) as u8;
            carry /= 2;
            i += 1;
        }
        while carry != 0 {
            carry += i32::from(self.number[i]);
            self.number[i] = (carry & 1) as u8;
            carry /= 2;
            i += 1;
        }
        self.contract();
    }

    /// VTK: `vtkLargeInteger::Minus`.
    fn minus(&mut self, n: &Self) {
        let m = usize::max(self.sig, n.sig);
        self.expand(m);
        let mut i = 0;
        let mut carry = 0_i32;
        while i <= n.sig {
            carry += i32::from(self.number[i]) - i32::from(n.number[i]);
            self.number[i] = ((carry + 2) & 1) as u8;
            carry = if carry < 0 { -1 } else { 0 };
            i += 1;
        }
        while carry != 0 {
            carry += i32::from(self.number[i]);
            self.number[i] = ((carry + 2) & 1) as u8;
            carry = if carry < 0 { -1 } else { 0 };
            i += 1;
        }
        self.contract();
    }

    fn is_zero_bool(&self) -> bool {
        self.sig == 0 && self.number[0] == 0
    }
}

impl Default for LargeInteger {
    fn default() -> Self {
        Self::new()
    }
}

impl From<i64> for LargeInteger {
    fn from(value: i64) -> Self {
        Self::from_long_long(value)
    }
}

impl From<u64> for LargeInteger {
    fn from(value: u64) -> Self {
        Self::from_unsigned_long_long(value)
    }
}

impl From<i32> for LargeInteger {
    fn from(value: i32) -> Self {
        Self::from_int(value)
    }
}

impl From<u32> for LargeInteger {
    fn from(value: u32) -> Self {
        Self::from_unsigned_int(value)
    }
}

impl PartialOrd for LargeInteger {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LargeInteger {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            std::cmp::Ordering::Equal
        } else if self.negative && !other.negative {
            std::cmp::Ordering::Less
        } else if !self.negative && other.negative {
            std::cmp::Ordering::Greater
        } else if self.negative {
            if self.is_smaller(other) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        } else if self.is_smaller(other) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

impl AddAssign<&LargeInteger> for LargeInteger {
    fn add_assign(&mut self, n: &LargeInteger) {
        if !(self.negative ^ n.negative) {
            self.plus(n);
        } else {
            if self.is_smaller(n) {
                let m = self.clone();
                *self = n.clone();
                self.minus(&m);
            } else {
                self.minus(n);
            }
            if self.is_zero_bool() {
                self.negative = false;
            }
        }
    }
}

impl AddAssign<LargeInteger> for LargeInteger {
    fn add_assign(&mut self, rhs: LargeInteger) {
        *self += &rhs;
    }
}

impl AddAssign<i32> for LargeInteger {
    fn add_assign(&mut self, rhs: i32) {
        *self += &LargeInteger::from(rhs);
    }
}

impl SubAssign<&LargeInteger> for LargeInteger {
    fn sub_assign(&mut self, n: &LargeInteger) {
        if self.negative ^ n.negative {
            self.plus(n);
        } else {
            if self.is_smaller(n) {
                let m = self.clone();
                *self = n.clone();
                self.minus(&m);
                self.complement();
            } else {
                self.minus(n);
            }
            if self.is_zero_bool() {
                self.negative = false;
            }
        }
    }
}

impl SubAssign<LargeInteger> for LargeInteger {
    fn sub_assign(&mut self, rhs: LargeInteger) {
        *self -= &rhs;
    }
}

impl SubAssign<i32> for LargeInteger {
    fn sub_assign(&mut self, rhs: i32) {
        *self -= &LargeInteger::from(rhs);
    }
}

impl ShlAssign<i32> for LargeInteger {
    fn shl_assign(&mut self, n: i32) {
        if n < 0 {
            *self >>= -n;
            return;
        }
        let n = n as usize;
        self.expand(self.sig + n);
        for i in (n..=self.sig).rev() {
            self.number[i] = self.number[i - n];
        }
        for i in 0..n {
            self.number[i] = 0;
        }
        self.contract();
    }
}

impl ShrAssign<i32> for LargeInteger {
    fn shr_assign(&mut self, n: i32) {
        if n < 0 {
            *self <<= -n;
            return;
        }
        let n = n as usize;
        if self.sig >= n {
            for i in 0..=self.sig - n {
                self.number[i] = self.number[i + n];
            }
        }
        let start = self.sig.saturating_sub(n).saturating_add(1);
        for i in start..=self.sig {
            self.number[i] = 0;
        }
        self.sig = start.saturating_sub(1);
        if self.is_zero_bool() {
            self.negative = false;
        }
    }
}

impl MulAssign<&LargeInteger> for LargeInteger {
    fn mul_assign(&mut self, n: &LargeInteger) {
        let mut c = LargeInteger::new();
        let m2 = self.sig + n.sig + 1;
        self.expand(m2);
        if n.is_smaller(self) {
            for i in 0..=n.sig {
                if n.number[i] == 1 {
                    c.plus(self);
                }
                *self <<= 1;
            }
        } else {
            let mut m = n.clone();
            let last = self.sig;
            for i in 0..=last {
                if self.number[i] == 1 {
                    c.plus(&m);
                }
                m <<= 1;
            }
        }
        if c.is_zero_bool() {
            c.negative = false;
        } else {
            c.negative = self.negative ^ n.negative;
        }
        *self = c;
        self.contract();
    }
}

impl MulAssign<LargeInteger> for LargeInteger {
    fn mul_assign(&mut self, rhs: LargeInteger) {
        *self *= &rhs;
    }
}

impl DivAssign<&LargeInteger> for LargeInteger {
    fn div_assign(&mut self, n: &LargeInteger) {
        if n.is_zero_bool() {
            return;
        }
        let mut c = LargeInteger::new();
        let mut m = n.clone();
        m <<= i32::max(self.sig as i32 - n.sig as i32, 0);
        let mut i = LargeInteger::from(1_i32);
        i <<= self.sig as i32 - n.sig as i32;
        while i > LargeInteger::new() {
            if !m.is_greater(self) {
                self.minus(&m);
                c += &i;
            }
            m >>= 1;
            i >>= 1;
        }
        if c.is_zero_bool() {
            c.negative = false;
        } else {
            c.negative = self.negative ^ n.negative;
        }
        *self = c;
    }
}

impl DivAssign<LargeInteger> for LargeInteger {
    fn div_assign(&mut self, rhs: LargeInteger) {
        *self /= &rhs;
    }
}

impl RemAssign<&LargeInteger> for LargeInteger {
    fn rem_assign(&mut self, n: &LargeInteger) {
        if n.is_zero_bool() {
            return;
        }
        let mut m = n.clone();
        let diff = self.sig as i32 - n.sig as i32;
        if diff < 0 {
            return;
        }
        m <<= diff;
        for _ in (0..=diff).rev() {
            if !m.is_greater(self) {
                self.minus(&m);
            }
            m >>= 1;
        }
        if self.is_zero_bool() {
            self.negative = false;
        }
    }
}

impl RemAssign<LargeInteger> for LargeInteger {
    fn rem_assign(&mut self, rhs: LargeInteger) {
        *self %= &rhs;
    }
}

impl BitAndAssign<&LargeInteger> for LargeInteger {
    fn bitand_assign(&mut self, n: &LargeInteger) {
        let m = usize::max(self.sig, n.sig);
        self.expand(m);
        for i in (0..=usize::min(self.sig, n.sig)).rev() {
            self.number[i] &= n.number[i];
        }
        self.contract();
    }
}

impl BitAndAssign<LargeInteger> for LargeInteger {
    fn bitand_assign(&mut self, rhs: LargeInteger) {
        *self &= &rhs;
    }
}

impl BitOrAssign<&LargeInteger> for LargeInteger {
    fn bitor_assign(&mut self, n: &LargeInteger) {
        let m = usize::max(self.sig, n.sig);
        self.expand(m);
        for i in (0..=usize::min(self.sig, n.sig)).rev() {
            self.number[i] |= n.number[i];
        }
        self.contract();
    }
}

impl BitOrAssign<LargeInteger> for LargeInteger {
    fn bitor_assign(&mut self, rhs: LargeInteger) {
        *self |= &rhs;
    }
}

impl BitXorAssign<&LargeInteger> for LargeInteger {
    fn bitxor_assign(&mut self, n: &LargeInteger) {
        let m = usize::max(self.sig, n.sig);
        self.expand(m);
        for i in (0..=usize::min(self.sig, n.sig)).rev() {
            self.number[i] ^= n.number[i];
        }
        self.contract();
    }
}

impl BitXorAssign<LargeInteger> for LargeInteger {
    fn bitxor_assign(&mut self, rhs: LargeInteger) {
        *self ^= &rhs;
    }
}

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident) => {
        impl $trait<&LargeInteger> for LargeInteger {
            type Output = LargeInteger;

            fn $method(mut self, rhs: &LargeInteger) -> Self::Output {
                self.$assign_method(rhs);
                self
            }
        }

        impl $trait<LargeInteger> for LargeInteger {
            type Output = LargeInteger;

            fn $method(mut self, rhs: LargeInteger) -> Self::Output {
                self.$assign_method(&rhs);
                self
            }
        }

        impl $trait<&LargeInteger> for &LargeInteger {
            type Output = LargeInteger;

            fn $method(self, rhs: &LargeInteger) -> Self::Output {
                let mut c = self.clone();
                c.$assign_method(rhs);
                c
            }
        }

        impl $trait<LargeInteger> for &LargeInteger {
            type Output = LargeInteger;

            fn $method(self, rhs: LargeInteger) -> Self::Output {
                let mut c = self.clone();
                c.$assign_method(&rhs);
                c
            }
        }
    };
}

impl_binary_op!(Add, add, AddAssign, add_assign);
impl_binary_op!(Sub, sub, SubAssign, sub_assign);
impl_binary_op!(Mul, mul, MulAssign, mul_assign);
impl_binary_op!(Div, div, DivAssign, div_assign);
impl_binary_op!(Rem, rem, RemAssign, rem_assign);
impl_binary_op!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_binary_op!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_binary_op!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl Shl<i32> for LargeInteger {
    type Output = LargeInteger;

    fn shl(mut self, rhs: i32) -> Self::Output {
        self <<= rhs;
        self
    }
}

impl Shl<i32> for &LargeInteger {
    type Output = LargeInteger;

    fn shl(self, rhs: i32) -> Self::Output {
        let mut c = self.clone();
        c <<= rhs;
        c
    }
}

impl Shr<i32> for LargeInteger {
    type Output = LargeInteger;

    fn shr(mut self, rhs: i32) -> Self::Output {
        self >>= rhs;
        self
    }
}

impl Shr<i32> for &LargeInteger {
    type Output = LargeInteger;

    fn shr(self, rhs: i32) -> Self::Output {
        let mut c = self.clone();
        c >>= rhs;
        c
    }
}

impl fmt::Display for LargeInteger {
    fn fmt(&self, s: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negative {
            s.write_str("-")?;
        }
        for i in (0..=self.sig).rev() {
            s.write_str(if self.number[i] == 0 { "0" } else { "1" })?;
        }
        Ok(())
    }
}

impl FromStr for LargeInteger {
    type Err = std::convert::Infallible;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut chars = input
            .chars()
            .skip_while(|c| *c == ' ' || *c == '\n' || *c == '\r')
            .peekable();
        let mut n = LargeInteger::new();
        while matches!(chars.peek(), Some('-' | '+')) {
            if chars.next() == Some('-') {
                n.negative = !n.negative;
            }
        }
        let mut digits = Vec::new();
        while matches!(chars.peek(), Some('0' | '1')) {
            digits.push((chars.next() == Some('1')) as u8);
        }
        if !digits.is_empty() {
            n.expand(digits.len() - 1);
            n.sig = digits.len() - 1;
            for (i, digit) in digits.into_iter().rev().enumerate() {
                n.number[i] = digit;
            }
            n.contract();
            if n.is_zero_bool() {
                n.negative = false;
            }
        }
        Ok(n)
    }
}
