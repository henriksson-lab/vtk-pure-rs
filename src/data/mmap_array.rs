//! File-backed data array loading and binary I/O utilities.
//!
//! Provides `MmapDataArray` for deferred loading of binary f64 data from files,
//! `MmapPointCloud` for loading XYZ point clouds, and raw binary
//! read/write helpers for `DataArray<f64>`.
//!
//! Despite the historical type names, these loaders perform checked eager
//! reads into owned Rust vectors. They do not expose OS-backed memory maps.

use std::fs::File;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::data::{CellArray, DataArray, Points, PolyData};

/// Descriptor for a binary f64 array stored in a file.
///
/// The file is not opened until [`load`](MmapDataArray::load) is called,
/// enabling deferred loading of large datasets. Loading copies the requested
/// range into memory; this is not an OS memory map.
#[derive(Debug, Clone)]
pub struct MmapDataArray {
    /// Path to the binary file.
    pub path: PathBuf,
    /// Byte offset where the array data begins.
    pub offset: u64,
    /// Number of tuples in the array.
    pub num_tuples: usize,
    /// Number of f64 components per tuple.
    pub num_components: usize,
}

impl MmapDataArray {
    /// Create a new descriptor.
    pub fn new(
        path: impl Into<PathBuf>,
        offset: u64,
        num_tuples: usize,
        num_components: usize,
    ) -> Self {
        Self {
            path: path.into(),
            offset,
            num_tuples,
            num_components,
        }
    }

    /// Read the data from the file and return a `DataArray<f64>`.
    pub fn load(&self) -> std::io::Result<DataArray<f64>> {
        if self.num_components == 0 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "num_components must be > 0",
            ));
        }
        let total = self
            .num_tuples
            .checked_mul(self.num_components)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    "array tuple/component count overflows",
                )
            })?;
        let byte_len = total
            .checked_mul(8)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "array byte length overflows"))?;
        let mut file = File::open(&self.path)?;
        ensure_available(&file, self.offset, byte_len)?;
        file.seek(SeekFrom::Start(self.offset))?;
        let mut buf = vec![0u8; byte_len];
        file.read_exact(&mut buf)?;
        let data: Vec<f64> = buf
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
            .collect();
        Ok(DataArray::from_vec("mmap_array", data, self.num_components))
    }
}

/// Descriptor for a binary XYZ point cloud stored as packed f64 triples.
#[derive(Debug, Clone)]
pub struct MmapPointCloud {
    /// Path to the binary file.
    pub path: PathBuf,
    /// Byte offset where the point data begins.
    pub offset: u64,
    /// Number of points (each point is 3 consecutive f64 values).
    pub num_points: usize,
}

impl MmapPointCloud {
    pub fn new(path: impl Into<PathBuf>, offset: u64, num_points: usize) -> Self {
        Self {
            path: path.into(),
            offset,
            num_points,
        }
    }

    /// Load the points from the file and return a `PolyData` with vertex cells.
    pub fn load(&self) -> std::io::Result<PolyData> {
        let byte_len = self
            .num_points
            .checked_mul(3)
            .and_then(|n| n.checked_mul(8))
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "point-cloud byte length overflows")
            })?;
        let mut file = File::open(&self.path)?;
        ensure_available(&file, self.offset, byte_len)?;
        file.seek(SeekFrom::Start(self.offset))?;
        let mut buf = vec![0u8; byte_len];
        file.read_exact(&mut buf)?;

        let mut points = Points::<f64>::new();
        let mut verts = CellArray::new();
        for i in 0..self.num_points {
            let base = i * 3 * 8;
            let x = f64::from_le_bytes(buf[base..base + 8].try_into().unwrap());
            let y = f64::from_le_bytes(buf[base + 8..base + 16].try_into().unwrap());
            let z = f64::from_le_bytes(buf[base + 16..base + 24].try_into().unwrap());
            points.push([x, y, z]);
            verts.push_cell(&[i as i64]);
        }

        let mut pd = PolyData::new();
        pd.points = points;
        pd.verts = verts;
        Ok(pd)
    }
}

fn ensure_available(file: &File, offset: u64, byte_len: usize) -> std::io::Result<()> {
    let byte_len = u64::try_from(byte_len)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "byte length does not fit in u64"))?;
    let end = offset
        .checked_add(byte_len)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "offset plus byte length overflows"))?;
    let file_len = file.metadata()?.len();
    if end > file_len {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "requested binary array range extends past end of file",
        ));
    }
    Ok(())
}

/// Write a `DataArray<f64>` as raw little-endian binary.
pub fn write_binary_array(path: &Path, array: &DataArray<f64>) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for &v in array.as_slice() {
        file.write_all(&v.to_le_bytes())?;
    }
    file.flush()?;
    Ok(())
}

/// Read a raw little-endian f64 binary file into a `DataArray<f64>`.
pub fn read_binary_array(
    path: &Path,
    name: &str,
    num_components: usize,
) -> std::io::Result<DataArray<f64>> {
    if num_components == 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "num_components must be > 0",
        ));
    }
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    if !buf.len().is_multiple_of(8) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "file length is not a whole number of f64 values",
        ));
    }
    let values = buf.len() / 8;
    if !values.is_multiple_of(num_components) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "value count is not divisible by num_components",
        ));
    }
    let data: Vec<f64> = buf
        .chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect();
    Ok(DataArray::from_vec(name, data, num_components))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_read_roundtrip() {
        let dir = std::env::temp_dir().join("vtk_mmap_test_roundtrip");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("array.bin");

        let orig = DataArray::<f64>::from_vec("temp", vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3);
        write_binary_array(&path, &orig).unwrap();
        let loaded = read_binary_array(&path, "temp", 3).unwrap();
        assert_eq!(loaded.num_tuples(), 2);
        assert_eq!(loaded.num_components(), 3);
        assert_eq!(loaded.as_slice(), orig.as_slice());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mmap_data_array_load() {
        let dir = std::env::temp_dir().join("vtk_mmap_test_load");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("data.bin");

        // Write header bytes then array data
        let mut f = File::create(&path).unwrap();
        f.write_all(&[0u8; 16]).unwrap(); // 16-byte header
        for v in &[10.0f64, 20.0, 30.0] {
            f.write_all(&v.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();

        let desc = MmapDataArray::new(&path, 16, 3, 1);
        let arr = desc.load().unwrap();
        assert_eq!(arr.num_tuples(), 3);
        assert_eq!(arr.as_slice(), &[10.0, 20.0, 30.0]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mmap_point_cloud_load() {
        let dir = std::env::temp_dir().join("vtk_mmap_test_pc");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("points.bin");

        let mut f = File::create(&path).unwrap();
        for v in &[1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0] {
            f.write_all(&v.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();

        let pc = MmapPointCloud::new(&path, 0, 2);
        let pd = pc.load().unwrap();
        assert_eq!(pd.points.len(), 2);
        assert_eq!(pd.verts.num_cells(), 2);
        let p0 = pd.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-12);
        assert!((p0[1] - 2.0).abs() < 1e-12);
        assert!((p0[2] - 3.0).abs() < 1e-12);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mmap_data_array_rejects_short_file_before_decode() {
        let dir = std::env::temp_dir().join("vtk_mmap_test_short");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("data.bin");

        let mut f = File::create(&path).unwrap();
        f.write_all(&1.0f64.to_le_bytes()).unwrap();
        f.flush().unwrap();

        let desc = MmapDataArray::new(&path, 0, 2, 1);
        let err = desc.load().unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_binary_array_rejects_partial_tuple() {
        let dir = std::env::temp_dir().join("vtk_mmap_test_partial_tuple");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("array.bin");

        let mut f = File::create(&path).unwrap();
        for v in &[1.0f64, 2.0, 3.0] {
            f.write_all(&v.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();

        let err = read_binary_array(&path, "bad", 2).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_binary_array_rejects_trailing_bytes() {
        let dir = std::env::temp_dir().join("vtk_mmap_test_trailing_bytes");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("array.bin");

        let mut f = File::create(&path).unwrap();
        f.write_all(&[1, 2, 3]).unwrap();
        f.flush().unwrap();

        let err = read_binary_array(&path, "bad", 1).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
