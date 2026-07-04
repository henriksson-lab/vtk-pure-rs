use std::io::BufRead;
use std::path::Path;

use crate::data::RectilinearGrid;
use crate::types::VtkError;

use crate::io::xml::binary;
use crate::io::xml::vtp_reader::{
    detect_format, extract_appended_base64, extract_appended_raw_from_bytes, extract_attr,
    extract_section_with_tag, extract_vtk_header_type,
    parse_attribute_arrays_with_hints_and_header_type, parse_from_appended_with_header_type,
    DataFormat,
};

/// Reader for VTK XML RectilinearGrid format (.vtr).
///
/// Supports ASCII, binary (base64-encoded), and appended data formats.
pub struct VtrReader;

impl VtrReader {
    pub fn read(path: &Path) -> Result<RectilinearGrid, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<RectilinearGrid, VtkError> {
        let mut bytes = Vec::new();
        let mut reader = reader;
        reader.read_to_end(&mut bytes).map_err(VtkError::Io)?;
        let content = String::from_utf8_lossy(&bytes);

        // Extract appended data section if present
        let appended_raw = extract_appended_raw_from_bytes(&bytes);
        let appended_b64 = extract_appended_base64(&content);
        let header_type = extract_vtk_header_type(&content);

        let piece_dims = read_piece_or_primary_dimensions(&content, "RectilinearGrid")?;

        let mut coords: [Option<Vec<f64>>; 3] = [None, None, None];

        // Parse Coordinates section
        let has_coordinates = if let Some((_, coords_section)) =
            extract_section_with_tag(&content, "Coordinates")
        {
            let mut search_pos = 0;
            let mut coord_index = 0usize;
            while let Some(da_start) = coords_section[search_pos..].find("<DataArray") {
                let abs_start = search_pos + da_start;
                let tag_end = coords_section[abs_start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed tag".into()))?;
                let tag = &coords_section[abs_start..abs_start + tag_end + 1];
                let (da_content, next_search_pos) = if tag.trim_end().ends_with("/>") {
                    ("", abs_start + tag_end + 1)
                } else {
                    let content_start = abs_start + tag_end + 1;
                    let content_end = coords_section[content_start..]
                        .find("</DataArray>")
                        .ok_or_else(|| VtkError::Parse("missing close".into()))?;
                    (
                        coords_section[content_start..content_start + content_end].trim(),
                        content_start + content_end + "</DataArray>".len(),
                    )
                };

                let name = extract_attr(tag, "Name").unwrap_or_default();
                let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Float64".to_string());

                let values: Vec<f64> = match detect_format(tag) {
                    DataFormat::Ascii => da_content
                        .split_whitespace()
                        .map(|s| {
                            s.parse().map_err(|_| {
                                VtkError::Parse(format!("invalid coordinate value: {}", s))
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
                        any_data_array_to_f64(&arr)
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
                        any_data_array_to_f64(&arr)
                    }
                };

                match coord_index {
                    0 => coords[0] = Some(values),
                    1 => coords[1] = Some(values),
                    2 => coords[2] = Some(values),
                    _ => {}
                }

                coord_index += 1;
                search_pos = next_search_pos;
            }

            if coord_index != 3 {
                return Err(VtkError::Parse(format!(
                    "Coordinates element has {coord_index} arrays, expected 3"
                )));
            }
            true
        } else {
            false
        };

        let expected_dims = piece_dims;
        if !has_coordinates && expected_dims.iter().all(|&dim| dim > 0) {
            return Err(VtkError::Parse(
                "A piece is missing its Coordinates element.".into(),
            ));
        }

        let x_coords = coords[0].take().unwrap_or_else(|| vec![0.0]);
        let y_coords = coords[1].take().unwrap_or_else(|| vec![0.0]);
        let z_coords = coords[2].take().unwrap_or_else(|| vec![0.0]);

        for (axis, (actual, expected)) in [x_coords.len(), y_coords.len(), z_coords.len()]
            .into_iter()
            .zip(expected_dims)
            .enumerate()
        {
            if actual != expected {
                return Err(VtkError::Parse(format!(
                    "coordinate axis {axis} has {actual} tuples, expected {expected}"
                )));
            }
        }

        let mut grid = RectilinearGrid::from_coords(x_coords, y_coords, z_coords);

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

/// Extract f64 values from any data array.
fn any_data_array_to_f64(arr: &crate::data::AnyDataArray) -> Vec<f64> {
    let nt = arr.num_tuples();
    let nc = arr.num_components();
    let mut result = Vec::with_capacity(nt * nc);
    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        result.extend_from_slice(&buf);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray as DA, DataSet};
    use crate::io::xml::VtrWriter;

    #[test]
    fn roundtrip_vtr() {
        let mut grid =
            RectilinearGrid::from_coords(vec![0.0, 1.0, 3.0], vec![0.0, 2.0], vec![0.0, 5.0]);
        let n = grid.num_points();
        let scalars: Vec<f64> = (0..n).map(|i| i as f64).collect();
        let arr = DA::from_vec("idx", scalars, 1);
        grid.point_data_mut().add_array(arr.into());
        grid.point_data_mut().set_active_scalars("idx");

        let mut buf = Vec::new();
        VtrWriter::write_to(&mut buf, &grid).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtrReader::read_from(reader).unwrap();

        assert_eq!(result.dimensions(), [3, 2, 2]);
        assert_eq!(result.x_coords(), &[0.0, 1.0, 3.0]);
        assert_eq!(result.y_coords(), &[0.0, 2.0]);
        assert_eq!(result.z_coords(), &[0.0, 5.0]);

        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.num_tuples(), 12);
    }

    #[test]
    fn reads_coordinates_by_position_and_attribute_hints() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian">
  <RectilinearGrid WholeExtent="0 1 0 1 0 0">
    <Piece Extent="0 1 0 1 0 0">
      <PointData Scalars="ids" Vectors="velocity">
        <DataArray type="UInt16" Name="ids" NumberOfComponents="1" format="ascii">1 2 3 4</DataArray>
        <DataArray type="Float64" Name="velocity" NumberOfComponents="3" format="ascii">1 0 0 0 1 0 0 0 1 1 1 1</DataArray>
      </PointData>
      <Coordinates>
        <DataArray type="Float64" Name="not_x" format="ascii">0 2</DataArray>
        <DataArray type="Float64" Name="not_y" format="ascii">10 20</DataArray>
        <DataArray type="Float64" Name="not_z" format="ascii">5</DataArray>
      </Coordinates>
    </Piece>
  </RectilinearGrid>
</VTKFile>"#;

        let result = VtrReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        assert_eq!(result.x_coords(), &[0.0, 2.0]);
        assert_eq!(result.y_coords(), &[10.0, 20.0]);
        assert_eq!(result.z_coords(), &[5.0]);
        assert!(matches!(
            result.point_data().scalars().unwrap(),
            crate::data::AnyDataArray::U16(_)
        ));
        assert_eq!(result.point_data().vectors().unwrap().name(), "velocity");
    }

    #[test]
    fn rejects_missing_coordinates_for_non_empty_piece() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian">
  <RectilinearGrid WholeExtent="0 1 0 1 0 0">
    <Piece Extent="0 1 0 1 0 0">
      <PointData>
      </PointData>
      <CellData>
      </CellData>
    </Piece>
  </RectilinearGrid>
</VTKFile>"#;

        let err = VtrReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("Coordinates"));
    }

    #[test]
    fn rejects_piece_without_extent() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian">
  <RectilinearGrid WholeExtent="0 0 0 0 0 0">
    <Piece>
      <Coordinates>
        <DataArray type="Float64" Name="x" format="ascii">0</DataArray>
        <DataArray type="Float64" Name="y" format="ascii">0</DataArray>
        <DataArray type="Float64" Name="z" format="ascii">0</DataArray>
      </Coordinates>
    </Piece>
  </RectilinearGrid>
</VTKFile>"#;

        let err = VtrReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("extent"));
    }

    #[test]
    fn reads_raw_appended_coordinates_without_utf8_conversion() {
        let mut appended = Vec::new();
        appended.extend_from_slice(&16u32.to_le_bytes());
        appended.extend_from_slice(&0.0f64.to_le_bytes());
        appended.extend_from_slice(&2.0f64.to_le_bytes());
        let y_offset = appended.len();
        appended.extend_from_slice(&8u32.to_le_bytes());
        appended.extend_from_slice(&10.0f64.to_le_bytes());
        let z_offset = appended.len();
        appended.extend_from_slice(&8u32.to_le_bytes());
        appended.extend_from_slice(&5.0f64.to_le_bytes());
        appended.push(0xff);

        let mut xml = format!(
            r#"<?xml version="1.0"?>
<VTKFile type="RectilinearGrid" version="1.0" byte_order="LittleEndian" header_type="UInt32">
  <RectilinearGrid WholeExtent="0 1 0 0 0 0">
    <Piece Extent="0 1 0 0 0 0">
      <Coordinates>
        <DataArray type="Float64" Name="x" format="appended" offset="0"/>
        <DataArray type="Float64" Name="y" format="appended" offset="{y_offset}"/>
        <DataArray type="Float64" Name="z" format="appended" offset="{z_offset}"/>
      </Coordinates>
    </Piece>
  </RectilinearGrid>
  <AppendedData encoding="raw">_"#
        )
        .into_bytes();
        xml.extend_from_slice(&appended);
        xml.extend_from_slice(b"</AppendedData>\n</VTKFile>");

        let result =
            VtrReader::read_from(std::io::BufReader::new(std::io::Cursor::new(xml))).unwrap();

        assert_eq!(result.x_coords(), &[0.0, 2.0]);
        assert_eq!(result.y_coords(), &[10.0]);
        assert_eq!(result.z_coords(), &[5.0]);
    }
}
