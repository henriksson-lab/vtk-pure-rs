use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, RectilinearGrid};
use crate::types::VtkError;

use crate::io::xml::binary;

/// Writer for VTK XML RectilinearGrid format (.vtr) with binary encoding.
pub struct VtrBinaryWriter;

impl VtrBinaryWriter {
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
            "<VTKFile type=\"RectilinearGrid\" version=\"1.0\" byte_order=\"LittleEndian\">"
        )?;
        writeln!(w, "  <RectilinearGrid WholeExtent=\"{ext}\">")?;
        writeln!(w, "    <Piece Extent=\"{ext}\">")?;

        writeln!(w, "      <Coordinates>")?;
        write_coord_binary(w, "x", grid.x_coords())?;
        write_coord_binary(w, "y", grid.y_coords())?;
        write_coord_binary(w, "z", grid.z_coords())?;
        writeln!(w, "      </Coordinates>")?;

        if grid.point_data().num_arrays() > 0 {
            write_binary_attrs(w, "PointData", grid.point_data())?;
        }
        if grid.cell_data().num_arrays() > 0 {
            write_binary_attrs(w, "CellData", grid.cell_data())?;
        }

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </RectilinearGrid>")?;
        writeln!(w, "</VTKFile>")?;
        Ok(())
    }
}

fn write_coord_binary<W: Write>(w: &mut W, name: &str, coords: &[f64]) -> Result<(), VtkError> {
    let arr = AnyDataArray::F64(DataArray::from_vec(name, coords.to_vec(), 1));
    let encoded = binary::encode_data_array_binary(&arr);
    writeln!(w, "        <DataArray type=\"Float64\" Name=\"{name}\" format=\"binary\">{encoded}</DataArray>")?;
    Ok(())
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
    use crate::data::RectilinearGrid;

    #[test]
    fn roundtrip_vtr_binary() {
        let grid = RectilinearGrid::from_coords(vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![0.0]);
        let mut buf = Vec::new();
        VtrBinaryWriter::write_to(&mut buf, &grid).unwrap();

        let xml = String::from_utf8(buf.clone()).unwrap();
        assert!(xml.contains("format=\"binary\""));

        let reader = std::io::BufReader::new(&buf[..]);
        let result = crate::io::xml::VtrReader::read_from(reader).unwrap();
        assert_eq!(result.dimensions(), grid.dimensions());
    }
}
