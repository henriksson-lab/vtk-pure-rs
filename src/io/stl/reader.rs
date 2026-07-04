use std::collections::HashMap;
use std::path::Path;

use crate::data::{CellArray, Points, PolyData};
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
        // VTK treats an STL whose first five bytes are "solid" as ASCII first,
        // then falls back to binary for malformed files with a binary header that
        // happens to start with that word.
        if data.len() >= 5 && &data[..5] == b"solid" {
            match Self::read_ascii(data) {
                Ok(poly_data) => return Ok(poly_data),
                Err(_) => {
                    return Self::read_binary(data);
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

        #[derive(Clone, Copy)]
        enum ScanState {
            Solid,
            Facet,
            Loop,
            Verts,
            EndLoop,
            EndFacet,
        }

        let mut state = ScanState::Solid;
        let mut solid_seen = false;
        let mut vert_off = 0usize;
        let mut tri_points = [[0.0; 3]; 3];

        for raw_line in text.lines() {
            let line = raw_line.trim_start();
            if line.is_empty() {
                continue;
            }
            let mut split = line.splitn(2, char::is_whitespace);
            let cmd = split.next().unwrap_or("").to_ascii_lowercase();
            let arg = split.next().unwrap_or("").trim_start();

            match state {
                ScanState::Solid => {
                    if cmd != "solid" {
                        return Err(parse_expected("solid", &cmd));
                    }
                    solid_seen = true;
                    state = ScanState::Facet;
                }
                ScanState::Facet => {
                    if cmd == "color" {
                        continue;
                    }
                    if cmd == "facet" {
                        state = ScanState::Loop;
                    } else if cmd == "endsolid" {
                        state = ScanState::Solid;
                    } else {
                        return Err(parse_expected("facet", &cmd));
                    }
                }
                ScanState::Loop => {
                    if cmd != "outer" {
                        return Err(parse_expected("outer loop", &cmd));
                    }
                    state = ScanState::Verts;
                    vert_off = 0;
                }
                ScanState::Verts => {
                    if cmd != "vertex" {
                        return Err(parse_expected("vertex", &cmd));
                    }
                    tri_points[vert_off] = read_vertex(arg)?;
                    vert_off += 1;
                    if vert_off == 3 {
                        state = ScanState::EndLoop;
                    }
                }
                ScanState::EndLoop => {
                    if cmd != "endloop" {
                        return Err(parse_expected("endloop", &cmd));
                    }
                    state = ScanState::EndFacet;
                }
                ScanState::EndFacet => {
                    if cmd != "endfacet" {
                        return Err(parse_expected("endfacet", &cmd));
                    }
                    let base = points.len() as i64;
                    for p in tri_points {
                        points.push(p);
                    }
                    polys.push_cell(&[base, base + 1, base + 2]);
                    state = ScanState::Facet;
                }
            }
        }

        if !solid_seen {
            return Err(VtkError::Parse(
                "Premature EOF while reading 'solid'".into(),
            ));
        }
        match state {
            ScanState::Solid | ScanState::Facet => {}
            ScanState::Loop => {
                return Err(VtkError::Parse(
                    "Premature EOF while reading 'outer loop'".into(),
                ))
            }
            ScanState::Verts => {
                return Err(VtkError::Parse(
                    "Premature EOF while reading 'vertex'".into(),
                ))
            }
            ScanState::EndLoop => {
                return Err(VtkError::Parse(
                    "Premature EOF while reading 'endloop'".into(),
                ))
            }
            ScanState::EndFacet => {
                return Err(VtkError::Parse(
                    "Premature EOF while reading 'endfacet'".into(),
                ))
            }
        }

        Ok(merge_points(points, polys))
    }

    fn read_binary(data: &[u8]) -> Result<PolyData, VtkError> {
        if data.len() < 84 {
            return Err(VtkError::Parse("STL binary too short".into()));
        }

        let _num_triangles_field = u32::from_le_bytes(data[80..84].try_into().unwrap()) as usize;
        let remaining = data.len() - 84;

        if remaining % 50 != 0 {
            return Err(VtkError::Parse(
                "STL binary remaining file length bad".into(),
            ));
        }
        let num_triangles_file = remaining / 50;
        let mut points = Points::<f64>::new();
        let mut polys = CellArray::new();

        let mut offset = 84;
        for _ in 0..num_triangles_file {
            // Normal: 3 x f32 LE
            let nx = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as f64;
            let ny = f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as f64;
            let nz = f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as f64;
            offset += 12;
            if !nx.is_finite() || !ny.is_finite() || !nz.is_finite() {
                return Err(VtkError::Parse("Normal vector non-finite".into()));
            }

            let base = points.len() as i64;
            // 3 vertices: each 3 x f32 LE
            for _ in 0..3 {
                let x = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as f64;
                let y = f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap()) as f64;
                let z =
                    f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as f64;
                offset += 12;
                if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                    return Err(VtkError::Parse("vertex non-finite".into()));
                }
                points.push([x, y, z]);
            }
            // Attribute byte count (skip)
            offset += 2;

            polys.push_cell(&[base, base + 1, base + 2]);
        }

        Ok(merge_points(points, polys))
    }
}

fn merge_points(points: Points<f64>, polys: CellArray) -> PolyData {
    let mut merged_points = Points::<f64>::new();
    let mut merged_polys = CellArray::new();
    let mut locator = HashMap::<[u64; 3], i64>::new();

    for cell in polys.iter() {
        let mut nodes = [0_i64; 3];
        for (i, &point_id) in cell.iter().take(3).enumerate() {
            let point = points.get(point_id as usize);
            let key = point_key(point);
            let next_id = merged_points.len() as i64;
            let node = *locator.entry(key).or_insert_with(|| {
                merged_points.push(point);
                next_id
            });
            nodes[i] = node;
        }

        if nodes[0] != nodes[1] && nodes[0] != nodes[2] && nodes[1] != nodes[2] {
            merged_polys.push_cell(&nodes);
        }
    }

    let mut pd = PolyData::new();
    pd.points = merged_points;
    pd.polys = merged_polys;
    pd
}

fn point_key(point: [f64; 3]) -> [u64; 3] {
    point.map(|v| if v == 0.0 { 0.0 } else { v }.to_bits())
}

fn parse_expected(expected: &str, found: &str) -> VtkError {
    VtkError::Parse(format!(
        "Parse error. Expecting '{}' found '{}'",
        expected, found
    ))
}

fn read_vertex(arg: &str) -> Result<[f64; 3], VtkError> {
    let values: Vec<f64> = arg
        .split_whitespace()
        .take(3)
        .map(|s| {
            s.parse::<f64>()
                .map_err(|_| VtkError::Parse("Parse error. Expecting 'vertex'".into()))
        })
        .collect::<Result<_, _>>()?;
    if values.len() != 3 {
        return Err(VtkError::Parse("Parse error. Expecting 'vertex'".into()));
    }
    Ok([values[0], values[1], values[2]])
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
        assert_eq!(result.points.len(), 4);

        // Check first vertex (f32 precision)
        let p0 = result.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-5);
        assert!((p0[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn ascii_allows_color_before_facet() {
        let data = b"solid colored\ncolor 1 0 0\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\nendloop\nendfacet\nendsolid\n";
        let result = StlReader::read_from(data).unwrap();
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn binary_uses_file_size_when_count_mismatches() {
        let mut data = vec![0u8; 84];
        data[80..84].copy_from_slice(&1u32.to_le_bytes());
        let result = StlReader::read_from(&data).unwrap();
        assert_eq!(result.polys.num_cells(), 0);
    }
}
