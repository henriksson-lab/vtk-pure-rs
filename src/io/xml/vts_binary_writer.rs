use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, StructuredGrid};
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML StructuredGrid format (.vts) with binary encoding.
pub struct VtsBinaryWriter;

impl VtsBinaryWriter {
    pub fn write(path: &Path, grid: &StructuredGrid) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, grid)
    }

    pub fn write_to<W: Write>(w: &mut W, grid: &StructuredGrid) -> Result<(), VtkError> {
        let dims = grid.dimensions();
        let ext = format!("0 {} 0 {} 0 {}", dims[0] - 1, dims[1] - 1, dims[2] - 1);

        writeln!(w, "<?xml version=\"1.0\"?>")?;
        writeln!(
            w,
            "<VTKFile type=\"StructuredGrid\" version=\"1.0\" byte_order=\"LittleEndian\">"
        )?;
        writeln!(w, "  <StructuredGrid WholeExtent=\"{ext}\">")?;
        writeln!(w, "    <Piece Extent=\"{ext}\">")?;

        // Points
        let n = grid.points.len();
        let mut pts_data = Vec::with_capacity(n * 3);
        for i in 0..n {
            let p = grid.points.get(i);
            pts_data.extend_from_slice(&p);
        }
        let pts_arr = AnyDataArray::F64(DataArray::from_vec("Points", pts_data, 3));
        let pts_encoded = binary::encode_data_array_binary(&pts_arr);
        writeln!(w, "      <Points>")?;
        writeln!(w, "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"binary\">{pts_encoded}</DataArray>")?;
        writeln!(w, "      </Points>")?;

        if grid.point_data().num_arrays() > 0 {
            write_binary_attrs(w, "PointData", grid.point_data())?;
        }
        if grid.cell_data().num_arrays() > 0 {
            write_binary_attrs(w, "CellData", grid.cell_data())?;
        }

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </StructuredGrid>")?;
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
    use crate::data::{Points, StructuredGrid};

    #[test]
    fn roundtrip_vts_binary() {
        let mut pts = Points::new();
        for j in 0..2 {
            for i in 0..3 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        let grid = StructuredGrid::from_dimensions_and_points([3, 2, 1], pts);

        let mut buf = Vec::new();
        VtsBinaryWriter::write_to(&mut buf, &grid).unwrap();

        let xml = String::from_utf8(buf.clone()).unwrap();
        assert!(xml.contains("format=\"binary\""));

        let reader = std::io::BufReader::new(&buf[..]);
        let result = crate::io::xml::VtsReader::read_from(reader).unwrap();
        assert_eq!(result.dimensions(), grid.dimensions());
        assert_eq!(result.points.len(), 6);
    }
}
