use std::io::BufRead;
use std::path::Path;

use crate::data::UnstructuredGrid;
use crate::types::{CellType, VtkError};

use crate::io::xml::binary;
use crate::io::xml::vtp_reader::{
    any_data_array_to_i64, data_array_to_points, detect_format, extract_appended_base64,
    extract_appended_raw, extract_attr, extract_section, extract_section_with_tag,
    extract_vtk_header_type, parse_attribute_arrays_with_hints_and_header_type,
    parse_from_appended_with_header_type, parse_points_ascii, DataFormat,
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

    pub fn read_from<R: BufRead>(mut reader: R) -> Result<UnstructuredGrid, VtkError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(VtkError::Io)?;
        let content = String::from_utf8_lossy(&bytes).into_owned();

        let piece_tag = extract_open_tag(&content, "Piece")
            .ok_or_else(|| VtkError::Parse("missing Piece element".into()))?;
        let number_of_points = extract_required_usize_attr(&piece_tag, "NumberOfPoints")?;
        let number_of_cells = extract_required_usize_attr(&piece_tag, "NumberOfCells")?;

        // Extract appended data section if present
        let appended_raw =
            extract_appended_raw_bytes(&bytes).or_else(|| extract_appended_raw(&content));
        let appended_b64 = extract_appended_base64(&content);
        let header_type = extract_vtk_header_type(&content);

        let mut grid = UnstructuredGrid::new();

        // Extract Points
        if let Some(points_section) = extract_section(&content, "Points") {
            let data_array_count = points_section.matches("<DataArray").count();
            if data_array_count == 1 {
                let da_start = points_section.find("<DataArray").unwrap();
                let tag_end = points_section[da_start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed DataArray tag".into()))?;
                let tag = &points_section[da_start..da_start + tag_end + 1];
                let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Float64".to_string());

                let da_content;
                if tag.trim_end().ends_with("/>") {
                    da_content = "";
                } else {
                    let content_start = da_start + tag_end + 1;
                    let content_end = points_section[content_start..]
                        .find("</DataArray>")
                        .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
                    da_content = points_section[content_start..content_start + content_end].trim();
                }

                match detect_format(tag) {
                    DataFormat::Ascii => {
                        let pts = parse_points_ascii(da_content)?;
                        for i in 0..pts.len() {
                            grid.points.push(pts.get(i));
                        }
                    }
                    DataFormat::Binary => {
                        let arr = binary::parse_binary_data_array_with_header_type(
                            da_content,
                            "Points",
                            &type_str,
                            3,
                            header_type.as_deref(),
                        )?;
                        let pts = data_array_to_points(&arr)?;
                        for i in 0..pts.len() {
                            grid.points.push(pts.get(i));
                        }
                    }
                    DataFormat::Appended(offset) => {
                        let arr = parse_from_appended_with_header_type(
                            appended_raw.as_deref(),
                            appended_b64.as_deref(),
                            offset,
                            "Points",
                            &type_str,
                            3,
                            header_type.as_deref(),
                        )?;
                        let pts = data_array_to_points(&arr)?;
                        for i in 0..pts.len() {
                            grid.points.push(pts.get(i));
                        }
                    }
                }
            } else if number_of_points > 0 {
                return Err(VtkError::Parse(
                    "A piece is missing its Points element or element does not have exactly 1 array."
                        .into(),
                ));
            }
        } else if number_of_points > 0 {
            return Err(VtkError::Parse(
                "A piece is missing its Points element or element does not have exactly 1 array."
                    .into(),
            ));
        }

        if grid.points.len() != number_of_points {
            return Err(VtkError::Parse(format!(
                "Piece declares {} points but {} were read",
                number_of_points,
                grid.points.len()
            )));
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

                let (da_content, next_search_pos) = if tag.trim_end().ends_with("/>") {
                    ("", abs_start + tag_end + 1)
                } else {
                    let content_start = abs_start + tag_end + 1;
                    let content_end = cells_section[content_start..]
                        .find("</DataArray>")
                        .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
                    (
                        cells_section[content_start..content_start + content_end].trim(),
                        content_start + content_end + "</DataArray>".len(),
                    )
                };

                let name = extract_attr(tag, "Name").unwrap_or_default();
                let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Int64".to_string());

                let values: Vec<i64> = match detect_format(tag) {
                    DataFormat::Ascii => da_content
                        .split_whitespace()
                        .map(|s| {
                            s.parse().map_err(|_| {
                                VtkError::Parse(format!("invalid cell integer: {}", s))
                            })
                        })
                        .collect::<Result<_, _>>()?,
                    DataFormat::Binary => {
                        let arr = binary::parse_binary_data_array_with_header_type(
                            da_content,
                            &name,
                            &type_str,
                            1,
                            header_type.as_deref(),
                        )?;
                        any_data_array_to_i64(&arr)
                    }
                    DataFormat::Appended(offset) => {
                        let arr = parse_from_appended_with_header_type(
                            appended_raw.as_deref(),
                            appended_b64.as_deref(),
                            offset,
                            &name,
                            &type_str,
                            1,
                            header_type.as_deref(),
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

                search_pos = next_search_pos;
            }

            // Build cells from connectivity + offsets + types
            let mut prev_offset: usize = 0;
            if !offsets.is_empty() && connectivity.is_empty() {
                return Err(VtkError::Parse(
                    "Cannot read cell connectivity because the \"connectivity\" array could not be found.".into(),
                ));
            }
            if !connectivity.is_empty() && offsets.is_empty() {
                return Err(VtkError::Parse(
                    "Cannot read cell offsets because the \"offsets\" array could not be found."
                        .into(),
                ));
            }
            if offsets.len() != number_of_cells {
                return Err(VtkError::Parse(format!(
                    "cell offsets length {} does not match NumberOfCells {}",
                    offsets.len(),
                    number_of_cells
                )));
            }
            if offsets.len() != types.len() {
                return Err(VtkError::Parse(format!(
                    "cell types length {} does not match offsets length {}",
                    types.len(),
                    offsets.len()
                )));
            }
            for (i, &offset) in offsets.iter().enumerate() {
                if offset < 0 {
                    return Err(VtkError::Parse("cell offset is negative".into()));
                }
                let end = offset as usize;
                if end < prev_offset {
                    return Err(VtkError::Parse(
                        "cell offsets are not non-decreasing".into(),
                    ));
                }
                if end > connectivity.len() {
                    return Err(VtkError::Parse(
                        "cell offsets exceed connectivity length".into(),
                    ));
                }
                let cell_pts = &connectivity[prev_offset..end];
                let ct = CellType::from_u8(types[i]).ok_or_else(|| {
                    VtkError::Parse(format!("unknown VTK cell type: {}", types[i]))
                })?;
                grid.push_cell(ct, cell_pts);
                prev_offset = end;
            }
            if prev_offset != connectivity.len() {
                return Err(VtkError::Parse(
                    "cell offsets do not consume all connectivity entries".into(),
                ));
            }
        } else if number_of_cells > 0 {
            return Err(VtkError::Parse(
                "A piece is missing its Cells element.".into(),
            ));
        }

        // Extract PointData
        if let Some((tag, pd_section)) = extract_section_with_tag(&content, "PointData") {
            parse_attribute_arrays_with_hints_and_header_type(
                &pd_section,
                grid.point_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                Some(&tag),
                header_type.as_deref(),
            )?;
        }

        // Extract CellData
        if let Some((tag, cd_section)) = extract_section_with_tag(&content, "CellData") {
            parse_attribute_arrays_with_hints_and_header_type(
                &cd_section,
                grid.cell_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                Some(&tag),
                header_type.as_deref(),
            )?;
        }

        Ok(grid)
    }
}

fn extract_open_tag(content: &str, tag: &str) -> Option<String> {
    let open_pattern = format!("<{}", tag);
    let start = content.find(&open_pattern)?;
    let tag_end = content[start..].find('>')?;
    Some(content[start..start + tag_end + 1].to_string())
}

fn extract_appended_raw_bytes(content: &[u8]) -> Option<Vec<u8>> {
    let section_start = find_bytes(content, b"<AppendedData")?;
    let tag_end_rel = content[section_start..].iter().position(|&b| b == b'>')?;
    let tag_end = section_start + tag_end_rel;
    let tag = String::from_utf8_lossy(&content[section_start..=tag_end]);

    let encoding = extract_attr(&tag, "encoding").unwrap_or_default();
    if encoding != "raw" {
        return None;
    }

    let after_tag = &content[tag_end + 1..];
    let underscore_pos = after_tag.iter().position(|&b| b == b'_')?;
    let data_start = underscore_pos + 1;
    let end_tag = find_bytes(after_tag, b"</AppendedData>")?;
    if data_start >= end_tag {
        return None;
    }
    Some(after_tag[data_start..end_tag].to_vec())
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn extract_required_usize_attr(tag: &str, attr_name: &str) -> Result<usize, VtkError> {
    let value = extract_attr(tag, attr_name)
        .ok_or_else(|| VtkError::Parse(format!("Piece is missing its {attr_name} attribute")))?;
    value
        .parse()
        .map_err(|_| VtkError::Parse(format!("invalid {attr_name} attribute: {value}")))
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

    #[test]
    fn read_vtu_with_attribute_hints() {
        let xml = br#"<?xml version="1.0"?>
<VTKFile type="UnstructuredGrid" version="1.0" byte_order="LittleEndian">
  <UnstructuredGrid>
    <Piece NumberOfPoints="3" NumberOfCells="1">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">
          0 0 0 1 0 0 0 1 0
        </DataArray>
      </Points>
      <Cells>
        <DataArray type="Int64" Name="connectivity" format="ascii">0 1 2</DataArray>
        <DataArray type="Int64" Name="offsets" format="ascii">3</DataArray>
        <DataArray type="UInt8" Name="types" format="ascii">5</DataArray>
      </Cells>
      <PointData Scalars="temp" Vectors="vel">
        <DataArray type="Float64" Name="vel" NumberOfComponents="3" format="ascii">
          1 0 0 0 1 0 0 0 1
        </DataArray>
        <DataArray type="Float64" Name="temp" NumberOfComponents="1" format="ascii">
          10 20 30
        </DataArray>
      </PointData>
    </Piece>
  </UnstructuredGrid>
</VTKFile>"#;

        let reader = std::io::BufReader::new(&xml[..]);
        let result = VtuReader::read_from(reader).unwrap();

        assert_eq!(result.point_data().scalars().unwrap().name(), "temp");
        assert_eq!(result.point_data().vectors().unwrap().name(), "vel");
    }

    #[test]
    fn read_rejects_malformed_cells() {
        let xml = br#"<?xml version="1.0"?>
<VTKFile type="UnstructuredGrid" version="1.0" byte_order="LittleEndian">
  <UnstructuredGrid>
    <Piece NumberOfPoints="3" NumberOfCells="1">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0 1 0 0 0 1 0</DataArray>
      </Points>
      <Cells>
        <DataArray type="Int64" Name="connectivity" format="ascii">0 1 2</DataArray>
        <DataArray type="Int64" Name="offsets" format="ascii">4</DataArray>
        <DataArray type="UInt8" Name="types" format="ascii">5</DataArray>
      </Cells>
    </Piece>
  </UnstructuredGrid>
</VTKFile>"#;

        let reader = std::io::BufReader::new(&xml[..]);
        assert!(VtuReader::read_from(reader).is_err());
    }

    #[test]
    fn read_rejects_points_element_without_exactly_one_array() {
        let xml = br#"<?xml version="1.0"?>
<VTKFile type="UnstructuredGrid" version="1.0" byte_order="LittleEndian">
  <UnstructuredGrid>
    <Piece NumberOfPoints="1" NumberOfCells="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">1 1 1</DataArray>
      </Points>
    </Piece>
  </UnstructuredGrid>
</VTKFile>"#;

        let err = VtuReader::read_from(std::io::BufReader::new(&xml[..])).unwrap_err();
        assert!(format!("{err}").contains("Points"));
    }
}
