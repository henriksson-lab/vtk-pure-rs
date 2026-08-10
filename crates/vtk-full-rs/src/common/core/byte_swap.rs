use std::{io, mem, slice};

/// VTK `vtkByteSwap`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ByteSwap;

pub trait ByteSwapValue: Copy {}

macro_rules! impl_byte_swap_value {
    ($($ty:ty),* $(,)?) => {
        $(impl ByteSwapValue for $ty {})*
    };
}

impl_byte_swap_value!(f32, f64, i8, u8, i16, u16, i32, u32, i64, u64);

impl ByteSwap {
    /// VTK: `vtkByteSwap::New`.
    pub fn new() -> Self {
        Self
    }

    /// VTK: `vtkByteSwap::SwapLE`.
    pub fn swap_le<T: ByteSwapValue>(value: &mut T) {
        if cfg!(target_endian = "big") {
            reverse_value_bytes(value);
        }
    }

    /// VTK: `vtkByteSwap::SwapBE`.
    pub fn swap_be<T: ByteSwapValue>(value: &mut T) {
        if cfg!(target_endian = "little") {
            reverse_value_bytes(value);
        }
    }

    /// VTK: `vtkByteSwap::SwapLERange`.
    pub fn swap_le_range<T: ByteSwapValue>(values: &mut [T]) {
        if cfg!(target_endian = "big") {
            swap_range(values);
        }
    }

    /// VTK: `vtkByteSwap::SwapBERange`.
    pub fn swap_be_range<T: ByteSwapValue>(values: &mut [T]) {
        if cfg!(target_endian = "little") {
            swap_range(values);
        }
    }

    /// VTK: `vtkByteSwap::SwapLERangeWrite`.
    pub fn swap_le_range_write<T: ByteSwapValue, W: io::Write>(
        values: &[T],
        writer: &mut W,
    ) -> io::Result<()> {
        if cfg!(target_endian = "big") {
            swap_range_write(values, writer)
        } else {
            write_native_range(values, writer)
        }
    }

    /// VTK: `vtkByteSwap::SwapBERangeWrite`.
    pub fn swap_be_range_write<T: ByteSwapValue, W: io::Write>(
        values: &[T],
        writer: &mut W,
    ) -> io::Result<()> {
        if cfg!(target_endian = "little") {
            swap_range_write(values, writer)
        } else {
            write_native_range(values, writer)
        }
    }

    /// VTK: `vtkByteSwap::Swap2LE`.
    pub fn swap2_le(bytes: &mut [u8]) {
        Self::swap_n_le(bytes, 2);
    }

    /// VTK: `vtkByteSwap::Swap4LE`.
    pub fn swap4_le(bytes: &mut [u8]) {
        Self::swap_n_le(bytes, 4);
    }

    /// VTK: `vtkByteSwap::Swap8LE`.
    pub fn swap8_le(bytes: &mut [u8]) {
        Self::swap_n_le(bytes, 8);
    }

    /// VTK: `vtkByteSwap::Swap2BE`.
    pub fn swap2_be(bytes: &mut [u8]) {
        Self::swap_n_be(bytes, 2);
    }

    /// VTK: `vtkByteSwap::Swap4BE`.
    pub fn swap4_be(bytes: &mut [u8]) {
        Self::swap_n_be(bytes, 4);
    }

    /// VTK: `vtkByteSwap::Swap8BE`.
    pub fn swap8_be(bytes: &mut [u8]) {
        Self::swap_n_be(bytes, 8);
    }

    /// VTK: `vtkByteSwap::Swap2LERange`.
    pub fn swap2_le_range(bytes: &mut [u8], num: usize) {
        Self::swap_n_le_range(bytes, num, 2);
    }

    /// VTK: `vtkByteSwap::Swap4LERange`.
    pub fn swap4_le_range(bytes: &mut [u8], num: usize) {
        Self::swap_n_le_range(bytes, num, 4);
    }

    /// VTK: `vtkByteSwap::Swap8LERange`.
    pub fn swap8_le_range(bytes: &mut [u8], num: usize) {
        Self::swap_n_le_range(bytes, num, 8);
    }

    /// VTK: `vtkByteSwap::Swap2BERange`.
    pub fn swap2_be_range(bytes: &mut [u8], num: usize) {
        Self::swap_n_be_range(bytes, num, 2);
    }

    /// VTK: `vtkByteSwap::Swap4BERange`.
    pub fn swap4_be_range(bytes: &mut [u8], num: usize) {
        Self::swap_n_be_range(bytes, num, 4);
    }

    /// VTK: `vtkByteSwap::Swap8BERange`.
    pub fn swap8_be_range(bytes: &mut [u8], num: usize) {
        Self::swap_n_be_range(bytes, num, 8);
    }

    /// VTK: `vtkByteSwap::SwapWrite2LERange`.
    pub fn swap_write2_le_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        Self::swap_write_n_le_range(bytes, num, 2, writer)
    }

    /// VTK: `vtkByteSwap::SwapWrite4LERange`.
    pub fn swap_write4_le_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        Self::swap_write_n_le_range(bytes, num, 4, writer)
    }

    /// VTK: `vtkByteSwap::SwapWrite8LERange`.
    pub fn swap_write8_le_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        Self::swap_write_n_le_range(bytes, num, 8, writer)
    }

    /// VTK: `vtkByteSwap::SwapWrite2BERange`.
    pub fn swap_write2_be_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        Self::swap_write_n_be_range(bytes, num, 2, writer)
    }

    /// VTK: `vtkByteSwap::SwapWrite4BERange`.
    pub fn swap_write4_be_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        Self::swap_write_n_be_range(bytes, num, 4, writer)
    }

    /// VTK: `vtkByteSwap::SwapWrite8BERange`.
    pub fn swap_write8_be_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        Self::swap_write_n_be_range(bytes, num, 8, writer)
    }

    /// VTK: `vtkByteSwap::SwapVoidRange`.
    pub fn swap_void_range(buffer: &mut [u8], num_words: usize, word_size: usize) {
        assert!(word_size > 0, "VTK byte-swap word size must be positive");
        let byte_count = num_words
            .checked_mul(word_size)
            .expect("VTK byte-swap byte count overflow");
        assert!(
            buffer.len() >= byte_count,
            "VTK byte-swap range exceeds buffer length"
        );

        for word in buffer[..byte_count].chunks_exact_mut(word_size) {
            word.reverse();
        }
    }

    fn swap_n_le(bytes: &mut [u8], word_size: usize) {
        if cfg!(target_endian = "big") {
            swap_first_word(bytes, word_size);
        }
    }

    fn swap_n_be(bytes: &mut [u8], word_size: usize) {
        if cfg!(target_endian = "little") {
            swap_first_word(bytes, word_size);
        }
    }

    fn swap_n_le_range(bytes: &mut [u8], num: usize, word_size: usize) {
        if cfg!(target_endian = "big") {
            swap_n_range(bytes, num, word_size);
        }
    }

    fn swap_n_be_range(bytes: &mut [u8], num: usize, word_size: usize) {
        if cfg!(target_endian = "little") {
            swap_n_range(bytes, num, word_size);
        }
    }

    fn swap_write_n_le_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        word_size: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        if cfg!(target_endian = "big") {
            swap_n_range_write(bytes, num, word_size, writer)
        } else {
            write_n_range(bytes, num, word_size, writer)
        }
    }

    fn swap_write_n_be_range<W: io::Write>(
        bytes: &[u8],
        num: usize,
        word_size: usize,
        writer: &mut W,
    ) -> io::Result<()> {
        if cfg!(target_endian = "little") {
            swap_n_range_write(bytes, num, word_size, writer)
        } else {
            write_n_range(bytes, num, word_size, writer)
        }
    }
}

fn swap_range<T: ByteSwapValue>(values: &mut [T]) {
    for value in values {
        reverse_value_bytes(value);
    }
}

fn reverse_value_bytes<T: ByteSwapValue>(value: &mut T) {
    value_bytes_mut(value).reverse();
}

fn swap_first_word(bytes: &mut [u8], word_size: usize) {
    assert!(
        bytes.len() >= word_size,
        "VTK byte-swap word exceeds buffer length"
    );
    bytes[..word_size].reverse();
}

fn swap_n_range(bytes: &mut [u8], num: usize, word_size: usize) {
    let byte_count = byte_count(num, word_size);
    assert!(
        bytes.len() >= byte_count,
        "VTK byte-swap range exceeds buffer length"
    );
    for word in bytes[..byte_count].chunks_exact_mut(word_size) {
        word.reverse();
    }
}

fn write_n_range<W: io::Write>(
    bytes: &[u8],
    num: usize,
    word_size: usize,
    writer: &mut W,
) -> io::Result<()> {
    let byte_count = byte_count(num, word_size);
    assert!(
        bytes.len() >= byte_count,
        "VTK byte-swap write range exceeds buffer length"
    );
    writer.write_all(&bytes[..byte_count])
}

fn swap_n_range_write<W: io::Write>(
    bytes: &[u8],
    num: usize,
    word_size: usize,
    writer: &mut W,
) -> io::Result<()> {
    let byte_count = byte_count(num, word_size);
    assert!(
        bytes.len() >= byte_count,
        "VTK byte-swap write range exceeds buffer length"
    );
    for word in bytes[..byte_count].chunks_exact(word_size) {
        for byte in word.iter().rev() {
            writer.write_all(slice::from_ref(byte))?;
        }
    }
    Ok(())
}

fn write_native_range<T: ByteSwapValue, W: io::Write>(
    values: &[T],
    writer: &mut W,
) -> io::Result<()> {
    writer.write_all(values_as_bytes(values))
}

fn swap_range_write<T: ByteSwapValue, W: io::Write>(
    values: &[T],
    writer: &mut W,
) -> io::Result<()> {
    for value in values {
        for byte in value_bytes(value).iter().rev() {
            writer.write_all(slice::from_ref(byte))?;
        }
    }
    Ok(())
}

fn byte_count(num: usize, word_size: usize) -> usize {
    num.checked_mul(word_size)
        .expect("VTK byte-swap byte count overflow")
}

fn value_bytes<T: ByteSwapValue>(value: &T) -> &[u8] {
    unsafe { slice::from_raw_parts((value as *const T).cast::<u8>(), mem::size_of::<T>()) }
}

fn value_bytes_mut<T: ByteSwapValue>(value: &mut T) -> &mut [u8] {
    unsafe { slice::from_raw_parts_mut((value as *mut T).cast::<u8>(), mem::size_of::<T>()) }
}

fn values_as_bytes<T: ByteSwapValue>(values: &[T]) -> &[u8] {
    unsafe { slice::from_raw_parts(values.as_ptr().cast::<u8>(), mem::size_of_val(values)) }
}
