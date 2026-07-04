use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};
use crate::types::VtkError;

/// Writer for VTK XML ImageData format (.vti).
pub struct VtiWriter;

impl VtiWriter {
    pub fn write(path: &Path, data: &ImageData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(w: &mut W, data: &ImageData) -> Result<(), VtkError> {
        let ext = data.extent();
        let spacing = data.spacing();
        let origin = data.origin();

        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"ImageData\" version=\"1.0\" byte_order=\"LittleEndian\" header_type=\"UInt32\">"
        )?;
        writeln!(
            w,
            "  <ImageData WholeExtent=\"{} {} {} {} {} {}\" Origin=\"{} {} {}\" Spacing=\"{} {} {}\" Direction=\"1 0 0 0 1 0 0 0 1\">",
            ext[0], ext[1], ext[2], ext[3], ext[4], ext[5],
            origin[0], origin[1], origin[2],
            spacing[0], spacing[1], spacing[2],
        )?;
        writeln!(
            w,
            "    <Piece Extent=\"{} {} {} {} {} {}\">",
            ext[0], ext[1], ext[2], ext[3], ext[4], ext[5],
        )?;

        write_data_section(w, "PointData", data.point_data())?;
        write_data_section(w, "CellData", data.cell_data())?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </ImageData>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
    }
}

fn write_data_section<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    let scalars_name = attrs.scalars().map(|a| a.name().to_string());
    let normals_name = attrs.normals().map(|a| a.name().to_string());
    let vectors_name = attrs.vectors().map(|a| a.name().to_string());
    let mut attrs_str = String::new();
    if let Some(ref name) = scalars_name {
        attrs_str.push_str(&format!(" Scalars=\"{}\"", xml_escape_attr(name)));
    }
    if let Some(ref name) = normals_name {
        attrs_str.push_str(&format!(" Normals=\"{}\"", xml_escape_attr(name)));
    }
    if let Some(ref name) = vectors_name {
        attrs_str.push_str(&format!(" Vectors=\"{}\"", xml_escape_attr(name)));
    }

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
    let type_name = match arr.scalar_type() {
        crate::types::ScalarType::F32 => "Float32",
        crate::types::ScalarType::F64 => "Float64",
        crate::types::ScalarType::I8 => "Int8",
        crate::types::ScalarType::I16 => "Int16",
        crate::types::ScalarType::I32 => "Int32",
        crate::types::ScalarType::I64 => "Int64",
        crate::types::ScalarType::U8 => "UInt8",
        crate::types::ScalarType::U16 => "UInt16",
        crate::types::ScalarType::U32 => "UInt32",
        crate::types::ScalarType::U64 => "UInt64",
    };

    writeln!(
        w,
        "        <DataArray type=\"{}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"ascii\">",
        type_name,
        xml_escape_attr(arr.name()),
        arr.num_components()
    )?;
    write!(w, "          ")?;
    write_array_values_ascii(w, arr)?;
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;
    Ok(())
}

fn write_array_values_ascii<W: Write>(w: &mut W, arr: &AnyDataArray) -> Result<(), VtkError> {
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
    use crate::data::{DataArray, ImageData};

    #[test]
    fn write_simple_vti() {
        let mut img = ImageData::with_dimensions(3, 4, 5);
        img.set_spacing([0.5, 0.5, 0.5]);
        img.set_origin([1.0, 2.0, 3.0]);

        let n = img.num_points();
        let scalars: Vec<f64> = (0..n).map(|i| i as f64 * 0.1).collect();
        let arr = DataArray::from_vec("density", scalars, 1);
        img.point_data_mut().add_array(arr.into());
        img.point_data_mut().set_active_scalars("density");

        let mut buf = Vec::new();
        VtiWriter::write_to(&mut buf, &img).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<VTKFile type=\"ImageData\""));
        assert!(output.contains("header_type=\"UInt32\""));
        assert!(output.contains("WholeExtent=\"0 2 0 3 0 4\""));
        assert!(output.contains("Origin=\"1 2 3\""));
        assert!(output.contains("Spacing=\"0.5 0.5 0.5\""));
        assert!(output.contains("Name=\"density\""));
    }

    #[test]
    fn writes_empty_data_sections_like_vtk_xml_writer() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let mut buf = Vec::new();
        VtiWriter::write_to(&mut buf, &img).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<PointData>"));
        assert!(output.contains("</PointData>"));
        assert!(output.contains("<CellData>"));
        assert!(output.contains("</CellData>"));
    }

    #[test]
    fn writes_integer_arrays_without_float_conversion() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        let ids = DataArray::from_vec("ids", vec![9_007_199_254_740_993u64; 3], 1);
        img.point_data_mut().add_array(ids.into());

        let mut buf = Vec::new();
        VtiWriter::write_to(&mut buf, &img).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("9007199254740993 "));
        assert!(!output.contains("9007199254740992 "));
    }
}
