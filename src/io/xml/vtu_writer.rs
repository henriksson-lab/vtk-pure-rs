use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, DataSetAttributes, UnstructuredGrid};
use crate::types::ScalarType;
use crate::types::VtkError;

/// Writer for VTK XML UnstructuredGrid format (.vtu).
///
/// Produces ASCII XML files compatible with ParaView and other VTK-based tools.
pub struct VtuWriter;

impl VtuWriter {
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
            "    <Piece NumberOfPoints=\"{}\" NumberOfCells=\"{}\">",
            n_points, n_cells
        )?;

        write_data_section(w, "PointData", grid.point_data())?;
        write_data_section(w, "CellData", grid.cell_data())?;
        write_points(w, grid, n_points)?;
        write_cells(w, grid, n_cells)?;

        writeln!(w, "    </Piece>")?;
        writeln!(w, "  </UnstructuredGrid>")?;
        writeln!(w, "</VTKFile>")?;

        Ok(())
    }
}

fn write_points<W: Write>(
    w: &mut W,
    grid: &UnstructuredGrid,
    n_points: usize,
) -> Result<(), VtkError> {
    writeln!(w, "      <Points>")?;
    writeln!(
        w,
        "        <DataArray type=\"Float64\" NumberOfComponents=\"3\" format=\"ascii\">"
    )?;
    write!(w, "          ")?;
    for i in 0..n_points {
        let p = grid.points.get(i);
        write!(w, "{} {} {} ", p[0], p[1], p[2])?;
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;
    writeln!(w, "      </Points>")?;
    Ok(())
}

fn write_cells<W: Write>(
    w: &mut W,
    grid: &UnstructuredGrid,
    n_cells: usize,
) -> Result<(), VtkError> {
    writeln!(w, "      <Cells>")?;

    writeln!(
        w,
        "        <DataArray type=\"Int64\" Name=\"connectivity\" format=\"ascii\">"
    )?;
    write!(w, "          ")?;
    for i in 0..n_cells {
        for &id in grid.cell_points(i) {
            write!(w, "{} ", id)?;
        }
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;

    writeln!(
        w,
        "        <DataArray type=\"Int64\" Name=\"offsets\" format=\"ascii\">"
    )?;
    write!(w, "          ")?;
    let mut offset: i64 = 0;
    for i in 0..n_cells {
        offset += grid.cell_points(i).len() as i64;
        write!(w, "{} ", offset)?;
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;

    writeln!(
        w,
        "        <DataArray type=\"UInt8\" Name=\"types\" format=\"ascii\">"
    )?;
    write!(w, "          ")?;
    for i in 0..n_cells {
        write!(w, "{} ", grid.cell_type(i) as u8)?;
    }
    writeln!(w)?;
    writeln!(w, "        </DataArray>")?;

    writeln!(w, "      </Cells>")?;
    Ok(())
}

fn write_data_section<W: Write>(
    w: &mut W,
    section: &str,
    attrs: &DataSetAttributes,
) -> Result<(), VtkError> {
    let attrs_str = data_attribute_string(attrs);

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
    writeln!(
        w,
        "        <DataArray type=\"{}\" Name=\"{}\" NumberOfComponents=\"{}\" format=\"ascii\">",
        xml_scalar_type(arr.scalar_type()),
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
    use crate::data::{DataArray, UnstructuredGrid};
    use crate::types::CellType;

    #[test]
    fn write_single_tetra_vtu() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let mut buf = Vec::new();
        VtuWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<VTKFile type=\"UnstructuredGrid\""));
        assert!(output.contains("NumberOfPoints=\"4\""));
        assert!(output.contains("NumberOfCells=\"1\""));
        assert!(output.contains("Name=\"connectivity\""));
        assert!(output.contains("Name=\"offsets\""));
        assert!(output.contains("Name=\"types\""));
    }

    #[test]
    fn write_with_scalars_vtu() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.5, 1.0, 0.0]);
        grid.points.push([0.5, 0.5, 1.0]);
        grid.push_cell(CellType::Tetra, &[0, 1, 2, 3]);

        let scalars = DataArray::from_vec("temp", vec![10.0, 20.0, 30.0, 40.0], 1);
        grid.point_data_mut().add_array(scalars.into());
        grid.point_data_mut().set_active_scalars("temp");

        let mut buf = Vec::new();
        VtuWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<PointData Scalars=\"temp\">"));
        assert!(output.contains("Name=\"temp\""));
    }

    #[test]
    fn write_preserves_attribute_hints_escaping_and_integer_types() {
        let mut grid = UnstructuredGrid::new();
        grid.points.push([0.0, 0.0, 0.0]);
        grid.points.push([1.0, 0.0, 0.0]);
        grid.points.push([0.0, 1.0, 0.0]);
        grid.push_cell(CellType::Triangle, &[0, 1, 2]);

        let ids = DataArray::from_vec("id&tag", vec![1u16, 2, 3], 1);
        let vectors =
            DataArray::from_vec("vel", vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 3);
        grid.point_data_mut().add_array(ids.into());
        grid.point_data_mut().add_array(vectors.into());
        grid.point_data_mut().set_active_scalars("id&tag");
        grid.point_data_mut().set_active_vectors("vel");

        let mut buf = Vec::new();
        VtuWriter::write_to(&mut buf, &grid).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("<PointData Scalars=\"id&amp;tag\" Vectors=\"vel\">"));
        assert!(output.contains("type=\"UInt16\" Name=\"id&amp;tag\""));
        assert!(output.contains("          1 2 3 "));
    }
}
