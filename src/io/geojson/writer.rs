use crate::data::PolyData;
use std::io::Write;

/// Write a PolyData as GeoJSON FeatureCollection.
///
/// Polygons become Polygon features, lines become LineString features,
/// and lone vertices become Point features.
pub fn write_geojson<W: Write>(w: &mut W, mesh: &PolyData) -> std::io::Result<()> {
    writeln!(w, "{{")?;
    writeln!(w, "  \"type\": \"FeatureCollection\",")?;
    writeln!(w, "  \"features\": [")?;

    let mut first = true;

    // Write polygon features
    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        if !first {
            writeln!(w, ",")?;
        } else {
            first = false;
        }
        write!(
            w,
            "    {{\"type\":\"Feature\",\"geometry\":{{\"type\":\"Polygon\",\"coordinates\":[["
        )?;
        for (i, &pid) in cell.iter().enumerate() {
            let p = mesh.points.get(pid as usize);
            if i > 0 {
                write!(w, ",")?;
            }
            write!(w, "[{},{}]", p[0], p[1])?;
        }
        // Close the ring
        let p0 = mesh.points.get(cell[0] as usize);
        write!(w, ",[{},{}]", p0[0], p0[1])?;
        write!(w, "]]}},\"properties\":{{}}}}")?;
    }

    // Write line features
    for cell in mesh.lines.iter() {
        if cell.len() < 2 {
            continue;
        }
        if !first {
            writeln!(w, ",")?;
        } else {
            first = false;
        }
        write!(
            w,
            "    {{\"type\":\"Feature\",\"geometry\":{{\"type\":\"LineString\",\"coordinates\":["
        )?;
        for (i, &pid) in cell.iter().enumerate() {
            let p = mesh.points.get(pid as usize);
            if i > 0 {
                write!(w, ",")?;
            }
            write!(w, "[{},{}]", p[0], p[1])?;
        }
        write!(w, "]}},\"properties\":{{}}}}")?;
    }

    // Write vertex features as points
    for cell in mesh.verts.iter() {
        for &pid in cell {
            if !first {
                writeln!(w, ",")?;
            } else {
                first = false;
            }
            let p = mesh.points.get(pid as usize);
            write!(w, "    {{\"type\":\"Feature\",\"geometry\":{{\"type\":\"Point\",\"coordinates\":[{},{}]}},\"properties\":{{}}}}", p[0], p[1])?;
        }
    }

    writeln!(w)?;
    writeln!(w, "  ]")?;
    writeln!(w, "}}")?;
    Ok(())
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
        assert!(s.contains("FeatureCollection"));
        assert!(s.contains("Polygon"));
    }

    #[test]
    fn write_line() {
        let mesh = PolyData::from_lines(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]], vec![[0, 1]]);
        let mut buf = Vec::new();
        write_geojson(&mut buf, &mesh).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("LineString"));
    }
}
