//! Collective operations on distributed datasets (serial stubs).
//!
//! These functions work on local data. When the `mpi` feature is enabled,
//! `mpi_backend` provides actual distributed implementations.

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};

/// Gather PolyData from all partitions into one (serial: just merge).
pub fn gather_poly_data(partitions: &[PolyData]) -> PolyData {
    if partitions.is_empty() {
        return PolyData::new();
    }
    if partitions.len() == 1 {
        return partitions[0].clone();
    }

    let mut result = partitions[0].clone();
    for part in &partitions[1..] {
        let offset = result.points.len() as i64;
        for i in 0..part.points.len() {
            result.points.push(part.points.get(i));
        }
        append_cells_with_offset(&mut result.verts, &part.verts, offset);
        append_cells_with_offset(&mut result.lines, &part.lines, offset);
        append_cells_with_offset(&mut result.polys, &part.polys, offset);
        append_cells_with_offset(&mut result.strips, &part.strips, offset);
    }
    gather_attributes(
        partitions,
        result.point_data_mut(),
        |pd| pd.point_data(),
        |pd| pd.points.len(),
    );
    gather_cell_attributes(partitions, result.cell_data_mut());
    result
}

/// Reduce a scalar across all partitions (serial: just compute from local data).
#[derive(Debug, Clone, Copy)]
pub enum ReduceOp {
    Sum,
    Min,
    Max,
    Mean,
}

/// Reduce a scalar value across partitions.
pub fn reduce_scalar(values: &[f64], op: ReduceOp) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    match op {
        ReduceOp::Sum => values.iter().sum(),
        ReduceOp::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        ReduceOp::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        ReduceOp::Mean => values.iter().sum::<f64>() / values.len() as f64,
    }
}

/// Broadcast a PolyData from rank 0 to all (serial: identity).
pub fn broadcast_poly_data(data: &PolyData) -> PolyData {
    data.clone()
}

/// Scatter: distribute parts of a PolyData to N ranks (serial: decompose by cell index).
pub fn scatter_poly_data(data: &PolyData, num_ranks: usize) -> Vec<PolyData> {
    if num_ranks == 0 {
        return Vec::new();
    }
    let nc = data.total_cells();
    let per_rank = (nc + num_ranks - 1) / num_ranks;
    let mut parts = Vec::new();

    for r in 0..num_ranks {
        let start = r * per_rank;
        let end = ((r + 1) * per_rank).min(nc);
        if start >= nc {
            break;
        }

        let mut point_map = vec![i64::MAX; data.points.len()];
        let mut point_ids = Vec::new();
        let mut pts = crate::data::Points::<f64>::new();
        let mut pd = PolyData::new();
        let mut flat_cell_id = 0;
        append_selected_cells(
            data,
            &data.verts,
            &mut pd.verts,
            start,
            end,
            &mut flat_cell_id,
            &mut point_map,
            &mut point_ids,
            &mut pts,
        );
        append_selected_cells(
            data,
            &data.lines,
            &mut pd.lines,
            start,
            end,
            &mut flat_cell_id,
            &mut point_map,
            &mut point_ids,
            &mut pts,
        );
        append_selected_cells(
            data,
            &data.polys,
            &mut pd.polys,
            start,
            end,
            &mut flat_cell_id,
            &mut point_map,
            &mut point_ids,
            &mut pts,
        );
        append_selected_cells(
            data,
            &data.strips,
            &mut pd.strips,
            start,
            end,
            &mut flat_cell_id,
            &mut point_map,
            &mut point_ids,
            &mut pts,
        );
        pd.points = pts;
        copy_selected_attributes(data.point_data(), pd.point_data_mut(), &point_ids);
        copy_selected_attributes(
            data.cell_data(),
            pd.cell_data_mut(),
            &(start..end).collect::<Vec<_>>(),
        );
        parts.push(pd);
    }

    parts
}

fn append_selected_cells(
    source: &PolyData,
    source_cells: &CellArray,
    target_cells: &mut CellArray,
    start: usize,
    end: usize,
    flat_cell_id: &mut usize,
    point_map: &mut [i64],
    point_ids: &mut Vec<usize>,
    points: &mut crate::data::Points<f64>,
) {
    for cell in source_cells {
        if *flat_cell_id >= start && *flat_cell_id < end {
            for &vid in cell {
                let vi = vid as usize;
                if point_map[vi] == i64::MAX {
                    point_map[vi] = points.len() as i64;
                    point_ids.push(vi);
                    points.push(source.points.get(vi));
                }
            }
            let remapped: Vec<i64> = cell.iter().map(|&v| point_map[v as usize]).collect();
            target_cells.push_cell(&remapped);
        }
        *flat_cell_id += 1;
    }
}

fn append_cells_with_offset(dst: &mut CellArray, src: &CellArray, offset: i64) {
    for cell in src {
        let remapped: Vec<i64> = cell.iter().map(|&v| v + offset).collect();
        dst.push_cell(&remapped);
    }
}

fn gather_attributes(
    partitions: &[PolyData],
    target: &mut DataSetAttributes,
    attrs: impl Fn(&PolyData) -> &DataSetAttributes,
    expected_tuples: impl Fn(&PolyData) -> usize,
) {
    let Some(first) = partitions.first() else {
        return;
    };
    let first_attrs = attrs(first);
    for first_array in first_attrs.iter() {
        if first_array.num_tuples() != expected_tuples(first) {
            continue;
        }
        let mut arrays = Vec::with_capacity(partitions.len());
        let mut compatible = true;
        for part in partitions {
            let Some(array) = attrs(part).get_array(first_array.name()) else {
                compatible = false;
                break;
            };
            if array.scalar_type() != first_array.scalar_type()
                || array.num_components() != first_array.num_components()
                || array.num_tuples() != expected_tuples(part)
            {
                compatible = false;
                break;
            }
            arrays.push(array);
        }
        if compatible {
            if let Some(array) = concat_arrays(&arrays) {
                let name = array.name().to_string();
                target.add_array(array);
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
    }
}

fn gather_cell_attributes(partitions: &[PolyData], target: &mut DataSetAttributes) {
    let Some(first) = partitions.first() else {
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

        let mut arrays = Vec::with_capacity(partitions.len());
        for part in partitions {
            let Some(array) = part.cell_data().get_array(first_array.name()) else {
                arrays.clear();
                break;
            };
            if array.scalar_type() != first_array.scalar_type()
                || array.num_components() != first_array.num_components()
                || array.num_tuples() != part.total_cells()
            {
                arrays.clear();
                break;
            }
            arrays.push(array);
        }
        if arrays.len() != partitions.len() {
            continue;
        }

        let mut ranges = Vec::with_capacity(partitions.len() * cell_ranges.len());
        for range_fn in cell_ranges {
            for part in partitions {
                ranges.push(range_fn(part));
            }
        }

        if let Some(array) = concat_array_ranges(first_array, &arrays, &ranges, partitions.len()) {
            let name = array.name().to_string();
            target.add_array(array);
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
}

fn concat_arrays(arrays: &[&AnyDataArray]) -> Option<AnyDataArray> {
    let first = *arrays.first()?;
    macro_rules! concat_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(first_array) = first else {
                unreachable!();
            };
            let mut data = Vec::new();
            for array in arrays {
                let AnyDataArray::$variant(array) = *array else {
                    return None;
                };
                data.extend_from_slice(array.as_slice());
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                first_array.name(),
                data,
                first_array.num_components(),
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

fn copy_selected_attributes(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if array.num_tuples() <= tuple_ids.iter().copied().max().unwrap_or(0) {
            continue;
        }
        if let Some(sliced) = slice_array(array, tuple_ids) {
            let name = sliced.name().to_string();
            target.add_array(sliced);
            if source.scalars().map(|a| a.name()) == Some(name.as_str()) {
                target.set_active_scalars(&name);
            }
            if source.vectors().map(|a| a.name()) == Some(name.as_str()) {
                target.set_active_vectors(&name);
            }
            if source.normals().map(|a| a.name()) == Some(name.as_str()) {
                target.set_active_normals(&name);
            }
        }
    }
}

fn slice_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! slice_variant {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                data.extend_from_slice($arr.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $arr.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(a) => slice_variant!(a, F32),
        AnyDataArray::F64(a) => slice_variant!(a, F64),
        AnyDataArray::I8(a) => slice_variant!(a, I8),
        AnyDataArray::I16(a) => slice_variant!(a, I16),
        AnyDataArray::I32(a) => slice_variant!(a, I32),
        AnyDataArray::I64(a) => slice_variant!(a, I64),
        AnyDataArray::U8(a) => slice_variant!(a, U8),
        AnyDataArray::U16(a) => slice_variant!(a, U16),
        AnyDataArray::U32(a) => slice_variant!(a, U32),
        AnyDataArray::U64(a) => slice_variant!(a, U64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_two() {
        let a = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let b = PolyData::from_triangles(
            vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let merged = gather_poly_data(&[a, b]);
        assert_eq!(merged.points.len(), 6);
        assert_eq!(merged.polys.num_cells(), 2);
    }

    #[test]
    fn reduce_ops() {
        assert_eq!(reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Sum), 6.0);
        assert_eq!(reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Min), 1.0);
        assert_eq!(reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Max), 3.0);
        assert!((reduce_scalar(&[1.0, 2.0, 3.0], ReduceOp::Mean) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn scatter_roundtrip() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let parts = scatter_poly_data(&pd, 2);
        assert_eq!(parts.len(), 2);
        let merged = gather_poly_data(&parts);
        assert_eq!(merged.polys.num_cells(), 2);
    }

    #[test]
    fn gather_mixed_cell_data_by_cell_type() {
        let mut a = PolyData::new();
        a.points.push([0.0, 0.0, 0.0]);
        a.points.push([1.0, 0.0, 0.0]);
        a.points.push([0.0, 1.0, 0.0]);
        a.verts.push_cell(&[0]);
        a.polys.push_cell(&[0, 1, 2]);
        a.strips.push_cell(&[0, 1, 2]);
        a.cell_data_mut()
            .add_array(DataArray::from_vec("id", vec![10i32, 20, 30], 1).into());

        let mut b = PolyData::new();
        b.points.push([2.0, 0.0, 0.0]);
        b.points.push([3.0, 0.0, 0.0]);
        b.points.push([2.0, 1.0, 0.0]);
        b.verts.push_cell(&[0]);
        b.polys.push_cell(&[0, 1, 2]);
        b.strips.push_cell(&[0, 1, 2]);
        b.cell_data_mut()
            .add_array(DataArray::from_vec("id", vec![11i32, 21, 31], 1).into());

        let merged = gather_poly_data(&[a, b]);
        let ids = merged.cell_data().get_array("id").unwrap();
        let AnyDataArray::I32(ids) = ids else {
            panic!("expected i32 cell data");
        };
        assert_eq!(ids.as_slice(), &[10, 11, 20, 21, 30, 31]);
    }
}
