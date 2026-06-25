use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};

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

    let total_pts: usize = inputs.iter().map(|p| p.points.len()).sum();

    // Bulk copy point data via extend_from_slice on flat backing buffer.
    // Avoids per-point get()/push() overhead (single memcpy per input mesh).
    let mut pts_flat = Vec::with_capacity(total_pts * 3);
    for &input in &inputs {
        pts_flat.extend_from_slice(input.points.as_flat_slice());
    }

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

fn merge_cells(inputs: &[&PolyData], get: impl Fn(&PolyData) -> &CellArray) -> CellArray {
    let total_cells: usize = inputs.iter().map(|p| get(p).num_cells()).sum();
    if total_cells == 0 {
        return CellArray::new();
    }

    let total_conn: usize = inputs.iter().map(|p| get(p).connectivity_len()).sum();
    let mut offsets = Vec::with_capacity(total_cells + 1);
    let mut conn = Vec::with_capacity(total_conn);
    offsets.push(0i64);

    let mut pt_off: i64 = 0;
    for &input in inputs {
        let cells = get(input);
        let src_conn = cells.connectivity();
        if pt_off == 0 {
            conn.extend_from_slice(src_conn);
        } else {
            // Bulk offset: extend then add offset in-place
            let start = conn.len();
            conn.extend_from_slice(src_conn);
            for v in &mut conn[start..] {
                *v += pt_off;
            }
        }
        let base = *offsets.last().unwrap();
        let src_off = cells.offsets();
        if base == 0 {
            offsets.extend_from_slice(&src_off[1..]);
        } else {
            let start = offsets.len();
            offsets.extend_from_slice(&src_off[1..]);
            for v in &mut offsets[start..] {
                *v += base;
            }
        }
        pt_off += input.points.len() as i64;
    }
    CellArray::from_raw(offsets, conn)
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
        ($variant:ident) => {{
            let AnyDataArray::$variant(first_array) = first else {
                unreachable!();
            };
            let nc = first_array.num_components();
            let mut data = Vec::new();
            for (range_idx, &(start, end)) in ranges.iter().enumerate() {
                if start == end {
                    continue;
                }
                let array_idx = range_idx % range_stride;
                let AnyDataArray::$variant(array) = arrays[array_idx] else {
                    return None;
                };
                if end > array.num_tuples() {
                    return None;
                }
                data.extend_from_slice(&array.as_slice()[start * nc..end * nc]);
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                first_array.name(),
                data,
                nc,
            )))
        }};
    }
    match first {
        AnyDataArray::F32(_) => concat_variant!(F32),
        AnyDataArray::F64(_) => concat_variant!(F64),
        AnyDataArray::I8(_) => concat_variant!(I8),
        AnyDataArray::I16(_) => concat_variant!(I16),
        AnyDataArray::I32(_) => concat_variant!(I32),
        AnyDataArray::I64(_) => concat_variant!(I64),
        AnyDataArray::U8(_) => concat_variant!(U8),
        AnyDataArray::U16(_) => concat_variant!(U16),
        AnyDataArray::U32(_) => concat_variant!(U32),
        AnyDataArray::U64(_) => concat_variant!(U64),
    }
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
