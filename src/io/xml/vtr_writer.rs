use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataSetAttributes, RectilinearGrid};
use crate::types::ScalarType;
use crate::types::VtkError;

/// Writer for VTK XML RectilinearGrid format (.vtr).
pub struct VtrWriter;

impl VtrWriter {
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
        writeln!(w, "  <RectilinearGrid WholeExtent=\"{}\">", ext)?;
        writeln!(w, "    <Piece Extent=\"{}\">", ext)?;

        // Point data
        write_data_section(w, "PointData", grid.point_data())?;

        // Cell data
        write_data_section(w, "CellData", grid.cell_data())?;

        // Coordinates
        writeln!(w, "      <Coordinates>")?;
        write_coord_array(w, "x", grid.x_coords())?;
        write_coord_array(w, "y", grid.y_coords())?;
        write_coord_array(w, "z", grid.z_coords())?;
        writeln!(w, "      </Coordinates>")?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </RectilinearGrid>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
    }
}

fn write_coord_array<W: Write>(w: &mut W, name: &str, coords: &[f64]) -> Result<(), VtkError> {
    writeln!(
        w,
        "        <DataArray type=\"Float64\" Name=\"{}\" format=\"ascii\">",
        name
    )?;
    write!(w, "          ")?;
    for &v in coords {
        write!(w, "{} ", v)?;
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;
    Ok(())
}

fn write_data_section<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    let attrs_str = data_attribute_string(attrs);
    writeln!(w, "      <{}{}>", section, attrs_str)?;
    for i in 0..attrs.num_arrays() {
        if let Some(arr) = attrs.get_array_by_index(i) {
            write_any_data_array(w, arr)?;
        }
    }
    writeln!(w, "      </{}>", section)?;
    Ok(())
}

fn write_any_data_array<W: Write>(w: &mut W, arr: &AnyDataArray) -> Result<(), VtkError> {
    writeln!(
        w,
        "        <DataArray type=\"{}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"ascii\">",
        xml_scalar_type(arr.scalar_type()),
        xml_escape_attr(arr.name()),
        arr.num_components()
    )?;
    write!(w, "          ")?;
    write_ascii_values(w, arr)?;
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;
    Ok(())
}

fn write_ascii_values<W: Write>(w: &mut W, arr: &AnyDataArray) -> Result<(), VtkError> {
    macro_rules! write_array {
        ($array:expr) => {{
            for i in 0..$array.num_tuples() {
                for v in $array.tuple(i) {
                    write!(w, "{} ", v)?;
                }
            }
        }};
    }

    match arr {
        AnyDataArray::F32(a) => write_array!(a),
        AnyDataArray::F64(a) => write_array!(a),
        AnyDataArray::I8(a) => write_array!(a),
        AnyDataArray::I16(a) => write_array!(a),
        AnyDataArray::I32(a) => write_array!(a),
        AnyDataArray::I64(a) => write_array!(a),
        AnyDataArray::U8(a) => write_array!(a),
        AnyDataArray::U16(a) => write_array!(a),
        AnyDataArray::U32(a) => write_array!(a),
        AnyDataArray::U64(a) => write_array!(a),
    }
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
    fn write_simple_vtr() {
        let grid = RectilinearGrid::from_coords(vec![0.0, 1.0, 3.0], vec![0.0, 2.0], vec![0.0]);
        let mut buf = Vec::new();
        VtrWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("<VTKFile type=\"RectilinearGrid\""));
        assert!(output.contains("header_type=\"UInt32\""));
        assert!(output.contains("WholeExtent=\"0 2 0 1 0 0\""));
        assert!(output.contains("<PointData>"));
        assert!(output.contains("<CellData>"));
        assert!(output.contains("Name=\"x\""));
    }

    #[test]
    fn writes_attribute_hints_and_integer_types() {
        let mut grid = RectilinearGrid::from_coords(vec![0.0, 1.0], vec![0.0], vec![0.0]);
        let ids = DataArray::from_vec("id&tag", vec![7u16, 8], 1);
        grid.point_data_mut().add_array(ids.into());
        grid.point_data_mut().set_active_scalars("id&tag");

        let mut buf = Vec::new();
        VtrWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<PointData Scalars=\"id&amp;tag\">"));
        assert!(output.contains("type=\"UInt16\" Name=\"id&amp;tag\""));
        assert!(output.contains(">"));
        assert!(output.contains("7 8"));
    }
}
