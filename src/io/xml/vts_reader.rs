use std::io::BufRead;
use std::path::Path;

use crate::data::{Points, StructuredGrid};
use crate::types::VtkError;

use crate::io::xml::binary;
use crate::io::xml::vtp_reader::{
    data_array_to_points, detect_format, extract_appended_base64, extract_appended_raw_from_bytes,
    extract_attr, extract_section, extract_section_with_tag, extract_vtk_header_type,
    parse_attribute_arrays_with_hints_and_header_type, parse_from_appended_with_header_type,
    parse_points_ascii, DataFormat,
};

/// Reader for VTK XML StructuredGrid format (.vts).
///
/// Supports ASCII, binary (base64-encoded), and appended data formats.
pub struct VtsReader;

impl VtsReader {
    pub fn read(path: &Path) -> Result<StructuredGrid, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(mut reader: R) -> Result<StructuredGrid, VtkError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(VtkError::Io)?;
        let content = String::from_utf8_lossy(&bytes).into_owned();

        // Extract appended data section if present
        let appended_raw = extract_appended_raw_from_bytes(&bytes);
        let appended_b64 = extract_appended_base64(&content);
        let header_type = extract_vtk_header_type(&content);

        // Parse extent from Piece tag. The dataset WholeExtent may cover more
        // than the currently read piece; VTK allocates and reads by piece extent.
        let dims = read_piece_or_primary_dimensions(&content, "StructuredGrid")?;

        // Parse Points
        let mut points = Points::new();
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

                points = match detect_format(tag) {
                    DataFormat::Ascii => parse_points_ascii(da_content)?,
                    DataFormat::Binary => {
                        let arr = binary::parse_binary_data_array_with_header_type(
                            da_content,
                            "Points",
                            &type_str,
                            3,
                            header_type.as_deref(),
                        )?;
                        data_array_to_points(&arr)?
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
                        data_array_to_points(&arr)?
                    }
                };
            }
        }

        let expected = dims[0] * dims[1] * dims[2];
        if points.is_empty() && expected > 0 {
            return Err(VtkError::Parse(
                "A piece is missing its Points element or element does not have exactly 1 array."
                    .into(),
            ));
        }
        if points.len() != expected {
            return Err(VtkError::Parse(format!(
                "expected {} points for dims {:?}, got {}",
                expected,
                dims,
                points.len()
            )));
        }

        let mut grid = StructuredGrid::from_dimensions_and_points(dims, points);

        // Parse PointData
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

        // Parse CellData
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

fn find_tag(content: &str, tag: &str) -> Option<String> {
    let pat = format!("<{}", tag);
    let start = content.find(&pat)?;
    let end = content[start..].find('>')?;
    Some(content[start..start + end + 1].to_string())
}

fn read_piece_or_primary_dimensions(content: &str, primary: &str) -> Result<[usize; 3], VtkError> {
    if let Some(piece_tag) = find_tag(content, "Piece") {
        let extent = extract_attr(&piece_tag, "Extent")
            .ok_or_else(|| VtkError::Parse("Piece has no extent.".into()))?;
        return extent_dimensions(&extent)
            .ok_or_else(|| VtkError::Parse("Extent attribute is not 6 integers.".into()));
    }

    let primary_tag = find_tag(content, primary)
        .ok_or_else(|| VtkError::Parse(format!("missing {primary} element")))?;
    let extent = extract_attr(&primary_tag, "WholeExtent")
        .ok_or_else(|| VtkError::Parse(format!("{primary} element has no WholeExtent.")))?;
    extent_dimensions(&extent)
        .ok_or_else(|| VtkError::Parse("WholeExtent attribute is not 6 integers.".into()))
}

fn extent_dimensions(extent: &str) -> Option<[usize; 3]> {
    let vals: Vec<i64> = extent
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    if vals.len() != 6 {
        return None;
    }
    Some([
        (vals[1] - vals[0] + 1).max(0) as usize,
        (vals[3] - vals[2] + 1).max(0) as usize,
        (vals[5] - vals[4] + 1).max(0) as usize,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray as DA, DataSet};
    use crate::io::xml::VtsWriter;

    #[test]
    fn roundtrip_vts() {
        let mut pts = Points::new();
        for j in 0..2 {
            for i in 0..3 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        let mut grid = StructuredGrid::from_dimensions_and_points([3, 2, 1], pts);
        let scalars: Vec<f64> = (0..6).map(|i| i as f64).collect();
        grid.point_data_mut()
            .add_array(DA::from_vec("idx", scalars, 1).into());
        grid.point_data_mut().set_active_scalars("idx");

        let mut buf = Vec::new();
        VtsWriter::write_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtsReader::read_from(reader).unwrap();

        assert_eq!(result.dimensions(), [3, 2, 1]);
        assert_eq!(result.num_points(), 6);
        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 6);
    }

    #[test]
    fn reads_appended_base64_uint64_header_and_attribute_hints() {
        let mut appended = Vec::new();
        appended.extend_from_slice(&24u64.to_le_bytes());
        for v in [1.0f64, 2.0, 3.0] {
            appended.extend_from_slice(&v.to_le_bytes());
        }
        let ids_offset = appended.len();
        appended.extend_from_slice(&2u64.to_le_bytes());
        appended.extend_from_slice(&7u16.to_le_bytes());

        let encoded = binary::base64_encode(&appended);
        let xml = format!(
            r#"<?xml version="1.0"?>
<VTKFile type="StructuredGrid" version="1.0" byte_order="LittleEndian" header_type="UInt64">
  <StructuredGrid WholeExtent="0 0 0 0 0 0">
    <Piece Extent="0 0 0 0 0 0">
      <PointData Scalars="ids">
        <DataArray type="UInt16" Name="ids" NumberOfComponents="1" format="appended" offset="{ids_offset}"/>
      </PointData>
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="appended" offset="0"/>
      </Points>
    </Piece>
  </StructuredGrid>
  <AppendedData encoding="base64">_{encoded}</AppendedData>
</VTKFile>"#
        );

        let result = VtsReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
        assert!(matches!(
            result.point_data().scalars().unwrap(),
            crate::data::AnyDataArray::U16(_)
        ));
    }

    #[test]
    fn reads_raw_appended_binary_without_utf8_conversion() {
        let mut xml = br#"<?xml version="1.0"?>
<VTKFile type="StructuredGrid" version="1.0" byte_order="LittleEndian" header_type="UInt32">
  <StructuredGrid WholeExtent="0 0 0 0 0 0">
    <Piece Extent="0 0 0 0 0 0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="appended" offset="0"/>
      </Points>
    </Piece>
  </StructuredGrid>
  <AppendedData encoding="raw">_"#
            .to_vec();
        xml.extend_from_slice(&24u32.to_le_bytes());
        for v in [1.0f64, 2.0, 3.0] {
            xml.extend_from_slice(&v.to_le_bytes());
        }
        xml.extend_from_slice(b"</AppendedData>\n</VTKFile>");

        let result = VtsReader::read_from(std::io::BufReader::new(&xml[..])).unwrap();

        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn uses_piece_extent_for_dimensions() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="StructuredGrid" version="1.0" byte_order="LittleEndian">
  <StructuredGrid WholeExtent="0 3 0 3 0 0">
    <Piece Extent="0 1 0 0 0 0">
      <PointData>
      </PointData>
      <CellData>
      </CellData>
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0 1 0 0</DataArray>
      </Points>
    </Piece>
  </StructuredGrid>
</VTKFile>"#;

        let result = VtsReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();
        assert_eq!(result.dimensions(), [2, 1, 1]);
    }

    #[test]
    fn rejects_piece_without_extent() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="StructuredGrid" version="1.0" byte_order="LittleEndian">
  <StructuredGrid WholeExtent="0 0 0 0 0 0">
    <Piece>
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
      </Points>
    </Piece>
  </StructuredGrid>
</VTKFile>"#;

        let err = VtsReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("extent"));
    }

    #[test]
    fn rejects_points_element_without_exactly_one_array() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="StructuredGrid" version="1.0" byte_order="LittleEndian">
  <StructuredGrid WholeExtent="0 0 0 0 0 0">
    <Piece Extent="0 0 0 0 0 0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">1 1 1</DataArray>
      </Points>
    </Piece>
  </StructuredGrid>
</VTKFile>"#;

        let err = VtsReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("Points"));
    }
}
