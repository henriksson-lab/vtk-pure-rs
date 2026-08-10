use super::PixelExtent;
use crate::common::core::{VtkChar, VtkDataType};

/// VTK: `vtkPixelTransfer`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PixelTransfer;

impl PixelTransfer {
    /// VTK: `vtkPixelTransfer::Blit(const vtkPixelExtent&, int, int, void*, int, void*)`.
    pub fn blit(
        ext: &PixelExtent,
        n_comps: i32,
        src_type: i32,
        src_data: &[u8],
        dest_type: i32,
        dest_data: &mut [u8],
    ) -> i32 {
        Self::blit_with_extents(
            ext, ext, ext, ext, n_comps, src_type, src_data, n_comps, dest_type, dest_data,
        )
    }

    /// VTK: `vtkPixelTransfer::Blit(const vtkPixelExtent&, const vtkPixelExtent&, const vtkPixelExtent&, const vtkPixelExtent&, int, int, void*, int, int, void*)`.
    pub fn blit_with_extents(
        src_whole: &PixelExtent,
        src_subset: &PixelExtent,
        dest_whole: &PixelExtent,
        dest_subset: &PixelExtent,
        n_src_comps: i32,
        src_type: i32,
        src_data: &[u8],
        n_dest_comps: i32,
        dest_type: i32,
        dest_data: &mut [u8],
    ) -> i32 {
        let Some(src_type) = VtkDataType::from_id(src_type) else {
            return 0;
        };
        let Some(dest_type) = VtkDataType::from_id(dest_type) else {
            return 0;
        };
        if !src_type.is_numeric() || !dest_type.is_numeric() || n_src_comps < 0 || n_dest_comps < 0
        {
            return 0;
        }

        blit_bytes(
            src_whole,
            src_subset,
            dest_whole,
            dest_subset,
            n_src_comps as usize,
            src_type,
            src_data,
            n_dest_comps as usize,
            dest_type,
            dest_data,
        )
    }

    /// VTK: `vtkPixelTransfer::Blit<SOURCE_TYPE, DEST_TYPE>`.
    pub fn blit_typed<SourceType, DestType>(
        src_whole: &PixelExtent,
        src_subset: &PixelExtent,
        dest_whole: &PixelExtent,
        dest_subset: &PixelExtent,
        n_src_comps: i32,
        src_data: &[SourceType],
        n_dest_comps: i32,
        dest_data: &mut [DestType],
    ) -> i32
    where
        SourceType: PixelScalar,
        DestType: PixelScalar,
    {
        if n_src_comps < 0 || n_dest_comps < 0 {
            return -1;
        }
        blit_slices(
            src_whole,
            src_subset,
            dest_whole,
            dest_subset,
            n_src_comps as usize,
            src_data,
            n_dest_comps as usize,
            dest_data,
        )
    }
}

pub trait PixelScalar: Copy {
    fn zero() -> Self;
    fn to_f64(self) -> f64;
    fn from_f64(value: f64) -> Self;
}

macro_rules! impl_pixel_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl PixelScalar for $ty {
                fn zero() -> Self {
                    0 as $ty
                }

                fn to_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(value: f64) -> Self {
                    value as $ty
                }
            }
        )*
    };
}

impl PixelScalar for bool {
    fn zero() -> Self {
        false
    }

    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    fn from_f64(value: f64) -> Self {
        value != 0.0
    }
}

impl_pixel_scalar!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

fn blit_slices<SourceType, DestType>(
    src_whole: &PixelExtent,
    src_subset: &PixelExtent,
    dest_whole: &PixelExtent,
    dest_subset: &PixelExtent,
    n_src_comps: usize,
    src_data: &[SourceType],
    n_dest_comps: usize,
    dest_data: &mut [DestType],
) -> i32
where
    SourceType: PixelScalar,
    DestType: PixelScalar,
{
    if !has_required_typed_len(src_whole, n_src_comps, src_data.len())
        || !has_required_typed_len(dest_whole, n_dest_comps, dest_data.len())
    {
        return -1;
    }

    if src_whole == src_subset && dest_whole == dest_subset && n_src_comps == n_dest_comps {
        let n = src_whole.number_of_pixels() * n_src_comps;
        for i in 0..n {
            dest_data[i] = DestType::from_f64(src_data[i].to_f64());
        }
    } else {
        let src_whole_size = src_whole.size();
        let swnx = src_whole_size[0];
        let dest_whole_size = dest_whole.size();
        let dwnx = dest_whole_size[0];

        let mut src_ext = *src_subset;
        src_ext.shift_by_extent(src_whole);
        let mut dest_ext = *dest_subset;
        dest_ext.shift_by_extent(dest_whole);

        let nxny = src_ext.size();
        let n_copy_comps = n_src_comps.min(n_dest_comps);

        for j in 0..nxny[1] {
            let sjj = swnx * (src_ext[2] + j) + src_ext[0];
            let djj = dwnx * (dest_ext[2] + j) + dest_ext[0];
            for i in 0..nxny[0] {
                let sidx = n_src_comps * (sjj + i) as usize;
                let didx = n_dest_comps * (djj + i) as usize;
                for p in 0..n_copy_comps {
                    dest_data[didx + p] = DestType::from_f64(src_data[sidx + p].to_f64());
                }
                for p in n_copy_comps..n_dest_comps {
                    dest_data[didx + p] = DestType::zero();
                }
            }
        }
    }
    0
}

#[allow(clippy::too_many_arguments)]
fn blit_bytes(
    src_whole: &PixelExtent,
    src_subset: &PixelExtent,
    dest_whole: &PixelExtent,
    dest_subset: &PixelExtent,
    n_src_comps: usize,
    src_type: VtkDataType,
    src_data: &[u8],
    n_dest_comps: usize,
    dest_type: VtkDataType,
    dest_data: &mut [u8],
) -> i32 {
    if !has_required_byte_len(src_whole, n_src_comps, src_type, src_data.len())
        || !has_required_byte_len(dest_whole, n_dest_comps, dest_type, dest_data.len())
    {
        return -1;
    }

    if src_whole == src_subset && dest_whole == dest_subset && n_src_comps == n_dest_comps {
        let n = src_whole.number_of_pixels() * n_src_comps;
        for i in 0..n {
            let value = read_scalar(src_data, i, src_type);
            write_scalar(dest_data, i, dest_type, value);
        }
    } else {
        let src_whole_size = src_whole.size();
        let swnx = src_whole_size[0];
        let dest_whole_size = dest_whole.size();
        let dwnx = dest_whole_size[0];

        let mut src_ext = *src_subset;
        src_ext.shift_by_extent(src_whole);
        let mut dest_ext = *dest_subset;
        dest_ext.shift_by_extent(dest_whole);

        let nxny = src_ext.size();
        let n_copy_comps = n_src_comps.min(n_dest_comps);

        for j in 0..nxny[1] {
            let sjj = swnx * (src_ext[2] + j) + src_ext[0];
            let djj = dwnx * (dest_ext[2] + j) + dest_ext[0];
            for i in 0..nxny[0] {
                let sidx = n_src_comps * (sjj + i) as usize;
                let didx = n_dest_comps * (djj + i) as usize;
                for p in 0..n_copy_comps {
                    let value = read_scalar(src_data, sidx + p, src_type);
                    write_scalar(dest_data, didx + p, dest_type, value);
                }
                for p in n_copy_comps..n_dest_comps {
                    write_scalar(dest_data, didx + p, dest_type, 0.0);
                }
            }
        }
    }
    0
}

fn has_required_typed_len(ext: &PixelExtent, n_comps: usize, len: usize) -> bool {
    len >= ext.number_of_pixels().saturating_mul(n_comps)
}

fn has_required_byte_len(
    ext: &PixelExtent,
    n_comps: usize,
    data_type: VtkDataType,
    len: usize,
) -> bool {
    len >= ext
        .number_of_pixels()
        .saturating_mul(n_comps)
        .saturating_mul(data_type.size())
}

fn read_scalar(data: &[u8], index: usize, data_type: VtkDataType) -> f64 {
    let offset = index * data_type.size();
    match data_type {
        VtkDataType::Bit => f64::from(data[offset] != 0),
        VtkDataType::Char => VtkChar::from_ne_bytes([data[offset]]) as f64,
        VtkDataType::UnsignedChar => data[offset] as f64,
        VtkDataType::Short => i16::from_ne_bytes([data[offset], data[offset + 1]]) as f64,
        VtkDataType::UnsignedShort => u16::from_ne_bytes([data[offset], data[offset + 1]]) as f64,
        VtkDataType::Int => i32::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as f64,
        VtkDataType::UnsignedInt => u32::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as f64,
        VtkDataType::Long | VtkDataType::IdType | VtkDataType::LongLong => i64::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as f64,
        VtkDataType::UnsignedLong | VtkDataType::UnsignedLongLong => u64::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as f64,
        VtkDataType::Float => f32::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as f64,
        VtkDataType::Double => f64::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]),
        VtkDataType::SignedChar => i8::from_ne_bytes([data[offset]]) as f64,
        _ => 0.0,
    }
}

fn write_scalar(data: &mut [u8], index: usize, data_type: VtkDataType, value: f64) {
    let offset = index * data_type.size();
    match data_type {
        VtkDataType::Bit => data[offset] = u8::from(value != 0.0),
        VtkDataType::Char => data[offset] = (value as VtkChar).to_ne_bytes()[0],
        VtkDataType::UnsignedChar => data[offset] = value as u8,
        VtkDataType::Short => {
            data[offset..offset + 2].copy_from_slice(&(value as i16).to_ne_bytes())
        }
        VtkDataType::UnsignedShort => {
            data[offset..offset + 2].copy_from_slice(&(value as u16).to_ne_bytes());
        }
        VtkDataType::Int => data[offset..offset + 4].copy_from_slice(&(value as i32).to_ne_bytes()),
        VtkDataType::UnsignedInt => {
            data[offset..offset + 4].copy_from_slice(&(value as u32).to_ne_bytes());
        }
        VtkDataType::Long | VtkDataType::IdType | VtkDataType::LongLong => {
            data[offset..offset + 8].copy_from_slice(&(value as i64).to_ne_bytes());
        }
        VtkDataType::UnsignedLong | VtkDataType::UnsignedLongLong => {
            data[offset..offset + 8].copy_from_slice(&(value as u64).to_ne_bytes());
        }
        VtkDataType::Float => {
            data[offset..offset + 4].copy_from_slice(&(value as f32).to_ne_bytes());
        }
        VtkDataType::Double => data[offset..offset + 8].copy_from_slice(&value.to_ne_bytes()),
        VtkDataType::SignedChar => data[offset] = (value as i8).to_ne_bytes()[0],
        _ => {}
    }
}
