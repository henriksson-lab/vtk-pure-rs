use std::io::BufRead;
use std::path::Path;

use crate::data::RectilinearGrid;
use crate::types::VtkError;

use crate::io::xml::binary;
use crate::io::xml::vtp_reader::{
    detect_format, extract_appended_base64, extract_appended_raw, extract_attr, extract_section,
    parse_attribute_arrays, parse_from_appended, DataFormat,
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
        let content: String = reader
            .lines()
            .collect::<Result<Vec<_>, _>>()
            .map_err(VtkError::Io)?
            .join("\n");

        // Extract appended data section if present
        let appended_raw = extract_appended_raw(&content);
        let appended_b64 = extract_appended_base64(&content);

        let mut x_coords = vec![0.0];
        let mut y_coords = vec![0.0];
        let mut z_coords = vec![0.0];

        // Parse Coordinates section
        if let Some(coords_section) = extract_section(&content, "Coordinates") {
            let mut search_pos = 0;
            while let Some(da_start) = coords_section[search_pos..].find("<DataArray") {
                let abs_start = search_pos + da_start;
                let tag_end = coords_section[abs_start..]
                    .find('>')
                    .ok_or_else(|| VtkError::Parse("unclosed tag".into()))?;
                let tag = &coords_section[abs_start..abs_start + tag_end + 1];
                let content_start = abs_start + tag_end + 1;
                let content_end = coords_section[content_start..]
                    .find("</DataArray>")
                    .ok_or_else(|| VtkError::Parse("missing close".into()))?;
                let da_content = coords_section[content_start..content_start + content_end].trim();

                let name = extract_attr(tag, "Name").unwrap_or_default();
                let type_str = extract_attr(tag, "type").unwrap_or_else(|| "Float64".to_string());

                let values: Vec<f64> = match detect_format(tag) {
                    DataFormat::Ascii => da_content
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect(),
                    DataFormat::Binary => {
                        let arr = binary::parse_binary_data_array(da_content, &name, &type_str, 1)?;
                        any_data_array_to_f64(&arr)
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
                        any_data_array_to_f64(&arr)
                    }
                };

                match name.as_str() {
                    "x" => x_coords = values,
                    "y" => y_coords = values,
                    "z" => z_coords = values,
                    _ => {}
                }

                search_pos = content_start + content_end + "</DataArray>".len();
            }
        }

        if x_coords.is_empty() {
            x_coords = vec![0.0];
        }
        if y_coords.is_empty() {
            y_coords = vec![0.0];
        }
        if z_coords.is_empty() {
            z_coords = vec![0.0];
        }

        let mut grid = RectilinearGrid::from_coords(x_coords, y_coords, z_coords);

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
}
