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
            "<VTKFile type=\"ImageData\" version=\"1.0\" byte_order=\"LittleEndian\">"
        )?;
        writeln!(w, "  <ImageData WholeExtent=\"{} {} {} {} {} {}\" Origin=\"{} {} {}\" Spacing=\"{} {} {}\">",
            ext[0], ext[1], ext[2], ext[3], ext[4], ext[5],
            origin[0], origin[1], origin[2],
            spacing[0], spacing[1], spacing[2],
        )?;
        writeln!(
            w,
            "    <Piece Extent=\"{} {} {} {} {} {}\">",
            ext[0], ext[1], ext[2], ext[3], ext[4], ext[5],
        )?;

        if data.point_data().num_arrays() > 0 {
            write_binary_attrs(w, "PointData", data.point_data())?;
        }
        if data.cell_data().num_arrays() > 0 {
            write_binary_attrs(w, "CellData", data.cell_data())?;
        }

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
    writeln!(w, "      <{section}>")?;
    for i in 0..attrs.num_arrays() {
        if let Some(arr) = attrs.get_array_by_index(i) {
            let type_name = match arr.scalar_type() {
                crate::types::ScalarType::F32 => "Float32",
                crate::types::ScalarType::F64 => "Float64",
                crate::types::ScalarType::I32 => "Int32",
                crate::types::ScalarType::I64 => "Int64",
                crate::types::ScalarType::U8 => "UInt8",
                _ => "Float64",
            };
            let encoded = binary::encode_data_array_binary(arr);
            writeln!(w, "        <DataArray type=\"{type_name}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"binary\">{encoded}</DataArray>",
                arr.name(), arr.num_components())?;
        }
    }
    writeln!(w, "      </{section}>")?;
    Ok(())
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
