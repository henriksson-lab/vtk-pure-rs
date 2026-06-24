use std::path::Path;

use crate::data::{CellArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for binary glTF (.glb) format.
///
/// Reads glTF 2.0 binary files and extracts the first mesh as PolyData.
/// Supports positions (VEC3 Float32) and indices (SCALAR UInt16/UInt32).
pub struct GlbReader;

impl GlbReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let data = std::fs::read(path)?;
        Self::read_from(&data)
    }

    pub fn read_from(data: &[u8]) -> Result<PolyData, VtkError> {
        if data.len() < 12 {
            return Err(VtkError::Parse("file too short for GLB header".into()));
        }

        // Check magic
        if &data[0..4] != b"glTF" {
            return Err(VtkError::Parse("not a GLB file".into()));
        }

        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if version != 2 {
            return Err(VtkError::Parse(format!("unsupported glTF version: {version}")).into());
        }

        // Parse chunks
        let mut json_data = &[] as &[u8];
        let mut bin_data = &[] as &[u8];
        let mut offset = 12;

        while offset + 8 <= data.len() {
            let chunk_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            let chunk_type = u32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            let chunk_start = offset + 8;
            let chunk_end = (chunk_start + chunk_len).min(data.len());

            match chunk_type {
                0x4E4F534A => json_data = &data[chunk_start..chunk_end], // JSON
                0x004E4942 => bin_data = &data[chunk_start..chunk_end],  // BIN
                _ => {}
            }

            offset = chunk_end;
            // Align to 4 bytes
            offset = (offset + 3) & !3;
        }

        if json_data.is_empty() {
            return Err(VtkError::Parse("no JSON chunk in GLB".into()));
        }

        let json_str = std::str::from_utf8(json_data)
            .map_err(|_| VtkError::Parse("invalid UTF-8 in JSON chunk".into()))?;

        parse_gltf_json(json_str.trim(), bin_data)
    }
}

fn parse_gltf_json(json: &str, bin: &[u8]) -> Result<PolyData, VtkError> {
    // Minimal JSON parsing for glTF structure
    let accessors = extract_json_array(json, "accessors")?;
    let buffer_views = extract_json_array(json, "bufferViews")?;

    // Find first mesh primitive
    let meshes_str = extract_json_array(json, "meshes")?;
    let first_mesh = first_json_object(&meshes_str)?;
    let primitives_str = extract_json_array(&first_mesh, "primitives")?;
    let prim = first_json_object(&primitives_str)?;

    // Get indices accessor
    let indices_acc_idx = extract_json_int(&prim, "indices")
        .ok_or_else(|| VtkError::Parse("no indices in primitive".into()))?;

    // Get position accessor
    let attrs = extract_json_object(&prim, "attributes")?;
    let pos_acc_idx = extract_json_int(&attrs, "POSITION")
        .ok_or_else(|| VtkError::Parse("no POSITION attribute".into()))?;

    // Parse position data
    let pos_acc = nth_json_object(&accessors, pos_acc_idx)?;
    let pos_bv_idx = extract_json_int(&pos_acc, "bufferView")
        .ok_or_else(|| VtkError::Parse("no bufferView in accessor".into()))?;
    let _pos_count = extract_json_int(&pos_acc, "count").unwrap_or(0);

    let pos_bv = nth_json_object(&buffer_views, pos_bv_idx)?;
    let pos_offset = extract_json_int(&pos_bv, "byteOffset").unwrap_or(0);
    let pos_length = extract_json_int(&pos_bv, "byteLength").unwrap_or(0);

    let pos_bytes = &bin[pos_offset..pos_offset + pos_length];
    let mut points = Points::new();
    for chunk in pos_bytes.chunks_exact(12) {
        let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f64;
        let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) as f64;
        let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]) as f64;
        points.push([x, y, z]);
    }

    // Parse index data
    let idx_acc = nth_json_object(&accessors, indices_acc_idx)?;
    let idx_bv_idx = extract_json_int(&idx_acc, "bufferView")
        .ok_or_else(|| VtkError::Parse("no bufferView for indices".into()))?;
    let idx_comp_type = extract_json_int(&idx_acc, "componentType").unwrap_or(5125);
    let _idx_count = extract_json_int(&idx_acc, "count").unwrap_or(0);

    let idx_bv = nth_json_object(&buffer_views, idx_bv_idx)?;
    let idx_offset = extract_json_int(&idx_bv, "byteOffset").unwrap_or(0);
    let idx_length = extract_json_int(&idx_bv, "byteLength").unwrap_or(0);

    let idx_bytes = &bin[idx_offset..idx_offset + idx_length];
    let indices: Vec<u32> = match idx_comp_type {
        5123 => {
            // UInt16
            idx_bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]) as u32)
                .collect()
        }
        5125 => {
            // UInt32
            idx_bytes
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect()
        }
        _ => {
            return Err(VtkError::Parse(format!(
                "unsupported index type: {idx_comp_type}"
            )))
        }
    };

    // Build polys (triangles)
    let mut polys = CellArray::new();
    for tri in indices.chunks_exact(3) {
        polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    Ok(pd)
}

// Minimal JSON helpers (no dependency needed for glTF's simple structure)

fn extract_json_array(json: &str, key: &str) -> Result<String, VtkError> {
    let pattern = format!("\"{}\"", key);
    let pos = json
        .find(&pattern)
        .ok_or_else(|| VtkError::Parse(format!("key '{key}' not found")))?;
    let after = &json[pos + pattern.len()..];
    let colon = after
        .find(':')
        .ok_or_else(|| VtkError::Parse("expected colon".into()))?;
    let after_colon = after[colon + 1..].trim_start();
    if !after_colon.starts_with('[') {
        return Err(VtkError::Parse(format!("'{key}' is not an array")));
    }
    extract_balanced(after_colon, '[', ']')
}

fn extract_json_object(json: &str, key: &str) -> Result<String, VtkError> {
    let pattern = format!("\"{}\"", key);
    let pos = json
        .find(&pattern)
        .ok_or_else(|| VtkError::Parse(format!("key '{key}' not found")))?;
    let after = &json[pos + pattern.len()..];
    let colon = after
        .find(':')
        .ok_or_else(|| VtkError::Parse("expected colon".into()))?;
    let after_colon = after[colon + 1..].trim_start();
    if after_colon.starts_with('{') {
        extract_balanced(after_colon, '{', '}')
    } else {
        Err(VtkError::Parse(format!("'{key}' is not an object")))
    }
}

fn extract_json_int(json: &str, key: &str) -> Option<usize> {
    let pattern = format!("\"{}\"", key);
    let pos = json.find(&pattern)?;
    let after = &json[pos + pattern.len()..];
    let colon = after.find(':')?;
    let val_str = after[colon + 1..].trim_start();
    let end = val_str.find(|c: char| !c.is_ascii_digit())?;
    val_str[..end].parse().ok()
}

fn extract_balanced(s: &str, open: char, close: char) -> Result<String, VtkError> {
    let mut depth = 0;
    let mut start = None;
    for (i, c) in s.char_indices() {
        if c == open {
            if depth == 0 {
                start = Some(i);
            }
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Ok(s[start.unwrap()..=i].to_string());
            }
        }
    }
    Err(VtkError::Parse("unbalanced brackets".into()))
}

fn first_json_object(array: &str) -> Result<String, VtkError> {
    nth_json_object(array, 0)
}

fn nth_json_object(array: &str, n: usize) -> Result<String, VtkError> {
    let inner = &array[1..array.len() - 1]; // strip [ ]
    let mut depth = 0;
    let mut start = None;
    let mut count = 0;

    for (i, c) in inner.char_indices() {
        match c {
            '{' | '[' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    if count == n {
                        return Ok(inner[start.unwrap()..=i].to_string());
                    }
                    count += 1;
                }
            }
            _ => {}
        }
    }
    Err(VtkError::Parse(format!(
        "object index {n} not found in array"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::gltf::GlbWriter;

    #[test]
    fn roundtrip_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        GlbWriter::write_to(&mut buf, &pd).unwrap();

        let result = GlbReader::read_from(&buf).unwrap();
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);

        let p1 = result.points.get(1);
        assert!((p1[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn roundtrip_quad_mesh() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );

        let mut buf = Vec::new();
        GlbWriter::write_to(&mut buf, &pd).unwrap();

        let result = GlbReader::read_from(&buf).unwrap();
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn invalid_magic() {
        assert!(GlbReader::read_from(b"notglTF!").is_err());
    }
}
