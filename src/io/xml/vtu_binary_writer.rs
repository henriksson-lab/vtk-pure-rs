use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, UnstructuredGrid};
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
            "<VTKFile type=\"UnstructuredGrid\" version=\"1.0\" byte_order=\"LittleEndian\">"
        )?;
        writeln!(w, "  <UnstructuredGrid>")?;

        let n_points = grid.points.len();
        let n_cells = grid.cells().num_cells();

        writeln!(
            w,
            "    <Piece NumberOfPoints=\"{n_points}\" NumberOfCells=\"{n_cells}\">"
        )?;

        // Points
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

        // PointData
        if grid.point_data().num_arrays() > 0 {
            write_binary_attrs(w, "PointData", grid.point_data())?;
        }

        // CellData
        if grid.cell_data().num_arrays() > 0 {
            write_binary_attrs(w, "CellData", grid.cell_data())?;
        }

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
    use crate::data::{CellArray, Points, UnstructuredGrid};
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
}
