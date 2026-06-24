use std::io::BufRead;
use std::path::Path;

use crate::data::{Points, StructuredGrid};
use crate::types::VtkError;

use crate::io::xml::binary;
use crate::io::xml::vtp_reader::{
    data_array_to_points, detect_format, extract_appended_base64, extract_appended_raw,
    extract_attr, extract_section, parse_attribute_arrays, parse_from_appended, parse_points_ascii,
    DataFormat,
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

    pub fn read_from<R: BufRead>(reader: R) -> Result<StructuredGrid, VtkError> {
        let content: String = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(VtkError::Io)?
            .join("\n");

        // Extract appended data section if present
        let appended_raw = extract_appended_raw(&content);
        let appended_b64 = extract_appended_base64(&content);

        // Parse extent from StructuredGrid tag
        let mut dims = [1usize; 3];
        if let Some(sg_tag) = find_tag(&content, "StructuredGrid") {
            if let Some(ext_str) = extract_attr(&sg_tag, "WholeExtent") {
                let vals: Vec<i64> = ext_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if vals.len() == 6 {
                    dims[0] = (vals[1] - vals[0] + 1).max(1) as usize;
                    dims[1] = (vals[3] - vals[2] + 1).max(1) as usize;
                    dims[2] = (vals[5] - vals[4] + 1).max(1) as usize;
                }
            }
        }

        // Parse Points
        let mut points = Points::new();
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

                points = match detect_format(tag) {
                    DataFormat::Ascii => parse_points_ascii(da_content)?,
                    DataFormat::Binary => {
                        let arr =
                            binary::parse_binary_data_array(da_content, "Points", &type_str, 3)?;
                        data_array_to_points(&arr)?
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
                        data_array_to_points(&arr)?
                    }
                };
            }
        }

        let expected = dims[0] * dims[1] * dims[2];
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
        if let Some(pd_section) = extract_section(&content, "PointData") {
            parse_attribute_arrays(
                &pd_section,
                grid.point_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
            )?;
        }

        // Parse CellData
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

fn find_tag(content: &str, tag: &str) -> Option<String> {
    let pat = format!("<{}", tag);
    let start = content.find(&pat)?;
    let end = content[start..].find('>')?;
    Some(content[start..start + end + 1].to_string())
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
}
