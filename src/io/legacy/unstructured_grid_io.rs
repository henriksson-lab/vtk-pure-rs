use std::io::{BufRead, Write};
use std::path::Path;

use crate::data::{
    AnyDataArray, DataArray, DataObject, DataSetAttributes, FieldData, UnstructuredGrid,
};
use crate::types::{CellType, ScalarType, VtkError};

/// Write an UnstructuredGrid to VTK legacy format (ASCII).
pub fn write_unstructured_grid(path: &Path, grid: &UnstructuredGrid) -> Result<(), VtkError> {
    let file = std::fs::File::create(path)?;
    let mut w = std::io::BufWriter::new(file);
    write_unstructured_grid_to(&mut w, grid)
}

pub fn write_unstructured_grid_to<W: Write>(
    w: &mut W,
    grid: &UnstructuredGrid,
) -> Result<(), VtkError> {
    writeln!(w, "# vtk DataFile Version 4.2")?;
    writeln!(w, "vtk-rs UnstructuredGrid")?;
    writeln!(w, "ASCII")?;
    writeln!(w, "DATASET UNSTRUCTURED_GRID")?;
    write_field_data(w, grid.field_data())?;

    // Points
    let n_points = grid.points.len();
    writeln!(w, "POINTS {} double", n_points)?;
    for i in 0..n_points {
        let p = grid.points.get(i);
        writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
    }

    let n_cells = grid.cells().num_cells();
    if n_cells > 0 {
        // Cells
        let total_size = n_cells + grid.cells().connectivity_len();
        writeln!(w, "CELLS {} {}", n_cells, total_size)?;
        for i in 0..n_cells {
            let pts = grid.cell_points(i);
            write!(w, "{}", pts.len())?;
            for &id in pts {
                write!(w, " {}", id)?;
            }
            writeln!(w)?;
        }

        // Cell types
        writeln!(w, "CELL_TYPES {}", n_cells)?;
        for i in 0..n_cells {
            writeln!(w, "{}", grid.cell_type(i) as u8)?;
        }
    }

    // Cell data
    if grid.cell_data().num_arrays() > 0 {
        writeln!(w, "CELL_DATA {}", n_cells)?;
        write_attributes(w, grid.cell_data())?;
    }

    // Point data
    if grid.point_data().num_arrays() > 0 {
        writeln!(w, "POINT_DATA {}", n_points)?;
        write_attributes(w, grid.point_data())?;
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

/// Read an UnstructuredGrid from VTK legacy format.
pub fn read_unstructured_grid(path: &Path) -> Result<UnstructuredGrid, VtkError> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    read_unstructured_grid_from(reader)
}

pub fn read_unstructured_grid_from<R: BufRead>(reader: R) -> Result<UnstructuredGrid, VtkError> {
    let mut lines_iter = reader.lines();

    // Header
    let version = next_line(&mut lines_iter)?;
    if !version.starts_with("# vtk DataFile Version") {
        return Err(VtkError::Parse("not a VTK file".into()));
    }
    let _description = next_line(&mut lines_iter)?;
    let file_type = next_line(&mut lines_iter)?;
    if file_type.trim().to_uppercase() != "ASCII" {
        return Err(VtkError::Unsupported(
            "only ASCII UnstructuredGrid reading supported".into(),
        ));
    }
    let dataset_line = next_line(&mut lines_iter)?;
    let tokens: Vec<&str> = dataset_line.split_whitespace().collect();
    if tokens.len() < 2 || tokens[1].to_uppercase() != "UNSTRUCTURED_GRID" {
        return Err(VtkError::Parse(format!(
            "expected UNSTRUCTURED_GRID, got: {}",
            dataset_line
        )));
    }

    let mut grid = UnstructuredGrid::new();
    let mut cell_connectivity: Vec<Vec<i64>> = Vec::new();
    let mut cell_types_raw: Vec<u8> = Vec::new();

    // Collect remaining lines
    let mut remaining: Vec<String> = Vec::new();
    for line in lines_iter {
        remaining.push(line.map_err(VtkError::Io)?);
    }

    let mut idx = 0;
    while idx < remaining.len() {
        let line = remaining[idx].trim().to_string();
        idx += 1;
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0].to_uppercase().as_str() {
            "POINTS" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let mut values = Vec::with_capacity(n * 3);
                while values.len() < n * 3 && idx < remaining.len() {
                    for token in remaining[idx].split_whitespace() {
                        let v = token.parse::<f64>().map_err(|_| {
                            VtkError::Parse(format!("invalid point coordinate: {}", token))
                        })?;
                        values.push(v);
                        if values.len() == n * 3 {
                            break;
                        }
                    }
                    idx += 1;
                }
                if values.len() != n * 3 {
                    return Err(VtkError::Parse(format!(
                        "expected {} point coordinates, got {}",
                        n * 3,
                        values.len()
                    )));
                }
                for i in 0..n {
                    grid.points
                        .push([values[i * 3], values[i * 3 + 1], values[i * 3 + 2]]);
                }
            }
            "FIELD" => {
                parse_field_data(&remaining, &mut idx, grid.field_data_mut(), &parts)?;
            }
            "CELLS" => {
                let n_cells: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let total_size: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                let mut raw = Vec::with_capacity(total_size);
                while raw.len() < total_size && idx < remaining.len() {
                    for token in remaining[idx].split_whitespace() {
                        let v = token.parse::<i64>().map_err(|_| {
                            VtkError::Parse(format!("invalid cell integer: {}", token))
                        })?;
                        raw.push(v);
                        if raw.len() == total_size {
                            break;
                        }
                    }
                    idx += 1;
                }
                if raw.len() != total_size {
                    return Err(VtkError::Parse(format!(
                        "expected {} cell integers, got {}",
                        total_size,
                        raw.len()
                    )));
                }

                let mut raw_idx = 0;
                for _ in 0..n_cells {
                    if raw_idx >= raw.len() {
                        return Err(VtkError::Parse("missing cell size".into()));
                    }
                    let npts = raw[raw_idx];
                    raw_idx += 1;
                    if npts < 0 {
                        return Err(VtkError::Parse(format!("invalid cell size: {}", npts)));
                    }
                    let npts = npts as usize;
                    if raw_idx + npts > raw.len() {
                        return Err(VtkError::Parse(format!(
                            "cell expects {} point ids, only {} remain",
                            npts,
                            raw.len().saturating_sub(raw_idx)
                        )));
                    }
                    let ids = raw[raw_idx..raw_idx + npts].to_vec();
                    raw_idx += npts;
                    cell_connectivity.push(ids);
                }
                if raw_idx != raw.len() {
                    return Err(VtkError::Parse(format!(
                        "{} extra cell integers",
                        raw.len() - raw_idx
                    )));
                }
            }
            "CELL_TYPES" => {
                let n_cells: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let mut count = 0;
                while count < n_cells && idx < remaining.len() {
                    for token in remaining[idx].split_whitespace() {
                        let v = token.parse::<u8>().map_err(|_| {
                            VtkError::Parse(format!("invalid cell type: {}", token))
                        })?;
                        cell_types_raw.push(v);
                        count += 1;
                        if count == n_cells {
                            break;
                        }
                    }
                    idx += 1;
                }
                if count != n_cells {
                    return Err(VtkError::Parse(format!(
                        "expected {} cell types, got {}",
                        n_cells, count
                    )));
                }
            }
            "POINT_DATA" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                if n != grid.points.len() {
                    return Err(VtkError::Parse(format!(
                        "Number of points don't match data values: {} != {}",
                        n,
                        grid.points.len()
                    )));
                }
                idx = parse_data_section(&remaining, idx, n, grid.point_data_mut())?;
            }
            "CELL_DATA" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                if n != cell_connectivity.len() {
                    return Err(VtkError::Parse(format!(
                        "Number of cells don't match data values: {} != {}",
                        n,
                        cell_connectivity.len()
                    )));
                }
                idx = parse_data_section(&remaining, idx, n, grid.cell_data_mut())?;
            }
            _ => {}
        }
    }

    // Build cells from connectivity + types
    for (i, conn) in cell_connectivity.iter().enumerate() {
        let raw_type = cell_types_raw
            .get(i)
            .ok_or_else(|| VtkError::Parse(format!("missing cell type for cell {}", i)))?;
        let ct = CellType::from_u8(*raw_type)
            .ok_or_else(|| VtkError::Parse(format!("unknown cell type: {}", raw_type)))?;
        grid.push_cell(ct, conn);
    }

    Ok(grid)
}

fn parse_data_section(
    lines: &[String],
    mut idx: usize,
    n: usize,
    attrs: &mut DataSetAttributes,
) -> Result<usize, VtkError> {
    while idx < lines.len() {
        let l = lines[idx].trim().to_string();
        let p: Vec<&str> = l.split_whitespace().collect();
        if p.is_empty() {
            idx += 1;
            continue;
        }
        if p[0].to_uppercase() == "COLOR_SCALARS" {
            idx += 1;
            let name = p
                .get(1)
                .copied()
                .ok_or_else(|| VtkError::Parse("missing COLOR_SCALARS name".into()))?;
            let nc: usize = p
                .get(2)
                .ok_or_else(|| VtkError::Parse("missing COLOR_SCALARS component count".into()))?
                .parse()
                .map_err(|_| VtkError::Parse("invalid COLOR_SCALARS component count".into()))?;
            let values = read_scalar_values(lines, &mut idx, n * nc)?;
            let data: Vec<u8> = values
                .iter()
                .map(|&v| (255.0 * v + 0.5).clamp(0.0, 255.0) as u8)
                .collect();
            let arr = AnyDataArray::U8(DataArray::from_vec(name, data, nc));
            let arr_name = arr.name().to_string();
            attrs.add_array(arr);
            if attrs.scalars().is_none() {
                attrs.set_active_scalars(&arr_name);
            }
        } else if matches!(p[0].to_uppercase().as_str(), "VECTORS" | "NORMALS") {
            idx += 1;
            let name = p
                .get(1)
                .copied()
                .ok_or_else(|| VtkError::Parse("missing vector/normal name".into()))?;
            let type_name = p
                .get(2)
                .copied()
                .ok_or_else(|| VtkError::Parse("missing vector/normal type".into()))?;
            let scalar_type = ScalarType::from_vtk_name(type_name)
                .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;
            let values = read_scalar_values(lines, &mut idx, n * 3)?;
            let arr = array_from_f64_values(name, scalar_type, values, 3);
            let arr_name = arr.name().to_string();
            let is_normals = p[0].eq_ignore_ascii_case("NORMALS");
            attrs.add_array(arr);
            if is_normals {
                attrs.set_active_normals(&arr_name);
            } else {
                attrs.set_active_vectors(&arr_name);
            }
        } else if p[0].to_uppercase() == "FIELD" {
            idx += 1;
            parse_attribute_field_data(lines, &mut idx, attrs, &p)?;
        } else if p[0].to_uppercase() == "SCALARS" {
            idx += 1;
            let name = p
                .get(1)
                .copied()
                .ok_or_else(|| VtkError::Parse("missing SCALARS name".into()))?;
            let type_name = p
                .get(2)
                .copied()
                .ok_or_else(|| VtkError::Parse("missing SCALARS type".into()))?;
            let nc: usize = p
                .get(3)
                .map(|s| s.parse())
                .transpose()
                .map_err(|_| VtkError::Parse("invalid SCALARS component count".into()))?
                .unwrap_or(1);

            if idx >= lines.len() || !lines[idx].trim().to_uppercase().starts_with("LOOKUP_TABLE") {
                return Err(VtkError::Parse(
                    "LOOKUP_TABLE must be specified with scalar".into(),
                ));
            }
            idx += 1;

            let values = read_scalar_values(lines, &mut idx, n * nc)?;
            let scalar_type = ScalarType::from_vtk_name(type_name)
                .ok_or_else(|| VtkError::Parse(format!("unknown scalar type: {}", type_name)))?;
            let arr = array_from_f64_values(name, scalar_type, values, nc);
            let arr_name = arr.name().to_string();
            attrs.add_array(arr);
            if attrs.scalars().is_none() {
                attrs.set_active_scalars(&arr_name);
            }
        } else if matches!(
            p[0].to_uppercase().as_str(),
            "POINT_DATA" | "CELL_DATA" | "POINTS" | "CELLS" | "CELL_TYPES"
        ) {
            break;
        } else {
            idx += 1;
        }
    }
    Ok(idx)
}

fn parse_attribute_field_data(
    lines: &[String],
    idx: &mut usize,
    attrs: &mut DataSetAttributes,
    header_parts: &[&str],
) -> Result<(), VtkError> {
    let num_arrays: usize = header_parts
        .get(2)
        .ok_or_else(|| VtkError::Parse("missing FIELD array count".into()))?
        .parse()
        .map_err(|_| VtkError::Parse("invalid FIELD array count".into()))?;
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

        let name = parts
            .get(0)
            .copied()
            .ok_or_else(|| VtkError::Parse("missing FIELD array name".into()))?;
        let num_components: usize = parts
            .get(1)
            .ok_or_else(|| VtkError::Parse("missing FIELD component count".into()))?
            .parse()
            .map_err(|_| VtkError::Parse("invalid FIELD component count".into()))?;
        let num_tuples: usize = parts
            .get(2)
            .ok_or_else(|| VtkError::Parse("missing FIELD tuple count".into()))?
            .parse()
            .map_err(|_| VtkError::Parse("invalid FIELD tuple count".into()))?;
        let type_name = parts
            .get(3)
            .copied()
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
    let num_arrays: usize = header_parts
        .get(2)
        .ok_or_else(|| VtkError::Parse("missing FIELD array count".into()))?
        .parse()
        .map_err(|_| VtkError::Parse("invalid FIELD array count".into()))?;
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

        let name = parts
            .get(0)
            .copied()
            .ok_or_else(|| VtkError::Parse("missing FIELD array name".into()))?;
        let num_components: usize = parts
            .get(1)
            .ok_or_else(|| VtkError::Parse("missing FIELD component count".into()))?
            .parse()
            .map_err(|_| VtkError::Parse("invalid FIELD component count".into()))?;
        let num_tuples: usize = parts
            .get(2)
            .ok_or_else(|| VtkError::Parse("missing FIELD tuple count".into()))?
            .parse()
            .map_err(|_| VtkError::Parse("invalid FIELD tuple count".into()))?;
        let type_name = parts
            .get(3)
            .copied()
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

fn read_scalar_values(
    lines: &[String],
    idx: &mut usize,
    total: usize,
) -> Result<Vec<f64>, VtkError> {
    let mut values = Vec::with_capacity(total);
    while values.len() < total && *idx < lines.len() {
        for token in lines[*idx].split_whitespace() {
            let v = token
                .parse::<f64>()
                .map_err(|_| VtkError::Parse(format!("invalid scalar value: {}", token)))?;
            values.push(v);
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
    use crate::data::DataSet;

    #[test]
    fn roundtrip_tetra() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let mut buf = Vec::new();
        write_unstructured_grid_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = read_unstructured_grid_from(reader).unwrap();

        assert_eq!(result.num_points(), 4);
        assert_eq!(result.num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Tetra);
        assert_eq!(result.cell_points(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn roundtrip_mixed_cells() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.points.push([2.0, 0.0, 0.0]);

        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Triangle, &[1, 4, 2]);

        let mut buf = Vec::new();
        write_unstructured_grid_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = read_unstructured_grid_from(reader).unwrap();

        assert_eq!(result.num_cells(), 2);
        assert_eq!(result.cell_type(0), CellType::Tetra);
        assert_eq!(result.cell_type(1), CellType::Triangle);
    }

    #[test]
    fn roundtrip_with_scalars() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let scalars = DataArray::from_vec("temperature", vec![10.0, 20.0, 30.0, 40.0], 1);
        grid.point_data_mut().add_array(scalars.into());
        grid.point_data_mut().set_active_scalars("temperature");

        let mut buf = Vec::new();
        write_unstructured_grid_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = read_unstructured_grid_from(reader).unwrap();

        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 4);
        let mut val = [0.0f64];
        s.tuple_as_f64(2, &mut val);
        assert!((val[0] - 30.0).abs() < 1e-6);
    }

    #[test]
    fn reads_cells_as_token_stream() {
        let input = b"# vtk DataFile Version 4.2
token stream cells
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 4 double
0 0 0 1 0 0
0 1 0 0 0 1
CELLS 1 5
4 0
1 2
3
CELL_TYPES 1
10
";

        let reader = std::io::BufReader::new(&input[..]);
        let result = read_unstructured_grid_from(reader).unwrap();

        assert_eq!(result.num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Tetra);
        assert_eq!(result.cell_points(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn rejects_short_points() {
        let input = b"# vtk DataFile Version 4.2
short points
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 2 double
0 0 0
";

        let reader = std::io::BufReader::new(&input[..]);
        assert!(read_unstructured_grid_from(reader).is_err());
    }

    #[test]
    fn rejects_invalid_cell_type_token() {
        let input = b"# vtk DataFile Version 4.2
bad cell type
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 3 double
0 0 0 1 0 0 0 1 0
CELLS 1 4
3 0 1 2
CELL_TYPES 1
not_a_type
";

        let reader = std::io::BufReader::new(&input[..]);
        assert!(read_unstructured_grid_from(reader).is_err());
    }

    #[test]
    fn writes_cell_data_before_point_data() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);
        let point_scalars = DataArray::from_vec("point_values", vec![1.0f64, 2.0, 3.0], 1);
        let cell_scalars = DataArray::from_vec("cell_values", vec![4.0f64], 1);
        grid.point_data_mut().add_array(point_scalars.into());
        grid.cell_data_mut().add_array(cell_scalars.into());

        let mut buf = Vec::new();
        write_unstructured_grid_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();

        let cell_data = output.find("CELL_DATA 1").unwrap();
        let point_data = output.find("POINT_DATA 3").unwrap();
        assert!(cell_data < point_data);
    }

    #[test]
    fn rejects_cell_data_count_mismatch() {
        let input = b"# vtk DataFile Version 4.2
bad count
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 3 double
0 0 0 1 0 0 0 1 0
CELLS 1 4
3 0 1 2
CELL_TYPES 1
5
CELL_DATA 2
SCALARS cell_values double
LOOKUP_TABLE default
1 2
";

        let reader = std::io::BufReader::new(&input[..]);
        assert!(read_unstructured_grid_from(reader).is_err());
    }

    #[test]
    fn rejects_missing_lookup_table() {
        let input = b"# vtk DataFile Version 4.2
bad scalar
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 3 double
0 0 0 1 0 0 0 1 0
CELLS 1 4
3 0 1 2
CELL_TYPES 1
5
POINT_DATA 3
SCALARS point_values double
1 2 3
";

        let reader = std::io::BufReader::new(&input[..]);
        assert!(read_unstructured_grid_from(reader).is_err());
    }

    #[test]
    fn roundtrips_numeric_field_data() {
        let mut grid = UnstructuredGrid::new();
        grid.field_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec("step", vec![7], 1)));

        let mut buf = Vec::new();
        write_unstructured_grid_to(&mut buf, &grid).unwrap();
        let reader = std::io::BufReader::new(&buf[..]);
        let result = read_unstructured_grid_from(reader).unwrap();

        let step = result.field_data().get_array("step").unwrap();
        assert_eq!(step.to_f64_vec(), vec![7.0]);
    }

    #[test]
    fn reads_attribute_field_data() {
        let input = b"# vtk DataFile Version 4.2
attribute field
ASCII
DATASET UNSTRUCTURED_GRID
POINTS 3 double
0 0 0 1 0 0 0 1 0
CELLS 1 4
3 0 1 2
CELL_TYPES 1
5
POINT_DATA 3
FIELD FieldData 1
ids 1 3 int
1 2 3
CELL_DATA 1
FIELD FieldData 1
cid 1 1 int
9
";

        let reader = std::io::BufReader::new(&input[..]);
        let grid = read_unstructured_grid_from(reader).unwrap();

        assert_eq!(
            grid.point_data().get_array("ids").unwrap().to_f64_vec(),
            vec![1.0, 2.0, 3.0]
        );
        assert_eq!(
            grid.cell_data().get_array("cid").unwrap().to_f64_vec(),
            vec![9.0]
        );
    }
}
