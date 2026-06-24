//! LAS (LIDAR) point cloud reader for vtk-rs.
//!
//! Reads LAS 1.2 format binary files (point formats 0-3).
//! Does not support compressed LAZ files.

use crate::data::{AnyDataArray, DataArray, Points, PolyData};
use std::io::{Read, Seek, SeekFrom};

/// Read a LAS file and return a PolyData point cloud.
pub fn read_las<R: Read + Seek>(reader: &mut R) -> Result<PolyData, String> {
    // Read public header
    let mut sig = [0u8; 4];
    reader.read_exact(&mut sig).map_err(|e| e.to_string())?;
    if &sig != b"LASF" {
        return Err("not a LAS file (missing LASF signature)".into());
    }

    // Skip to relevant header fields
    let mut header = [0u8; 223]; // remaining header bytes (after signature)
    reader.read_exact(&mut header).map_err(|e| e.to_string())?;

    let _point_format = header[100]; // offset 104 from start - 4
    let point_record_len = u16::from_le_bytes([header[101], header[102]]);
    let num_points =
        u32::from_le_bytes([header[103], header[104], header[105], header[106]]) as usize;
    let offset_to_data =
        u32::from_le_bytes([header[92], header[93], header[94], header[95]]) as u64;

    let x_scale = f64::from_le_bytes(header[127..135].try_into().unwrap());
    let y_scale = f64::from_le_bytes(header[135..143].try_into().unwrap());
    let z_scale = f64::from_le_bytes(header[143..151].try_into().unwrap());
    let x_offset = f64::from_le_bytes(header[151..159].try_into().unwrap());
    let y_offset = f64::from_le_bytes(header[159..167].try_into().unwrap());
    let z_offset = f64::from_le_bytes(header[167..175].try_into().unwrap());

    // Seek to point data
    reader
        .seek(SeekFrom::Start(offset_to_data))
        .map_err(|e| e.to_string())?;

    let mut points = Points::<f64>::new();
    let mut intensity_data = Vec::with_capacity(num_points);
    let mut classification_data = Vec::with_capacity(num_points);

    let mut point_buf = vec![0u8; point_record_len as usize];

    for _ in 0..num_points {
        if reader.read_exact(&mut point_buf).is_err() {
            break;
        }

        // All point formats start with X, Y, Z as i32
        let xi = i32::from_le_bytes(point_buf[0..4].try_into().unwrap());
        let yi = i32::from_le_bytes(point_buf[4..8].try_into().unwrap());
        let zi = i32::from_le_bytes(point_buf[8..12].try_into().unwrap());

        let x = xi as f64 * x_scale + x_offset;
        let y = yi as f64 * y_scale + y_offset;
        let z = zi as f64 * z_scale + z_offset;

        points.push([x, y, z]);

        // Intensity (u16 at offset 12)
        if point_buf.len() >= 14 {
            let intensity = u16::from_le_bytes([point_buf[12], point_buf[13]]);
            intensity_data.push(intensity as f64);
        }

        // Classification (u8 at offset 15)
        if point_buf.len() >= 16 {
            classification_data.push(point_buf[15] as f64);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;

    if !intensity_data.is_empty() {
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Intensity",
                intensity_data,
                1,
            )));
    }
    if !classification_data.is_empty() {
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Classification",
                classification_data,
                1,
            )));
    }

    Ok(mesh)
}

/// Read a LAS file from a file path.
pub fn read_las_file(path: &std::path::Path) -> Result<PolyData, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_las(&mut file)
}

/// Write a minimal LAS 1.2 file from a PolyData point cloud.
pub fn write_las<W: std::io::Write>(w: &mut W, mesh: &PolyData) -> Result<(), String> {
    let n = mesh.points.len();
    if n == 0 {
        return Ok(());
    }

    // Compute scale and offset
    let mut min = mesh.points.get(0);
    let mut max = min;
    for i in 1..n {
        let p = mesh.points.get(i);
        for j in 0..3 {
            min[j] = min[j].min(p[j]);
            max[j] = max[j].max(p[j]);
        }
    }

    let scale = [
        if (max[0] - min[0]).abs() > 1e-15 {
            (max[0] - min[0]) / i32::MAX as f64
        } else {
            0.001
        },
        if (max[1] - min[1]).abs() > 1e-15 {
            (max[1] - min[1]) / i32::MAX as f64
        } else {
            0.001
        },
        if (max[2] - min[2]).abs() > 1e-15 {
            (max[2] - min[2]) / i32::MAX as f64
        } else {
            0.001
        },
    ];
    let offset = min;

    // Build header (227 bytes for LAS 1.2 point format 0)
    let mut header = vec![0u8; 227];
    header[0..4].copy_from_slice(b"LASF");
    header[24] = 1; // version major
    header[25] = 2; // version minor
                    // Header size = 227
    header[94..96].copy_from_slice(&227u16.to_le_bytes()); // header size
                                                           // Offset to point data
    header[96..100].copy_from_slice(&227u32.to_le_bytes());
    // Point format 0
    header[104] = 0;
    // Point record length = 20 (x,y,z=12 + intensity=2 + misc=6)
    header[105..107].copy_from_slice(&20u16.to_le_bytes());
    // Num points
    header[107..111].copy_from_slice(&(n as u32).to_le_bytes());
    // Scale factors
    header[131..139].copy_from_slice(&scale[0].to_le_bytes());
    header[139..147].copy_from_slice(&scale[1].to_le_bytes());
    header[147..155].copy_from_slice(&scale[2].to_le_bytes());
    // Offsets
    header[155..163].copy_from_slice(&offset[0].to_le_bytes());
    header[163..171].copy_from_slice(&offset[1].to_le_bytes());
    header[171..179].copy_from_slice(&offset[2].to_le_bytes());
    // Min/max
    header[179..187].copy_from_slice(&max[0].to_le_bytes());
    header[187..195].copy_from_slice(&min[0].to_le_bytes());
    header[195..203].copy_from_slice(&max[1].to_le_bytes());
    header[203..211].copy_from_slice(&min[1].to_le_bytes());
    header[211..219].copy_from_slice(&max[2].to_le_bytes());
    header[219..227].copy_from_slice(&min[2].to_le_bytes());

    w.write_all(&header).map_err(|e| e.to_string())?;

    // Write points (format 0: 20 bytes each)
    for i in 0..n {
        let p = mesh.points.get(i);
        let xi = ((p[0] - offset[0]) / scale[0]) as i32;
        let yi = ((p[1] - offset[1]) / scale[1]) as i32;
        let zi = ((p[2] - offset[2]) / scale[2]) as i32;

        w.write_all(&xi.to_le_bytes()).map_err(|e| e.to_string())?;
        w.write_all(&yi.to_le_bytes()).map_err(|e| e.to_string())?;
        w.write_all(&zi.to_le_bytes()).map_err(|e| e.to_string())?;
        w.write_all(&[0u8; 8]).map_err(|e| e.to_string())?; // intensity + padding
    }

    Ok(())
}

/// Write LAS to file path.
pub fn write_las_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::new(file);
    write_las(&mut w, mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mesh = PolyData::from_points(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]);

        let mut buf = Vec::new();
        write_las(&mut buf, &mesh).unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let loaded = read_las(&mut cursor).unwrap();
        assert_eq!(loaded.points.len(), 3);

        // Check approximate coordinate preservation
        let p = loaded.points.get(0);
        assert!((p[0] - 1.0).abs() < 0.01, "x={}", p[0]);
        assert!((p[1] - 2.0).abs() < 0.01, "y={}", p[1]);
        assert!((p[2] - 3.0).abs() < 0.01, "z={}", p[2]);
    }

    #[test]
    fn empty_point_cloud() {
        let mesh = PolyData::new();
        let mut buf = Vec::new();
        write_las(&mut buf, &mesh).unwrap();
        assert!(buf.is_empty()); // no output for empty
    }
}
