use std::io::BufRead;
use std::path::Path;

use crate::data::UnstructuredGrid;
use crate::types::{CellType, VtkError};

use crate::io::xml::binary;
use crate::io::xml::vtp_reader::{
    any_data_array_to_i64, detect_format, extract_appended_base64, extract_appended_raw,
    extract_attr, extract_section, parse_attribute_arrays, parse_from_appended, DataFormat,
};

/// Reader for VTK XML UnstructuredGrid format (.vtu).
///
/// Supports ASCII, binary (base64-encoded), and appended data formats.
pub struct VtuReader;

impl VtuReader {
    pub fn read(path: &Path) -> Result<UnstructuredGrid, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<UnstructuredGrid, VtkError> {
        let content: String = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(VtkError::Io)?
            .join("\n");

        // Extract appended data section if present
        let appended_raw = extract_appended_raw(&content);
        let appended_b64 = extract_appended_base64(&content);

        let mut grid = UnstructuredGrid::new();

        // Extract Points
        if let Some(points_section) = extract_section(&content, "Points") {
            if let Some(da_start) = points_section.find("<DataArray") {
                let tag_end = points_section[da_start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed DataArray tag".into()))?;
                let tag = &points_section[da_start..da_start + tag_end + 1];
                let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Float64".to_string());

                let content_start = da_start + tag_end + 1;
                let content_end = points_section[content_start..]
                    .find("</DataArray>")
                    .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
                let da_content = points_section[content_start..content_start + content_end].trim();

                match detect_format(tag) {
                    DataFormat::Ascii => {
                        let values: Vec<f64> = da_content
                            .split_whitespace()
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        for chunk in values.chunks(3) {
                            if chunk.len() == 3 {
                                grid.points.push([chunk[0], chunk[1], chunk[2]]);
                            }
                        }
                    }
                    DataFormat::Binary => {
                        let arr =
                            binary::parse_binary_data_array(da_content, "Points", &type_str, 3)?;
                        let pts = crate::io::xml::vtp_reader::data_array_to_points(&arr)?;
                        for i in 0..pts.len() {
                            grid.points.push(pts.get(i));
                        }
                    }
                    DataFormat::Appended(offset) => {
                        let arr = parse_from_appended(
                            appended_raw.as_deref(),
                            appended_b64.as_deref(),
                            offset,
                            "Points",
                            &type_str,
                            3,
                        )?;
                        let pts = crate::io::xml::vtp_reader::data_array_to_points(&arr)?;
                        for i in 0..pts.len() {
                            grid.points.push(pts.get(i));
                        }
                    }
                }
            }
        }

        // Extract Cells
        if let Some(cells_section) = extract_section(&content, "Cells") {
            let mut connectivity: Vec<i64> = Vec::new();
            let mut offsets: Vec<i64> = Vec::new();
            let mut types: Vec<u8> = Vec::new();

            let mut search_pos = 0;
            while let Some(da_start) = cells_section[search_pos..].find("<DataArray") {
                let abs_start = search_pos + da_start;
                let tag_end = cells_section[abs_start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed DataArray tag".into()))?;
                let tag = &cells_section[abs_start..abs_start + tag_end + 1];

                let content_start = abs_start + tag_end + 1;
                let content_end = cells_section[content_start..]
                    .find("</DataArray>")
                    .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
                let da_content = cells_section[content_start..content_start + content_end].trim();

                let name = extract_attr(tag, "Name").unwrap_or_default();
                let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Int64".to_string());

                let values: Vec<i64> = match detect_format(tag) {
                    DataFormat::Ascii => da_content
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect(),
                    DataFormat::Binary => {
                        let arr = binary::parse_binary_data_array(da_content, &name, &type_str, 1)?;
                        any_data_array_to_i64(&arr)
                    }
                    DataFormat::Appended(offset) => {
                        let arr = parse_from_appended(
                            appended_raw.as_deref(),
                            appended_b64.as_deref(),
                            offset,
                            &name,
                            &type_str,
                            1,
                        )?;
                        any_data_array_to_i64(&arr)
                    }
                };

                match name.as_str() {
                    "connectivity" => connectivity = values,
                    "offsets" => offsets = values,
                    "types" => types = values.into_iter().map(|v| v as u8).collect(),
                    _ => {}
                }

                search_pos = content_start + content_end + "</DataArray>".len();
            }

            // Build cells from connectivity + offsets + types
            let mut prev_offset: usize = 0;
            for (i, &offset) in offsets.iter().enumerate() {
                let end = offset as usize;
                if end <= connectivity.len() && prev_offset < end {
                    let cell_pts = &connectivity[prev_offset..end];
                    let ct = types
                        .get(i)
                        .and_then(|&v| CellType::from_u8(v))
                        .unwrap_or(CellType::Triangle);
                    grid.push_cell(ct, cell_pts);
                }
                prev_offset = end;
            }
        }

        // Extract PointData
        if let Some(pd_section) = extract_section(&content, "PointData") {
            parse_attribute_arrays(
                &pd_section,
                grid.point_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
            )?;
        }

        // Extract CellData
        if let Some(cd_section) = extract_section(&content, "CellData") {
            parse_attribute_arrays(
                &cd_section,
                grid.cell_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
            )?;
        }

        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray as DA, DataSet};
    use crate::io::xml::VtuWriter;
    use crate::types::CellType;

    #[test]
    fn roundtrip_vtu_tetra() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let mut buf = Vec::new();
        VtuWriter::write_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtuReader::read_from(reader).unwrap();

        assert_eq!(result.num_points(), 4);
        assert_eq!(result.num_cells(), 1);
        assert_eq!(result.cell_type(0), CellType::Tetra);
        assert_eq!(result.cell_points(0), &[0, 1, 2, 3]);
    }

    #[test]
    fn roundtrip_vtu_mixed() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.points.push([2.0, 0.0, 0.0]);

        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        grid.push_cell(CellType::Triangle, &[1, 4, 2]);

        let mut buf = Vec::new();
        VtuWriter::write_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtuReader::read_from(reader).unwrap();

        assert_eq!(result.num_cells(), 2);
        assert_eq!(result.cell_type(0), CellType::Tetra);
        assert_eq!(result.cell_type(1), CellType::Triangle);
    }

    #[test]
    fn roundtrip_vtu_with_scalars() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let scalars = DA::from_vec("temp", vec![10.0, 20.0, 30.0], 1);
        grid.point_data_mut().add_array(scalars.into());
        grid.point_data_mut().set_active_scalars("temp");

        let mut buf = Vec::new();
        VtuWriter::write_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtuReader::read_from(reader).unwrap();

        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.name(), "temp");
        assert_eq!(s.num_tuples(), 3);
    }
}
