use std::io::{BufRead, Write};
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, UnstructuredGrid};
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

    // Points
    let n_points = grid.points.len();
    writeln!(w, "POINTS {} double", n_points)?;
    for i in 0..n_points {
        let p = grid.points.get(i);
        writeln!(w, "{} {} {}", p[0], p[1], p[2])?;
    }

    // Cells
    let n_cells = grid.cells().num_cells();
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

    // Point data
    if grid.point_data().num_arrays() > 0 {
        writeln!(w, "POINT_DATA {}", n_points)?;
        write_attributes(w, grid.point_data())?;
    }

    // Cell data
    if grid.cell_data().num_arrays() > 0 {
        writeln!(w, "CELL_DATA {}", n_cells)?;
        write_attributes(w, grid.cell_data())?;
    }

    Ok(())
}

fn write_attributes<W: Write>(w: &mut W, attrs: &DataSetAttributes) -> Result<(), VtkError> {
    for i in 0..attrs.num_arrays() {
        if let Some(arr) = attrs.get_array_by_index(i) {
            write_scalars_ascii(w, arr)?;
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
                        if let Ok(v) = token.parse::<f64>() {
                            values.push(v);
                        }
                    }
                    idx += 1;
                }
                for i in 0..n {
                    grid.points
                        .push([values[i * 3], values[i * 3 + 1], values[i * 3 + 2]]);
                }
            }
            "CELLS" => {
                let n_cells: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                for _ in 0..n_cells {
                    if idx >= remaining.len() {
                        break;
                    }
                    let cell_line = remaining[idx].trim().to_string();
                    idx += 1;
                    let cell_tokens: Vec<&str> = cell_line.split_whitespace().collect();
                    if cell_tokens.is_empty() {
                        continue;
                    }
                    let npts: usize = cell_tokens[0].parse().unwrap_or(0);
                    let ids: Vec<i64> = cell_tokens[1..=npts]
                        .iter()
                        .map(|s| s.parse().unwrap_or(0))
                        .collect();
                    cell_connectivity.push(ids);
                }
            }
            "CELL_TYPES" => {
                let n_cells: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let mut count = 0;
                while count < n_cells && idx < remaining.len() {
                    for token in remaining[idx].split_whitespace() {
                        if let Ok(v) = token.parse::<u8>() {
                            cell_types_raw.push(v);
                            count += 1;
                        }
                    }
                    idx += 1;
                }
            }
            "POINT_DATA" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                idx = parse_data_section(&remaining, idx, n, grid.point_data_mut());
            }
            "CELL_DATA" => {
                let n: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                idx = parse_data_section(&remaining, idx, n, grid.cell_data_mut());
            }
            _ => {}
        }
    }

    // Build cells from connectivity + types
    for (i, conn) in cell_connectivity.iter().enumerate() {
        let ct = cell_types_raw
            .get(i)
            .and_then(|&v| CellType::from_u8(v))
            .unwrap_or(CellType::Triangle);
        grid.push_cell(ct, conn);
    }

    Ok(grid)
}

fn parse_data_section(
    lines: &[String],
    mut idx: usize,
    n: usize,
    attrs: &mut DataSetAttributes,
) -> usize {
    while idx < lines.len() {
        let l = lines[idx].trim().to_string();
        let p: Vec<&str> = l.split_whitespace().collect();
        if p.is_empty() {
            idx += 1;
            continue;
        }
        if p[0].to_uppercase() == "SCALARS" {
            idx += 1;
            let name = p.get(1).copied().unwrap_or("data");
            let type_name = p.get(2).copied().unwrap_or("double");
            let nc: usize = p.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);

            // Skip LOOKUP_TABLE line
            if idx < lines.len() && lines[idx].trim().to_uppercase().starts_with("LOOKUP_TABLE") {
                idx += 1;
            }

            let mut values = Vec::with_capacity(n * nc);
            while values.len() < n * nc && idx < lines.len() {
                for token in lines[idx].split_whitespace() {
                    if let Ok(v) = token.parse::<f64>() {
                        values.push(v);
                    }
                }
                idx += 1;
            }

            let scalar_type = ScalarType::from_vtk_name(type_name).unwrap_or(ScalarType::F64);
            let arr = match scalar_type {
                ScalarType::F32 => {
                    let data: Vec<f32> = values.iter().map(|&v| v as f32).collect();
                    AnyDataArray::F32(DataArray::from_vec(name, data, nc))
                }
                _ => AnyDataArray::F64(DataArray::from_vec(name, values, nc)),
            };
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
    idx
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
}
