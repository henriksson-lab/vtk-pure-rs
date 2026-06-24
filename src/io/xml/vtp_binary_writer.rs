use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, PolyData};
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML PolyData format (.vtp) with binary (base64) encoding.
///
/// Produces compact binary-encoded VTP files compatible with ParaView.
pub struct VtpBinaryWriter;

impl VtpBinaryWriter {
    pub fn write(path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"PolyData\" version=\"1.0\" byte_order=\"LittleEndian\">"
        )?;
        writeln!(w, "  <PolyData>")?;

        let n_points = data.points.len();
        let n_polys = data.polys.num_cells();

        writeln!(
            w,
            "    <Piece NumberOfPoints=\"{}\" NumberOfVerts=\"0\" NumberOfLines=\"0\" NumberOfPolys=\"{}\" NumberOfStrips=\"0\">",
            n_points, n_polys
        )?;

        // Points (Float64, 3 components, binary)
        writeln!(w, "      <Points>")?;
        let points_arr = points_to_data_array(data);
        let points_encoded = binary::encode_data_array_binary(&points_arr);
        writeln!(
            w,
            "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"binary\">{}</DataArray>",
            points_encoded
        )?;
        writeln!(w, "      </Points>")?;

        // Polys
        if n_polys > 0 {
            writeln!(w, "      <Polys>")?;
            write_binary_cell_section(w, &data.polys)?;
            writeln!(w, "      </Polys>")?;
        }

        // PointData
        if data.point_data().num_arrays() > 0 {
            write_binary_data_section(w, "PointData", data.point_data())?;
        }

        // CellData
        if data.cell_data().num_arrays() > 0 {
            write_binary_data_section(w, "CellData", data.cell_data())?;
        }

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </PolyData>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
    }
}

fn points_to_data_array(data: &PolyData) -> AnyDataArray {
    let n = data.points.len();
    let mut values = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = data.points.get(i);
        values.extend_from_slice(&p);
    }
    AnyDataArray::F64(DataArray::from_vec("Points", values, 3))
}

fn write_binary_cell_section<W: Write>(w: &mut W, cells: &CellArray) -> Result<(), VtkError> {
    // Connectivity
    let mut conn = Vec::new();
    for cell in cells.iter() {
        for &id in cell {
            conn.push(id);
        }
    }
    let conn_arr = AnyDataArray::I64(DataArray::from_vec("connectivity", conn, 1));
    let conn_encoded = binary::encode_data_array_binary(&conn_arr);
    writeln!(
        w,
        "        <DataArray type=\"Int64\" Name=\"connectivity\" format=\"binary\">{}</DataArray>",
        conn_encoded
    )?;

    // Offsets
    let mut offsets = Vec::new();
    let mut offset = 0i64;
    for cell in cells.iter() {
        offset += cell.len() as i64;
        offsets.push(offset);
    }
    let off_arr = AnyDataArray::I64(DataArray::from_vec("offsets", offsets, 1));
    let off_encoded = binary::encode_data_array_binary(&off_arr);
    writeln!(
        w,
        "        <DataArray type=\"Int64\" Name=\"offsets\" format=\"binary\">{}</DataArray>",
        off_encoded
    )?;

    Ok(())
}

fn write_binary_data_section<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    writeln!(w, "      <{}>", section)?;
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
            writeln!(
                w,
                "        <DataArray type=\"{}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"binary\">{}</DataArray>",
                type_name, arr.name(), arr.num_components(), encoded
            )?;
        }
    }
    writeln!(w, "      </{}>", section)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::xml::VtpReader;

    #[test]
    fn binary_vtp_roundtrip() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let mut buf = Vec::new();
        VtpBinaryWriter::write_to(&mut buf, &pd).unwrap();

        let xml = String::from_utf8(buf.clone()).unwrap();
        assert!(xml.contains("format=\"binary\""));

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtpReader::read_from(reader).unwrap();
        assert_eq!(result.points.len(), 3);
        assert_eq!(result.polys.num_cells(), 1);

        let p1 = result.points.get(1);
        assert!((p1[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn binary_vtp_with_scalars() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let s = DataArray::from_vec("temp", vec![10.0f64, 20.0, 30.0], 1);
        pd.point_data_mut().add_array(s.into());

        let mut buf = Vec::new();
        VtpBinaryWriter::write_to(&mut buf, &pd).unwrap();

        let reader = std::io::BufReader::new(&buf[..]);
        let result = VtpReader::read_from(reader).unwrap();
        let arr = result.point_data().get_array("temp").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        let mut v = [0.0f64];
        arr.tuple_as_f64(1, &mut v);
        assert!((v[0] - 20.0).abs() < 1e-6);
    }
}
