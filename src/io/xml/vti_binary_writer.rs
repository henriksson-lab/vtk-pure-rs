use std::io::Write;
use std::path::Path;

use crate::data::{DataSetAttributes, ImageData};
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML ImageData format (.vti) with binary encoding.
pub struct VtiBinaryWriter;

impl VtiBinaryWriter {
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
        writeln!(w, "  <ImageData WholeExtent=\"{} {} {} {} {} {}\" Origin=\"{} {} {}\" Spacing=\"{} {} {}\" Direction=\"1 0 0 0 1 0 0 0 1\">",
            ext[0], ext[1], ext[2], ext[3], ext[4], ext[5],
            origin[0], origin[1], origin[2],
            spacing[0], spacing[1], spacing[2],
        )?;
        writeln!(
            w,
            "    <Piece Extent=\"{} {} {} {} {} {}\">",
            ext[0], ext[1], ext[2], ext[3], ext[4], ext[5],
        )?;

        write_binary_attrs(w, "PointData", data.point_data())?;
        write_binary_attrs(w, "CellData", data.cell_data())?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </ImageData>")?;
        writeln!(w, "</VTKFile>")?;
        Ok(())
    }
}

fn write_binary_attrs<W: Write>(
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

    writeln!(w, "      <{section}{attrs_str}>")?;
    for i in 0..attrs.num_arrays() {
        if let Some(arr) = attrs.get_array_by_index(i) {
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
            let encoded = binary::encode_data_array_binary(arr);
            writeln!(w, "        <DataArray type=\"{type_name}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"binary\">{encoded}</DataArray>",
                xml_escape_attr(arr.name()), arr.num_components())?;
        }
    }
    writeln!(w, "      </{section}>")?;
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
    use crate::data::{AnyDataArray, DataArray, ImageData};

    #[test]
    fn write_vti_binary() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let scalars = DataArray::from_vec("density", vec![1.0f64; 27], 1);
        img.point_data_mut().add_array(AnyDataArray::F64(scalars));

        let mut buf = Vec::new();
        VtiBinaryWriter::write_to(&mut buf, &img).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("format=\"binary\""));
        assert!(xml.contains("ImageData"));
        assert!(xml.contains("<CellData>"));
    }

    #[test]
    fn writes_empty_data_sections_like_vtk_xml_writer() {
        let img = ImageData::with_dimensions(2, 2, 2);
        let mut buf = Vec::new();
        VtiBinaryWriter::write_to(&mut buf, &img).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains("<PointData>"));
        assert!(xml.contains("</PointData>"));
        assert!(xml.contains("<CellData>"));
        assert!(xml.contains("</CellData>"));
    }

    #[test]
    fn roundtrip_vti_binary() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        let scalars = DataArray::from_vec("temp", (0..27).map(|i| i as f64).collect(), 1);
        img.point_data_mut().add_array(AnyDataArray::F64(scalars));
        img.point_data_mut().set_active_scalars("temp");

        let mut buf = Vec::new();
        VtiBinaryWriter::write_to(&mut buf, &img).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = crate::io::xml::VtiReader::read_from(reader).unwrap();
        let arr = result.point_data().get_array("temp").unwrap();
        assert_eq!(arr.num_tuples(), 27);
        let mut v = [0.0f64];
        arr.tuple_as_f64(13, &mut v);
        assert!((v[0] - 13.0).abs() < 1e-6);
    }
}
