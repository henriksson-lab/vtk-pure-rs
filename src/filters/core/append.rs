use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;
use rayon::prelude::*;
use std::ptr;

const APPEND_PAR_MIN_VALUES: usize = 1_000_000;

fn uninit_scalar_vec<T: Scalar>(len: usize) -> Vec<T> {
    let mut data = Vec::with_capacity(len);
    unsafe {
        data.set_len(len);
    }
    data
}

#[derive(Clone, Copy)]
struct UnsafeSliceWriter<T> {
    ptr: *mut T,
}

unsafe impl<T: Send> Send for UnsafeSliceWriter<T> {}
unsafe impl<T: Sync> Sync for UnsafeSliceWriter<T> {}

impl<T> UnsafeSliceWriter<T> {
    fn new(slice: &mut [T]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
        }
    }

    #[inline(always)]
    unsafe fn copy_from_slice(&self, dst_start: usize, src: &[T])
    where
        T: Copy,
    {
        ptr::copy_nonoverlapping(src.as_ptr(), self.ptr.add(dst_start), src.len());
    }

    #[inline(always)]
    unsafe fn write(&self, idx: usize, value: T) {
        *self.ptr.add(idx) = value;
    }
}

#[derive(Clone, Copy)]
struct InputRange<'a> {
    input: &'a PolyData,
    point_start: usize,
}

#[derive(Clone, Copy)]
struct CellRange<'a> {
    cells: &'a CellArray,
    point_start: i64,
    cell_start: usize,
    conn_start: usize,
}

/// Merge multiple PolyData into one. Bulk memcpy via flat slices for speed.
pub fn append(inputs: &[&PolyData]) -> PolyData {
    if inputs.is_empty() {
        return PolyData::new();
    }
    if inputs.len() == 1 {
        return inputs[0].clone();
    }

    let inputs: Vec<&PolyData> = inputs
        .iter()
        .copied()
        .filter(|input| !input.points.is_empty())
        .collect();
    if inputs.is_empty() {
        return PolyData::new();
    }
    if inputs.len() == 1 {
        return inputs[0].clone();
    }

    let mut input_ranges = Vec::with_capacity(inputs.len());
    let mut total_pts = 0usize;
    for &input in &inputs {
        input_ranges.push(InputRange {
            input,
            point_start: total_pts,
        });
        total_pts += input.points.len();
    }

    let mut pts_flat = uninit_scalar_vec(total_pts * 3);
    copy_points(&input_ranges, &mut pts_flat);

    let polys = merge_cells(&inputs, |p| &p.polys);
    let lines = merge_cells(&inputs, |p| &p.lines);
    let verts = merge_cells(&inputs, |p| &p.verts);
    let strips = merge_cells(&inputs, |p| &p.strips);

    let mut output = PolyData::new();
    output.points = Points::from_flat_vec(pts_flat);
    output.polys = polys;
    output.lines = lines;
    output.verts = verts;
    output.strips = strips;

    append_point_data(output.point_data_mut(), &inputs);
    append_cell_data(output.cell_data_mut(), &inputs);
    output
}

fn copy_points(ranges: &[InputRange<'_>], out: &mut [f64]) {
    if out.len() < APPEND_PAR_MIN_VALUES {
        for range in ranges {
            let src = range.input.points.as_flat_slice();
            let dst_start = range.point_start * 3;
            out[dst_start..dst_start + src.len()].copy_from_slice(src);
        }
        return;
    }

    let writer = UnsafeSliceWriter::new(out);
    ranges.par_iter().for_each(|range| {
        let src = range.input.points.as_flat_slice();
        unsafe {
            writer.copy_from_slice(range.point_start * 3, src);
        }
    });
}

fn merge_cells(inputs: &[&PolyData], get: impl Fn(&PolyData) -> &CellArray) -> CellArray {
    if inputs.iter().all(|input| get(input).is_empty()) {
        return CellArray::new();
    }

    let mut ranges = Vec::with_capacity(inputs.len());
    let mut total_cells = 0usize;
    let mut total_conn = 0usize;
    let mut point_start = 0i64;
    for &input in inputs {
        let cells = get(input);
        ranges.push(CellRange {
            cells,
            point_start,
            cell_start: total_cells,
            conn_start: total_conn,
        });
        total_cells += cells.num_cells();
        total_conn += cells.connectivity_len();
        point_start += input.points.len() as i64;
    }
    if total_cells == 0 {
        return CellArray::new();
    }

    let mut offsets = uninit_scalar_vec(total_cells + 1);
    let mut conn = uninit_scalar_vec(total_conn);
    offsets[0] = 0;
    copy_cell_ranges(&ranges, &mut offsets, &mut conn);
    CellArray::from_raw(offsets, conn)
}

fn copy_cell_ranges(ranges: &[CellRange<'_>], offsets: &mut [i64], conn: &mut [i64]) {
    if offsets.len() + conn.len() < APPEND_PAR_MIN_VALUES {
        for range in ranges {
            if range.cells.is_empty() {
                continue;
            }
            let src_offsets = range.cells.offsets();
            let src_conn = range.cells.connectivity();
            let conn_start = range.conn_start as i64;
            for (local_idx, &src_offset) in src_offsets[1..].iter().enumerate() {
                offsets[range.cell_start + local_idx + 1] = conn_start + src_offset;
            }
            if range.point_start == 0 {
                conn[range.conn_start..range.conn_start + src_conn.len()].copy_from_slice(src_conn);
            } else {
                for (local_idx, &point_id) in src_conn.iter().enumerate() {
                    conn[range.conn_start + local_idx] = point_id + range.point_start;
                }
            }
        }
        return;
    }

    let offsets_writer = UnsafeSliceWriter::new(offsets);
    let conn_writer = UnsafeSliceWriter::new(conn);
    ranges.par_iter().for_each(|range| {
        if range.cells.is_empty() {
            return;
        }
        let src_offsets = range.cells.offsets();
        let src_conn = range.cells.connectivity();
        let conn_start = range.conn_start as i64;
        unsafe {
            for (local_idx, &src_offset) in src_offsets[1..].iter().enumerate() {
                offsets_writer.write(range.cell_start + local_idx + 1, conn_start + src_offset);
            }
            if range.point_start == 0 {
                conn_writer.copy_from_slice(range.conn_start, src_conn);
            } else {
                for (local_idx, &point_id) in src_conn.iter().enumerate() {
                    conn_writer.write(range.conn_start + local_idx, point_id + range.point_start);
                }
            }
        }
    });
}

fn append_point_data(target: &mut DataSetAttributes, inputs: &[&PolyData]) {
    let Some(first) = inputs.first() else {
        return;
    };
    let first_attrs = first.point_data();
    for first_array in first_attrs.iter() {
        if first_array.num_tuples() != first.points.len() {
            continue;
        }
        let ranges: Vec<(usize, usize)> =
            inputs.iter().map(|input| (0, input.points.len())).collect();
        append_compatible_array(
            target,
            inputs,
            first_array,
            |input| input.point_data(),
            &ranges,
        );
    }
}

fn append_cell_data(target: &mut DataSetAttributes, inputs: &[&PolyData]) {
    let Some(first) = inputs.first() else {
        return;
    };
    let first_attrs = first.cell_data();
    for first_array in first_attrs.iter() {
        if first_array.num_tuples() != first.total_cells() {
            continue;
        }

        let cell_ranges = [
            |pd: &PolyData| (0, pd.verts.num_cells()),
            |pd: &PolyData| {
                let start = pd.verts.num_cells();
                (start, start + pd.lines.num_cells())
            },
            |pd: &PolyData| {
                let start = pd.verts.num_cells() + pd.lines.num_cells();
                (start, start + pd.polys.num_cells())
            },
            |pd: &PolyData| {
                let start = pd.verts.num_cells() + pd.lines.num_cells() + pd.polys.num_cells();
                (start, start + pd.strips.num_cells())
            },
        ];

        let mut ranges = Vec::with_capacity(inputs.len() * cell_ranges.len());
        for range_fn in cell_ranges {
            for input in inputs {
                ranges.push(range_fn(input));
            }
        }
        append_compatible_array(
            target,
            inputs,
            first_array,
            |input| input.cell_data(),
            &ranges,
        );
    }
}

fn append_compatible_array(
    target: &mut DataSetAttributes,
    inputs: &[&PolyData],
    first_array: &AnyDataArray,
    attrs: impl Fn(&PolyData) -> &DataSetAttributes,
    ranges: &[(usize, usize)],
) {
    let mut arrays = Vec::with_capacity(inputs.len());
    for input in inputs {
        let Some(array) = attrs(input).get_array(first_array.name()) else {
            return;
        };
        if array.scalar_type() != first_array.scalar_type()
            || array.num_components() != first_array.num_components()
        {
            return;
        }
        arrays.push(array);
    }

    if let Some(array) = concat_array_ranges(first_array, &arrays, ranges, inputs.len()) {
        let name = array.name().to_string();
        target.add_array(array);
        let first_attrs = attrs(inputs[0]);
        if first_attrs.scalars().map(|a| a.name()) == Some(name.as_str()) {
            target.set_active_scalars(&name);
        }
        if first_attrs.vectors().map(|a| a.name()) == Some(name.as_str()) {
            target.set_active_vectors(&name);
        }
        if first_attrs.normals().map(|a| a.name()) == Some(name.as_str()) {
            target.set_active_normals(&name);
        }
    }
}

fn concat_array_ranges(
    first: &AnyDataArray,
    arrays: &[&AnyDataArray],
    ranges: &[(usize, usize)],
    range_stride: usize,
) -> Option<AnyDataArray> {
    macro_rules! concat_variant {
        ($variant:ident, $ty:ty) => {{
            let AnyDataArray::$variant(first_array) = first else {
                unreachable!();
            };
            let nc = first_array.num_components();
            let total_tuples: usize = ranges
                .iter()
                .map(|&(start, end)| end.saturating_sub(start))
                .sum();
            let mut data = uninit_scalar_vec::<$ty>(total_tuples * nc);
            concat_ranges_into(&mut data, arrays, ranges, range_stride, nc, |array| {
                let AnyDataArray::$variant(array) = array else {
                    return None;
                };
                Some(array.as_slice())
            })?;
            Some(AnyDataArray::$variant(DataArray::from_vec(
                first_array.name(),
                data,
                nc,
            )))
        }};
    }
    match first {
        AnyDataArray::F32(_) => concat_variant!(F32, f32),
        AnyDataArray::F64(_) => concat_variant!(F64, f64),
        AnyDataArray::I8(_) => concat_variant!(I8, i8),
        AnyDataArray::I16(_) => concat_variant!(I16, i16),
        AnyDataArray::I32(_) => concat_variant!(I32, i32),
        AnyDataArray::I64(_) => concat_variant!(I64, i64),
        AnyDataArray::U8(_) => concat_variant!(U8, u8),
        AnyDataArray::U16(_) => concat_variant!(U16, u16),
        AnyDataArray::U32(_) => concat_variant!(U32, u32),
        AnyDataArray::U64(_) => concat_variant!(U64, u64),
    }
}

fn concat_ranges_into<T: Scalar>(
    out: &mut [T],
    arrays: &[&AnyDataArray],
    ranges: &[(usize, usize)],
    range_stride: usize,
    nc: usize,
    get_slice: impl Fn(&AnyDataArray) -> Option<&[T]> + Send + Sync,
) -> Option<()> {
    let mut tuple_offsets = Vec::with_capacity(ranges.len());
    let mut tuple_offset = 0usize;
    for (range_idx, &(start, end)) in ranges.iter().enumerate() {
        let array_idx = range_idx % range_stride;
        let array = get_slice(arrays[array_idx])?;
        if end > array.len() / nc {
            return None;
        }
        tuple_offsets.push(tuple_offset);
        tuple_offset += end.saturating_sub(start);
    }

    if out.len() < APPEND_PAR_MIN_VALUES {
        for (range_idx, &(start, end)) in ranges.iter().enumerate() {
            if start == end {
                continue;
            }
            let array_idx = range_idx % range_stride;
            let array = get_slice(arrays[array_idx])?;
            let src = &array[start * nc..end * nc];
            let dst_start = tuple_offsets[range_idx] * nc;
            out[dst_start..dst_start + src.len()].copy_from_slice(src);
        }
        return Some(());
    }

    let writer = UnsafeSliceWriter::new(out);
    ranges
        .par_iter()
        .enumerate()
        .for_each(|(range_idx, &(start, end))| {
            if start == end {
                return;
            }
            let array_idx = range_idx % range_stride;
            let array = get_slice(arrays[array_idx]).expect("array type checked before copy");
            let src = &array[start * nc..end * nc];
            unsafe {
                writer.copy_from_slice(tuple_offsets[range_idx] * nc, src);
            }
        });

    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn append_two() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = append(&[&a, &b]);
        assert_eq!(r.points.len(), 6);
        assert_eq!(r.polys.num_cells(), 2);
        assert_eq!(r.polys.cell(1), &[3, 4, 5]);
    }

    #[test]
    fn append_strips_and_point_data() {
        let mut a = PolyData::new();
        a.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]);
        a.strips.push_cell(&[0, 1, 2]);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0, 3.0],
                1,
            )));

        let mut b = PolyData::new();
        b.points = Points::from_vec(vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [3.0, 1.0, 0.0]]);
        b.strips.push_cell(&[0, 1, 2]);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![4.0, 5.0, 6.0],
                1,
            )));

        let r = append(&[&a, &b]);
        assert_eq!(r.strips.num_cells(), 2);
        assert_eq!(r.strips.cell(1), &[3, 4, 5]);
        assert_eq!(
            r.point_data().get_array("s").unwrap().to_f64_vec(),
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
        );
    }

    #[test]
    fn append_ignores_empty_inputs_when_multiple_inputs_are_present() {
        let empty = PolyData::new();
        let mut full = PolyData::new();
        full.points = Points::from_vec(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        full.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![1.0, 2.0],
                1,
            )));

        let r = append(&[&empty, &full]);
        assert_eq!(r.points.len(), 2);
        assert_eq!(
            r.point_data().get_array("s").unwrap().to_f64_vec(),
            vec![1.0, 2.0]
        );
    }
}
