use std::io::{BufRead, Write};
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::{ScalarType, VtkError};

/// Write an ImageData to VTK legacy format (STRUCTURED_POINTS).
pub fn write_image_data(path: &Path, data: &ImageData) -> Result<(), VtkError> {
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    write_image_data_to(&mut w, data)
}

pub fn write_image_data_to<W: Write>(w: &mut W, data: &ImageData) -> Result<(), VtkError> {
    let dims = data.dimensions();
    let spacing = data.spacing();
    let origin = data.origin();

    writeln!(w, "# vtk DataFile Version 4.2")?;
    writeln!(w, "vtk-rs ImageData")?;
    writeln!(w, "ASCII")?;
    writeln!(w, "DATASET STRUCTURED_POINTS")?;
    writeln!(w, "DIMENSIONS {} {} {}", dims[0], dims[1], dims[2])?;
    writeln!(w, "SPACING {} {} {}", spacing[0], spacing[1], spacing[2])?;
    writeln!(w, "ORIGIN {} {} {}", origin[0], origin[1], origin[2])?;

    let n = data.num_points();
    if data.point_data().num_arrays() > 0 {
        writeln!(w, "POINT_DATA {}", n)?;
        for i in 0..data.point_data().num_arrays() {
            if let Some(arr) = data.point_data().get_array_by_index(i) {
                write_scalars_ascii(w, arr)?;
            }
        }
    }

    Ok(())
}

fn write_scalars_ascii<W: Write>(w: &mut W, arr: &AnyDataArray) -> Result<(), VtkError> {
    let nc = arr.num_components();
    let nt = arr.num_tuples();
    let type_name = arr.scalar_type().vtk_name();

    if nc == 1 {
        writeln!(w, "SCALARS {} {}", arr.name(), type_name)?;
    } else {
        writeln!(w, "SCALARS {} {} {}", arr.name(), type_name, nc)?;
    }
    writeln!(w, "LOOKUP_TABLE default")?;

    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        for (j, v) in buf.iter().enumerate() {
            if j > 0 {
                write!(w, " ")?;
            }
            write!(w, "{}", v)?;
        }
        writeln!(w)?;
    }
    Ok(())
}

/// Read an ImageData from VTK legacy format (STRUCTURED_POINTS).
pub fn read_image_data(path: &Path) -> Result<ImageData, VtkError> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    read_image_data_from(reader)
}

pub fn read_image_data_from<R: BufRead>(reader: R) -> Result<ImageData, VtkError> {
    let mut lines = reader.lines();

    // Header
    let version = next_line(&mut lines)?;
    if !version.starts_with("# vtk DataFile Version") {
        return Err(VtkError::Parse("not a VTK file".into()));
    }
    let _description = next_line(&mut lines)?;
    let file_type = next_line(&mut lines)?;
    if file_type.trim().to_uppercase() != "ASCII" {
        return Err(VtkError::Unsupported(
            "only ASCII ImageData supported".into(),
        ));
    }
    let dataset_line = next_line(&mut lines)?;
    let tokens: Vec<&str> = dataset_line.split_whitespace().collect();
    if tokens.len() < 2 || tokens[1].to_uppercase() != "STRUCTURED_POINTS" {
        return Err(VtkError::Parse(format!(
            "expected STRUCTURED_POINTS, got: {}",
            dataset_line
        )));
    }

    let mut dims = [1usize; 3];
    let mut spacing = [1.0f64; 3];
    let mut origin = [0.0f64; 3];
    let mut image = ImageData::new();

    // Parse remaining lines
    let mut remaining_lines: Vec<String> = Vec::new();
    for line in lines {
        remaining_lines.push(line.map_err(VtkError::Io)?);
    }

    let mut idx = 0;
    while idx < remaining_lines.len() {
        let line = remaining_lines[idx].trim().to_string();
        idx += 1;
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_uppercase().as_str() {
            "DIMENSIONS" => {
                if parts.len() >= 4 {
                    dims[0] = parts[1].parse().unwrap_or(1);
                    dims[1] = parts[2].parse().unwrap_or(1);
                    dims[2] = parts[3].parse().unwrap_or(1);
                }
            }
            "SPACING" | "ASPECT_RATIO" => {
                if parts.len() >= 4 {
                    spacing[0] = parts[1].parse().unwrap_or(1.0);
                    spacing[1] = parts[2].parse().unwrap_or(1.0);
                    spacing[2] = parts[3].parse().unwrap_or(1.0);
                }
            }
            "ORIGIN" => {
                if parts.len() >= 4 {
                    origin[0] = parts[1].parse().unwrap_or(0.0);
                    origin[1] = parts[2].parse().unwrap_or(0.0);
                    origin[2] = parts[3].parse().unwrap_or(0.0);
                }
            }
            "POINT_DATA" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                // Parse scalar arrays
                while idx < remaining_lines.len() {
                    let l = remaining_lines[idx].trim().to_string();
                    let p: Vec<&str> = l.split_whitespace().collect();
                    if p.is_empty() {
                        idx += 1;
                        continue;
                    }
                    if p[0].to_uppercase() == "SCALARS" {
                        idx += 1;
                        let name = p.get(1).unwrap_or(&"data");
                        let type_name = p.get(2).unwrap_or(&"double");
                        let nc: usize = p.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);

                        // Skip LOOKUP_TABLE line
                        if idx < remaining_lines.len()
                            && remaining_lines[idx]
                                .trim()
                                .to_uppercase()
                                .starts_with("LOOKUP_TABLE")
                        {
                            idx += 1;
                        }

                        // Read n * nc values
                        let mut values = Vec::with_capacity(n * nc);
                        while values.len() < n * nc && idx < remaining_lines.len() {
                            for token in remaining_lines[idx].split_whitespace() {
                                if let Ok(v) = token.parse::<f64>() {
                                    values.push(v);
                                }
                            }
                            idx += 1;
                        }

                        let scalar_type =
                            ScalarType::from_vtk_name(type_name).unwrap_or(ScalarType::F64);
                        let arr = match scalar_type {
                            ScalarType::F32 => {
                                let data: Vec<f32> = values.iter().map(|&v| v as f32).collect();
                                AnyDataArray::F32(DataArray::from_vec(*name, data, nc))
                            }
                            _ => AnyDataArray::F64(DataArray::from_vec(*name, values, nc)),
                        };
                        let arr_name = arr.name().to_string();
                        image.point_data_mut().add_array(arr);
                        if image.point_data().scalars().is_none() {
                            image.point_data_mut().set_active_scalars(&arr_name);
                        }
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    image.set_extent([
        0,
        dims[0] as i64 - 1,
        0,
        dims[1] as i64 - 1,
        0,
        dims[2] as i64 - 1,
    ]);
    image.set_spacing(spacing);
    image.set_origin(origin);

    Ok(image)
}

fn next_line(
    lines: &mut impl Iterator<Item = Result<String, std::io::Error>>,
) -> Result<String, VtkError> {
    lines
        .next()
        .ok_or_else(|| VtkError::Parse("unexpected end of file".into()))?
        .map_err(VtkError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_image_data() {
        let mut img = ImageData::with_dimensions(3, 4, 5);
        img.set_spacing([0.5, 0.5, 0.5]);
        img.set_origin([1.0, 2.0, 3.0]);

        let n = img.num_points();
        let scalars: Vec<f64> = (0..n).map(|i| i as f64 * 0.1).collect();
        let arr = DataArray::from_vec("density", scalars, 1);
        img.point_data_mut().add_array(arr.into());
        img.point_data_mut().set_active_scalars("density");

        let mut buf = Vec::new();
        write_image_data_to(&mut buf, &img).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = read_image_data_from(reader).unwrap();

        assert_eq!(result.dimensions(), [3, 4, 5]);
        assert_eq!(result.spacing(), [0.5, 0.5, 0.5]);
        assert_eq!(result.origin(), [1.0, 2.0, 3.0]);

        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 60);
        let mut val = [0.0f64];
        s.tuple_as_f64(10, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-6);
    }
}
