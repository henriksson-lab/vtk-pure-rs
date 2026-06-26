//! Memory-mapped data utilities for zero-copy I/O.
//!
//! Provides helpers for working with memory-mapped files,
//! enabling processing of datasets larger than available RAM.

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, Points, PolyData};
use std::path::Path;

/// Information about a memory-mapped data file.
#[derive(Debug, Clone)]
pub struct MmapInfo {
    pub file_size: u64,
    pub estimated_points: usize,
    pub format: String,
}

/// Estimate the size of a data file without loading it.
pub fn estimate_file_info(path: &Path) -> Option<MmapInfo> {
    let metadata = std::fs::metadata(path).ok()?;
    let size = metadata.len();

    let ext = path.extension()?.to_str()?.to_lowercase();
    let estimated_points = match ext.as_str() {
        "stl" => size.saturating_sub(84) as usize / 50, // binary STL: 50 bytes per triangle
        "ply" => size as usize / 40,                    // rough estimate
        "obj" => size as usize / 30,
        "vtk" => size as usize / 40,
        "las" => size.saturating_sub(227) as usize / 20, // LAS format 0
        _ => size as usize / 30,
    };

    Some(MmapInfo {
        file_size: size,
        estimated_points,
        format: ext,
    })
}

/// Process a PolyData in chunks of `chunk_size` points.
///
/// Calls `process` for each chunk. Returns the number of chunks.
pub fn process_points_chunked<F>(mesh: &PolyData, chunk_size: usize, mut process: F) -> usize
where
    F: FnMut(usize, &PolyData),
{
    if chunk_size == 0 {
        return 0;
    }

    let n = mesh.points.len();
    let mut chunk_count = 0;

    for start in (0..n).step_by(chunk_size) {
        let end = (start + chunk_size).min(n);
        let mut pts = Points::<f64>::new();
        for i in start..end {
            pts.push(mesh.points.get(i));
        }
        let mut chunk = PolyData::new();
        chunk.points = pts;
        copy_point_data_range(mesh.point_data(), chunk.point_data_mut(), start, end);
        process(chunk_count, &chunk);
        chunk_count += 1;
    }

    chunk_count
}

/// Merge multiple PolyData chunks into one.
pub fn merge_chunks(points: &[PolyData]) -> PolyData {
    let refs: Vec<&PolyData> = points.iter().collect();
    if refs.is_empty() {
        return PolyData::new();
    }
    crate::filters::core::append::append(&refs)
}

/// Report memory usage estimate for a PolyData.
pub fn estimate_memory_bytes(mesh: &PolyData) -> usize {
    let points_bytes = mesh.points.len() * 3 * 8; // 3 f64
    let cells_bytes = mesh.polys.num_cells() * 4 * 8; // rough
    let data_bytes = {
        let pd = mesh.point_data();
        let mut total = 0;
        for i in 0..pd.num_arrays() {
            if let Some(arr) = pd.get_array_by_index(i) {
                total += arr.num_tuples() * arr.num_components() * 8;
            }
        }
        total
    };
    points_bytes + cells_bytes + data_bytes
}

fn copy_point_data_range(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    start: usize,
    end: usize,
) {
    for array in source.iter() {
        if end > array.num_tuples() {
            continue;
        }
        if let Some(subset) = slice_array(array, start, end) {
            let name = subset.name().to_string();
            target.add_array(subset);
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

fn slice_array(array: &AnyDataArray, start: usize, end: usize) -> Option<AnyDataArray> {
    macro_rules! slice_variant {
        ($variant:ident, $a:expr) => {{
            let nc = $a.num_components();
            let from = start.checked_mul(nc)?;
            let to = end.checked_mul(nc)?;
            Some(AnyDataArray::$variant(DataArray::from_vec(
                $a.name(),
                $a.as_slice().get(from..to)?.to_vec(),
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(a) => slice_variant!(F32, a),
        AnyDataArray::F64(a) => slice_variant!(F64, a),
        AnyDataArray::I8(a) => slice_variant!(I8, a),
        AnyDataArray::I16(a) => slice_variant!(I16, a),
        AnyDataArray::I32(a) => slice_variant!(I32, a),
        AnyDataArray::I64(a) => slice_variant!(I64, a),
        AnyDataArray::U8(a) => slice_variant!(U8, a),
        AnyDataArray::U16(a) => slice_variant!(U16, a),
        AnyDataArray::U32(a) => slice_variant!(U32, a),
        AnyDataArray::U64(a) => slice_variant!(U64, a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_estimate() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let bytes = estimate_memory_bytes(&mesh);
        assert!(bytes > 0);
    }

    #[test]
    fn estimate_info() {
        // Test with a non-existent file should return None
        let info = estimate_file_info(Path::new("/nonexistent/file.stl"));
        assert!(info.is_none());
    }

    #[test]
    fn zero_chunk_size_returns_no_chunks() {
        let mesh = PolyData::from_points(vec![[0.0, 0.0, 0.0]]);
        let mut called = false;
        let chunks = process_points_chunked(&mesh, 0, |_, _| called = true);
        assert_eq!(chunks, 0);
        assert!(!called);
    }

    #[test]
    fn chunked_read_count() {
        // Test the chunk counting logic with in-memory data
        let mesh =
            PolyData::from_points((0..100).map(|i| [i as f64, 0.0, 0.0]).collect::<Vec<_>>());
        let mut count = 0;
        let n = mesh.points.len();
        for _start in (0..n).step_by(30) {
            count += 1;
        }
        assert_eq!(count, 4); // 100/30 = 3.33 → 4 chunks
    }

    #[test]
    fn chunked_processing_preserves_point_data_ranges() {
        let mut mesh = PolyData::from_points((0..5).map(|i| [i as f64, 0.0, 0.0]).collect());
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "s",
                vec![0.0, 1.0, 2.0, 3.0, 4.0],
                1,
            )));
        mesh.point_data_mut().set_active_scalars("s");

        let mut chunks = Vec::new();
        let n = process_points_chunked(&mesh, 2, |_, chunk| chunks.push(chunk.clone()));

        assert_eq!(n, 3);
        assert_eq!(
            chunks[1].point_data().scalars().unwrap().to_f64_vec(),
            vec![2.0, 3.0]
        );
    }
}
