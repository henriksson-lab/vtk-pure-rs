use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::VtkError;

/// Reader for EnSight Gold format (ASCII).
///
/// Reads a `.case` file, then loads the referenced `.geo` geometry file
/// and optional variable files.
pub struct EnSightReader;

impl EnSightReader {
    /// Read an EnSight Gold case file and return the PolyData mesh.
    pub fn read(case_path: &Path) -> Result<PolyData, VtkError> {
        let case_dir = case_path.parent().unwrap_or(Path::new("."));
        let case_content = std::fs::read_to_string(case_path)?;

        let geo_name = parse_case_geo(&case_content)?;
        let variables = parse_case_variables(&case_content);

        let geo_path = case_dir.join(&geo_name);
        let mut pd = read_geometry(&geo_path)?;

        for (var_type, var_name, var_file) in &variables {
            let var_path = case_dir.join(var_file);
            match var_type.as_str() {
                "scalar" => {
                    let arr = read_scalar_variable(&var_path, var_name, pd.points.len())?;
                    pd.point_data_mut().add_array(arr);
                }
                "vector" => {
                    let arr = read_vector_variable(&var_path, var_name, pd.points.len())?;
                    pd.point_data_mut().add_array(arr);
                }
                _ => {}
            }
        }

        Ok(pd)
    }
}

fn parse_case_geo(content: &str) -> Result<String, VtkError> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("model:") {
            let after_colon = trimmed.split_once(':').map(|(_, rest)| rest).unwrap_or("");
            let parts: Vec<&str> = after_colon.split_whitespace().collect();
            match parts.as_slice() {
                [file] => return Ok(unquote_filename(file)),
                [time_set, file] if time_set.parse::<i32>().is_ok() => {
                    return Ok(unquote_filename(file));
                }
                [time_set, file_set, file, ..]
                    if time_set.parse::<i32>().is_ok() && file_set.parse::<i32>().is_ok() =>
                {
                    return Ok(unquote_filename(file));
                }
                _ => {}
            }
        }
    }
    Err(VtkError::Parse(
        "no geometry model found in case file".into(),
    ))
}

fn parse_case_variables(content: &str) -> Vec<(String, String, String)> {
    let mut vars = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("scalar per node:") || trimmed.starts_with("vector per node:") {
            let var_type = if trimmed.starts_with("scalar") {
                "scalar"
            } else {
                "vector"
            };
            let after_colon = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
            let parts: Vec<&str> = after_colon.split_whitespace().collect();
            let (name_idx, file_idx) = if parts.len() >= 4
                && parts[0].parse::<i32>().is_ok()
                && parts[1].parse::<i32>().is_ok()
            {
                (2, 3)
            } else if parts.len() >= 3 && parts[0].parse::<i32>().is_ok() {
                (1, 2)
            } else {
                (0, 1)
            };
            if parts.len() > file_idx {
                vars.push((
                    var_type.to_string(),
                    parts[name_idx].to_string(),
                    unquote_filename(parts[file_idx]),
                ));
            }
        }
    }
    vars
}

fn unquote_filename(filename: &str) -> String {
    filename.chars().filter(|&c| c != '"').collect()
}

fn read_geometry(path: &Path) -> Result<PolyData, VtkError> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut pd = PolyData::new();
    let mut i = 0;
    let node_ids_listed = lines
        .iter()
        .take_while(|line| !line.trim().starts_with("coordinates"))
        .any(|line| {
            let trimmed = line.trim();
            trimmed == "node id given" || trimmed == "node id ignore"
        });

    // Skip header lines until "coordinates"
    while i < lines.len() && !lines[i].trim().starts_with("coordinates") {
        i += 1;
    }
    if i >= lines.len() {
        return Err(VtkError::Parse("no coordinates section found".into()));
    }
    i += 1;

    // Number of points
    let n_pts: usize = lines
        .get(i)
        .ok_or_else(|| VtkError::Parse("missing point count".into()))?
        .trim()
        .parse()
        .map_err(|_| VtkError::Parse("invalid point count".into()))?;
    i += 1;
    if node_ids_listed {
        i += n_pts;
    }

    // Read X, Y, Z coordinates separately
    let mut xs = Vec::with_capacity(n_pts);
    let mut ys = Vec::with_capacity(n_pts);
    let mut zs = Vec::with_capacity(n_pts);

    for point in 0..n_pts {
        let v = parse_f64_line(lines.get(i), "x coordinate", point)?;
        xs.push(v);
        i += 1;
    }
    for point in 0..n_pts {
        let v = parse_f64_line(lines.get(i), "y coordinate", point)?;
        ys.push(v);
        i += 1;
    }
    for point in 0..n_pts {
        let v = parse_f64_line(lines.get(i), "z coordinate", point)?;
        zs.push(v);
        i += 1;
    }

    let mut points = Points::new();
    for j in 0..n_pts {
        points.push([xs[j], ys[j], zs[j]]);
    }
    pd.points = points;

    // Read element sections
    while i < lines.len() {
        let line = lines[i].trim();
        if line == "tria3" {
            i += 1;
            let n_cells: usize = lines
                .get(i)
                .ok_or_else(|| VtkError::Parse("missing tria3 cell count".into()))?
                .trim()
                .parse()
                .map_err(|_| VtkError::Parse("invalid tria3 cell count".into()))?;
            i += 1;
            let mut polys = CellArray::new();
            if let Some(line) = lines.get(i) {
                if parse_i64_fields(line)
                    .map(|parts| parts.len() < 3)
                    .unwrap_or(true)
                {
                    i += n_cells;
                }
            }
            for cell in 0..n_cells {
                let line = lines
                    .get(i)
                    .ok_or_else(|| VtkError::Parse(format!("missing tria3 cell {cell}")))?;
                let parts = parse_i64_fields(line)
                    .map_err(|_| VtkError::Parse(format!("invalid tria3 cell: {line}")))?;
                if parts.len() < 3 {
                    return Err(VtkError::Parse(format!("invalid tria3 cell: {line}")));
                }
                // Convert from 1-based to 0-based
                polys.push_cell(&[parts[0] - 1, parts[1] - 1, parts[2] - 1]);
                i += 1;
            }
            pd.polys = polys;
        } else {
            i += 1;
        }
    }

    Ok(pd)
}

fn parse_i64_fields(line: &str) -> Result<Vec<i64>, std::num::ParseIntError> {
    line.split_whitespace().map(|s| s.parse::<i64>()).collect()
}

fn read_scalar_variable(path: &Path, name: &str, n_pts: usize) -> Result<AnyDataArray, VtkError> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Skip header (description, part, part_number, coordinates)
    let mut i = 0;
    while i < lines.len() && !lines[i].trim().starts_with("coordinates") {
        i += 1;
    }
    if i >= lines.len() {
        return Err(VtkError::Parse(
            "no coordinates section found in scalar variable".into(),
        ));
    }
    i += 1;

    let values = read_f64_values(&lines, i, n_pts, "scalar value")?;

    Ok(AnyDataArray::F64(DataArray::from_vec(name, values, 1)))
}

fn read_vector_variable(path: &Path, name: &str, n_pts: usize) -> Result<AnyDataArray, VtkError> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() && !lines[i].trim().starts_with("coordinates") {
        i += 1;
    }
    if i >= lines.len() {
        return Err(VtkError::Parse(
            "no coordinates section found in vector variable".into(),
        ));
    }
    i += 1;

    let mut xs = Vec::with_capacity(n_pts);
    let mut ys = Vec::with_capacity(n_pts);
    let mut zs = Vec::with_capacity(n_pts);

    let mut next_line = i;
    xs.extend(read_f64_values_from(
        &lines,
        &mut next_line,
        n_pts,
        "vector x value",
    )?);
    ys.extend(read_f64_values_from(
        &lines,
        &mut next_line,
        n_pts,
        "vector y value",
    )?);
    zs.extend(read_f64_values_from(
        &lines,
        &mut next_line,
        n_pts,
        "vector z value",
    )?);

    let mut data = Vec::with_capacity(n_pts * 3);
    for j in 0..n_pts {
        data.push(xs[j]);
        data.push(ys[j]);
        data.push(zs[j]);
    }

    Ok(AnyDataArray::F64(DataArray::from_vec(name, data, 3)))
}

fn parse_f64_line(line: Option<&&str>, what: &str, index: usize) -> Result<f64, VtkError> {
    let line = line.ok_or_else(|| VtkError::Parse(format!("missing {what} {index}")))?;
    line.trim()
        .parse::<f64>()
        .map_err(|_| VtkError::Parse(format!("invalid {what} {index}: {line}")))
}

fn read_f64_values(
    lines: &[&str],
    start: usize,
    count: usize,
    what: &str,
) -> Result<Vec<f64>, VtkError> {
    let mut line_index = start;
    read_f64_values_from(lines, &mut line_index, count, what)
}

fn read_f64_values_from(
    lines: &[&str],
    line_index: &mut usize,
    count: usize,
    what: &str,
) -> Result<Vec<f64>, VtkError> {
    let mut values = Vec::with_capacity(count);
    while values.len() < count {
        let line = lines
            .get(*line_index)
            .ok_or_else(|| VtkError::Parse(format!("missing {what} {}", values.len())))?;
        let fields = parse_f64_fields(line)
            .map_err(|_| VtkError::Parse(format!("invalid {what} {}: {line}", values.len())))?;
        if fields.is_empty() {
            return Err(VtkError::Parse(format!(
                "invalid {what} {}: {line}",
                values.len()
            )));
        }
        for value in fields {
            if values.len() == count {
                break;
            }
            values.push(value);
        }
        *line_index += 1;
    }
    Ok(values)
}

fn parse_f64_fields(line: &str) -> Result<Vec<f64>, std::num::ParseFloatError> {
    let whitespace_fields: Result<Vec<f64>, _> =
        line.split_whitespace().map(|s| s.parse::<f64>()).collect();
    match whitespace_fields {
        Ok(fields) if fields.len() != 1 || line.trim().len() <= 12 => Ok(fields),
        _ => line
            .as_bytes()
            .chunks(12)
            .map(|chunk| {
                let field = std::str::from_utf8(chunk).unwrap_or("").trim();
                field.parse::<f64>()
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray as DA;
    use crate::io::ensight::EnSightWriter;

    #[test]
    fn roundtrip_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let dir = std::env::temp_dir().join("vtk_ensight_rt_test");
        let _ = std::fs::remove_dir_all(&dir);

        EnSightWriter::write(&dir, "rt", &pd).unwrap();
        let result = EnSightReader::read(&dir.join("rt.case")).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);

        let p = result.points.get(1);
        assert!((p[0] - 1.0).abs() < 0.01);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn roundtrip_with_scalar() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = DA::from_vec("temp", vec![10.0f64, 20.0, 30.0], 1);
        pd.point_data_mut().add_array(s.into());

        let dir = std::env::temp_dir().join("vtk_ensight_scalar_rt");
        let _ = std::fs::remove_dir_all(&dir);

        EnSightWriter::write(&dir, "data", &pd).unwrap();
        let result = EnSightReader::read(&dir.join("data.case")).unwrap();

        let arr = result.point_data().get_array("temp").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert!((buf[0] - 20.0).abs() < 0.1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_case_geo_accepts_time_and_file_sets() {
        assert_eq!(
            parse_case_geo("GEOMETRY\nmodel: 2 7 mesh.geo\n").unwrap(),
            "mesh.geo"
        );
        assert_eq!(
            parse_case_geo("GEOMETRY\nmodel: 2 transient.*****.geo\n").unwrap(),
            "transient.*****.geo"
        );
        assert_eq!(
            parse_case_geo("GEOMETRY\nmodel: \"mesh.geo\"\n").unwrap(),
            "mesh.geo"
        );
    }

    #[test]
    fn parse_case_variables_accept_time_and_file_sets() {
        let vars = parse_case_variables(
            "VARIABLE\nscalar per node: 2 7 pressure \"pressure.scl\"\nvector per node: 3 velocity velocity.vec\n",
        );
        assert_eq!(
            vars,
            vec![
                (
                    "scalar".to_string(),
                    "pressure".to_string(),
                    "pressure.scl".to_string()
                ),
                (
                    "vector".to_string(),
                    "velocity".to_string(),
                    "velocity.vec".to_string()
                ),
            ]
        );
    }

    #[test]
    fn tria3_skips_optional_element_ids() {
        let dir = std::env::temp_dir().join("vtk_ensight_tria3_element_ids");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let geo = dir.join("mesh.geo");
        std::fs::write(
            &geo,
            "EnSight Gold\nGenerated\npart\n1\ncoordinates\n3\n0\n1\n0\n0\n0\n1\n0\n0\n0\ntria3\n1\n42\n1 2 3\n",
        )
        .unwrap();

        let result = read_geometry(&geo).unwrap();

        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.iter().next().unwrap(), &[0, 1, 2]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn geometry_skips_listed_node_ids() {
        let dir = std::env::temp_dir().join("vtk_ensight_node_ids_given");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let geo = dir.join("mesh.geo");
        std::fs::write(
            &geo,
            "EnSight Gold\nGenerated\nnode id given\nelement id off\npart\n1\ncoordinates\n3\n101\n102\n103\n0\n1\n0\n0\n0\n1\n0\n0\n0\ntria3\n1\n1 2 3\n",
        )
        .unwrap();

        let result = read_geometry(&geo).unwrap();

        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
        assert_eq!(result.points.get(1), [1.0, 0.0, 0.0]);
        assert_eq!(result.points.get(2), [0.0, 1.0, 0.0]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scalar_variable_reads_packed_values() {
        let dir = std::env::temp_dir().join("vtk_ensight_scalar_packed");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let var = dir.join("temp.scl");
        std::fs::write(&var, "temp\npart\n1\ncoordinates\n1.0 2.0 3.0\n").unwrap();

        let arr = read_scalar_variable(&var, "temp", 3).unwrap();

        assert_eq!(arr.num_tuples(), 3);
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 3.0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn vector_variable_reads_fixed_width_component_blocks() {
        let dir = std::env::temp_dir().join("vtk_ensight_vector_fixed_width");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let var = dir.join("vel.vec");
        std::fs::write(
            &var,
            "vel\npart\n1\ncoordinates\n-1.00000e+00-2.00000e+00\n 3.00000e+00 4.00000e+00\n 5.00000e+00 6.00000e+00\n",
        )
        .unwrap();

        let arr = read_vector_variable(&var, "vel", 2).unwrap();

        assert_eq!(arr.num_tuples(), 2);
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [-2.0, 4.0, 6.0]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
