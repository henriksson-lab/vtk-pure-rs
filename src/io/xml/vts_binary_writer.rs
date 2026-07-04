use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, StructuredGrid};
use crate::types::ScalarType;
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML StructuredGrid format (.vts) with binary encoding.
pub struct VtsBinaryWriter;

impl VtsBinaryWriter {
    pub fn write(path: &Path, grid: &StructuredGrid) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, grid)
    }

    pub fn write_to<W: Write>(w: &mut W, grid: &StructuredGrid) -> Result<(), VtkError> {
        let dims = grid.dimensions();
        let ext = extent_string(dims);

        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"StructuredGrid\" version=\"1.0\" byte_order=\"LittleEndian\" header_type=\"UInt32\">"
        )?;
        writeln!(w, "  <StructuredGrid WholeExtent=\"{ext}\">")?;
        writeln!(w, "    <Piece Extent=\"{ext}\">")?;

        write_binary_attrs(w, "PointData", grid.point_data())?;
        write_binary_attrs(w, "CellData", grid.cell_data())?;

        // Points
        let n = grid.points.len();
        let mut pts_data = Vec::with_capacity(n * 3);
        for i in 0..n {
            let p = grid.points.get(i);
            pts_data.extend_from_slice(&p);
        }
        let pts_arr = AnyDataArray::F64(DataArray::from_vec("Points", pts_data, 3));
        let pts_encoded = binary::encode_data_array_binary(&pts_arr);
        writeln!(w, "      <Points>")?;
        writeln!(w, "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"binary\">{pts_encoded}</DataArray>")?;
        writeln!(w, "      </Points>")?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </StructuredGrid>")?;
        writeln!(w, "</VTKFile>")?;
        Ok(())
    }
}

fn extent_string(dims: [usize; 3]) -> String {
    if dims.contains(&0) {
        "0 -1 0 -1 0 -1".to_string()
    } else {
        format!("0 {} 0 {} 0 {}", dims[0] - 1, dims[1] - 1, dims[2] - 1)
    }
}

fn write_binary_attrs<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    let attrs_str = data_attribute_string(attrs);
    writeln!(w, "      <{section}{attrs_str}>")?;
    for i in 0..attrs.num_arrays() {
        if let Some(arr) = attrs.get_array_by_index(i) {
            let encoded = binary::encode_data_array_binary(arr);
            writeln!(w, "        <DataArray type=\"{type_name}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"binary\">{encoded}</DataArray>",
                xml_escape_attr(arr.name()), arr.num_components(), type_name = xml_scalar_type(arr.scalar_type()))?;
        }
    }
    writeln!(w, "      </{section}>")?;
    Ok(())
}

fn data_attribute_string(attrs: &DataSetAttributes) -> String {
    let mut attrs_str = String::new();
    if let Some(arr) = attrs.scalars() {
        attrs_str.push_str(&format!(" Scalars=\"{}\"", xml_escape_attr(arr.name())));
    }
    if let Some(arr) = attrs.normals() {
        attrs_str.push_str(&format!(" Normals=\"{}\"", xml_escape_attr(arr.name())));
    }
    if let Some(arr) = attrs.vectors() {
        attrs_str.push_str(&format!(" Vectors=\"{}\"", xml_escape_attr(arr.name())));
    }
    attrs_str
}

fn xml_scalar_type(scalar_type: ScalarType) -> &'static str {
    match scalar_type {
        ScalarType::F32 => "Float32",
        ScalarType::F64 => "Float64",
        ScalarType::I8 => "Int8",
        ScalarType::I16 => "Int16",
        ScalarType::I32 => "Int32",
        ScalarType::I64 => "Int64",
        ScalarType::U8 => "UInt8",
        ScalarType::U16 => "UInt16",
        ScalarType::U32 => "UInt32",
        ScalarType::U64 => "UInt64",
    }
}

fn xml_escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray, Points, StructuredGrid};

    #[test]
    fn roundtrip_vts_binary() {
        let mut pts = Points::new();
        for j in 0..2 {
            for i in 0..3 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        let grid = StructuredGrid::from_dimensions_and_points([3, 2, 1], pts);

        let mut buf = Vec::new();
        VtsBinaryWriter::write_to(&mut buf, &grid).unwrap();

        let xml = String::from_utf8(buf.clone()).unwrap();
        assert!(xml.contains("format=\"binary\""));
        assert!(xml.contains("<PointData>"));
        assert!(xml.contains("<CellData>"));

        let reader = std::io::BufReader::new(&buf[..]);
        let result = crate::io::xml::VtsReader::read_from(reader).unwrap();
        assert_eq!(result.dimensions(), grid.dimensions());
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn binary_vts_writes_attribute_hints_and_integer_types() {
        let mut pts = Points::new();
        pts.push([0.0, 0.0, 0.0]);
        let mut grid = StructuredGrid::from_dimensions_and_points([1, 1, 1], pts);
        let ids = DataArray::from_vec("id&tag", vec![7u16], 1);
        grid.point_data_mut().add_array(ids.into());
        grid.point_data_mut().set_active_scalars("id&tag");

        let mut buf = Vec::new();
        VtsBinaryWriter::write_to(&mut buf, &grid).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains("header_type=\"UInt32\""));
        assert!(xml.contains("<PointData Scalars=\"id&amp;tag\">"));
        assert!(xml.contains("type=\"UInt16\" Name=\"id&amp;tag\""));
    }

    #[test]
    fn binary_writes_empty_extent_without_underflow() {
        let grid = StructuredGrid::new();
        let mut buf = Vec::new();
        VtsBinaryWriter::write_to(&mut buf, &grid).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("WholeExtent=\"0 -1 0 -1 0 -1\""));
        assert!(xml.contains("Piece Extent=\"0 -1 0 -1 0 -1\""));
    }
}
