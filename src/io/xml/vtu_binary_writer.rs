use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, UnstructuredGrid};
use crate::types::ScalarType;
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML UnstructuredGrid format (.vtu) with binary encoding.
pub struct VtuBinaryWriter;

impl VtuBinaryWriter {
    pub fn write(path: &Path, grid: &UnstructuredGrid) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, grid)
    }

    pub fn write_to<W: Write>(w: &mut W, grid: &UnstructuredGrid) -> Result<(), VtkError> {
        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"UnstructuredGrid\" version=\"1.0\" byte_order=\"LittleEndian\" header_type=\"UInt32\">"
        )?;
        writeln!(w, "  <UnstructuredGrid>")?;

        let n_points = grid.points.len();
        let n_cells = grid.cells().num_cells();

        writeln!(
            w,
            "    <Piece NumberOfPoints=\"{n_points}\" NumberOfCells=\"{n_cells}\">"
        )?;

        write_binary_attrs(w, "PointData", grid.point_data())?;
        write_binary_attrs(w, "CellData", grid.cell_data())?;

        writeln!(w, "      <Points>")?;
        let mut pts_data = Vec::with_capacity(n_points * 3);
        for i in 0..n_points {
            let p = grid.points.get(i);
            pts_data.extend_from_slice(&p);
        }
        let pts_arr = AnyDataArray::F64(DataArray::from_vec("Points", pts_data, 3));
        let pts_encoded = binary::encode_data_array_binary(&pts_arr);
        writeln!(w, "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"binary\">{pts_encoded}</DataArray>")?;
        writeln!(w, "      </Points>")?;

        // Cells
        writeln!(w, "      <Cells>")?;

        // Connectivity
        let mut conn = Vec::new();
        for i in 0..n_cells {
            for &id in grid.cell_points(i) {
                conn.push(id);
            }
        }
        let conn_arr = AnyDataArray::I64(DataArray::from_vec("connectivity", conn, 1));
        writeln!(w, "        <DataArray type=\"Int64\" Name=\"connectivity\" format=\"binary\">{}</DataArray>",
            binary::encode_data_array_binary(&conn_arr))?;

        // Offsets
        let mut offsets = Vec::new();
        let mut off = 0i64;
        for i in 0..n_cells {
            off += grid.cell_points(i).len() as i64;
            offsets.push(off);
        }
        let off_arr = AnyDataArray::I64(DataArray::from_vec("offsets", offsets, 1));
        writeln!(
            w,
            "        <DataArray type=\"Int64\" Name=\"offsets\" format=\"binary\">{}</DataArray>",
            binary::encode_data_array_binary(&off_arr)
        )?;

        // Types
        let types: Vec<u8> = (0..n_cells).map(|i| grid.cell_type(i) as u8).collect();
        let types_arr = AnyDataArray::U8(DataArray::from_vec("types", types, 1));
        writeln!(
            w,
            "        <DataArray type=\"UInt8\" Name=\"types\" format=\"binary\">{}</DataArray>",
            binary::encode_data_array_binary(&types_arr)
        )?;

        writeln!(w, "      </Cells>")?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </UnstructuredGrid>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
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
    use crate::data::{DataArray, Points, UnstructuredGrid};
    use crate::types::CellType;

    fn make_tet() -> UnstructuredGrid {
        let mut pts = Points::new();
        pts.push([0.0, 0.0, 0.0]);
        pts.push([1.0, 0.0, 0.0]);
        pts.push([0.0, 1.0, 0.0]);
        pts.push([0.0, 0.0, 1.0]);
        let mut ug = UnstructuredGrid::new();
        ug.points = pts;
        ug.push_cell(CellType::Tetra, &[0, 1, 2, 3]);
        ug
    }

    #[test]
    fn write_tet_binary() {
        let ug = make_tet();
        let mut buf = Vec::new();
        VtuBinaryWriter::write_to(&mut buf, &ug).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("format=\"binary\""));
        assert!(xml.contains("header_type=\"UInt32\""));
        assert!(xml.contains("UnstructuredGrid"));
    }

    #[test]
    fn roundtrip_tet_binary() {
        let ug = make_tet();
        let mut buf = Vec::new();
        VtuBinaryWriter::write_to(&mut buf, &ug).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = crate::io::xml::VtuReader::read_from(reader).unwrap();
        assert_eq!(result.points.len(), 4);
        assert_eq!(result.cells().num_cells(), 1);
    }

    #[test]
    fn write_binary_preserves_attribute_hints_escaping_and_integer_types() {
        let mut ug = make_tet();
        let ids = DataArray::from_vec("id&tag", vec![1u16, 2, 3, 4], 1);
        ug.point_data_mut().add_array(ids.into());
        ug.point_data_mut().set_active_scalars("id&tag");

        let mut buf = Vec::new();
        VtuBinaryWriter::write_to(&mut buf, &ug).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        assert!(xml.contains("<PointData Scalars=\"id&amp;tag\">"));
        assert!(xml.contains("type=\"UInt16\" Name=\"id&amp;tag\""));
    }
}
