use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, RectilinearGrid};
use crate::types::ScalarType;
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML RectilinearGrid format (.vtr) with binary encoding.
pub struct VtrBinaryWriter;

impl VtrBinaryWriter {
    pub fn write(path: &Path, grid: &RectilinearGrid) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, grid)
    }

    pub fn write_to<W: Write>(w: &mut W, grid: &RectilinearGrid) -> Result<(), VtkError> {
        let dims = grid.dimensions();
        let ext = format!("0 {} 0 {} 0 {}", dims[0] - 1, dims[1] - 1, dims[2] - 1);

        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"RectilinearGrid\" version=\"1.0\" byte_order=\"LittleEndian\" header_type=\"UInt32\">"
        )?;
        writeln!(w, "  <RectilinearGrid WholeExtent=\"{ext}\">")?;
        writeln!(w, "    <Piece Extent=\"{ext}\">")?;

        write_binary_attrs(w, "PointData", grid.point_data())?;
        write_binary_attrs(w, "CellData", grid.cell_data())?;

        writeln!(w, "      <Coordinates>")?;
        write_coord_binary(w, "x", grid.x_coords())?;
        write_coord_binary(w, "y", grid.y_coords())?;
        write_coord_binary(w, "z", grid.z_coords())?;
        writeln!(w, "      </Coordinates>")?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </RectilinearGrid>")?;
        writeln!(w, "</VTKFile>")?;
        Ok(())
    }
}

fn write_coord_binary<W: Write>(w: &mut W, name: &str, coords: &[f64]) -> Result<(), VtkError> {
    let arr = AnyDataArray::F64(DataArray::from_vec(name, coords.to_vec(), 1));
    let encoded = binary::encode_data_array_binary(&arr);
    writeln!(w, "        <DataArray type=\"Float64\" Name=\"{name}\" format=\"binary\">{encoded}</DataArray>")?;
    Ok(())
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
    use crate::data::{DataArray, RectilinearGrid};

    #[test]
    fn roundtrip_vtr_binary() {
        let grid = RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![0.0]);
        let mut buf = Vec::new();
        VtrBinaryWriter::write_to(&mut buf, &grid).unwrap();

        let xml = String::from_utf8(buf.clone()).unwrap();
        assert!(xml.contains("format=\"binary\""));

        let reader = std::io::BufReader::new(&buf[..]);
        let result = crate::io::xml::VtrReader::read_from(reader).unwrap();
        assert_eq!(result.dimensions(), grid.dimensions());
    }

    #[test]
    fn binary_vtr_writes_attribute_hints_and_integer_types() {
        let mut grid = RectilinearGrid::from_coords(vec![0.0, 1.0], vec![0.0, 1.0], vec![0.0, 1.0]);
        let ids = DataArray::from_vec("id&tag", vec![1u16, 2, 3, 4, 5, 6, 7, 8], 1);
        grid.point_data_mut().add_array(ids.into());
        grid.point_data_mut().set_active_scalars("id&tag");

        let mut buf = Vec::new();
        VtrBinaryWriter::write_to(&mut buf, &grid).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains("<PointData Scalars=\"id&amp;tag\">"));
        assert!(xml.contains("type=\"UInt16\" Name=\"id&amp;tag\""));
    }
}
