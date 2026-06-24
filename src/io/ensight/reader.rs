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
                    if let Ok(arr) = read_scalar_variable(&var_path, var_name, pd.points.len()) {
                        pd.point_data_mut().add_array(arr);
                    }
                }
                "vector" => {
                    if let Ok(arr) = read_vector_variable(&var_path, var_name, pd.points.len()) {
                        pd.point_data_mut().add_array(arr);
                    }
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
            let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Ok(parts[1].trim().to_string());
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
            if parts.len() >= 2 {
                vars.push((
                    var_type.to_string(),
                    parts[0].to_string(),
                    parts[1].to_string(),
                ));
            }
        }
    }
    vars
}

fn read_geometry(path: &Path) -> Result<PolyData, VtkError> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut pd = PolyData::new();
    let mut i = 0;

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

    // Read X, Y, Z coordinates separately
    let mut xs = Vec::with_capacity(n_pts);
    let mut ys = Vec::with_capacity(n_pts);
    let mut zs = Vec::with_capacity(n_pts);

    for _ in 0..n_pts {
        let v: f64 = lines.get(i).unwrap_or(&"0").trim().parse().unwrap_or(0.0);
        xs.push(v);
        i += 1;
    }
    for _ in 0..n_pts {
        let v: f64 = lines.get(i).unwrap_or(&"0").trim().parse().unwrap_or(0.0);
        ys.push(v);
        i += 1;
    }
    for _ in 0..n_pts {
        let v: f64 = lines.get(i).unwrap_or(&"0").trim().parse().unwrap_or(0.0);
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
            let n_cells: usize = lines.get(i).unwrap_or(&"0").trim().parse().unwrap_or(0);
            i += 1;
            let mut polys = CellArray::new();
            for _ in 0..n_cells {
                let parts: Vec<i64> = lines
                    .get(i)
                    .unwrap_or(&"")
                    .split_whitespace()
                    .filter_map(|s| s.parse::<i64>().ok())
                    .collect();
                if parts.len() >= 3 {
                    // Convert from 1-based to 0-based
                    polys.push_cell(&[parts[0] - 1, parts[1] - 1, parts[2] - 1]);
                }
                i += 1;
            }
            pd.polys = polys;
        } else {
            i += 1;
        }
    }

    Ok(pd)
}

fn read_scalar_variable(path: &Path, name: &str, n_pts: usize) -> Result<AnyDataArray, VtkError> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Skip header (description, part, part_number, coordinates)
    let mut i = 0;
    while i < lines.len() && !lines[i].trim().starts_with("coordinates") {
        i += 1;
    }
    i += 1;

    let mut values = Vec::with_capacity(n_pts);
    for _ in 0..n_pts {
        let v: f64 = lines.get(i).unwrap_or(&"0").trim().parse().unwrap_or(0.0);
        values.push(v);
        i += 1;
    }

    Ok(AnyDataArray::F64(DataArray::from_vec(name, values, 1)))
}

fn read_vector_variable(path: &Path, name: &str, n_pts: usize) -> Result<AnyDataArray, VtkError> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() && !lines[i].trim().starts_with("coordinates") {
        i += 1;
    }
    i += 1;

    let mut xs = Vec::with_capacity(n_pts);
    let mut ys = Vec::with_capacity(n_pts);
    let mut zs = Vec::with_capacity(n_pts);

    for _ in 0..n_pts {
        xs.push(
            lines
                .get(i)
                .unwrap_or(&"0")
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0),
        );
        i += 1;
    }
    for _ in 0..n_pts {
        ys.push(
            lines
                .get(i)
                .unwrap_or(&"0")
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0),
        );
        i += 1;
    }
    for _ in 0..n_pts {
        zs.push(
            lines
                .get(i)
                .unwrap_or(&"0")
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0),
        );
        i += 1;
    }

    let mut data = Vec::with_capacity(n_pts * 3);
    for j in 0..n_pts {
        data.push(xs[j]);
        data.push(ys[j]);
        data.push(zs[j]);
    }

    Ok(AnyDataArray::F64(DataArray::from_vec(name, data, 3)))
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
}
