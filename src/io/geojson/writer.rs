use crate::data::{CellArray, PolyData};
use std::io::Write;

/// Write a PolyData as GeoJSON Feature with GeometryCollection.
///
/// Polygons become MultiPolygon geometries, lines become MultiLineString
/// geometries, and vertices become MultiPoint geometries.
pub fn write_geojson<W: Write>(w: &mut W, mesh: &PolyData) -> std::io::Result<()> {
    writeln!(w, "{{")?;
    writeln!(w, "\"type\": \"Feature\",")?;
    writeln!(w, "\"properties\": {{\"ScalarFormat\": \"none\"}},")?;
    writeln!(w, "\"geometry\":")?;
    writeln!(w, "{{")?;
    writeln!(w, "\"type\": \"GeometryCollection\",")?;
    writeln!(w, "\"geometries\":")?;
    writeln!(w, "[")?;

    let mut first = true;

    if has_cells(&mesh.verts, 1) {
        write_comma_if_needed(w, &mut first)?;
        writeln!(w, "{{")?;
        writeln!(w, "\"type\": \"MultiPoint\",")?;
        writeln!(w, "\"coordinates\":")?;
        writeln!(w, "[")?;
        let mut first_point = true;
        for cell in mesh.verts.iter() {
            for &pid in cell {
                if !first_point {
                    writeln!(w, ",")?;
                } else {
                    first_point = false;
                }
                write_position(w, mesh, pid)?;
            }
        }
        writeln!(w)?;
        writeln!(w, "]")?;
        write!(w, "}}")?;
    }

    if has_cells(&mesh.lines, 2) {
        write_comma_if_needed(w, &mut first)?;
        writeln!(w, "{{")?;
        writeln!(w, "\"type\": \"MultiLineString\",")?;
        writeln!(w, "\"coordinates\":")?;
        writeln!(w, "[")?;
        let mut first_cell = true;
        for cell in mesh.lines.iter().filter(|cell| cell.len() >= 2) {
            if !first_cell {
                writeln!(w, ",")?;
            } else {
                first_cell = false;
            }
            write!(w, "[ ")?;
            for (i, &pid) in cell.iter().enumerate() {
                if i > 0 {
                    write!(w, ",")?;
                }
                write_position(w, mesh, pid)?;
            }
            write!(w, "]")?;
        }
        writeln!(w)?;
        writeln!(w, "]")?;
        write!(w, "}}")?;
    }

    if has_cells(&mesh.polys, 3) {
        write_comma_if_needed(w, &mut first)?;
        writeln!(w, "{{")?;
        writeln!(w, "\"type\": \"MultiPolygon\",")?;
        writeln!(w, "\"coordinates\":")?;
        writeln!(w, "[")?;
        let mut first_cell = true;
        for cell in mesh.polys.iter().filter(|cell| cell.len() >= 3) {
            if !first_cell {
                writeln!(w, ",")?;
            } else {
                first_cell = false;
            }
            write!(w, "[[ ")?;
            for (i, &pid) in cell.iter().enumerate() {
                if i > 0 {
                    write!(w, ",")?;
                }
                write_position(w, mesh, pid)?;
            }
            write!(w, " ]]")?;
        }
        writeln!(w)?;
        writeln!(w, "]")?;
        write!(w, "}}")?;
    }

    writeln!(w)?;
    writeln!(w, "]")?;
    writeln!(w, "}}")?;
    writeln!(w, "}}")?;
    Ok(())
}

fn has_cells(cells: &CellArray, min_len: usize) -> bool {
    cells.iter().any(|cell| cell.len() >= min_len)
}

fn write_comma_if_needed<W: Write>(w: &mut W, first: &mut bool) -> std::io::Result<()> {
    if !*first {
        writeln!(w, ",")?;
    } else {
        *first = false;
    }
    Ok(())
}

fn write_position<W: Write>(w: &mut W, mesh: &PolyData, pid: i64) -> std::io::Result<()> {
    let p = mesh.points.get(pid as usize);
    write!(w, "[")?;
    write_coordinate(w, p[0])?;
    write!(w, ",")?;
    write_coordinate(w, p[1])?;
    write!(w, ",")?;
    write_coordinate(w, p[2])?;
    write!(w, "]")
}

fn write_coordinate<W: Write>(w: &mut W, value: f64) -> std::io::Result<()> {
    if value.is_nan() {
        write!(w, "null")
    } else {
        write!(w, "{value}")
    }
}

/// Write GeoJSON to a file path.
#[allow(dead_code)]
pub fn write_geojson_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut w = std::io::BufWriter::new(file);
    write_geojson(&mut w, mesh).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_polygon() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_geojson(&mut buf, &mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"type\": \"Feature\""));
        assert!(s.contains("GeometryCollection"));
        assert!(s.contains("MultiPolygon"));
    }

    #[test]
    fn write_line() {
        let mesh = PolyData::from_lines(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]], vec![[0, 1]]);
        let mut buf = Vec::new();
        write_geojson(&mut buf, &mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("MultiLineString"));
    }

    #[test]
    fn write_nan_coordinate_as_null() {
        let mesh = PolyData::from_triangles(
            vec![[f64::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_geojson(&mut buf, &mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("[null,0,0]"));
        assert!(!s.contains("NaN"));
    }
}
