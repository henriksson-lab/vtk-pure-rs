use std::io::BufRead;
use std::path::Path;

use crate::data::{CellArray, DataArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for STL (stereolithography) format.
///
/// Automatically detects ASCII vs binary format.
pub struct StlReader;

impl StlReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let data = std::fs::read(path)?;
        Self::read_from(&data)
    }

    pub fn read_from(data: &[u8]) -> Result<PolyData, VtkError> {
        // Detect format: ASCII STL starts with "solid"
        if data.len() > 5 && &data[..5] == b"solid" {
            // Could still be binary if "solid" appears in header.
            // Check if it looks like ASCII by searching for "facet" after the first line.
            if let Some(newline) = data.iter().position(|&b| b == b'\n') {
                let after_header = &data[newline + 1..];
                let trimmed = trim_start(after_header);
                if trimmed.starts_with(b"facet") || trimmed.starts_with(b"endsolid") {
                    return Self::read_ascii(data);
                }
            }
        }
        Self::read_binary(data)
    }

    fn read_ascii(data: &[u8]) -> Result<PolyData, VtkError> {
        let text = std::str::from_utf8(data)
            .map_err(|e| VtkError::Parse(format!("invalid UTF-8: {}", e)))?;

        let mut points = Points::<f64>::new();
        let mut polys = CellArray::new();
        let mut normals_arr = DataArray::<f64>::new("Normals", 3);

        let reader = std::io::BufReader::new(text.as_bytes());
        let mut current_normal = [0.0f64; 3];
        let mut tri_points = Vec::with_capacity(3);

        for line in reader.lines() {
            let line = line.map_err(VtkError::Io)?;
            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix("facet normal") {
                let parts: Vec<f64> = rest
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() == 3 {
                    current_normal = [parts[0], parts[1], parts[2]];
                }
                tri_points.clear();
            } else if let Some(rest) = trimmed.strip_prefix("vertex") {
                let parts: Vec<f64> = rest
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() == 3 {
                    tri_points.push([parts[0], parts[1], parts[2]]);
                }
            } else if trimmed == "endfacet" && tri_points.len() == 3 {
                let base = points.len() as i64;
                for &p in &tri_points {
                    points.push(p);
                    normals_arr.push_tuple(&current_normal);
                }
                polys.push_cell(&[base, base + 1, base + 2]);
            }
        }

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        if normals_arr.num_tuples() > 0 {
            pd.point_data_mut().add_array(normals_arr.into());
            pd.point_data_mut().set_active_normals("Normals");
        }
        Ok(pd)
    }

    fn read_binary(data: &[u8]) -> Result<PolyData, VtkError> {
        if data.len() < 84 {
            return Err(VtkError::Parse("STL binary too short".into()));
        }

        // Skip 80-byte header
        let num_triangles = u32::from_le_bytes(data[80..84].try_into().unwrap()) as usize;

        let expected = 84 + num_triangles * 50;
        if data.len() < expected {
            return Err(VtkError::Parse(format!(
                "STL binary truncated: expected {} bytes, got {}",
                expected,
                data.len()
            )));
        }

        let mut points = Points::<f64>::new();
        let mut polys = CellArray::new();
        let mut normals_arr = DataArray::<f64>::new("Normals", 3);

        let mut offset = 84;
        for _ in 0..num_triangles {
            // Normal: 3 x f32 LE
            let nx = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as f64;
            let ny = f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as f64;
            let nz = f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as f64;
            offset += 12;

            let base = points.len() as i64;
            // 3 vertices: each 3 x f32 LE
            for _ in 0..3 {
                let x = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as f64;
                let y = f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as f64;
                let z =
                    f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as f64;
                offset += 12;
                points.push([x, y, z]);
                normals_arr.push_tuple(&[nx, ny, nz]);
            }
            // Attribute byte count (skip)
            offset += 2;

            polys.push_cell(&[base, base + 1, base + 2]);
        }

        let mut pd = PolyData::new();
        pd.points = points;
        pd.polys = polys;
        pd.point_data_mut().add_array(normals_arr.into());
        pd.point_data_mut().set_active_normals("Normals");
        Ok(pd)
    }
}

fn trim_start(data: &[u8]) -> &[u8] {
    let start = data
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(data.len());
    &data[start..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::stl::StlWriter;

    #[test]
    fn roundtrip_ascii() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let writer = StlWriter::ascii();
        let mut buf = Vec::new();
        writer.write_to(&mut buf, &pd).unwrap();

        let result = StlReader::read_from(&buf).unwrap();
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn roundtrip_binary() {
        let pd = PolyData::from_triangles(
            vec![
                [1.0, 2.0, 3.0],
                [4.0, 5.0, 6.0],
                [7.0, 8.0, 9.0],
                [10.0, 11.0, 12.0],
            ],
            vec![[0, 1, 2], [1, 2, 3]],
        );

        let writer = StlWriter::binary();
        let mut buf = Vec::new();
        writer.write_to(&mut buf, &pd).unwrap();

        let result = StlReader::read_from(&buf).unwrap();
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.points.len(), 6); // STL duplicates vertices

        // Check first vertex (f32 precision)
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-5);
        assert!((p0[1] - 2.0).abs() < 1e-5);
    }
}
