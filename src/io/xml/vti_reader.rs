use std::io::BufRead;
use std::path::Path;

use crate::data::ImageData;
use crate::types::VtkError;

use crate::io::xml::vtp_reader::{
    extract_appended_base64, extract_appended_raw, extract_attr, extract_section,
    parse_attribute_arrays,
};

/// Reader for VTK XML ImageData format (.vti).
///
/// Supports ASCII, binary (base64-encoded), and appended data formats.
pub struct VtiReader;

impl VtiReader {
    pub fn read(path: &Path) -> Result<ImageData, VtkError> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Self::read_from(reader)
    }

    pub fn read_from<R: BufRead>(reader: R) -> Result<ImageData, VtkError> {
        let content: String = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(VtkError::Io)?
            .join("\n");

        // Extract appended data section if present
        let appended_raw = extract_appended_raw(&content);
        let appended_b64 = extract_appended_base64(&content);

        let mut image = ImageData::new();

        // Parse ImageData attributes from the tag
        if let Some(id_tag) = find_tag(&content, "ImageData") {
            if let Some(extent_str) = extract_attr(&id_tag, "WholeExtent") {
                let vals: Vec<i64> = extent_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if vals.len() == 6 {
                    image.set_extent([vals[0], vals[1], vals[2], vals[3], vals[4], vals[5]]);
                }
            }
            if let Some(origin_str) = extract_attr(&id_tag, "Origin") {
                let vals: Vec<f64> = origin_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if vals.len() == 3 {
                    image.set_origin([vals[0], vals[1], vals[2]]);
                }
            }
            if let Some(spacing_str) = extract_attr(&id_tag, "Spacing") {
                let vals: Vec<f64> = spacing_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if vals.len() == 3 {
                    image.set_spacing([vals[0], vals[1], vals[2]]);
                }
            }
        }

        // Parse PointData
        if let Some(pd_section) = extract_section(&content, "PointData") {
            parse_attribute_arrays(
                &pd_section,
                image.point_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
            )?;
        }

        // Parse CellData
        if let Some(cd_section) = extract_section(&content, "CellData") {
            parse_attribute_arrays(
                &cd_section,
                image.cell_data_mut(),
                appended_raw.as_deref(),
                appended_b64.as_deref(),
            )?;
        }

        Ok(image)
    }
}

fn find_tag(content: &str, tag: &str) -> Option<String> {
    let pattern = format!("<{}", tag);
    let start = content.find(&pattern)?;
    let end = content[start..].find('>')?;
    Some(content[start..start + end + 1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray as DA, DataSet};
    use crate::io::xml::VtiWriter;

    #[test]
    fn roundtrip_vti() {
        let mut img = ImageData::with_dimensions(3, 4, 5);
        img.set_spacing([0.5, 0.5, 0.5]);
        img.set_origin([1.0, 2.0, 3.0]);

        let n = img.num_points();
        let scalars: Vec<f64> = (0..n).map(|i| i as f64 * 0.1).collect();
        let arr = DA::from_vec("density", scalars, 1);
        img.point_data_mut().add_array(arr.into());
        img.point_data_mut().set_active_scalars("density");

        let mut buf = Vec::new();
        VtiWriter::write_to(&mut buf, &img).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtiReader::read_from(reader).unwrap();

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
    fn roundtrip_vti_no_data() {
        let mut img = ImageData::with_dimensions(2, 2, 2);
        img.set_spacing([1.0, 2.0, 3.0]);

        let mut buf = Vec::new();
        VtiWriter::write_to(&mut buf, &img).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtiReader::read_from(reader).unwrap();

        assert_eq!(result.dimensions(), [2, 2, 2]);
        assert_eq!(result.spacing(), [1.0, 2.0, 3.0]);
    }
}
