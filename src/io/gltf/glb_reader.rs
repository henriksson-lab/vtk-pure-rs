use std::path::Path;

use std::collections::BTreeMap;

use crate::data::{CellArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for binary glTF (.glb) format.
///
/// Reads glTF 2.0 binary files and extracts the first mesh as PolyData.
/// Supports positions (VEC3 Float32), point/line/triangle primitives, and
/// unsigned byte/short/int indices.
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
        let file_length = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        if file_length != data.len() {
            return Err(VtkError::Parse(
                "GLB header length does not match file size".into(),
            ));
        }

        // Parse chunks
        let mut json_data = &[] as &[u8];
        let mut bin_data = &[] as &[u8];
        let mut offset = 12;
        let mut chunk_index = 0;

        while offset < data.len() {
            if offset + 8 > data.len() {
                return Err(VtkError::Parse("truncated GLB chunk header".into()));
            }
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
            let chunk_end = chunk_start
                .checked_add(chunk_len)
                .ok_or_else(|| VtkError::Parse("GLB chunk length overflow".into()))?;
            if chunk_end > data.len() {
                return Err(VtkError::Parse("truncated GLB chunk data".into()));
            }
            if chunk_index == 0 && chunk_type != 0x4E4F534A {
                return Err(VtkError::Parse("first GLB chunk is not JSON".into()));
            }

            match chunk_type {
                0x4E4F534A => json_data = &data[chunk_start..chunk_end], // JSON
                0x004E4942 => bin_data = &data[chunk_start..chunk_end],  // BIN
                _ => {}
            }

            offset = chunk_end;
            chunk_index += 1;
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
    let root = JsonParser::new(json).parse().map_err(VtkError::Parse)?;
    let accessors = root
        .member("accessors")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| VtkError::Parse("missing accessors array".into()))?;
    let buffer_views = root
        .member("bufferViews")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| VtkError::Parse("missing bufferViews array".into()))?;
    let meshes = root
        .member("meshes")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| VtkError::Parse("missing meshes array".into()))?;
    let first_mesh = meshes
        .first()
        .ok_or_else(|| VtkError::Parse("no mesh in glTF".into()))?;
    let primitives = first_mesh
        .member("primitives")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| VtkError::Parse("mesh missing primitives array".into()))?;
    let mut points = Points::new();
    let mut verts = CellArray::new();
    let mut lines = CellArray::new();
    let mut polys = CellArray::new();
    let mut strips = CellArray::new();

    for prim in primitives {
        let attrs = prim
            .member("attributes")
            .ok_or_else(|| VtkError::Parse("primitive missing attributes".into()))?;
        let pos_acc_idx = attrs
            .member_usize("POSITION")
            .ok_or_else(|| VtkError::Parse("no POSITION attribute".into()))?;
        let pos_acc = accessors
            .get(pos_acc_idx)
            .ok_or_else(|| VtkError::Parse(format!("POSITION accessor {pos_acc_idx} not found")))?;
        let primitive_point_offset = points.len() as i64;
        let primitive_point_count = read_positions(pos_acc, buffer_views, bin, &mut points)?;

        let mode = prim.member_usize("mode").unwrap_or(4);
        let indices = if let Some(indices_acc_idx) = prim.member_usize("indices") {
            let idx_acc = accessors.get(indices_acc_idx).ok_or_else(|| {
                VtkError::Parse(format!("index accessor {indices_acc_idx} not found"))
            })?;
            read_indices(idx_acc, buffer_views, bin)?
        } else {
            (0..primitive_point_count as u32).collect()
        };

        match mode {
            0 => {
                for &idx in &indices {
                    validate_index(idx, primitive_point_count)?;
                    verts.push_cell(&[primitive_point_offset + idx as i64]);
                }
            }
            1 => {
                for line in indices.chunks_exact(2) {
                    validate_index(line[0], primitive_point_count)?;
                    validate_index(line[1], primitive_point_count)?;
                    lines.push_cell(&[
                        primitive_point_offset + line[0] as i64,
                        primitive_point_offset + line[1] as i64,
                    ]);
                }
            }
            2 => {
                validate_indices(&indices, primitive_point_count)?;
                if !indices.is_empty() {
                    let mut cell = indices_to_cell(&indices, primitive_point_offset);
                    cell.push(primitive_point_offset + indices[0] as i64);
                    lines.push_cell(&cell);
                }
            }
            3 => {
                validate_indices(&indices, primitive_point_count)?;
                if !indices.is_empty() {
                    lines.push_cell(&indices_to_cell(&indices, primitive_point_offset));
                }
            }
            4 => {
                for tri in indices.chunks_exact(3) {
                    validate_index(tri[0], primitive_point_count)?;
                    validate_index(tri[1], primitive_point_count)?;
                    validate_index(tri[2], primitive_point_count)?;
                    polys.push_cell(&[
                        primitive_point_offset + tri[0] as i64,
                        primitive_point_offset + tri[1] as i64,
                        primitive_point_offset + tri[2] as i64,
                    ]);
                }
            }
            5 => {
                validate_indices(&indices, primitive_point_count)?;
                if !indices.is_empty() {
                    strips.push_cell(&indices_to_cell(&indices, primitive_point_offset));
                }
            }
            6 => {
                validate_indices(&indices, primitive_point_count)?;
                for i in 2..indices.len() {
                    polys.push_cell(&[
                        primitive_point_offset + indices[0] as i64,
                        primitive_point_offset + indices[i - 1] as i64,
                        primitive_point_offset + indices[i] as i64,
                    ]);
                }
            }
            _ => {
                return Err(VtkError::Unsupported(format!(
                    "unsupported glTF primitive mode: {mode}"
                )));
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.verts = verts;
    pd.lines = lines;
    pd.polys = polys;
    pd.strips = strips;
    Ok(pd)
}

fn validate_index(idx: u32, point_count: usize) -> Result<(), VtkError> {
    if idx as usize >= point_count {
        return Err(VtkError::Parse(format!(
            "primitive index {idx} is outside POSITION count {point_count}"
        )));
    }
    Ok(())
}

fn read_positions(
    accessor: &JsonValue,
    buffer_views: &[JsonValue],
    bin: &[u8],
    points: &mut Points<f64>,
) -> Result<usize, VtkError> {
    if accessor.member_str("type") != Some("VEC3")
        || accessor.member_usize("componentType") != Some(5126)
    {
        return Err(VtkError::Parse(
            "POSITION accessor must be VEC3 Float32".into(),
        ));
    }
    let count = accessor
        .member_usize("count")
        .ok_or_else(|| VtkError::Parse("POSITION accessor missing count".into()))?;
    let view = accessor_view(accessor, buffer_views, bin)?;
    let stride = view.byte_stride.unwrap_or(12);
    if stride < 12 {
        return Err(VtkError::Parse(
            "POSITION byteStride is smaller than VEC3 Float32".into(),
        ));
    }

    for i in 0..count {
        let start = view
            .offset
            .checked_add(i * stride)
            .ok_or_else(|| VtkError::Parse("POSITION byte offset overflow".into()))?;
        let chunk = checked_range(bin, start, 12, "POSITION accessor")?;
        let x = read_f32(chunk, 0) as f64;
        let y = read_f32(chunk, 4) as f64;
        let z = read_f32(chunk, 8) as f64;
        points.push([x, y, z]);
    }

    Ok(count)
}

fn validate_indices(indices: &[u32], point_count: usize) -> Result<(), VtkError> {
    for &idx in indices {
        validate_index(idx, point_count)?;
    }
    Ok(())
}

fn indices_to_cell(indices: &[u32], point_offset: i64) -> Vec<i64> {
    indices
        .iter()
        .map(|&idx| point_offset + idx as i64)
        .collect()
}

struct AccessorView {
    offset: usize,
    byte_stride: Option<usize>,
}

fn accessor_view(
    accessor: &JsonValue,
    buffer_views: &[JsonValue],
    bin: &[u8],
) -> Result<AccessorView, VtkError> {
    let bv_idx = accessor
        .member_usize("bufferView")
        .ok_or_else(|| VtkError::Parse("accessor missing bufferView".into()))?;
    let bv = buffer_views
        .get(bv_idx)
        .ok_or_else(|| VtkError::Parse(format!("bufferView {bv_idx} not found")))?;
    let bv_offset = bv.member_usize("byteOffset").unwrap_or(0);
    let bv_len = bv
        .member_usize("byteLength")
        .ok_or_else(|| VtkError::Parse("bufferView missing byteLength".into()))?;
    let acc_offset = accessor.member_usize("byteOffset").unwrap_or(0);
    let offset = bv_offset
        .checked_add(acc_offset)
        .ok_or_else(|| VtkError::Parse("accessor byte offset overflow".into()))?;
    checked_range(bin, bv_offset, bv_len, "bufferView")?;
    Ok(AccessorView {
        offset,
        byte_stride: bv.member_usize("byteStride"),
    })
}

fn read_indices(
    accessor: &JsonValue,
    buffer_views: &[JsonValue],
    bin: &[u8],
) -> Result<Vec<u32>, VtkError> {
    if accessor.member_str("type") != Some("SCALAR") {
        return Err(VtkError::Parse("index accessor must be SCALAR".into()));
    }
    let count = accessor
        .member_usize("count")
        .ok_or_else(|| VtkError::Parse("index accessor missing count".into()))?;
    let component_type = accessor
        .member_usize("componentType")
        .ok_or_else(|| VtkError::Parse("index accessor missing componentType".into()))?;
    let component_size = match component_type {
        5121 => 1,
        5123 => 2,
        5125 => 4,
        _ => {
            return Err(VtkError::Parse(format!(
                "unsupported index type: {component_type}"
            )))
        }
    };
    let view = accessor_view(accessor, buffer_views, bin)?;
    let stride = view.byte_stride.unwrap_or(component_size);
    if stride < component_size {
        return Err(VtkError::Parse(
            "index byteStride is smaller than component size".into(),
        ));
    }
    let mut indices = Vec::with_capacity(count);
    for i in 0..count {
        let start = view
            .offset
            .checked_add(i * stride)
            .ok_or_else(|| VtkError::Parse("index byte offset overflow".into()))?;
        let bytes = checked_range(bin, start, component_size, "index accessor")?;
        let idx = match component_type {
            5121 => bytes[0] as u32,
            5123 => u16::from_le_bytes([bytes[0], bytes[1]]) as u32,
            5125 => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            _ => unreachable!(),
        };
        indices.push(idx);
    }
    Ok(indices)
}

fn checked_range<'a>(
    data: &'a [u8],
    offset: usize,
    len: usize,
    what: &str,
) -> Result<&'a [u8], VtkError> {
    let end = offset
        .checked_add(len)
        .ok_or_else(|| VtkError::Parse(format!("{what} byte range overflow")))?;
    data.get(offset..end)
        .ok_or_else(|| VtkError::Parse(format!("{what} byte range is outside BIN chunk")))
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool,
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    fn member(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Object(values) => values.get(key),
            _ => None,
        }
    }

    fn member_usize(&self, key: &str) -> Option<usize> {
        match self.member(key)? {
            Self::Number(value) if *value >= 0.0 && value.fract() == 0.0 => Some(*value as usize),
            _ => None,
        }
    }

    fn member_str(&self, key: &str) -> Option<&str> {
        match self.member(key)? {
            Self::String(value) => Some(value),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<JsonValue, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.pos != self.input.len() {
            return Err("trailing characters after JSON value".to_string());
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'n') => self.parse_literal(b"null", JsonValue::Null),
            Some(b't') => self.parse_literal(b"true", JsonValue::Bool),
            Some(b'f') => self.parse_literal(b"false", JsonValue::Bool),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JsonValue::Number),
            Some(ch) => Err(format!("unexpected JSON byte {ch}")),
            None => Err("unexpected end of JSON".to_string()),
        }
    }

    fn parse_literal(&mut self, literal: &[u8], value: JsonValue) -> Result<JsonValue, String> {
        if self.input[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(value)
        } else {
            Err("invalid JSON literal".to_string())
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect(b'[')?;
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(b']') {
                break;
            }
            values.push(self.parse_value()?);
            self.skip_ws();
            if self.consume(b']') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(JsonValue::Array(values))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect(b'{')?;
        let mut values = BTreeMap::new();
        loop {
            self.skip_ws();
            if self.consume(b'}') {
                break;
            }
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(b':')?;
            values.insert(key, self.parse_value()?);
            self.skip_ws();
            if self.consume(b'}') {
                break;
            }
            self.expect(b',')?;
        }
        Ok(JsonValue::Object(values))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        while let Some(ch) = self.next() {
            match ch {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = self
                        .next()
                        .ok_or_else(|| "unterminated escape sequence".to_string())?;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.parse_unicode_escape()?),
                        _ => return Err("invalid string escape".to_string()),
                    }
                }
                _ => out.push(ch as char),
            }
        }
        Err("unterminated JSON string".to_string())
    }

    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        if self.pos + 4 > self.input.len() {
            return Err("short unicode escape".to_string());
        }
        let hex =
            std::str::from_utf8(&self.input[self.pos..self.pos + 4]).map_err(|e| e.to_string())?;
        self.pos += 4;
        let value = u16::from_str_radix(hex, 16).map_err(|e| e.to_string())?;
        char::from_u32(value as u32).ok_or_else(|| "invalid unicode escape".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        self.consume(b'-');
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.consume(b'.') {
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|e| e.to_string())?
            .parse::<f64>()
            .map_err(|e| e.to_string())
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(format!("expected JSON byte {expected}"))
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }
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

    #[test]
    fn rejects_out_of_range_indices() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        GlbWriter::write_to(&mut buf, &pd).unwrap();

        let bin_chunk_header =
            20 + u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as usize;
        let bin_start = bin_chunk_header + 8;
        let index_start = bin_start + 36;
        buf[index_start..index_start + 4].copy_from_slice(&99u32.to_le_bytes());

        assert!(GlbReader::read_from(&buf).is_err());
    }

    #[test]
    fn line_loop_appends_first_index_to_close_cell() {
        let mut bin = Vec::new();
        for point in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]] {
            for component in point {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        let indices_offset = bin.len();
        for index in [0u32, 1, 2] {
            bin.extend_from_slice(&index.to_le_bytes());
        }

        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0}},"indices":1,"mode":2}}]}}],"#,
                r#""accessors":["#,
                r#"{{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"}},"#,
                r#"{{"bufferView":1,"componentType":5125,"count":3,"type":"SCALAR"}}"#,
                r#"],"bufferViews":["#,
                r#"{{"buffer":0,"byteOffset":0,"byteLength":36,"byteStride":12}},"#,
                r#"{{"buffer":0,"byteOffset":{},"byteLength":12}}"#,
                r#"],"buffers":[{{"byteLength":{}}}]}}"#
            ),
            indices_offset,
            bin.len()
        );

        let mut glb = Vec::new();
        write_test_glb(&mut glb, json.as_bytes(), &bin);

        let result = GlbReader::read_from(&glb).unwrap();
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.lines.cell(0), &[0, 1, 2, 0]);
    }

    #[test]
    fn appends_points_for_each_primitive_position_accessor() {
        let mut bin = Vec::new();
        for point in [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [10.0, 0.0, 0.0],
            [11.0, 0.0, 0.0],
            [10.0, 1.0, 0.0],
        ] {
            for component in point {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }

        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"#,
                r#""meshes":[{{"primitives":["#,
                r#"{{"attributes":{{"POSITION":0}},"mode":4}},"#,
                r#"{{"attributes":{{"POSITION":1}},"mode":4}}"#,
                r#"]}}],"#,
                r#""accessors":["#,
                r#"{{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"}},"#,
                r#"{{"bufferView":1,"componentType":5126,"count":3,"type":"VEC3"}}"#,
                r#"],"bufferViews":["#,
                r#"{{"buffer":0,"byteOffset":0,"byteLength":36}},"#,
                r#"{{"buffer":0,"byteOffset":36,"byteLength":36}}"#,
                r#"],"buffers":[{{"byteLength":{}}}]}}"#
            ),
            bin.len()
        );

        let mut glb = Vec::new();
        write_test_glb(&mut glb, json.as_bytes(), &bin);

        let result = GlbReader::read_from(&glb).unwrap();
        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
        assert_eq!(result.polys.cell(1), &[3, 4, 5]);
    }

    fn write_test_glb(out: &mut Vec<u8>, json: &[u8], bin: &[u8]) {
        let json_padded_len = (json.len() + 3) & !3;
        let bin_padded_len = (bin.len() + 3) & !3;
        let total_len = 12 + 8 + json_padded_len + 8 + bin_padded_len;

        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&(total_len as u32).to_le_bytes());
        out.extend_from_slice(&(json_padded_len as u32).to_le_bytes());
        out.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
        out.extend_from_slice(json);
        out.resize(out.len() + json_padded_len - json.len(), b' ');
        out.extend_from_slice(&(bin_padded_len as u32).to_le_bytes());
        out.extend_from_slice(&0x004E4942u32.to_le_bytes());
        out.extend_from_slice(bin);
        out.resize(out.len() + bin_padded_len - bin.len(), 0);
    }
}
