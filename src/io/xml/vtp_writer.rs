use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};
use crate::types::VtkError;

/// Writer for VTK XML PolyData format (.vtp).
///
/// Produces ASCII XML files compatible with ParaView and other VTK-based tools.
pub struct VtpWriter;

impl VtpWriter {
    pub fn write(path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"PolyData\" version=\"1.0\" byte_order=\"LittleEndian\" header_type=\"UInt32\">"
        )?;
        writeln!(w, "  <PolyData>")?;

        let n_points = data.points.len();
        let n_verts = data.verts.num_cells();
        let n_lines = data.lines.num_cells();
        let n_polys = data.polys.num_cells();
        let n_strips = data.strips.num_cells();

        writeln!(
            w,
            "    <Piece NumberOfPoints=\"{}\" NumberOfVerts=\"{}\" NumberOfLines=\"{}\" NumberOfStrips=\"{}\" NumberOfPolys=\"{}\">",
            n_points, n_verts, n_lines, n_strips, n_polys
        )?;

        write_data_arrays(w, "PointData", data.point_data())?;
        write_data_arrays(w, "CellData", data.cell_data())?;

        // Points
        writeln!(w, "      <Points>")?;
        writeln!(
            w,
            "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"ascii\">"
        )?;
        write!(w, "          ")?;
        for i in 0..n_points {
            let p = data.points.get(i);
            write!(w, "{} {} {} ", p[0], p[1], p[2])?;
        }
        writeln!(w)?;
        writeln!(w, "        </DataArray>")?;
        writeln!(w, "      </Points>")?;

        // Cells
        write_cell_section(w, "Verts", &data.verts)?;
        write_cell_section(w, "Lines", &data.lines)?;
        write_cell_section(w, "Strips", &data.strips)?;
        write_cell_section(w, "Polys", &data.polys)?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </PolyData>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
    }
}

fn write_cell_section<W: Write>(w: &mut W, tag: &str, cells: &CellArray) -> Result<(), VtkError> {
    writeln!(w, "      <{}>", tag)?;

    // Connectivity
    writeln!(
        w,
        "        <DataArray type=\"Int64\" Name=\"connectivity\" format=\"ascii\">"
    )?;
    write!(w, "          ")?;
    for cell in cells.iter() {
        for &id in cell {
            write!(w, "{} ", id)?;
        }
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;

    // Offsets
    writeln!(
        w,
        "        <DataArray type=\"Int64\" Name=\"offsets\" format=\"ascii\">"
    )?;
    write!(w, "          ")?;
    let mut offset = 0i64;
    for cell in cells.iter() {
        offset += cell.len() as i64;
        write!(w, "{} ", offset)?;
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;

    writeln!(w, "      </{}>", tag)?;
    Ok(())
}

fn write_data_arrays<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    // Build attribute hints
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
    use crate::data::{DataArray, PolyData};

    #[test]
    fn write_triangle_vtp() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<VTKFile type=\"PolyData\""));
        assert!(output.contains("NumberOfPoints=\"3\""));
        assert!(output.contains("NumberOfPolys=\"1\""));
        assert!(output.contains("<Polys>"));
        assert!(output.contains("connectivity"));
        assert!(output.contains("offsets"));
    }

    #[test]
    fn write_with_scalars_vtp() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("temperature", vec![1.0f64, 2.0, 3.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("temperature");

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<PointData Scalars=\"temperature\">"));
        assert!(output.contains("Name=\"temperature\""));
        assert!(output.find("<PointData").unwrap() < output.find("<Points>").unwrap());
    }

    #[test]
    fn escapes_attribute_names() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("a&b", vec![1.0f64, 2.0, 3.0], 1);
        pd.point_data_mut().add_array(scalars.into());
        pd.point_data_mut().set_active_scalars("a&b");

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("Scalars=\"a&amp;b\""));
        assert!(output.contains("Name=\"a&amp;b\""));
    }

    #[test]
    fn writes_integer_arrays_without_float_conversion() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let ids = DataArray::from_vec("ids", vec![9_007_199_254_740_993u64; 3], 1);
        pd.point_data_mut().add_array(ids.into());

        let mut buf = Vec::new();
        VtpWriter::write_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("9007199254740993 "));
        assert!(!output.contains("9007199254740992 "));
    }
}
