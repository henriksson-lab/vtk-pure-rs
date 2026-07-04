use crate::data::{CellArray, Points, PolyData};
use std::collections::BTreeMap;
use std::io::Read;

/// Read GeoJSON and return a PolyData.
///
/// Supports Feature, FeatureCollection, Point, MultiPoint, LineString,
/// MultiLineString, Polygon, MultiPolygon, and GeometryCollection geometries.
pub fn read_geojson<R: Read>(reader: &mut R) -> Result<PolyData, String> {
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;

    let root = JsonParser::new(&text).parse()?;
    let mut builder = GeoJsonBuilder::default();
    parse_root(&root, &mut builder)?;

    let mut mesh = PolyData::new();
    mesh.points = builder.points;
    mesh.verts = builder.verts;
    mesh.lines = builder.lines;
    mesh.polys = builder.polys;
    Ok(mesh)
}

#[derive(Default)]
struct GeoJsonBuilder {
    points: Points<f64>,
    verts: CellArray,
    lines: CellArray,
    polys: CellArray,
}

fn parse_root(root: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    match object_string(root, "type").as_deref() {
        Some("FeatureCollection") => {
            let features = object_member(root, "features")
                .and_then(JsonValue::as_array)
                .ok_or_else(|| "FeatureCollection missing features array".to_string())?;
            for feature in features {
                parse_feature(feature, builder)?;
            }
        }
        Some("Feature") => parse_feature(root, builder)?,
        Some(
            "Point" | "MultiPoint" | "LineString" | "MultiLineString" | "Polygon" | "MultiPolygon"
            | "GeometryCollection",
        ) => parse_geometry(root, builder)?,
        Some(other) => return Err(format!("unsupported GeoJSON root type {other}")),
        None => return Err("GeoJSON root missing type".to_string()),
    }
    Ok(())
}

fn parse_feature(feature: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    if object_string(feature, "type").as_deref() != Some("Feature") {
        return Err("Feature expected".to_string());
    }
    let geometry =
        object_member(feature, "geometry").ok_or_else(|| "Feature missing geometry".to_string())?;
    if !matches!(geometry, JsonValue::Null) {
        parse_geometry(geometry, builder)?;
    }
    Ok(())
}

fn parse_geometry(geometry: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    let geom_type =
        object_string(geometry, "type").ok_or_else(|| "geometry missing type".to_string())?;

    if geom_type == "GeometryCollection" {
        let geometries = object_member(geometry, "geometries")
            .and_then(JsonValue::as_array)
            .ok_or_else(|| "GeometryCollection missing geometries array".to_string())?;
        for child in geometries {
            parse_geometry(child, builder)?;
        }
        return Ok(());
    }

    let coordinates = object_member(geometry, "coordinates")
        .ok_or_else(|| format!("{geom_type} missing coordinates"))?;
    match geom_type.as_str() {
        "Point" => extract_point(coordinates, builder),
        "MultiPoint" => extract_multi_point(coordinates, builder),
        "LineString" => extract_line_string(coordinates, builder),
        "MultiLineString" => extract_multi_line_string(coordinates, builder),
        "Polygon" => extract_polygon(coordinates, builder),
        "MultiPolygon" => extract_multi_polygon(coordinates, builder),
        _ => Err(format!("unsupported GeoJSON geometry type {geom_type}")),
    }
}

fn extract_point(value: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    let point = parse_position(value)?;
    let id = builder.points.len() as i64;
    builder.points.push(point);
    builder.verts.push_cell(&[id]);
    Ok(())
}

fn extract_multi_point(value: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    let positions = value
        .as_array()
        .ok_or_else(|| "MultiPoint coordinates must be an array".to_string())?;
    let mut ids = Vec::with_capacity(positions.len());
    for position in positions {
        let point = parse_position(position)?;
        let id = builder.points.len() as i64;
        builder.points.push(point);
        ids.push(id);
    }
    if !ids.is_empty() {
        builder.verts.push_cell(&ids);
    }
    Ok(())
}

fn extract_line_string(value: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    let ids = insert_positions(value, &mut builder.points)?;
    if !ids.is_empty() {
        builder.lines.push_cell(&ids);
    }
    Ok(())
}

fn extract_multi_line_string(
    value: &JsonValue,
    builder: &mut GeoJsonBuilder,
) -> Result<(), String> {
    let lines = value
        .as_array()
        .ok_or_else(|| "MultiLineString coordinates must be an array".to_string())?;
    for line in lines {
        extract_line_string(line, builder)?;
    }
    Ok(())
}

fn extract_polygon(value: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    let rings = value
        .as_array()
        .ok_or_else(|| "Polygon coordinates must be an array".to_string())?;
    let Some(exterior) = rings.first() else {
        return Err("Polygon must contain an exterior ring".to_string());
    };
    let ids = insert_ring_positions(exterior, &mut builder.points)?;
    if ids.len() >= 3 {
        builder.polys.push_cell(&ids);
    }
    Ok(())
}

fn extract_multi_polygon(value: &JsonValue, builder: &mut GeoJsonBuilder) -> Result<(), String> {
    let polygons = value
        .as_array()
        .ok_or_else(|| "MultiPolygon coordinates must be an array".to_string())?;
    for polygon in polygons {
        extract_polygon(polygon, builder)?;
    }
    Ok(())
}

fn insert_positions(value: &JsonValue, points: &mut Points<f64>) -> Result<Vec<i64>, String> {
    let positions = value
        .as_array()
        .ok_or_else(|| "coordinate list must be an array".to_string())?;
    let mut ids = Vec::with_capacity(positions.len());
    for position in positions {
        let point = parse_position(position)?;
        let id = points.len() as i64;
        points.push(point);
        ids.push(id);
    }
    Ok(ids)
}

fn insert_ring_positions(value: &JsonValue, points: &mut Points<f64>) -> Result<Vec<i64>, String> {
    let positions = value
        .as_array()
        .ok_or_else(|| "coordinate list must be an array".to_string())?;
    let n = if positions.len() > 1 && same_point_id_positions(value)? {
        positions.len() - 1
    } else {
        positions.len()
    };
    let mut ids = Vec::with_capacity(n);
    for position in &positions[..n] {
        let point = parse_position(position)?;
        let id = points.len() as i64;
        points.push(point);
        ids.push(id);
    }
    Ok(ids)
}

fn parse_position(value: &JsonValue) -> Result<[f64; 3], String> {
    let coords = value
        .as_array()
        .ok_or_else(|| "position must be an array".to_string())?;
    if coords.is_empty() || coords.len() > 3 {
        return Err(format!("position has {} dimensions", coords.len()));
    }
    let x = coords[0]
        .as_f64()
        .ok_or_else(|| "position x must be numeric".to_string())?;
    let y = match coords.get(1) {
        Some(value) => value
            .as_f64()
            .ok_or_else(|| "position y must be numeric".to_string())?,
        None => 0.0,
    };
    let z = match coords.get(2) {
        Some(value) => value
            .as_f64()
            .ok_or_else(|| "position z must be numeric".to_string())?,
        None => 0.0,
    };
    Ok([x, y, z])
}

fn same_point_id_positions(value: &JsonValue) -> Result<bool, String> {
    let positions = value
        .as_array()
        .ok_or_else(|| "coordinate list must be an array".to_string())?;
    let (Some(first), Some(last)) = (positions.first(), positions.last()) else {
        return Ok(false);
    };
    Ok(parse_position(first)? == parse_position(last)?)
}

fn object_member<'a>(value: &'a JsonValue, name: &str) -> Option<&'a JsonValue> {
    value.as_object()?.get(name)
}

fn object_string(value: &JsonValue, name: &str) -> Option<String> {
    object_member(value, name)?.as_string().map(str::to_string)
}

/// Read GeoJSON from a file path.
#[allow(dead_code)]
pub fn read_geojson_file(path: &std::path::Path) -> Result<PolyData, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_geojson(&mut file)
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

    fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
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

    #[test]
    fn read_polygon() {
        let json = r#"{"type":"FeatureCollection","features":[
            {"type":"Feature","geometry":{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]},"properties":{}}
        ]}"#;
        let mesh = read_geojson(&mut json.as_bytes()).unwrap();
        assert_eq!(mesh.points.len(), 4);
        assert_eq!(mesh.polys.num_cells(), 1);
        assert_eq!(mesh.polys.cell(0).len(), 4);
    }

    #[test]
    fn read_linestring() {
        let json = r#"{"type":"FeatureCollection","features":[
            {"type":"Feature","geometry":{"type":"LineString","coordinates":[[0,0],[1,1],[2,0]]},"properties":{}}
        ]}"#;
        let mesh = read_geojson(&mut json.as_bytes()).unwrap();
        assert_eq!(mesh.points.len(), 3);
        assert_eq!(mesh.lines.num_cells(), 1);
    }

    #[test]
    fn read_point_creates_vertex_cell() {
        let json = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[1,2,3]},"properties":{}}"#;
        let mesh = read_geojson(&mut json.as_bytes()).unwrap();
        assert_eq!(mesh.points.len(), 1);
        assert_eq!(mesh.verts.num_cells(), 1);
        assert_eq!(mesh.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn read_top_level_geometry() {
        let json = r#"{"type":"LineString","coordinates":[[0,0],[1,1]]}"#;
        let mesh = read_geojson(&mut json.as_bytes()).unwrap();
        assert_eq!(mesh.points.len(), 2);
        assert_eq!(mesh.lines.num_cells(), 1);
    }

    #[test]
    fn rejects_nonnumeric_position_components() {
        let json = r#"{"type":"Point","coordinates":[1,"bad"]}"#;
        let err = read_geojson(&mut json.as_bytes()).unwrap_err();
        assert_eq!(err, "position y must be numeric");
    }

    #[test]
    fn read_multi_geometry() {
        let json = r#"{"type":"Feature","geometry":{"type":"GeometryCollection","geometries":[
            {"type":"MultiPoint","coordinates":[[0,0],[1,1]]},
            {"type":"MultiLineString","coordinates":[[[0,0],[1,0]],[[2,0],[3,0]]]},
            {"type":"MultiPolygon","coordinates":[[[[0,0],[1,0],[1,1],[0,0]]]]}
        ]},"properties":{}}"#;
        let mesh = read_geojson(&mut json.as_bytes()).unwrap();
        assert_eq!(mesh.verts.num_cells(), 1);
        assert_eq!(mesh.lines.num_cells(), 2);
        assert_eq!(mesh.polys.num_cells(), 1);
    }

    #[test]
    fn roundtrip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        crate::io::geojson::write_geojson(&mut buf, &mesh).unwrap();
        let loaded = read_geojson(&mut &buf[..]).unwrap();
        assert_eq!(loaded.polys.num_cells(), 1);
        assert!(loaded.points.len() >= 3);
    }

    #[test]
    fn empty_collection() {
        let json = r#"{"type":"FeatureCollection","features":[]}"#;
        let mesh = read_geojson(&mut json.as_bytes()).unwrap();
        assert_eq!(mesh.points.len(), 0);
    }
}
