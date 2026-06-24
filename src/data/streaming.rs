//! Streaming / chunked iteration over large datasets.
//!
//! Enables processing datasets that are too large to hold fully in memory
//! by yielding smaller pieces one at a time.

use crate::data::{AnyDataArray, CellArray, DataArray, ImageData, Points, PolyData};

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
        chunk.verts = copy_cells_in_point_range(&self.data.verts, start, end);
        chunk.lines = copy_cells_in_point_range(&self.data.lines, start, end);
        chunk.polys = copy_cells_in_point_range(&self.data.polys, start, end);
        chunk.strips = copy_cells_in_point_range(&self.data.strips, start, end);
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
    let mut merged = PolyData::new();
    for chunk in iter {
        let offset = merged.points.len() as i64;
        for i in 0..chunk.points.len() {
            merged.points.push(chunk.points.get(i));
        }
        for c in 0..chunk.verts.num_cells() {
            let cell = chunk.verts.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.verts.push_cell(&shifted);
        }
        for c in 0..chunk.polys.num_cells() {
            let cell = chunk.polys.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.polys.push_cell(&shifted);
        }
        for c in 0..chunk.lines.num_cells() {
            let cell = chunk.lines.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.lines.push_cell(&shifted);
        }
        for c in 0..chunk.strips.num_cells() {
            let cell = chunk.strips.cell(c);
            let shifted: Vec<i64> = cell.iter().map(|&id| id + offset).collect();
            merged.strips.push_cell(&shifted);
        }
    }
    merged
}

fn copy_cells_in_point_range(cells: &CellArray, start: usize, end: usize) -> CellArray {
    let mut out = CellArray::new();
    for c in 0..cells.num_cells() {
        let cell = cells.cell(c);
        if cell.iter().all(|&id| id >= start as i64 && id < end as i64) {
            let remapped: Vec<i64> = cell.iter().map(|&id| id - start as i64).collect();
            out.push_cell(&remapped);
        }
    }
    out
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
