use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataSetAttributes, RectilinearGrid};
use crate::types::VtkError;

/// Writer for VTK XML RectilinearGrid format (.vtr).
pub struct VtrWriter;

impl VtrWriter {
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
        writeln!(w, "  <RectilinearGrid WholeExtent=\"{}\">", ext)?;
        writeln!(w, "    <Piece Extent=\"{}\">", ext)?;

        // Coordinates
        writeln!(w, "      <Coordinates>")?;
        write_coord_array(w, "x", grid.x_coords())?;
        write_coord_array(w, "y", grid.y_coords())?;
        write_coord_array(w, "z", grid.z_coords())?;
        writeln!(w, "      </Coordinates>")?;

        // Point data
        if grid.point_data().num_arrays() > 0 {
            write_data_section(w, "PointData", grid.point_data())?;
        }

        // Cell data
        if grid.cell_data().num_arrays() > 0 {
            write_data_section(w, "CellData", grid.cell_data())?;
        }

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </RectilinearGrid>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
    }
}

fn write_coord_array<W: Write>(w: &mut W, name: &str, coords: &[f64]) -> Result<(), VtkError> {
    writeln!(
        w,
        "        <DataArray type=\"Float64\" Name=\"{}\" format=\"ascii\">",
        name
    )?;
    write!(w, "          ")?;
    for &v in coords {
        write!(w, "{} ", v)?;
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;
    Ok(())
}

fn write_data_section<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    let scalars_name = attrs.scalars().map(|a| a.name().to_string());
    let mut attrs_str = String::new();
    if let Some(ref name) = scalars_name {
        attrs_str.push_str(&format!(" Scalars=\"{}\"", name));
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
        _ => "Float64",
    };
    writeln!(
        w,
        "        <DataArray type=\"{}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"ascii\">",
        type_name,
        arr.name(),
        arr.num_components()
    )?;
    write!(w, "          ")?;
    let nt = arr.num_tuples();
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    for i in 0..nt {
        arr.tuple_as_f64(i, &mut buf);
        for v in &buf {
            write!(w, "{} ", v)?;
        }
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray, DataSet, RectilinearGrid};

    #[test]
    fn write_simple_vtr() {
        let grid = RectilinearGrid::from_coords(vec![0.0, 1.0, 3.0], vec![0.0, 2.0], vec![0.0]);
        let mut buf = Vec::new();
        VtrWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("<VTKFile type=\"RectilinearGrid\""));
        assert!(output.contains("WholeExtent=\"0 2 0 1 0 0\""));
        assert!(output.contains("Name=\"x\""));
    }
}
