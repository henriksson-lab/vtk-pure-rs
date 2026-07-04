use std::io::{BufRead, Write};
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataObject, DataSetAttributes, FieldData, ImageData};
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
    write_field_data(w, data.field_data())?;
    writeln!(w, "DIMENSIONS {} {} {}", dims[0], dims[1], dims[2])?;
    writeln!(w, "SPACING {} {} {}", spacing[0], spacing[1], spacing[2])?;
    let extent = data.extent();
    writeln!(
        w,
        "ORIGIN {} {} {}",
        origin[0] + extent[0] as f64 * spacing[0],
        origin[1] + extent[2] as f64 * spacing[1],
        origin[2] + extent[4] as f64 * spacing[2]
    )?;

    if data.cell_data().num_arrays() > 0 {
        writeln!(w, "CELL_DATA {}", num_cells_from_dimensions(dims))?;
        write_attributes(w, data.cell_data())?;
    }
    let n = data.num_points();
    if data.point_data().num_arrays() > 0 {
        writeln!(w, "POINT_DATA {}", n)?;
        write_attributes(w, data.point_data())?;
    }

    Ok(())
}

fn write_attributes<W: Write>(w: &mut W, attrs: &DataSetAttributes) -> Result<(), VtkError> {
    let vectors_name = attrs.vectors().map(|a| a.name().to_string());
    let normals_name = attrs.normals().map(|a| a.name().to_string());

    for i in 0..attrs.num_arrays() {
        if let Some(arr) = attrs.get_array_by_index(i) {
            if normals_name.as_deref() == Some(arr.name()) && arr.num_components() == 3 {
                write_data_array_as_vectors(w, "NORMALS", arr)?;
            } else if vectors_name.as_deref() == Some(arr.name()) && arr.num_components() == 3 {
                write_data_array_as_vectors(w, "VECTORS", arr)?;
            } else {
                write_scalars_ascii(w, arr)?;
            }
        }
    }
    Ok(())
}

fn write_data_array_as_vectors<W: Write>(
    w: &mut W,
    keyword: &str,
    arr: &AnyDataArray,
) -> Result<(), VtkError> {
    writeln!(
        w,
        "{} {} {}",
        keyword,
        arr.name(),
        arr.scalar_type().vtk_name()
    )?;

    let mut buf = vec![0.0f64; arr.num_components()];
    for i in 0..arr.num_tuples() {
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

fn write_scalars_ascii<W: Write>(w: &mut W, arr: &AnyDataArray) -> Result<(), VtkError> {
    if let AnyDataArray::U8(a) = arr {
        writeln!(w, "COLOR_SCALARS {} {}", a.name(), a.num_components())?;
        for i in 0..a.num_tuples() {
            let t = a.tuple(i);
            for v in t {
                write!(w, "{} ", *v as f32 / 255.0)?;
            }
            writeln!(w)?;
        }
        return Ok(());
    }

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
    let mut extent: Option<[i64; 6]> = None;
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
                    dims[0] = parse_part(parts.get(1), "DIMENSIONS x")?;
                    dims[1] = parse_part(parts.get(2), "DIMENSIONS y")?;
                    dims[2] = parse_part(parts.get(3), "DIMENSIONS z")?;
                    extent = Some([
                        0,
                        dims[0] as i64 - 1,
                        0,
                        dims[1] as i64 - 1,
                        0,
                        dims[2] as i64 - 1,
                    ]);
                }
            }
            "FIELD" => {
                parse_field_data(&remaining_lines, &mut idx, image.field_data_mut(), &parts)?;
            }
            "EXTENT" => {
                if parts.len() >= 7 {
                    let parsed_extent = [
                        parse_part(parts.get(1), "EXTENT x min")?,
                        parse_part(parts.get(2), "EXTENT x max")?,
                        parse_part(parts.get(3), "EXTENT y min")?,
                        parse_part(parts.get(4), "EXTENT y max")?,
                        parse_part(parts.get(5), "EXTENT z min")?,
                        parse_part(parts.get(6), "EXTENT z max")?,
                    ];
                    dims = dimensions_from_extent(parsed_extent);
                    extent = Some(parsed_extent);
                }
            }
            "SPACING" | "ASPECT_RATIO" => {
                if parts.len() >= 4 {
                    spacing[0] = parse_part(parts.get(1), "SPACING x")?;
                    spacing[1] = parse_part(parts.get(2), "SPACING y")?;
                    spacing[2] = parse_part(parts.get(3), "SPACING z")?;
                }
            }
            "ORIGIN" => {
                if parts.len() >= 4 {
                    origin[0] = parse_part(parts.get(1), "ORIGIN x")?;
                    origin[1] = parse_part(parts.get(2), "ORIGIN y")?;
                    origin[2] = parse_part(parts.get(3), "ORIGIN z")?;
                }
            }
            "POINT_DATA" => {
                let n: usize = parse_part(parts.get(1), "POINT_DATA count")?;
                let expected = num_points_from_dimensions(dims);
                if n != expected {
                    return Err(VtkError::Parse(format!(
                        "Number of points don't match data values: {} != {}",
                        n, expected
                    )));
                }
                parse_scalar_arrays(&remaining_lines, &mut idx, image.point_data_mut(), n, true)?;
            }
            "CELL_DATA" => {
                let n: usize = parse_part(parts.get(1), "CELL_DATA count")?;
                let expected = num_cells_from_dimensions(dims);
                if n != expected {
                    return Err(VtkError::Parse(format!(
                        "Number of cells don't match data values: {} != {}",
                        n, expected
                    )));
                }
                parse_scalar_arrays(&remaining_lines, &mut idx, image.cell_data_mut(), n, true)?;
            }
            _ => {}
        }
    }

    image.set_extent(extent.unwrap_or([
        0,
        dims[0] as i64 - 1,
        0,
        dims[1] as i64 - 1,
        0,
        dims[2] as i64 - 1,
    ]));
    image.set_spacing(spacing);
    image.set_origin(origin);

    Ok(image)
}

fn write_field_data<W: Write>(w: &mut W, field_data: &FieldData) -> Result<(), VtkError> {
    let arrays: Vec<&AnyDataArray> = field_data.iter().collect();
    if arrays.is_empty() {
        return Ok(());
    }

    writeln!(w, "FIELD FieldData {}", arrays.len())?;
    for arr in arrays {
        writeln!(
            w,
            "{} {} {} {}",
            arr.name(),
            arr.num_components(),
            arr.num_tuples(),
            arr.scalar_type().vtk_name()
        )?;
        let mut buf = vec![0.0f64; arr.num_components()];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            for (j, v) in buf.iter().enumerate() {
                if j > 0 {
                    write!(w, " ")?;
                }
                write!(w, "{}", v)?;
            }
            writeln!(w)?;
        }
    }
    Ok(())
}

fn parse_field_data(
    lines: &[String],
    idx: &mut usize,
    field_data: &mut FieldData,
    header_parts: &[&str],
) -> Result<(), VtkError> {
    let num_arrays: usize = parse_part(header_parts.get(2), "FIELD array count")?;
    for _ in 0..num_arrays {
        while *idx < lines.len() && lines[*idx].trim().is_empty() {
            *idx += 1;
        }
        if *idx >= lines.len() {
            return Err(VtkError::Parse("unexpected end of FIELD data".into()));
        }

        let line = lines[*idx].trim();
        *idx += 1;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.first().copied() == Some("NULL_ARRAY") {
            continue;
        }

        let name = *parts
            .get(0)
            .ok_or_else(|| VtkError::Parse("missing FIELD array name".into()))?;
        let num_components: usize = parse_part(parts.get(1), "FIELD component count")?;
        let num_tuples: usize = parse_part(parts.get(2), "FIELD tuple count")?;
        let type_name = *parts
            .get(3)
            .ok_or_else(|| VtkError::Parse("missing FIELD scalar type".into()))?;
        let scalar_type = ScalarType::from_vtk_name(type_name)
            .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;
        let values = read_scalar_values(lines, idx, num_components * num_tuples)?;
        field_data.add_array(array_from_f64_values(
            name,
            scalar_type,
            values,
            num_components,
        ));
    }
    Ok(())
}

fn parse_scalar_arrays(
    lines: &[String],
    idx: &mut usize,
    attrs: &mut DataSetAttributes,
    n: usize,
    set_active_scalars: bool,
) -> Result<(), VtkError> {
    while *idx < lines.len() {
        let line = lines[*idx].trim();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            *idx += 1;
            continue;
        }
        match parts[0].to_uppercase().as_str() {
            "COLOR_SCALARS" => {
                *idx += 1;
                let name = *parts
                    .get(1)
                    .ok_or_else(|| VtkError::Parse("missing COLOR_SCALARS name".into()))?;
                let nc: usize = parse_part(parts.get(2), "COLOR_SCALARS component count")?;
                let values = read_scalar_values(lines, idx, n * nc)?;
                let data: Vec<u8> = values
                    .iter()
                    .map(|&v| (255.0 * v + 0.5).clamp(0.0, 255.0) as u8)
                    .collect();
                let arr = AnyDataArray::U8(DataArray::from_vec(name, data, nc));
                let arr_name = arr.name().to_string();
                attrs.add_array(arr);
                if set_active_scalars && attrs.scalars().is_none() {
                    attrs.set_active_scalars(&arr_name);
                }
                continue;
            }
            "VECTORS" | "NORMALS" => {
                *idx += 1;
                let name = *parts
                    .get(1)
                    .ok_or_else(|| VtkError::Parse("missing vector/normal name".into()))?;
                let type_name = *parts
                    .get(2)
                    .ok_or_else(|| VtkError::Parse("missing vector/normal type".into()))?;
                let values = read_scalar_values(lines, idx, n * 3)?;
                let scalar_type = ScalarType::from_vtk_name(type_name).ok_or_else(|| {
                    VtkError::Parse(format!("unknown scalar type: {}", type_name))
                })?;
                let arr = array_from_f64_values(name, scalar_type, values, 3);
                let arr_name = arr.name().to_string();
                let is_normals = parts[0].eq_ignore_ascii_case("NORMALS");
                attrs.add_array(arr);
                if is_normals {
                    attrs.set_active_normals(&arr_name);
                } else {
                    attrs.set_active_vectors(&arr_name);
                }
                continue;
            }
            "FIELD" => {
                *idx += 1;
                parse_attribute_field_data(lines, idx, attrs, &parts)?;
                continue;
            }
            "SCALARS" => {}
            _ => break,
        }

        if parts[0].eq_ignore_ascii_case("SCALARS") {
            *idx += 1;
            let name = *parts
                .get(1)
                .ok_or_else(|| VtkError::Parse("missing SCALARS name".into()))?;
            let type_name = *parts
                .get(2)
                .ok_or_else(|| VtkError::Parse("missing SCALARS type".into()))?;
            let nc: usize = parts
                .get(3)
                .map(|s| s.parse())
                .transpose()
                .map_err(|_| VtkError::Parse("invalid SCALARS component count".into()))?
                .unwrap_or(1);

            if *idx >= lines.len()
                || !lines[*idx]
                    .trim()
                    .to_uppercase()
                    .starts_with("LOOKUP_TABLE")
            {
                return Err(VtkError::Parse(
                    "LOOKUP_TABLE must be specified with scalar".into(),
                ));
            }
            *idx += 1;

            let values = read_scalar_values(lines, idx, n * nc)?;
            let scalar_type = ScalarType::from_vtk_name(type_name)
                .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;
            let arr = array_from_f64_values(name, scalar_type, values, nc);
            let arr_name = arr.name().to_string();
            attrs.add_array(arr);
            if set_active_scalars && attrs.scalars().is_none() {
                attrs.set_active_scalars(&arr_name);
            }
        }
    }
    Ok(())
}

fn parse_attribute_field_data(
    lines: &[String],
    idx: &mut usize,
    attrs: &mut DataSetAttributes,
    header_parts: &[&str],
) -> Result<(), VtkError> {
    let num_arrays: usize = parse_part(header_parts.get(2), "FIELD array count")?;
    for _ in 0..num_arrays {
        while *idx < lines.len() && lines[*idx].trim().is_empty() {
            *idx += 1;
        }
        if *idx >= lines.len() {
            return Err(VtkError::Parse("unexpected end of FIELD data".into()));
        }

        let line = lines[*idx].trim();
        *idx += 1;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.first().copied() == Some("NULL_ARRAY") {
            continue;
        }

        let name = *parts
            .get(0)
            .ok_or_else(|| VtkError::Parse("missing FIELD array name".into()))?;
        let num_components: usize = parse_part(parts.get(1), "FIELD component count")?;
        let num_tuples: usize = parse_part(parts.get(2), "FIELD tuple count")?;
        let type_name = *parts
            .get(3)
            .ok_or_else(|| VtkError::Parse("missing FIELD scalar type".into()))?;
        let scalar_type = ScalarType::from_vtk_name(type_name)
            .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;
        let values = read_scalar_values(lines, idx, num_components * num_tuples)?;
        attrs.add_array(array_from_f64_values(
            name,
            scalar_type,
            values,
            num_components,
        ));
    }
    Ok(())
}

fn read_scalar_values(
    lines: &[String],
    idx: &mut usize,
    total: usize,
) -> Result<Vec<f64>, VtkError> {
    let mut values = Vec::with_capacity(total);
    while values.len() < total && *idx < lines.len() {
        for token in lines[*idx].split_whitespace() {
            let value = token
                .parse::<f64>()
                .map_err(|_| VtkError::Parse(format!("invalid scalar value: {}", token)))?;
            values.push(value);
            if values.len() == total {
                break;
            }
        }
        *idx += 1;
    }
    if values.len() != total {
        return Err(VtkError::Parse(format!(
            "not enough scalar values: {} != {}",
            values.len(),
            total
        )));
    }
    Ok(values)
}

fn array_from_f64_values(
    name: &str,
    scalar_type: ScalarType,
    values: Vec<f64>,
    num_components: usize,
) -> AnyDataArray {
    match scalar_type {
        ScalarType::F32 => AnyDataArray::F32(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as f32).collect(),
            num_components,
        )),
        ScalarType::F64 => AnyDataArray::F64(DataArray::from_vec(name, values, num_components)),
        ScalarType::I8 => AnyDataArray::I8(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as i8).collect(),
            num_components,
        )),
        ScalarType::I16 => AnyDataArray::I16(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as i16).collect(),
            num_components,
        )),
        ScalarType::I32 => AnyDataArray::I32(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as i32).collect(),
            num_components,
        )),
        ScalarType::I64 => AnyDataArray::I64(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as i64).collect(),
            num_components,
        )),
        ScalarType::U8 => AnyDataArray::U8(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as u8).collect(),
            num_components,
        )),
        ScalarType::U16 => AnyDataArray::U16(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as u16).collect(),
            num_components,
        )),
        ScalarType::U32 => AnyDataArray::U32(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as u32).collect(),
            num_components,
        )),
        ScalarType::U64 => AnyDataArray::U64(DataArray::from_vec(
            name,
            values.into_iter().map(|v| v as u64).collect(),
            num_components,
        )),
    }
}

fn dimensions_from_extent(extent: [i64; 6]) -> [usize; 3] {
    [
        (extent[1] - extent[0] + 1).max(0) as usize,
        (extent[3] - extent[2] + 1).max(0) as usize,
        (extent[5] - extent[4] + 1).max(0) as usize,
    ]
}

fn num_points_from_dimensions(dims: [usize; 3]) -> usize {
    dims[0].saturating_mul(dims[1]).saturating_mul(dims[2])
}

fn num_cells_from_dimensions(dims: [usize; 3]) -> usize {
    if dims.iter().any(|&dim| dim == 0) {
        return 0;
    }

    let mut active_dims = 0;
    let mut cells = 1usize;
    for dim in dims {
        if dim > 1 {
            active_dims += 1;
            cells = cells.saturating_mul(dim - 1);
        }
    }
    if active_dims == 0 {
        0
    } else {
        cells
    }
}

fn parse_part<T: std::str::FromStr>(part: Option<&&str>, context: &str) -> Result<T, VtkError> {
    part.ok_or_else(|| VtkError::Parse(format!("missing {}", context)))?
        .parse()
        .map_err(|_| VtkError::Parse(format!("invalid {}", context)))
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

    #[test]
    fn reads_extent_and_cell_data() {
        let vtk = b"# vtk DataFile Version 4.2
extent image
ASCII
DATASET STRUCTURED_POINTS
EXTENT 2 4 3 5 0 0
SPACING 1 2 3
ORIGIN 4 5 6
CELL_DATA 4
SCALARS material unsigned_short
LOOKUP_TABLE default
1 2 3 4
";

        let reader = std::io::BufReader::new(&vtk[..]);
        let image = read_image_data_from(reader).unwrap();

        assert_eq!(image.extent(), [2, 4, 3, 5, 0, 0]);
        assert_eq!(image.dimensions(), [3, 3, 1]);
        let scalars = image.cell_data().scalars().unwrap();
        assert_eq!(scalars.name(), "material");
        assert_eq!(scalars.num_tuples(), 4);
    }

    #[test]
    fn rejects_point_data_count_mismatch() {
        let vtk = b"# vtk DataFile Version 4.2
bad count
ASCII
DATASET STRUCTURED_POINTS
DIMENSIONS 2 2 1
POINT_DATA 3
SCALARS density double
LOOKUP_TABLE default
1 2 3
";

        let reader = std::io::BufReader::new(&vtk[..]);
        assert!(read_image_data_from(reader).is_err());
    }

    #[test]
    fn rejects_cell_data_for_empty_dimension() {
        let vtk = b"# vtk DataFile Version 4.2
empty dimension
ASCII
DATASET STRUCTURED_POINTS
DIMENSIONS 0 4 5
CELL_DATA 12
SCALARS material double
LOOKUP_TABLE default
1 2 3 4 5 6 7 8 9 10 11 12
";

        let reader = std::io::BufReader::new(&vtk[..]);
        assert!(read_image_data_from(reader).is_err());
    }

    #[test]
    fn writes_cell_data_before_point_data() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        let point_scalars = DataArray::from_vec("point_values", vec![1.0f64, 2.0, 3.0, 4.0], 1);
        let cell_scalars = DataArray::from_vec("cell_values", vec![5.0f64], 1);
        img.point_data_mut().add_array(point_scalars.into());
        img.cell_data_mut().add_array(cell_scalars.into());

        let mut buf = Vec::new();
        write_image_data_to(&mut buf, &img).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let cell_data = output.find("CELL_DATA 1").unwrap();
        let point_data = output.find("POINT_DATA 4").unwrap();
        assert!(cell_data < point_data);
    }

    #[test]
    fn writes_dimensions_origin_at_extent_min_corner() {
        let mut img = ImageData::new();
        img.set_extent([2, 4, 3, 5, 1, 1]);
        img.set_spacing([0.5, 2.0, 10.0]);
        img.set_origin([10.0, 20.0, 30.0]);

        let mut buf = Vec::new();
        write_image_data_to(&mut buf, &img).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("DIMENSIONS 3 3 1"));
        assert!(output.contains("ORIGIN 11 26 40"));
    }

    #[test]
    fn roundtrips_numeric_field_data() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.field_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("step", vec![7], 1)));

        let mut buf = Vec::new();
        write_image_data_to(&mut buf, &img).unwrap();
        let reader = std::io::BufReader::new(&buf[..]);
        let result = read_image_data_from(reader).unwrap();

        let step = result.field_data().get_array("step").unwrap();
        assert_eq!(step.to_f64_vec(), vec![7.0]);
    }

    #[test]
    fn reads_attribute_field_data() {
        let vtk = b"# vtk DataFile Version 4.2
attribute field
ASCII
DATASET STRUCTURED_POINTS
DIMENSIONS 2 2 1
POINT_DATA 4
FIELD FieldData 1
ids 1 4 int
1 2 3 4
CELL_DATA 1
FIELD FieldData 1
cid 1 1 int
9
";

        let reader = std::io::BufReader::new(&vtk[..]);
        let image = read_image_data_from(reader).unwrap();

        assert_eq!(
            image.point_data().get_array("ids").unwrap().to_f64_vec(),
            vec![1.0, 2.0, 3.0, 4.0]
        );
        assert_eq!(
            image.cell_data().get_array("cid").unwrap().to_f64_vec(),
            vec![9.0]
        );
    }
}
