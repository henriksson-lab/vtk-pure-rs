use crate::data::{AnyDataArray, DataArray};
use crate::types::VtkError;

/// Decode base64 bytes (standard alphabet, no padding required).
pub fn base64_decode(input: &str) -> Result<Vec<u8>, VtkError> {
    let input = input.trim();
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut nbits: u32 = 0;

    for byte in input.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => continue,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => {
                return Err(VtkError::Parse(format!(
                    "invalid base64 char: {}",
                    byte as char
                )))
            }
        };
        buf = (buf << 6) | val as u32;
        nbits += 6;
        if nbits >= 8 {
            nbits -= 8;
            output.push((buf >> nbits) as u8);
            buf &= (1 << nbits) - 1;
        }
    }

    Ok(output)
}

/// Encode bytes to base64.
pub fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Encode an AnyDataArray to base64 binary format (4-byte header + data).
pub fn encode_data_array_binary(arr: &AnyDataArray) -> String {
    let raw = data_array_to_bytes(arr);
    let mut with_header = Vec::with_capacity(4 + raw.len());
    with_header.extend_from_slice(&(raw.len() as u32).to_le_bytes());
    with_header.extend_from_slice(&raw);
    base64_encode(&with_header)
}

/// Convert AnyDataArray to raw little-endian bytes.
pub fn data_array_to_bytes(arr: &AnyDataArray) -> Vec<u8> {
    let nt = arr.num_tuples();
    let nc = arr.num_components();
    let total = nt * nc;
    match arr {
        AnyDataArray::F32(a) => {
            let mut buf = Vec::with_capacity(total * 4);
            for i in 0..nt {
                let t = a.tuple(i);
                for &v in t {
                    buf.extend_from_slice(&v.to_le_bytes());
                }
            }
            buf
        }
        AnyDataArray::F64(a) => {
            let mut buf = Vec::with_capacity(total * 8);
            for i in 0..nt {
                let t = a.tuple(i);
                for &v in t {
                    buf.extend_from_slice(&v.to_le_bytes());
                }
            }
            buf
        }
        AnyDataArray::I32(a) => {
            let mut buf = Vec::with_capacity(total * 4);
            for i in 0..nt {
                let t = a.tuple(i);
                for &v in t {
                    buf.extend_from_slice(&v.to_le_bytes());
                }
            }
            buf
        }
        AnyDataArray::I64(a) => {
            let mut buf = Vec::with_capacity(total * 8);
            for i in 0..nt {
                let t = a.tuple(i);
                for &v in t {
                    buf.extend_from_slice(&v.to_le_bytes());
                }
            }
            buf
        }
        AnyDataArray::U8(a) => {
            let mut buf = Vec::with_capacity(total);
            for i in 0..nt {
                let t = a.tuple(i);
                buf.extend_from_slice(t);
            }
            buf
        }
        _ => {
            // Fallback: convert to f64
            let mut buf = Vec::with_capacity(total * 8);
            let mut tmp = vec![0.0f64; nc];
            for i in 0..nt {
                arr.tuple_as_f64(i, &mut tmp);
                for &v in &tmp {
                    buf.extend_from_slice(&v.to_le_bytes());
                }
            }
            buf
        }
    }
}

/// Parse raw binary bytes into an AnyDataArray.
///
/// VTK XML binary format: the data array content is base64-encoded.
/// The first 4 bytes (UInt32) or 8 bytes (UInt64) give the byte count of the data.
/// By default VTK uses a UInt32 header.
pub fn parse_binary_data_array(
    encoded: &str,
    name: &str,
    type_str: &str,
    num_components: usize,
) -> Result<AnyDataArray, VtkError> {
    let bytes = base64_decode(encoded)?;
    if bytes.len() < 4 {
        return Err(VtkError::Parse("binary data too short for header".into()));
    }

    // Read 4-byte header (number of data bytes)
    let data_bytes = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    let data_start = 4;
    let data_end = data_start + data_bytes;

    if data_end > bytes.len() {
        // Try 8-byte header
        if bytes.len() >= 8 {
            let data_bytes_64 = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]) as usize;
            let data_start_64 = 8;
            let data_end_64 = data_start_64 + data_bytes_64;
            if data_end_64 <= bytes.len() {
                return bytes_to_data_array(
                    &bytes[data_start_64..data_end_64],
                    name,
                    type_str,
                    num_components,
                );
            }
        }
        return Err(VtkError::Parse(format!(
            "binary data header says {} bytes but only {} available",
            data_bytes,
            bytes.len() - data_start
        )));
    }

    bytes_to_data_array(&bytes[data_start..data_end], name, type_str, num_components)
}

/// Parse binary data from an appended data section.
///
/// `appended_data` is the raw binary data after the underscore in `<AppendedData encoding="raw">`.
/// `offset` is the byte offset into this section.
pub fn parse_appended_data_array(
    appended_data: &[u8],
    offset: usize,
    name: &str,
    type_str: &str,
    num_components: usize,
) -> Result<AnyDataArray, VtkError> {
    if offset + 4 > appended_data.len() {
        return Err(VtkError::Parse("appended data offset out of range".into()));
    }

    let header = u32::from_le_bytes([
        appended_data[offset],
        appended_data[offset + 1],
        appended_data[offset + 2],
        appended_data[offset + 3],
    ]) as usize;

    let data_start = offset + 4;
    let data_end = data_start + header;
    if data_end > appended_data.len() {
        return Err(VtkError::Parse(
            "appended data extends beyond buffer".into(),
        ));
    }

    bytes_to_data_array(
        &appended_data[data_start..data_end],
        name,
        type_str,
        num_components,
    )
}

/// Parse base64-encoded appended data.
pub fn parse_appended_base64_data_array(
    appended_encoded: &str,
    offset: usize,
    name: &str,
    type_str: &str,
    num_components: usize,
) -> Result<AnyDataArray, VtkError> {
    let bytes = base64_decode(appended_encoded)?;
    parse_appended_data_array(&bytes, offset, name, type_str, num_components)
}

fn bytes_to_data_array(
    data: &[u8],
    name: &str,
    type_str: &str,
    nc: usize,
) -> Result<AnyDataArray, VtkError> {
    match type_str {
        "Float32" => {
            if data.len() % 4 != 0 {
                return Err(VtkError::Parse("Float32 data not aligned".into()));
            }
            let values: Vec<f32> = data
                .chunks_exact(4)
                .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            Ok(AnyDataArray::F32(DataArray::from_vec(name, values, nc)))
        }
        "Float64" => {
            if data.len() % 8 != 0 {
                return Err(VtkError::Parse("Float64 data not aligned".into()));
            }
            let values: Vec<f64> = data
                .chunks_exact(8)
                .map(|c| f64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
                .collect();
            Ok(AnyDataArray::F64(DataArray::from_vec(name, values, nc)))
        }
        "Int8" => {
            let values: Vec<i8> = data.iter().map(|&b| b as i8).collect();
            Ok(AnyDataArray::I8(DataArray::from_vec(name, values, nc)))
        }
        "UInt8" => Ok(AnyDataArray::U8(DataArray::from_vec(
            name,
            data.to_vec(),
            nc,
        ))),
        "Int16" => {
            if data.len() % 2 != 0 {
                return Err(VtkError::Parse("Int16 data not aligned".into()));
            }
            let values: Vec<i16> = data
                .chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect();
            Ok(AnyDataArray::I16(DataArray::from_vec(name, values, nc)))
        }
        "UInt16" => {
            if data.len() % 2 != 0 {
                return Err(VtkError::Parse("UInt16 data not aligned".into()));
            }
            let values: Vec<u16> = data
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            Ok(AnyDataArray::U16(DataArray::from_vec(name, values, nc)))
        }
        "Int32" => {
            if data.len() % 4 != 0 {
                return Err(VtkError::Parse("Int32 data not aligned".into()));
            }
            let values: Vec<i32> = data
                .chunks_exact(4)
                .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            Ok(AnyDataArray::I32(DataArray::from_vec(name, values, nc)))
        }
        "UInt32" => {
            if data.len() % 4 != 0 {
                return Err(VtkError::Parse("UInt32 data not aligned".into()));
            }
            let values: Vec<u32> = data
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            Ok(AnyDataArray::U32(DataArray::from_vec(name, values, nc)))
        }
        "Int64" => {
            if data.len() % 8 != 0 {
                return Err(VtkError::Parse("Int64 data not aligned".into()));
            }
            let values: Vec<i64> = data
                .chunks_exact(8)
                .map(|c| i64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
                .collect();
            Ok(AnyDataArray::I64(DataArray::from_vec(name, values, nc)))
        }
        "UInt64" => {
            if data.len() % 8 != 0 {
                return Err(VtkError::Parse("UInt64 data not aligned".into()));
            }
            let values: Vec<u64> = data
                .chunks_exact(8)
                .map(|c| u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
                .collect();
            Ok(AnyDataArray::U64(DataArray::from_vec(name, values, nc)))
        }
        _ => Err(VtkError::Parse(format!(
            "unsupported binary type: {type_str}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        let encoded = "AQID"; // [1, 2, 3]
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, vec![1, 2, 3]);
    }

    #[test]
    fn test_base64_decode_with_padding() {
        let encoded = "AQIDBA=="; // [1, 2, 3, 4]
        let decoded = base64_decode(encoded).unwrap();
        assert_eq!(decoded, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_parse_binary_float32() {
        // Build base64-encoded data: 4-byte header (12 = 3 floats * 4 bytes) + 3 f32 values
        let mut raw = Vec::new();
        raw.extend_from_slice(&12u32.to_le_bytes()); // header
        raw.extend_from_slice(&1.0f32.to_le_bytes());
        raw.extend_from_slice(&2.0f32.to_le_bytes());
        raw.extend_from_slice(&3.0f32.to_le_bytes());

        let encoded = base64_encode_for_test(&raw);
        let result = parse_binary_data_array(&encoded, "test", "Float32", 1).unwrap();
        assert!(matches!(&result, AnyDataArray::F32(_)));
        assert_eq!(result.num_tuples(), 3);
        let mut buf = [0.0f64];
        result.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 1.0).abs() < 1e-6);
        result.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-6);
    }

    fn base64_encode_for_test(data: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(CHARS[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }
}
