use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataSetAttributes, StructuredGrid};
use crate::types::VtkError;

/// Writer for VTK XML StructuredGrid format (.vts).
pub struct VtsWriter;

impl VtsWriter {
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
        writeln!(w, "  <StructuredGrid WholeExtent=\"{}\">", ext)?;
        writeln!(w, "    <Piece Extent=\"{}\">", ext)?;

        // Points
        writeln!(w, "      <Points>")?;
        writeln!(
            w,
            "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"ascii\">"
        )?;
        write!(w, "          ")?;
        for i in 0..grid.points.len() {
            let p = grid.points.get(i);
            write!(w, "{} {} {} ", p[0], p[1], p[2])?;
        }
        writeln!(w)?;
        writeln!(w, "        </DataArray>")?;
        writeln!(w, "      </Points>")?;

        if grid.point_data().num_arrays() > 0 {
            write_data_section(w, "PointData", grid.point_data())?;
        }
        if grid.cell_data().num_arrays() > 0 {
            write_data_section(w, "CellData", grid.cell_data())?;
        }

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </StructuredGrid>")?;
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
    use crate::data::{DataSet, Points, StructuredGrid};

    #[test]
    fn write_simple_vts() {
        let mut pts = Points::new();
        for j in 0..2 {
            for i in 0..3 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        let grid = StructuredGrid::from_dimensions_and_points([3, 2, 1], pts);
        let mut buf = Vec::new();
        VtsWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("<VTKFile type=\"StructuredGrid\""));
        assert!(output.contains("WholeExtent=\"0 2 0 1 0 0\""));
    }
}
