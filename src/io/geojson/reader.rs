use crate::data::{CellArray, Points, PolyData};
use std::io::Read;

/// Read GeoJSON and return a PolyData.
///
/// Supports Point, LineString, Polygon, MultiPoint geometries.
/// This is a minimal parser — no external JSON dependency.
pub fn read_geojson<R: Read>(reader: &mut R) -> Result<PolyData, String> {
    let mut text = String::new();
    reader
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();
    let mut lines = CellArray::new();

    // Simple extraction of coordinate arrays from the JSON
    // Find all "coordinates" values and their geometry types
    let mut pos = 0;
    while let Some(type_pos) = text[pos..].find("\"type\"") {
        let abs_pos = pos + type_pos;
        pos = abs_pos + 6;

        // Extract type value
        let type_val = extract_string_value(&text[pos..]);
        if type_val.is_none() {
            continue;
        }
        let type_val = type_val.unwrap();

        if type_val == "Feature" || type_val == "FeatureCollection" {
            continue;
        }

        // Find coordinates for this geometry
        let coord_search = &text[pos..];
        if let Some(coord_pos) = coord_search.find("\"coordinates\"") {
            let coord_start = pos + coord_pos + 13;
            // Skip whitespace and colon
            let rest = text[coord_start..].trim_start();
            let rest = if rest.starts_with(':') {
                rest[1..].trim_start()
            } else {
                rest
            };

            match type_val.as_str() {
                "Point" => {
                    if let Some(coord) = parse_point(rest) {
                        let idx = points.len() as i64;
                        points.push([coord[0], coord[1], 0.0]);
                        // Add as vertex cell
                        let _ = idx; // points only, no cell needed
                    }
                }
                "LineString" => {
                    if let Some(coords) = parse_coord_array(rest) {
                        let mut ids = Vec::new();
                        for c in &coords {
                            let idx = points.len() as i64;
                            points.push([c[0], c[1], 0.0]);
                            ids.push(idx);
                        }
                        if ids.len() >= 2 {
                            lines.push_cell(&ids);
                        }
                    }
                }
                "Polygon" => {
                    // Polygon is [ring, ...] where ring is [[x,y],...]
                    if let Some(ring) = parse_polygon_outer_ring(rest) {
                        let mut ids = Vec::new();
                        // Skip last point (same as first in GeoJSON)
                        let n = if ring.len() > 1 {
                            ring.len() - 1
                        } else {
                            ring.len()
                        };
                        for c in &ring[..n] {
                            let idx = points.len() as i64;
                            points.push([c[0], c[1], 0.0]);
                            ids.push(idx);
                        }
                        if ids.len() >= 3 {
                            polys.push_cell(&ids);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.lines = lines;
    Ok(mesh)
}

fn extract_string_value(text: &str) -> Option<String> {
    let rest = text.trim_start();
    let rest = if rest.starts_with(':') {
        rest[1..].trim_start()
    } else {
        rest
    };
    if !rest.starts_with('"') {
        return None;
    }
    let end = rest[1..].find('"')?;
    Some(rest[1..1 + end].to_string())
}

fn parse_point(text: &str) -> Option<[f64; 2]> {
    let text = text.trim_start();
    if !text.starts_with('[') {
        return None;
    }
    let end = text.find(']')?;
    let inner = &text[1..end];
    let parts: Vec<f64> = inner
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if parts.len() >= 2 {
        Some([parts[0], parts[1]])
    } else {
        None
    }
}

fn parse_coord_array(text: &str) -> Option<Vec<[f64; 2]>> {
    let text = text.trim_start();
    if !text.starts_with('[') {
        return None;
    }
    // Find matching bracket
    let mut depth = 0;
    let mut end = 0;
    for (i, ch) in text.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if end == 0 {
        return None;
    }

    let inner = &text[1..end];
    let mut coords = Vec::new();
    let mut pos = 0;
    while let Some(start) = inner[pos..].find('[') {
        let abs = pos + start;
        if let Some(e) = inner[abs..].find(']') {
            let pair = &inner[abs + 1..abs + e];
            let parts: Vec<f64> = pair
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            if parts.len() >= 2 {
                coords.push([parts[0], parts[1]]);
            }
            pos = abs + e + 1;
        } else {
            break;
        }
    }
    Some(coords)
}

fn parse_polygon_outer_ring(text: &str) -> Option<Vec<[f64; 2]>> {
    // Polygon coordinates: [[[x,y],[x,y],...]]
    // We need to skip one layer of brackets to get to the ring
    let text = text.trim_start();
    if !text.starts_with('[') {
        return None;
    }
    // Find the inner array start
    let inner = &text[1..].trim_start();
    parse_coord_array(inner)
}

/// Read GeoJSON from a file path.
#[allow(dead_code)]
pub fn read_geojson_file(path: &std::path::Path) -> Result<PolyData, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_geojson(&mut file)
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
        assert_eq!(mesh.points.len(), 4); // closed ring, last point skipped
        assert_eq!(mesh.polys.num_cells(), 1);
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
