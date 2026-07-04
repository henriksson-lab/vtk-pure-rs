use std::io::BufRead;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::VtkError;

use crate::io::xml::binary;

/// Reader for VTK XML PolyData format (.vtp).
///
/// Supports ASCII, binary (base64-encoded), and appended data formats.
pub struct VtpReader;

impl VtpReader {
    pub fn read(path: &Path) -> Result<PolyData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<PolyData, VtkError> {
        let mut bytes = Vec::new();
        let mut reader = reader;
        reader.read_to_end(&mut bytes).map_err(VtkError::Io)?;
        let content = String::from_utf8_lossy(&bytes);

        // Extract appended data section if present (raw binary after '_')
        let appended_raw = extract_appended_raw_from_bytes(&bytes);
        let appended_b64 = extract_appended_base64(&content);
        let header_type = extract_vtk_header_type(&content);

        let piece_tag = find_piece_or_primary_tag(&content, "PolyData")
            .ok_or_else(|| VtkError::Parse("missing PolyData element".into()))?;
        let number_of_points = extract_required_usize_attr(&piece_tag, "NumberOfPoints")?;
        let number_of_verts = extract_usize_attr(&piece_tag, "NumberOfVerts")?.unwrap_or(0);
        let number_of_lines = extract_usize_attr(&piece_tag, "NumberOfLines")?.unwrap_or(0);
        let number_of_strips = extract_usize_attr(&piece_tag, "NumberOfStrips")?.unwrap_or(0);
        let number_of_polys = extract_usize_attr(&piece_tag, "NumberOfPolys")?.unwrap_or(0);

        let mut pd = PolyData::new();

        // Extract Points
        if let Some(points_section) = extract_section(&content, "Points") {
            if points_section.matches("<DataArray").count() == 1 {
                pd.points = parse_points_section(
                    &points_section,
                    appended_raw.as_deref(),
                    appended_b64.as_deref(),
                    header_type.as_deref(),
                )?;
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
        if pd.points.len() != number_of_points {
            return Err(VtkError::Parse(format!(
                "Piece declares {} points but {} were read",
                number_of_points,
                pd.points.len()
            )));
        }

        // Extract cell sections
        if let Some(polys_section) = extract_section(&content, "Polys") {
            pd.polys = parse_cell_section(
                &polys_section,
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                header_type.as_deref(),
            )?;
            validate_cell_count("Polys", pd.polys.num_cells(), number_of_polys)?;
        }
        if let Some(lines_section) = extract_section(&content, "Lines") {
            pd.lines = parse_cell_section(
                &lines_section,
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                header_type.as_deref(),
            )?;
            validate_cell_count("Lines", pd.lines.num_cells(), number_of_lines)?;
        }
        if let Some(verts_section) = extract_section(&content, "Verts") {
            pd.verts = parse_cell_section(
                &verts_section,
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                header_type.as_deref(),
            )?;
            validate_cell_count("Verts", pd.verts.num_cells(), number_of_verts)?;
        }
        if let Some(strips_section) = extract_section(&content, "Strips") {
            pd.strips = parse_cell_section(
                &strips_section,
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                header_type.as_deref(),
            )?;
            validate_cell_count("Strips", pd.strips.num_cells(), number_of_strips)?;
        }

        // Extract PointData
        if let Some((tag, pd_section)) = extract_section_with_tag(&content, "PointData") {
            parse_attribute_arrays_with_hints_and_header_type(
                &pd_section,
                pd.point_data_mut(),
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
                pd.cell_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
                Some(&tag),
                header_type.as_deref(),
            )?;
        }

        Ok(pd)
    }
}

fn find_piece_or_primary_tag(content: &str, primary: &str) -> Option<String> {
    find_open_tag(content, "Piece").or_else(|| find_open_tag(content, primary))
}

fn find_open_tag(content: &str, tag: &str) -> Option<String> {
    let open_pattern = format!("<{}", tag);
    let start = content.find(&open_pattern)?;
    let tag_end = content[start..].find('>')?;
    Some(content[start..start + tag_end + 1].to_string())
}

fn extract_required_usize_attr(tag: &str, attr_name: &str) -> Result<usize, VtkError> {
    let value = extract_attr(tag, attr_name)
        .ok_or_else(|| VtkError::Parse(format!("Piece is missing its {attr_name} attribute")))?;
    value
        .parse()
        .map_err(|_| VtkError::Parse(format!("invalid {attr_name} attribute: {value}")))
}

fn extract_usize_attr(tag: &str, attr_name: &str) -> Result<Option<usize>, VtkError> {
    extract_attr(tag, attr_name)
        .map(|value| {
            value
                .parse()
                .map_err(|_| VtkError::Parse(format!("invalid {attr_name} attribute: {value}")))
        })
        .transpose()
}

fn validate_cell_count(tag: &str, actual: usize, expected: usize) -> Result<(), VtkError> {
    if actual != expected {
        return Err(VtkError::Parse(format!(
            "{tag} declares {expected} cells but {actual} were read"
        )));
    }
    Ok(())
}

/// Extract content between <tag> and </tag>.
pub(crate) fn extract_section(content: &str, tag: &str) -> Option<String> {
    extract_section_with_tag(content, tag).map(|(_, section)| section)
}

/// Extract an opening tag and content between <tag> and </tag>.
pub(crate) fn extract_section_with_tag(content: &str, tag: &str) -> Option<(String, String)> {
    let open_pattern = format!("<{}", tag);
    let close_pattern = format!("</{}>", tag);

    let start = content.find(&open_pattern)?;
    let after_open = &content[start..];
    let tag_end = after_open.find('>')?;
    let open_tag = after_open[..tag_end + 1].to_string();
    let content_start = start + tag_end + 1;

    let end = content[content_start..].find(&close_pattern)?;
    Some((
        open_tag,
        content[content_start..content_start + end].to_string(),
    ))
}

/// Extract attribute from a tag string.
pub(crate) fn extract_attr(tag: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=", attr_name);
    let start = tag.match_indices(&pattern).find_map(|(start, _)| {
        let is_name_boundary = start == 0
            || tag[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c == '<' || c.is_whitespace());
        is_name_boundary.then_some(start)
    })?;
    let mut value_start = start + pattern.len();
    while tag[value_start..].starts_with(char::is_whitespace) {
        value_start += tag[value_start..].chars().next()?.len_utf8();
    }
    let quote = tag[value_start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    value_start += quote.len_utf8();
    let end = tag[value_start..].find(quote)?;
    Some(xml_unescape_attr(&tag[value_start..value_start + end]))
}

fn xml_unescape_attr(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Extract raw appended data (encoding="raw") - bytes after the '_' marker.
pub(crate) fn extract_appended_raw(content: &str) -> Option<Vec<u8>> {
    extract_appended_raw_from_bytes(content.as_bytes())
}

pub(crate) fn extract_appended_raw_from_bytes(content: &[u8]) -> Option<Vec<u8>> {
    let section_start = find_bytes(content, b"<AppendedData")?;
    let tag_end_rel = content[section_start..].iter().position(|&b| b == b'>')?;
    let tag_end = section_start + tag_end_rel;
    let tag = std::str::from_utf8(&content[section_start..=tag_end]).ok()?;

    let encoding = extract_attr(tag, "encoding").unwrap_or_default();
    if encoding != "raw" {
        return None;
    }

    let mut data_start = tag_end + 1;
    while data_start < content.len() && content[data_start].is_ascii_whitespace() {
        data_start += 1;
    }
    if data_start < content.len() && content[data_start] == b'_' {
        data_start += 1;
    }
    let end_tag = find_bytes(&content[data_start..], b"</AppendedData>")?;
    if end_tag == 0 {
        return None;
    }
    Some(content[data_start..data_start + end_tag].to_vec())
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Extract base64-encoded appended data.
pub(crate) fn extract_appended_base64(content: &str) -> Option<String> {
    let section_start = content.find("<AppendedData")?;
    let tag_str = &content[section_start..];
    let tag_end = tag_str.find('>')?;
    let tag = &tag_str[..tag_end + 1];

    let encoding = extract_attr(tag, "encoding").unwrap_or_default();
    if encoding != "base64" {
        return None;
    }

    let after_tag = &content[section_start + tag_end + 1..];
    let underscore_pos = after_tag.find('_')?;
    let data_start = underscore_pos + 1;
    let end_tag = after_tag.find("</AppendedData>")?;
    if data_start >= end_tag {
        return None;
    }
    Some(after_tag[data_start..end_tag].to_string())
}

/// Determine the format of a DataArray tag.
pub(crate) enum DataFormat {
    Ascii,
    Binary,
    Appended(usize),
}

pub(crate) fn detect_format(tag: &str) -> DataFormat {
    let format = extract_attr(tag, "format").unwrap_or_else(|| "ascii".to_string());
    match format.as_str() {
        "binary" => DataFormat::Binary,
        "appended" => {
            let offset: usize = extract_attr(tag, "offset")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            DataFormat::Appended(offset)
        }
        _ => DataFormat::Ascii,
    }
}

pub(crate) fn extract_vtk_header_type(content: &str) -> Option<String> {
    let start = content.find("<VTKFile")?;
    let tag_end = content[start..].find('>')?;
    let tag = &content[start..start + tag_end + 1];
    extract_attr(tag, "header_type")
}

fn parse_points_section(
    section: &str,
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
    header_type: Option<&str>,
) -> Result<Points<f64>, VtkError> {
    // Find the DataArray tag in the section
    let da_start = section
        .find("<DataArray")
        .ok_or_else(|| VtkError::Parse("no DataArray in Points".into()))?;
    let tag_end = section[da_start..]
        .find('>')
        .ok_or_else(|| VtkError::Parse("unclosed DataArray tag".into()))?;
    let tag = &section[da_start..da_start + tag_end + 1];
    let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Float64".to_string());

    let content;
    if tag.trim_end().ends_with("/>") {
        content = "";
    } else {
        let content_start = da_start + tag_end + 1;
        let content_end = section[content_start..]
            .find("</DataArray>")
            .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
        content = section[content_start..content_start + content_end].trim();
    }

    match detect_format(tag) {
        DataFormat::Ascii => parse_points_ascii(content),
        DataFormat::Binary => {
            let arr = binary::parse_binary_data_array_with_header_type(
                content,
                "Points",
                &type_str,
                3,
                header_type,
            )?;
            data_array_to_points(&arr)
        }
        DataFormat::Appended(offset) => {
            let arr = parse_from_appended_with_header_type(
                appended_raw,
                appended_b64,
                offset,
                "Points",
                &type_str,
                3,
                header_type,
            )?;
            data_array_to_points(&arr)
        }
    }
}

pub(crate) fn data_array_to_points(arr: &AnyDataArray) -> Result<Points<f64>, VtkError> {
    let mut pts = Points::new();
    let nt = arr.num_tuples();
    let mut buf = [0.0f64; 3];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        pts.push(buf);
    }
    Ok(pts)
}

pub(crate) fn parse_points_ascii(data: &str) -> Result<Points<f64>, VtkError> {
    let values: Vec<f64> = data
        .split_whitespace()
        .map(|s| {
            s.parse()
                .map_err(|_| VtkError::Parse(format!("invalid point coordinate: {}", s)))
        })
        .collect::<Result<_, _>>()?;
    if values.len() % 3 != 0 {
        return Err(VtkError::Parse(format!(
            "POINTS data has {} values, expected a multiple of 3",
            values.len()
        )));
    }
    let mut pts = Points::new();
    for chunk in values.chunks(3) {
        pts.push([chunk[0], chunk[1], chunk[2]]);
    }
    Ok(pts)
}

fn parse_cell_section(
    section: &str,
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
    header_type: Option<&str>,
) -> Result<CellArray, VtkError> {
    let mut connectivity_values: Vec<i64> = Vec::new();
    let mut offsets_values: Vec<i64> = Vec::new();
    let mut found_connectivity = false;
    let mut found_offsets = false;

    let mut search_pos = 0;
    while let Some(da_start) = section[search_pos..].find("<DataArray") {
        let abs_start = search_pos + da_start;
        let tag_end = section[abs_start..]
            .find('>')
            .ok_or_else(|| VtkError::Parse("unclosed DataArray tag".into()))?;
        let tag = &section[abs_start..abs_start + tag_end + 1];

        let (content, next_search_pos) = if tag.trim_end().ends_with("/>") {
            ("", abs_start + tag_end + 1)
        } else {
            let content_start = abs_start + tag_end + 1;
            let content_end = section[content_start..]
                .find("</DataArray>")
                .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
            (
                section[content_start..content_start + content_end].trim(),
                content_start + content_end + "</DataArray>".len(),
            )
        };

        let name = extract_attr(tag, "Name").unwrap_or_default();
        let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Int64".to_string());

        let values: Vec<i64> = match detect_format(tag) {
            DataFormat::Ascii => content
                .split_whitespace()
                .map(|s| {
                    s.parse()
                        .map_err(|_| VtkError::Parse(format!("invalid cell integer: {}", s)))
                })
                .collect::<Result<_, _>>()?,
            DataFormat::Binary => {
                let arr = binary::parse_binary_data_array_with_header_type(
                    content,
                    &name,
                    &type_str,
                    1,
                    header_type,
                )?;
                any_data_array_to_i64(&arr)
            }
            DataFormat::Appended(offset) => {
                let arr = parse_from_appended_with_header_type(
                    appended_raw,
                    appended_b64,
                    offset,
                    &name,
                    &type_str,
                    1,
                    header_type,
                )?;
                any_data_array_to_i64(&arr)
            }
        };

        match name.as_str() {
            "connectivity" => {
                connectivity_values = values;
                found_connectivity = true;
            }
            "offsets" => {
                offsets_values = values;
                found_offsets = true;
            }
            _ => {}
        }

        search_pos = next_search_pos;
    }

    if found_offsets && !found_connectivity {
        return Err(VtkError::Parse(
            "Cannot read cell connectivity because the \"connectivity\" array could not be found."
                .into(),
        ));
    }
    if found_connectivity && !found_offsets {
        return Err(VtkError::Parse(
            "Cannot read cell offsets because the \"offsets\" array could not be found.".into(),
        ));
    }

    let mut cells = CellArray::new();
    let mut prev_offset = 0;
    for &offset in &offsets_values {
        if offset < prev_offset {
            return Err(VtkError::Parse(
                "cell offsets are not non-decreasing".into(),
            ));
        }
        if offset < 0 {
            return Err(VtkError::Parse("cell offset is negative".into()));
        }
        let start = prev_offset as usize;
        let end = offset as usize;
        if end > connectivity_values.len() {
            return Err(VtkError::Parse(
                "cell offsets exceed connectivity length".into(),
            ));
        }
        cells.push_cell(&connectivity_values[start..end]);
        prev_offset = offset;
    }
    if prev_offset as usize != connectivity_values.len() {
        return Err(VtkError::Parse(
            "cell offsets do not consume all connectivity entries".into(),
        ));
    }

    Ok(cells)
}

pub(crate) fn any_data_array_to_i64(arr: &AnyDataArray) -> Vec<i64> {
    match arr {
        AnyDataArray::F32(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::F64(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::I8(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::I16(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::I32(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::I64(a) => a.as_slice().to_vec(),
        AnyDataArray::U8(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::U16(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::U32(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
        AnyDataArray::U64(a) => a.as_slice().iter().map(|&v| v as i64).collect(),
    }
}

pub(crate) fn parse_from_appended(
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
    offset: usize,
    name: &str,
    type_str: &str,
    nc: usize,
) -> Result<AnyDataArray, VtkError> {
    if let Some(raw) = appended_raw {
        return binary::parse_appended_data_array(raw, offset, name, type_str, nc);
    }
    if let Some(b64) = appended_b64 {
        return binary::parse_appended_base64_data_array(b64, offset, name, type_str, nc);
    }
    Err(VtkError::Parse(
        "appended format specified but no AppendedData section found".into(),
    ))
}

pub(crate) fn parse_from_appended_with_header_type(
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
    offset: usize,
    name: &str,
    type_str: &str,
    nc: usize,
    header_type: Option<&str>,
) -> Result<AnyDataArray, VtkError> {
    if let Some(raw) = appended_raw {
        return binary::parse_appended_data_array_with_header_type(
            raw,
            offset,
            name,
            type_str,
            nc,
            header_type,
        );
    }
    if let Some(b64) = appended_b64 {
        return binary::parse_appended_base64_data_array_with_header_type(
            b64,
            offset,
            name,
            type_str,
            nc,
            header_type,
        );
    }
    Err(VtkError::Parse(
        "appended format specified but no AppendedData section found".into(),
    ))
}

pub(crate) fn parse_attribute_arrays(
    section: &str,
    attrs: &mut crate::data::DataSetAttributes,
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
) -> Result<(), VtkError> {
    parse_attribute_arrays_with_hints(section, attrs, appended_raw, appended_b64, None)
}

pub(crate) fn parse_attribute_arrays_with_hints(
    section: &str,
    attrs: &mut crate::data::DataSetAttributes,
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
    section_tag: Option<&str>,
) -> Result<(), VtkError> {
    parse_attribute_arrays_with_hints_and_header_type(
        section,
        attrs,
        appended_raw,
        appended_b64,
        section_tag,
        None,
    )
}

pub(crate) fn parse_attribute_arrays_with_hints_and_header_type(
    section: &str,
    attrs: &mut crate::data::DataSetAttributes,
    appended_raw: Option<&[u8]>,
    appended_b64: Option<&str>,
    section_tag: Option<&str>,
    header_type: Option<&str>,
) -> Result<(), VtkError> {
    let mut search_pos = 0;
    let has_scalars_hint = section_tag.and_then(|tag| extract_attr(tag, "Scalars"));
    while let Some(da_start) = section[search_pos..].find("<DataArray") {
        let abs_start = search_pos + da_start;
        let tag_end = section[abs_start..]
            .find('>')
            .ok_or_else(|| VtkError::Parse("unclosed DataArray tag".into()))?;
        let tag = &section[abs_start..abs_start + tag_end + 1];

        let (content, next_search_pos) = if tag.trim_end().ends_with("/>") {
            ("", abs_start + tag_end + 1)
        } else {
            let content_start = abs_start + tag_end + 1;
            let content_end = section[content_start..]
                .find("</DataArray>")
                .ok_or_else(|| VtkError::Parse("missing </DataArray>".into()))?;
            (
                section[content_start..content_start + content_end].trim(),
                content_start + content_end + "</DataArray>".len(),
            )
        };

        let name = extract_attr(tag, "Name").unwrap_or_else(|| "data".to_string());
        let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Float64".to_string());
        let nc: usize = extract_attr(tag, "NumberOfComponents")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let arr = match detect_format(tag) {
            DataFormat::Ascii => parse_ascii_data_array(content, &name, &type_str, nc)?,
            DataFormat::Binary => binary::parse_binary_data_array_with_header_type(
                content,
                &name,
                &type_str,
                nc,
                header_type,
            )?,
            DataFormat::Appended(offset) => parse_from_appended_with_header_type(
                appended_raw,
                appended_b64,
                offset,
                &name,
                &type_str,
                nc,
                header_type,
            )?,
        };

        let arr_name = arr.name().to_string();
        attrs.add_array(arr);
        if has_scalars_hint.is_none() && attrs.scalars().is_none() {
            attrs.set_active_scalars(&arr_name);
        }

        search_pos = next_search_pos;
    }

    if let Some(tag) = section_tag {
        if let Some(name) = extract_attr(tag, "Scalars") {
            attrs.set_active_scalars(&name);
        }
        if let Some(name) = extract_attr(tag, "Vectors") {
            attrs.set_active_vectors(&name);
        }
        if let Some(name) = extract_attr(tag, "Normals") {
            attrs.set_active_normals(&name);
        }
    }
    Ok(())
}

pub(crate) fn parse_ascii_data_array(
    content: &str,
    name: &str,
    type_str: &str,
    nc: usize,
) -> Result<AnyDataArray, VtkError> {
    fn parse_values<T: std::str::FromStr>(content: &str) -> Result<Vec<T>, VtkError> {
        content
            .split_whitespace()
            .map(|s| {
                s.parse()
                    .map_err(|_| VtkError::Parse(format!("invalid DataArray value: {}", s)))
            })
            .collect()
    }

    let arr = match type_str {
        "Float32" => AnyDataArray::F32(DataArray::from_vec(name, parse_values(content)?, nc)),
        "Float64" => AnyDataArray::F64(DataArray::from_vec(name, parse_values(content)?, nc)),
        "Int8" => AnyDataArray::I8(DataArray::from_vec(name, parse_values(content)?, nc)),
        "Int16" => AnyDataArray::I16(DataArray::from_vec(name, parse_values(content)?, nc)),
        "Int32" => AnyDataArray::I32(DataArray::from_vec(name, parse_values(content)?, nc)),
        "Int64" => AnyDataArray::I64(DataArray::from_vec(name, parse_values(content)?, nc)),
        "UInt8" => AnyDataArray::U8(DataArray::from_vec(name, parse_values(content)?, nc)),
        "UInt16" => AnyDataArray::U16(DataArray::from_vec(name, parse_values(content)?, nc)),
        "UInt32" => AnyDataArray::U32(DataArray::from_vec(name, parse_values(content)?, nc)),
        "UInt64" => AnyDataArray::U64(DataArray::from_vec(name, parse_values(content)?, nc)),
        other => {
            return Err(VtkError::Parse(format!(
                "unsupported DataArray type: {}",
                other
            )))
        }
    };
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataArray as DA;
    use crate::io::xml::VtpWriter;

    #[test]
    fn roundtrip_vtp_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtpReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);
    }

    #[test]
    fn roundtrip_vtp_with_scalars() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DA::from_vec("temperature", vec![10.0f64, 20.0, 30.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("temperature");

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtpReader::read_from(reader).unwrap();

        let s = result.point_data().scalars().unwrap();
        assert_eq!(s.name(), "temperature");
        assert_eq!(s.num_tuples(), 3);
    }

    #[test]
    fn reads_active_attribute_hints() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="1" NumberOfVerts="0" NumberOfLines="0" NumberOfPolys="0" NumberOfStrips="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
      </Points>
      <PointData Scalars="selected" Vectors="velocity">
        <DataArray type="Float64" Name="other" NumberOfComponents="1" format="ascii">1</DataArray>
        <DataArray type="Float64" Name="selected" NumberOfComponents="1" format="ascii">2</DataArray>
        <DataArray type="Float64" Name="velocity" NumberOfComponents="3" format="ascii">1 0 0</DataArray>
      </PointData>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let result = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        assert_eq!(result.point_data().scalars().unwrap().name(), "selected");
        assert_eq!(result.point_data().vectors().unwrap().name(), "velocity");
    }

    #[test]
    fn reads_single_quoted_xml_attributes() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type='PolyData' version='1.0' byte_order='LittleEndian'>
  <PolyData>
    <Piece NumberOfPoints='1' NumberOfVerts='0' NumberOfLines='0' NumberOfPolys='0' NumberOfStrips='0'>
      <Points>
        <DataArray type='Float64' NumberOfComponents='3' format='ascii'>0 0 0</DataArray>
      </Points>
      <PointData Scalars='ids'>
        <DataArray type='UInt16' Name='ids' NumberOfComponents='1' format='ascii'>7</DataArray>
      </PointData>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let result = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        assert_eq!(result.points.len(), 1);
        assert_eq!(result.point_data().scalars().unwrap().name(), "ids");
    }

    #[test]
    fn reads_ascii_integer_array_type() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="1" NumberOfVerts="0" NumberOfLines="0" NumberOfPolys="0" NumberOfStrips="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
      </Points>
      <PointData Scalars="ids">
        <DataArray type="UInt16" Name="ids" NumberOfComponents="1" format="ascii">7</DataArray>
      </PointData>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let result = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        assert!(matches!(
            result.point_data().scalars().unwrap(),
            AnyDataArray::U16(_)
        ));
    }

    #[test]
    fn reads_ascii_uint64_without_float_rounding() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="1" NumberOfVerts="0" NumberOfLines="0" NumberOfPolys="0" NumberOfStrips="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
      </Points>
      <PointData Scalars="ids">
        <DataArray type="UInt64" Name="ids" NumberOfComponents="1" format="ascii">9007199254740993</DataArray>
      </PointData>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let result = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        match result.point_data().scalars().unwrap() {
            AnyDataArray::U64(arr) => assert_eq!(arr.as_slice(), &[9_007_199_254_740_993]),
            other => panic!("unexpected array type: {:?}", other.scalar_type()),
        }
    }

    #[test]
    fn reads_raw_appended_binary_without_utf8_conversion() {
        let mut appended = Vec::new();
        appended.extend_from_slice(&24u32.to_le_bytes());
        for v in [0.0f64, 0.0, 0.0] {
            appended.extend_from_slice(&v.to_le_bytes());
        }
        let connectivity_offset = appended.len();
        appended.extend_from_slice(&8u32.to_le_bytes());
        appended.extend_from_slice(&0i64.to_le_bytes());
        let offsets_offset = appended.len();
        appended.extend_from_slice(&8u32.to_le_bytes());
        appended.extend_from_slice(&1i64.to_le_bytes());
        appended.push(0xff);

        let mut xml = format!(
            r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian" header_type="UInt32">
  <PolyData>
    <Piece NumberOfPoints="1" NumberOfVerts="1" NumberOfLines="0" NumberOfPolys="0" NumberOfStrips="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="appended" offset="0"/>
      </Points>
      <Verts>
        <DataArray type="Int64" Name="connectivity" format="appended" offset="{connectivity_offset}"/>
        <DataArray type="Int64" Name="offsets" format="appended" offset="{offsets_offset}"/>
      </Verts>
    </Piece>
  </PolyData>
  <AppendedData encoding="raw">_"#
        )
        .into_bytes();
        xml.extend_from_slice(&appended);
        xml.extend_from_slice(b"</AppendedData>\n</VTKFile>");

        let result =
            VtpReader::read_from(std::io::BufReader::new(std::io::Cursor::new(xml))).unwrap();

        assert_eq!(result.points.get(0), [0.0, 0.0, 0.0]);
        assert_eq!(result.verts.cell(0), &[0]);
    }

    #[test]
    fn reads_appended_base64_uint64_header() {
        let mut appended = Vec::new();
        appended.extend_from_slice(&24u64.to_le_bytes());
        for v in [1.0f64, 2.0, 3.0] {
            appended.extend_from_slice(&v.to_le_bytes());
        }
        let connectivity_offset = appended.len();
        appended.extend_from_slice(&8u64.to_le_bytes());
        appended.extend_from_slice(&0i64.to_le_bytes());
        let offsets_offset = appended.len();
        appended.extend_from_slice(&8u64.to_le_bytes());
        appended.extend_from_slice(&1i64.to_le_bytes());

        let encoded = binary::base64_encode(&appended);
        let xml = format!(
            r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian" header_type="UInt64">
  <PolyData>
    <Piece NumberOfPoints="1" NumberOfVerts="1" NumberOfLines="0" NumberOfPolys="0" NumberOfStrips="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="appended" offset="0"/>
      </Points>
      <Verts>
        <DataArray type="Int64" Name="connectivity" format="appended" offset="{connectivity_offset}"/>
        <DataArray type="Int64" Name="offsets" format="appended" offset="{offsets_offset}"/>
      </Verts>
    </Piece>
  </PolyData>
  <AppendedData encoding="base64">_{encoded}</AppendedData>
</VTKFile>"#
        );

        let result = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap();

        assert_eq!(result.points.get(0), [1.0, 2.0, 3.0]);
        assert_eq!(result.verts.cell(0), &[0]);
    }

    #[test]
    fn rejects_cell_section_missing_offsets() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="3" NumberOfVerts="0" NumberOfLines="0" NumberOfPolys="1" NumberOfStrips="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0 1 0 0 0 1 0</DataArray>
      </Points>
      <Polys>
        <DataArray type="Int64" Name="connectivity" format="ascii">0 1 2</DataArray>
      </Polys>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let err = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("offsets"));
    }

    #[test]
    fn rejects_missing_number_of_points_attribute() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPolys="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii"></DataArray>
      </Points>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let err = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("NumberOfPoints"));
    }

    #[test]
    fn rejects_points_element_without_exactly_one_array() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="1" NumberOfPolys="0">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0</DataArray>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">1 1 1</DataArray>
      </Points>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let err = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("Points"));
    }

    #[test]
    fn rejects_declared_poly_count_mismatch_when_section_is_read() {
        let xml = r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="1.0" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="3" NumberOfPolys="2">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="ascii">0 0 0 1 0 0 0 1 0</DataArray>
      </Points>
      <Polys>
        <DataArray type="Int64" Name="connectivity" format="ascii">0 1 2</DataArray>
        <DataArray type="Int64" Name="offsets" format="ascii">3</DataArray>
      </Polys>
    </Piece>
  </PolyData>
</VTKFile>"#;

        let err = VtpReader::read_from(std::io::BufReader::new(xml.as_bytes())).unwrap_err();
        assert!(format!("{err}").contains("Polys declares 2 cells"));
    }

    #[test]
    fn roundtrip_vtp_quad() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3], [1, 4, 5], [1, 5, 2]],
        );

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtpReader::read_from(reader).unwrap();

        assert_eq!(result.points.len(), 6);
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn read_binary_format_vtp() {
        // Construct a minimal VTP with format="binary" DataArrays
        // Points: 3 vertices, Float64, 3 components
        let mut points_raw = Vec::new();
        let point_data_bytes = 3 * 3 * 8u32; // 3 points * 3 components * 8 bytes
        points_raw.extend_from_slice(&point_data_bytes.to_le_bytes());
        for &v in &[0.0f64, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
            points_raw.extend_from_slice(&v.to_le_bytes());
        }
        let points_b64 = base64_encode_test(&points_raw);

        // Connectivity: [0,1,2]
        let mut conn_raw = Vec::new();
        let conn_bytes = 3 * 8u32;
        conn_raw.extend_from_slice(&conn_bytes.to_le_bytes());
        for &v in &[0i64, 1, 2] {
            conn_raw.extend_from_slice(&v.to_le_bytes());
        }
        let conn_b64 = base64_encode_test(&conn_raw);

        // Offsets: [3]
        let mut off_raw = Vec::new();
        let off_bytes = 1 * 8u32;
        off_raw.extend_from_slice(&off_bytes.to_le_bytes());
        off_raw.extend_from_slice(&3i64.to_le_bytes());
        let off_b64 = base64_encode_test(&off_raw);

        let xml = format!(
            r#"<?xml version="1.0"?>
<VTKFile type="PolyData" version="0.1" byte_order="LittleEndian">
  <PolyData>
    <Piece NumberOfPoints="3" NumberOfPolys="1">
      <Points>
        <DataArray type="Float64" NumberOfComponents="3" format="binary">{}</DataArray>
      </Points>
      <Polys>
        <DataArray type="Int64" Name="connectivity" format="binary">{}</DataArray>
        <DataArray type="Int64" Name="offsets" format="binary">{}</DataArray>
      </Polys>
    </Piece>
  </PolyData>
</VTKFile>"#,
            points_b64, conn_b64, off_b64
        );

        let reader = std::io::BufReader::new(xml.as_bytes());
        let result = VtpReader::read_from(reader).unwrap();
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.polys.cell(0), &[0, 1, 2]);

        let p1 = result.points.get(1);
        assert!((p1[0] - 1.0).abs() < 1e-10);
    }

    fn base64_encode_test(data: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
            let triple = (b0 << 16) | (b1 << 8) | b2;
            result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
            result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(CHARS[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }
        result
    }
}
