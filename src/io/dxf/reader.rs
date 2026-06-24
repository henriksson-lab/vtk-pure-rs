use crate::data::{CellArray, Points, PolyData};
use std::io::BufRead;

/// Reader for AutoCAD DXF format (3DFACE and LINE entities).
pub struct DxfReader<R: BufRead> {
    reader: R,
}

impl<R: BufRead> DxfReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Read a DXF file and return a PolyData mesh.
    pub fn read(&mut self) -> Result<PolyData, String> {
        let mut lines = Vec::new();
        let mut buf = String::new();
        loop {
            buf.clear();
            let n = self.reader.read_line(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            lines.push(buf.trim().to_string());
        }

        let mut points = Points::<f64>::new();
        let mut polys = CellArray::new();
        let mut line_cells = CellArray::new();
        let mut point_map: std::collections::HashMap<[i64; 3], usize> =
            std::collections::HashMap::new();

        let mut i = 0;
        while i < lines.len() {
            if lines[i] == "0" && i + 1 < lines.len() {
                let entity = &lines[i + 1];
                if entity == "3DFACE" {
                    i += 2;
                    let (face_pts, consumed) = parse_3dface(&lines[i..]);
                    i += consumed;

                    let mut ids = Vec::new();
                    for p in &face_pts {
                        let key = quantize(*p);
                        let idx = *point_map.entry(key).or_insert_with(|| {
                            let idx = points.len();
                            points.push(*p);
                            idx
                        });
                        ids.push(idx as i64);
                    }
                    // Deduplicate (3DFACE repeats p2 for triangles)
                    ids.dedup();
                    if ids.len() >= 3 {
                        polys.push_cell(&ids);
                    }
                } else if entity == "LINE" {
                    i += 2;
                    let (line_pts, consumed) = parse_line(&lines[i..]);
                    i += consumed;

                    if line_pts.len() == 2 {
                        let mut ids = Vec::new();
                        for p in &line_pts {
                            let key = quantize(*p);
                            let idx = *point_map.entry(key).or_insert_with(|| {
                                let idx = points.len();
                                points.push(*p);
                                idx
                            });
                            ids.push(idx as i64);
                        }
                        line_cells.push_cell(&ids);
                    }
                } else {
                    i += 2;
                }
            } else {
                i += 1;
            }
        }

        let mut mesh = PolyData::new();
        mesh.points = points;
        mesh.polys = polys;
        mesh.lines = line_cells;
        Ok(mesh)
    }
}

fn parse_3dface(lines: &[String]) -> (Vec<[f64; 3]>, usize) {
    let mut pts = [[0.0; 3]; 4];
    let mut i = 0;

    while i + 1 < lines.len() {
        let code: i32 = match lines[i].trim().parse() {
            Ok(c) => c,
            Err(_) => break,
        };
        if code == 0 {
            break;
        } // next entity

        let value = &lines[i + 1];
        i += 2;

        match code {
            10 => pts[0][0] = value.parse().unwrap_or(0.0),
            20 => pts[0][1] = value.parse().unwrap_or(0.0),
            30 => pts[0][2] = value.parse().unwrap_or(0.0),
            11 => pts[1][0] = value.parse().unwrap_or(0.0),
            21 => pts[1][1] = value.parse().unwrap_or(0.0),
            31 => pts[1][2] = value.parse().unwrap_or(0.0),
            12 => pts[2][0] = value.parse().unwrap_or(0.0),
            22 => pts[2][1] = value.parse().unwrap_or(0.0),
            32 => pts[2][2] = value.parse().unwrap_or(0.0),
            13 => pts[3][0] = value.parse().unwrap_or(0.0),
            23 => pts[3][1] = value.parse().unwrap_or(0.0),
            33 => pts[3][2] = value.parse().unwrap_or(0.0),
            _ => {}
        }
    }

    (pts.to_vec(), i)
}

fn parse_line(lines: &[String]) -> (Vec<[f64; 3]>, usize) {
    let mut pts = [[0.0; 3]; 2];
    let mut i = 0;

    while i + 1 < lines.len() {
        let code: i32 = match lines[i].trim().parse() {
            Ok(c) => c,
            Err(_) => break,
        };
        if code == 0 {
            break;
        }

        let value = &lines[i + 1];
        i += 2;

        match code {
            10 => pts[0][0] = value.parse().unwrap_or(0.0),
            20 => pts[0][1] = value.parse().unwrap_or(0.0),
            30 => pts[0][2] = value.parse().unwrap_or(0.0),
            11 => pts[1][0] = value.parse().unwrap_or(0.0),
            21 => pts[1][1] = value.parse().unwrap_or(0.0),
            31 => pts[1][2] = value.parse().unwrap_or(0.0),
            _ => {}
        }
    }

    (pts.to_vec(), i)
}

fn quantize(p: [f64; 3]) -> [i64; 3] {
    [
        (p[0] * 1e8) as i64,
        (p[1] * 1e8) as i64,
        (p[2] * 1e8) as i64,
    ]
}

/// Read a DXF file from a file path.
pub fn read_dxf_file(path: &std::path::Path) -> Result<PolyData, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    DxfReader::new(std::io::BufReader::new(file)).read()
}

/// Write a DXF file to a file path.
pub fn write_dxf_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    crate::io::dxf::DxfWriter::new(std::io::BufWriter::new(file))
        .write(mesh)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_triangle() {
        let mesh = PolyData::from_triangles(
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        crate::io::dxf::DxfWriter::new(&mut buf)
            .write(&mesh)
            .unwrap();

        let loaded = DxfReader::new(&buf[..]).read().unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);

        let p = loaded.points.get(0);
        assert!((p[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn roundtrip_quad() {
        let mesh = PolyData::from_quads(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2, 3]],
        );
        let mut buf = Vec::new();
        crate::io::dxf::DxfWriter::new(&mut buf)
            .write(&mesh)
            .unwrap();

        let loaded = DxfReader::new(&buf[..]).read().unwrap();
        assert_eq!(loaded.points.len(), 4);
        assert_eq!(loaded.polys.num_cells(), 1);
    }
}
