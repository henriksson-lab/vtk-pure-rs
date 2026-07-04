//! Streaming / chunked iteration over large datasets.
//!
//! Enables processing datasets that are too large to hold fully in memory
//! by yielding smaller pieces one at a time.

use crate::data::{
    AnyDataArray, CellArray, DataArray, DataSetAttributes, ImageData, Points, PolyData,
};

/// A generic data stream wrapping an iterator of chunks.
pub struct DataStream<T> {
    inner: Box<dyn Iterator<Item = T>>,
}

impl<T> DataStream<T> {
    /// Create a stream from any iterator.
    pub fn new(iter: impl Iterator<Item = T> + 'static) -> Self {
        Self {
            inner: Box::new(iter),
        }
    }
}

impl<T> Iterator for DataStream<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.inner.next()
    }
}

/// Streams a `PolyData` in chunks of N points.
///
/// Each yielded chunk contains at most `chunk_size` points. Cells are not split
/// across chunks; cells whose point ids all fall inside the current point range
/// are copied and remapped to the chunk-local point ids.
pub struct StreamingPolyData {
    data: PolyData,
    chunk_size: usize,
    cursor: usize,
}

impl StreamingPolyData {
    pub fn new(data: PolyData, chunk_size: usize) -> Self {
        assert!(chunk_size > 0);
        Self {
            data,
            chunk_size,
            cursor: 0,
        }
    }
}

impl Iterator for StreamingPolyData {
    type Item = PolyData;

    fn next(&mut self) -> Option<PolyData> {
        let n = self.data.points.len();
        if self.cursor >= n {
            return None;
        }
        let start = self.cursor;
        let end = (start + self.chunk_size).min(n);
        self.cursor = end;

        let mut pts = Points::<f64>::new();
        for i in start..end {
            pts.push(self.data.points.get(i));
        }

        let mut chunk = PolyData::new();
        chunk.points = pts;
        let (verts, vert_cell_ids) = copy_cells_in_point_range(&self.data.verts, start, end);
        let (lines, line_cell_ids) = copy_cells_in_point_range(&self.data.lines, start, end);
        let (polys, poly_cell_ids) = copy_cells_in_point_range(&self.data.polys, start, end);
        let (strips, strip_cell_ids) = copy_cells_in_point_range(&self.data.strips, start, end);

        chunk.verts = verts;
        chunk.lines = lines;
        chunk.polys = polys;
        chunk.strips = strips;

        let point_ids: Vec<usize> = (start..end).collect();
        copy_attribute_tuples(self.data.point_data(), chunk.point_data_mut(), &point_ids);

        let mut cell_ids = Vec::with_capacity(
            vert_cell_ids.len() + line_cell_ids.len() + poly_cell_ids.len() + strip_cell_ids.len(),
        );
        let line_offset = self.data.verts.num_cells();
        let poly_offset = line_offset + self.data.lines.num_cells();
        let strip_offset = poly_offset + self.data.polys.num_cells();
        cell_ids.extend(vert_cell_ids);
        cell_ids.extend(line_cell_ids.into_iter().map(|id| id + line_offset));
        cell_ids.extend(poly_cell_ids.into_iter().map(|id| id + poly_offset));
        cell_ids.extend(strip_cell_ids.into_iter().map(|id| id + strip_offset));
        copy_attribute_tuples(self.data.cell_data(), chunk.cell_data_mut(), &cell_ids);
        Some(chunk)
    }
}

/// Streams an `ImageData` as 2-D slices along the Z axis.
///
/// Each yielded slice is an `ImageData` with `nz = 1`, preserving the
/// original spacing and origin (adjusted for the slice Z position).
pub struct StreamingImageData {
    data: ImageData,
    z_cursor: usize,
}

impl StreamingImageData {
    pub fn new(data: ImageData) -> Self {
        Self { data, z_cursor: 0 }
    }
}

impl Iterator for StreamingImageData {
    type Item = ImageData;

    fn next(&mut self) -> Option<ImageData> {
        let dims = self.data.dimensions();
        let nz = dims[2];
        if self.z_cursor >= nz {
            return None;
        }
        let k = self.z_cursor;
        self.z_cursor += 1;

        let nx = dims[0];
        let ny = dims[1];
        let spacing = self.data.spacing();
        let origin = self.data.origin();

        let mut slice = ImageData::with_dimensions(nx, ny, 1);
        slice.set_spacing(spacing);
        slice.set_origin([origin[0], origin[1], origin[2] + k as f64 * spacing[2]]);

        // Copy scalar data for this slice if present
        if let Some(scalars) = self.data.point_data().scalars() {
            let slice_size = nx * ny;
            let base = k * slice_size;
            let nc = scalars.num_components();
            let mut values = Vec::with_capacity(slice_size * nc);
            let mut buf = vec![0.0f64; nc];
            for idx in base..(base + slice_size) {
                scalars.tuple_as_f64(idx, &mut buf);
                values.extend_from_slice(&buf);
            }
            let arr = DataArray::<f64>::from_vec(scalars.name(), values, nc);
            let scalar_name = arr.name().to_string();
            slice.point_data_mut().add_array(AnyDataArray::F64(arr));
            slice.point_data_mut().set_active_scalars(&scalar_name);
        }

        Some(slice)
    }
}

/// Merge all chunks from a streaming iterator back into a single `PolyData`.
pub fn collect_stream(iter: impl Iterator<Item = PolyData>) -> PolyData {
    let chunks: Vec<PolyData> = iter.collect();
    let mut merged = PolyData::new();
    let mut point_counts = Vec::with_capacity(chunks.len());

    for chunk in &chunks {
        let offset = merged.points.len() as i64;
        point_counts.push(chunk.points.len());
        for i in 0..chunk.points.len() {
            merged.points.push(chunk.points.get(i));
        }
        for c in 0..chunk.verts.num_cells() {
            let cell = chunk.verts.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.verts.push_cell(&shifted);
        }
        for c in 0..chunk.lines.num_cells() {
            let cell = chunk.lines.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.lines.push_cell(&shifted);
        }
        for c in 0..chunk.polys.num_cells() {
            let cell = chunk.polys.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.polys.push_cell(&shifted);
        }
        for c in 0..chunk.strips.num_cells() {
            let cell = chunk.strips.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.strips.push_cell(&shifted);
        }
    }

    concatenate_attributes(
        chunks.iter().map(|chunk| chunk.point_data()),
        &point_counts,
        merged.point_data_mut(),
    );
    concatenate_poly_data_cell_attributes(&chunks, merged.cell_data_mut());
    merged
}

fn copy_cells_in_point_range(
    cells: &CellArray,
    start: usize,
    end: usize,
) -> (CellArray, Vec<usize>) {
    let mut out = CellArray::new();
    let mut copied_cell_ids = Vec::new();
    for c in 0..cells.num_cells() {
        let cell = cells.cell(c);
        if cell.iter().all(|&id| id >= start as i64 && id < end as i64) {
            let remapped: Vec<i64> = cell.iter().map(|&id| id - start as i64).collect();
            out.push_cell(&remapped);
            copied_cell_ids.push(c);
        }
    }
    (out, copied_cell_ids)
}

fn copy_attribute_tuples(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
) {
    for array in source.iter() {
        if tuple_ids
            .iter()
            .all(|&tuple_id| tuple_id < array.num_tuples())
        {
            target.add_array(subset_array(array, tuple_ids));
        }
    }
    copy_active_attributes(source, target);
}

fn concatenate_attributes<'a>(
    sources: impl Iterator<Item = &'a DataSetAttributes>,
    tuple_counts: &[usize],
    target: &mut DataSetAttributes,
) {
    let sources: Vec<&DataSetAttributes> = sources.collect();
    let Some(first) = sources.first() else {
        return;
    };

    for array in first.iter() {
        let name = array.name();
        if sources
            .iter()
            .zip(tuple_counts)
            .all(|(attrs, &tuple_count)| {
                attrs
                    .get_array(name)
                    .map(|candidate| {
                        candidate.scalar_type() == array.scalar_type()
                            && candidate.num_components() == array.num_components()
                            && candidate.num_tuples() == tuple_count
                    })
                    .unwrap_or(false)
            })
        {
            let arrays: Vec<&AnyDataArray> = sources
                .iter()
                .map(|attrs| attrs.get_array(name).unwrap())
                .collect();
            target.add_array(concat_arrays(&arrays));
        }
    }
    copy_active_attributes(first, target);
}

fn concatenate_poly_data_cell_attributes(chunks: &[PolyData], target: &mut DataSetAttributes) {
    let Some(first) = chunks.first() else {
        return;
    };

    for array in first.cell_data().iter() {
        let name = array.name();
        if chunks.iter().all(|chunk| {
            chunk
                .cell_data()
                .get_array(name)
                .map(|candidate| {
                    candidate.scalar_type() == array.scalar_type()
                        && candidate.num_components() == array.num_components()
                        && candidate.num_tuples() == chunk.total_cells()
                })
                .unwrap_or(false)
        }) {
            let arrays: Vec<&AnyDataArray> = chunks
                .iter()
                .map(|chunk| chunk.cell_data().get_array(name).unwrap())
                .collect();
            let cell_counts: Vec<[usize; 4]> = chunks
                .iter()
                .map(|chunk| {
                    [
                        chunk.verts.num_cells(),
                        chunk.lines.num_cells(),
                        chunk.polys.num_cells(),
                        chunk.strips.num_cells(),
                    ]
                })
                .collect();
            target.add_array(concat_poly_data_cell_array(&arrays, &cell_counts));
        }
    }
    copy_active_attributes(first.cell_data(), target);
}

fn copy_active_attributes(source: &DataSetAttributes, target: &mut DataSetAttributes) {
    if let Some(array) = source.scalars() {
        target.set_active_scalars(array.name());
    }
    if let Some(array) = source.vectors() {
        target.set_active_vectors(array.name());
    }
    if let Some(array) = source.normals() {
        target.set_active_normals(array.name());
    }
    if let Some(array) = source.tcoords() {
        target.set_active_tcoords(array.name());
    }
    if let Some(array) = source.tensors() {
        target.set_active_tensors(array.name());
    }
    if let Some(array) = source.global_ids() {
        target.set_active_global_ids(array.name());
    }
    if let Some(array) = source.pedigree_ids() {
        target.set_active_pedigree_ids(array.name());
    }
    if let Some(array) = source.edge_flags() {
        target.set_active_edge_flags(array.name());
    }
    if let Some(array) = source.tangents() {
        target.set_active_tangents(array.name());
    }
    if let Some(array) = source.rational_weights() {
        target.set_active_rational_weights(array.name());
    }
    if let Some(array) = source.higher_order_degrees() {
        target.set_active_higher_order_degrees(array.name());
    }
    if let Some(array) = source.process_ids() {
        target.set_active_process_ids(array.name());
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> AnyDataArray {
    macro_rules! subset {
        ($arr:expr, $variant:ident) => {{
            let nc = $arr.num_components();
            let mut values = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                values.extend_from_slice($arr.tuple(tuple_id));
            }
            AnyDataArray::$variant(DataArray::from_vec($arr.name(), values, nc))
        }};
    }
    match array {
        AnyDataArray::F32(arr) => subset!(arr, F32),
        AnyDataArray::F64(arr) => subset!(arr, F64),
        AnyDataArray::I8(arr) => subset!(arr, I8),
        AnyDataArray::I16(arr) => subset!(arr, I16),
        AnyDataArray::I32(arr) => subset!(arr, I32),
        AnyDataArray::I64(arr) => subset!(arr, I64),
        AnyDataArray::U8(arr) => subset!(arr, U8),
        AnyDataArray::U16(arr) => subset!(arr, U16),
        AnyDataArray::U32(arr) => subset!(arr, U32),
        AnyDataArray::U64(arr) => subset!(arr, U64),
    }
}

fn concat_arrays(arrays: &[&AnyDataArray]) -> AnyDataArray {
    macro_rules! concat {
        ($variant:ident, $arrays:expr) => {{
            let AnyDataArray::$variant(first) = $arrays[0] else {
                unreachable!();
            };
            let nc = first.num_components();
            let mut values = Vec::new();
            for array in $arrays {
                let AnyDataArray::$variant(arr) = array else {
                    unreachable!();
                };
                values.extend_from_slice(arr.as_slice());
            }
            AnyDataArray::$variant(DataArray::from_vec(first.name(), values, nc))
        }};
    }
    match arrays[0] {
        AnyDataArray::F32(_) => concat!(F32, arrays),
        AnyDataArray::F64(_) => concat!(F64, arrays),
        AnyDataArray::I8(_) => concat!(I8, arrays),
        AnyDataArray::I16(_) => concat!(I16, arrays),
        AnyDataArray::I32(_) => concat!(I32, arrays),
        AnyDataArray::I64(_) => concat!(I64, arrays),
        AnyDataArray::U8(_) => concat!(U8, arrays),
        AnyDataArray::U16(_) => concat!(U16, arrays),
        AnyDataArray::U32(_) => concat!(U32, arrays),
        AnyDataArray::U64(_) => concat!(U64, arrays),
    }
}

fn concat_poly_data_cell_array(
    arrays: &[&AnyDataArray],
    cell_counts: &[[usize; 4]],
) -> AnyDataArray {
    macro_rules! concat_cells {
        ($variant:ident, $arrays:expr) => {{
            let AnyDataArray::$variant(first) = $arrays[0] else {
                unreachable!();
            };
            let nc = first.num_components();
            let total_tuples: usize = cell_counts
                .iter()
                .map(|counts| counts.iter().sum::<usize>())
                .sum();
            let mut values = Vec::with_capacity(total_tuples * nc);
            for family in 0..4 {
                for (array, counts) in $arrays.iter().zip(cell_counts) {
                    let AnyDataArray::$variant(arr) = array else {
                        unreachable!();
                    };
                    let start_tuple = counts[..family].iter().sum::<usize>();
                    let end_tuple = start_tuple + counts[family];
                    let start = start_tuple * nc;
                    let end = end_tuple * nc;
                    values.extend_from_slice(&arr.as_slice()[start..end]);
                }
            }
            AnyDataArray::$variant(DataArray::from_vec(first.name(), values, nc))
        }};
    }
    match arrays[0] {
        AnyDataArray::F32(_) => concat_cells!(F32, arrays),
        AnyDataArray::F64(_) => concat_cells!(F64, arrays),
        AnyDataArray::I8(_) => concat_cells!(I8, arrays),
        AnyDataArray::I16(_) => concat_cells!(I16, arrays),
        AnyDataArray::I32(_) => concat_cells!(I32, arrays),
        AnyDataArray::I64(_) => concat_cells!(I64, arrays),
        AnyDataArray::U8(_) => concat_cells!(U8, arrays),
        AnyDataArray::U16(_) => concat_cells!(U16, arrays),
        AnyDataArray::U32(_) => concat_cells!(U32, arrays),
        AnyDataArray::U64(_) => concat_cells!(U64, arrays),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_poly_data_chunks() {
        let points: Vec<[f64; 3]> = (0..10).map(|i| [i as f64, 0.0, 0.0]).collect();
        let pd = PolyData::from_points(points);
        let stream = StreamingPolyData::new(pd, 3);
        let chunks: Vec<PolyData> = stream.collect();
        assert_eq!(chunks.len(), 4); // 10/3 = 3.33 → 4 chunks
        assert_eq!(chunks[0].points.len(), 3);
        assert_eq!(chunks[3].points.len(), 1); // last chunk has 1 point
    }

    #[test]
    fn streaming_poly_data_preserves_whole_cells() {
        let mut pd = PolyData::from_points((0..6).map(|i| [i as f64, 0.0, 0.0]).collect());
        pd.verts.push_cell(&[0]);
        pd.lines.push_cell(&[1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.strips.push_cell(&[0, 1, 2]);

        let chunks: Vec<PolyData> = StreamingPolyData::new(pd, 3).collect();
        assert_eq!(chunks[0].verts.cell(0), &[0]);
        assert_eq!(chunks[0].lines.cell(0), &[1, 2]);
        assert_eq!(chunks[0].strips.cell(0), &[0, 1, 2]);
        assert_eq!(chunks[1].polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn collect_stream_preserves_all_cell_arrays() {
        let mut first = PolyData::from_points(vec![[0.0; 3], [1.0, 0.0, 0.0]]);
        first.lines.push_cell(&[0, 1]);
        let mut second = PolyData::from_points(vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
        second.strips.push_cell(&[0, 1]);

        let merged = collect_stream(vec![first, second].into_iter());
        assert_eq!(merged.lines.cell(0), &[0, 1]);
        assert_eq!(merged.strips.cell(0), &[2, 3]);
    }

    #[test]
    fn streaming_poly_data_preserves_attributes() {
        let mut pd = PolyData::from_points((0..4).map(|i| [i as f64, 0.0, 0.0]).collect());
        pd.lines.push_cell(&[0, 1]);
        pd.lines.push_cell(&[2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_ids",
                vec![10.0, 11.0, 12.0, 13.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("point_ids");
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_ids",
                vec![20, 21],
                1,
            )));

        let chunks: Vec<PolyData> = StreamingPolyData::new(pd, 2).collect();
        assert_eq!(chunks[0].point_data().scalars().unwrap().num_tuples(), 2);
        assert_eq!(
            chunks[1]
                .point_data()
                .get_array("point_ids")
                .unwrap()
                .statistics()
                .unwrap()
                .min,
            12.0
        );
        assert_eq!(
            chunks[1]
                .cell_data()
                .get_array("cell_ids")
                .unwrap()
                .num_tuples(),
            1
        );

        let merged = collect_stream(chunks.into_iter());
        assert_eq!(
            merged
                .point_data()
                .get_array("point_ids")
                .unwrap()
                .num_tuples(),
            4
        );
        assert_eq!(
            merged
                .cell_data()
                .get_array("cell_ids")
                .unwrap()
                .num_tuples(),
            2
        );
    }

    #[test]
    fn collect_stream_preserves_poly_data_cell_attribute_order() {
        let mut first = PolyData::from_points(vec![[0.0; 3], [1.0, 0.0, 0.0]]);
        first.verts.push_cell(&[0]);
        first.lines.push_cell(&[0, 1]);
        first
            .cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_ids",
                vec![10, 20],
                1,
            )));

        let mut second = PolyData::from_points(vec![[2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]);
        second.verts.push_cell(&[0]);
        second.lines.push_cell(&[0, 1]);
        second
            .cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cell_ids",
                vec![30, 40],
                1,
            )));

        let merged = collect_stream(vec![first, second].into_iter());
        let AnyDataArray::I32(arr) = merged.cell_data().get_array("cell_ids").unwrap() else {
            panic!("expected i32 cell_ids array");
        };

        assert_eq!(arr.as_slice(), &[10, 30, 20, 40]);
    }

    #[test]
    fn streaming_image_data_slices() {
        let img = ImageData::from_function(
            [4, 4, 3],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, y, z| x + y + z,
        );
        let stream = StreamingImageData::new(img);
        let slices: Vec<ImageData> = stream.collect();
        assert_eq!(slices.len(), 3);
        for s in &slices {
            assert_eq!(s.dimensions(), [4, 4, 1]);
            assert!(s.point_data().scalars().is_some());
        }
    }
}
