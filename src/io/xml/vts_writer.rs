use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, StructuredGrid};
use crate::types::ScalarType;
use crate::types::VtkError;

/// Writer for VTK XML StructuredGrid format (.vts).
pub struct VtsWriter;

impl VtsWriter {
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
        writeln!(w, "  <StructuredGrid WholeExtent=\"{}\">", ext)?;
        writeln!(w, "    <Piece Extent=\"{}\">", ext)?;

        write_data_section(w, "PointData", grid.point_data())?;
        write_data_section(w, "CellData", grid.cell_data())?;

        // Points
        writeln!(w, "      <Points>")?;
        writeln!(
            w,
            "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"ascii\">"
        )?;
        write!(w, "          ")?;
        for i in 0..grid.points.len() {
            let p = grid.points.get(i);
            write!(w, "{} {} {} ", p[0], p[1], p[2])?;
        }
        writeln!(w)?;
        writeln!(w, "        </DataArray>")?;
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
    match arr {
        AnyDataArray::F32(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::F64(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::I8(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::I16(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::I32(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::I64(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::U8(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::U16(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::U32(a) => write_typed_array_values_ascii(w, a),
        AnyDataArray::U64(a) => write_typed_array_values_ascii(w, a),
    }
}

fn write_typed_array_values_ascii<W: Write, T: crate::types::Scalar>(
    w: &mut W,
    arr: &DataArray<T>,
) -> Result<(), VtkError> {
    for value in arr.as_slice() {
        write!(w, "{} ", value)?;
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
    use crate::data::{DataArray, Points, StructuredGrid};

    #[test]
    fn write_simple_vts() {
        let mut pts = Points::new();
        for j in 0..2 {
            for i in 0..3 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        let grid = StructuredGrid::from_dimensions_and_points([3, 2, 1], pts);
        let mut buf = Vec::new();
        VtsWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("<VTKFile type=\"StructuredGrid\""));
        assert!(output.contains("header_type=\"UInt32\""));
        assert!(output.contains("WholeExtent=\"0 2 0 1 0 0\""));
        assert!(output.contains("<PointData>"));
        assert!(output.contains("<CellData>"));
    }

    #[test]
    fn writes_attribute_hints_and_integer_types() {
        let mut pts = Points::new();
        pts.push([0.0, 0.0, 0.0]);
        let mut grid = StructuredGrid::from_dimensions_and_points([1, 1, 1], pts);
        let ids = DataArray::from_vec("id&tag", vec![9_007_199_254_740_993u64], 1);
        grid.point_data_mut().add_array(ids.into());
        grid.point_data_mut().set_active_scalars("id&tag");

        let mut buf = Vec::new();
        VtsWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<PointData Scalars=\"id&amp;tag\">"));
        assert!(output.contains("type=\"UInt64\" Name=\"id&amp;tag\""));
        assert!(output.contains("9007199254740993 "));
    }

    #[test]
    fn writes_empty_extent_without_underflow() {
        let grid = StructuredGrid::new();
        let mut buf = Vec::new();
        VtsWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("WholeExtent=\"0 -1 0 -1 0 -1\""));
        assert!(output.contains("Piece Extent=\"0 -1 0 -1 0 -1\""));
    }
}
