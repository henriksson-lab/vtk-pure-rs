//! Facet file format reader and writer.
//!
//! VTK Facet ASCII format:
//! `FACET FILE ...`, number of parts, then for each part a point block and a
//! topology block. Cell connectivity in the file is 1-based and each cell line
//! ends with material and relative-part numbers.

use crate::data::{AnyDataArray, CellArray, DataArray, PolyData};
use std::io::{BufRead, Error, ErrorKind, Write};

/// Write PolyData as Facet format.
pub fn write_facet<W: Write>(w: &mut W, mesh: &PolyData) -> std::io::Result<()> {
    writeln!(w, "FACET FILE FROM VTK")?;
    writeln!(w, "1")?;
    write_part(w, "Element", mesh)
}

fn write_part<W: Write>(w: &mut W, part_name: &str, mesh: &PolyData) -> std::io::Result<()> {
    writeln!(w, "{part_name}")?;
    writeln!(w, "0")?;
    writeln!(w, "{} 0 0", mesh.points.len())?;

    for i in 0..mesh.points.len() {
        let xyz = mesh.points.get(i);
        writeln!(w, "{} {} {}", xyz[0], xyz[1], xyz[2])?;
    }

    writeln!(w, "1")?;
    writeln!(w, "{part_name}")?;

    let families = [
        !mesh.verts.is_empty(),
        !mesh.lines.is_empty(),
        !mesh.polys.is_empty(),
        !mesh.strips.is_empty(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if families > 1 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "multiple different cells in the poly data",
        ));
    }

    let material = 0;
    let part = 0;

    if !mesh.verts.is_empty() {
        let total_cells: usize = mesh.verts.iter().map(|cell| cell.len()).sum();
        writeln!(w, "{total_cells} 1")?;
        for cell in mesh.verts.iter() {
            for point_id in cell {
                writeln!(w, "{} {material} {part}", point_id + 1)?;
            }
        }
    } else if !mesh.lines.is_empty() {
        let total_cells: usize = mesh
            .lines
            .iter()
            .map(|cell| cell.len().saturating_sub(1))
            .sum();
        writeln!(w, "{total_cells} 2")?;
        for cell in mesh.lines.iter() {
            for pair in cell.windows(2) {
                writeln!(w, "{} {} {material} {part}", pair[0] + 1, pair[1] + 1)?;
            }
        }
    } else if !mesh.polys.is_empty() {
        let num_points = homogeneous_cell_size(&mesh.polys)?;
        writeln!(w, "{} {num_points}", mesh.polys.num_cells())?;
        for cell in mesh.polys.iter() {
            for point_id in cell {
                write!(w, "{} ", point_id + 1)?;
            }
            writeln!(w, "{material} {part}")?;
        }
    } else if !mesh.strips.is_empty() {
        let total_cells: usize = mesh
            .strips
            .iter()
            .map(|cell| cell.len().saturating_sub(2))
            .sum();
        writeln!(w, "{total_cells} 3")?;
        for cell in mesh.strips.iter() {
            for tri in cell.windows(3) {
                writeln!(
                    w,
                    "{} {} {} {material} {part}",
                    tri[0] + 1,
                    tri[1] + 1,
                    tri[2] + 1
                )?;
            }
        }
    } else {
        writeln!(w, "0 0")?;
    }

    Ok(())
}

fn homogeneous_cell_size(cells: &CellArray) -> std::io::Result<usize> {
    cells.is_homogeneous().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "found polygons with different order",
        )
    })
}

/// Read Facet format into PolyData.
pub fn read_facet<R: BufRead>(reader: R) -> Result<PolyData, String> {
    let mut lines = reader.lines();

    let header = next_line(&mut lines, "Cannot read file comment")?;
    if !header.starts_with("FACET FILE") {
        return Err("File does not start with FACET FILE".into());
    }

    let num_parts: usize = parse_line(&next_line(&mut lines, "Bad number of parts line")?)?;

    let mut mesh = PolyData::new();
    let mut vert_metadata = Vec::new();
    let mut line_metadata = Vec::new();
    let mut poly_metadata = Vec::new();

    for part in 0..num_parts {
        let part_name = next_line(&mut lines, "Cannot read part name")?;

        let cell_point_index: i32 = parse_line(&next_line(
            &mut lines,
            "Cannot read cell/point index or it is not 0",
        )?)?;
        if cell_point_index != 0 {
            return Err("Cannot read cell/point index or it is not 0".into());
        }

        let point_info_line = next_line(&mut lines, "Problem reading number of points")?;
        let point_info = parse_ints(&point_info_line)
            .map_err(|_| "Problem reading number of points".to_string())?;
        if point_info.len() < 3 || point_info[0] < 0 {
            return Err("Problem reading number of points".into());
        }

        let point_offset = mesh.points.len() as i64;
        for point in 0..point_info[0] as usize {
            let line = next_line(&mut lines, "Problem reading point")?;
            let xyz = parse_floats(&line)
                .map_err(|_| format!("Problem reading point: {point} {line}"))?;
            if xyz.len() < 3 {
                return Err(format!("Problem reading point: {point} {line}"));
            }
            mesh.points.push([xyz[0], xyz[1], xyz[2]]);
        }

        let cell_point_index: i32 = parse_line(&next_line(
            &mut lines,
            "Cannot read cell/point index or it is not 1",
        )?)?;
        if cell_point_index != 1 {
            return Err("Cannot read cell/point index or it is not 1".into());
        }

        let topo_part_name = next_line(
            &mut lines,
            "Cannot read part name or the part name does not match",
        )?;
        if topo_part_name != part_name {
            return Err("Cannot read part name or the part name does not match".into());
        }

        let topology_line = next_line(
            &mut lines,
            "Problem reading number of cells and points per cell",
        )?;
        let topology = parse_ints(&topology_line)
            .map_err(|_| "Problem reading number of cells and points per cell".to_string())?;
        if topology.len() < 2 || topology[0] < 0 || topology[1] < 0 {
            return Err("Problem reading number of cells and points per cell".into());
        }

        let num_cells = topology[0] as usize;
        let num_points_per_cell = topology[1] as usize;

        for cell in 0..num_cells {
            let line = next_line(&mut lines, "Cannot read cell")?;
            let values = parse_ints(&line).map_err(|_| format!("Cannot read cell: {cell}"))?;
            if values.len() < num_points_per_cell + 2 {
                return Err(format!("Cannot extract cell points for cell: {cell}"));
            }

            let mut point_list = Vec::with_capacity(num_points_per_cell);
            for val in values.iter().take(num_points_per_cell) {
                point_list.push(point_offset + *val - 1);
            }

            let metadata = (
                part as u32,
                values[num_points_per_cell] as u32,
                values[num_points_per_cell + 1] as u32,
            );
            match num_points_per_cell {
                1 => {
                    mesh.verts.push_cell(&point_list);
                    vert_metadata.push(metadata);
                }
                2 => {
                    mesh.lines.push_cell(&point_list);
                    line_metadata.push(metadata);
                }
                _ => {
                    mesh.polys.push_cell(&point_list);
                    poly_metadata.push(metadata);
                }
            }
        }
    }

    let mut materials = Vec::new();
    let mut relative_parts = Vec::new();
    let mut part_numbers = Vec::new();
    for (part_number, material, relative_part) in vert_metadata
        .into_iter()
        .chain(line_metadata)
        .chain(poly_metadata)
    {
        part_numbers.push(part_number);
        materials.push(material);
        relative_parts.push(relative_part);
    }

    if !materials.is_empty() {
        mesh.cell_data_mut()
            .add_array(AnyDataArray::U32(DataArray::from_vec(
                "PartNumber",
                part_numbers,
                1,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::U32(DataArray::from_vec(
                "Material", materials, 1,
            )));
        mesh.cell_data_mut()
            .add_array(AnyDataArray::U32(DataArray::from_vec(
                "RelativePartNumber",
                relative_parts,
                1,
            )));
    }

    Ok(mesh)
}

fn next_line<R: BufRead>(lines: &mut std::io::Lines<R>, message: &str) -> Result<String, String> {
    lines
        .next()
        .ok_or_else(|| message.to_string())?
        .map_err(|e| e.to_string())
}

fn parse_line<T: std::str::FromStr>(line: &str) -> Result<T, String> {
    line.trim()
        .parse::<T>()
        .map_err(|_| format!("failed to parse line: {line}"))
}

fn parse_ints(line: &str) -> Result<Vec<i64>, std::num::ParseIntError> {
    line.split_whitespace()
        .map(|tok| tok.parse::<i64>())
        .collect()
}

fn parse_floats(line: &str) -> Result<Vec<f64>, std::num::ParseFloatError> {
    line.split_whitespace()
        .map(|tok| tok.parse::<f64>())
        .collect()
}

pub fn read_facet_file(path: &std::path::Path) -> Result<PolyData, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_facet(std::io::BufReader::new(f))
}

pub fn write_facet_file(mesh: &PolyData, path: &std::path::Path) -> Result<(), String> {
    let f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    write_facet(&mut std::io::BufWriter::new(f), mesh).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_facet(&mut buf, &mesh).unwrap();
        let loaded = read_facet(&buf[..]).unwrap();
        assert_eq!(loaded.points.len(), 3);
        assert_eq!(loaded.polys.num_cells(), 1);
    }

    #[test]
    fn writes_vtk_facet_header_and_one_based_connectivity() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        write_facet(&mut buf, &mesh).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("FACET FILE FROM VTK\n1\n"));
        assert!(text.contains("\n1 2 3 0 0\n"));
    }
}
