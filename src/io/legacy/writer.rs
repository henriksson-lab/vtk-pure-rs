use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, CellArray, DataSetAttributes, PolyData};
use crate::types::VtkError;

/// File type for legacy VTK format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Ascii,
    Binary,
}

/// Writer for VTK legacy format (.vtk) files.
pub struct LegacyWriter {
    pub file_type: FileType,
    pub header: String,
}

impl Default for LegacyWriter {
    fn default() -> Self {
        Self {
            file_type: FileType::Ascii,
            header: "vtk-rs output".to_string(),
        }
    }
}

impl LegacyWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ascii() -> Self {
        Self {
            file_type: FileType::Ascii,
            ..Default::default()
        }
    }

    pub fn binary() -> Self {
        Self {
            file_type: FileType::Binary,
            ..Default::default()
        }
    }

    pub fn write_poly_data(&self, path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        self.write_poly_data_to(&mut w, data)
    }

    pub fn write_poly_data_to<W: Write>(&self, w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        // Header
        writeln!(w, "# vtk DataFile Version 4.2")?;
        writeln!(w, "{}", self.header)?;
        match self.file_type {
            FileType::Ascii => writeln!(w, "ASCII")?,
            FileType::Binary => writeln!(w, "BINARY")?,
        }
        writeln!(w, "DATASET POLYDATA")?;

        // Points
        self.write_points(w, data)?;

        // Cells
        if !data.verts.is_empty() {
            self.write_cells(w, "VERTICES", &data.verts)?;
        }
        if !data.lines.is_empty() {
            self.write_cells(w, "LINES", &data.lines)?;
        }
        if !data.polys.is_empty() {
            self.write_cells(w, "POLYGONS", &data.polys)?;
        }
        if !data.strips.is_empty() {
            self.write_cells(w, "TRIANGLE_STRIPS", &data.strips)?;
        }

        // Point data
        if data.point_data().num_arrays() > 0 {
            writeln!(w, "POINT_DATA {}", data.points.len())?;
            self.write_attributes(w, data.point_data())?;
        }

        // Cell data
        if data.cell_data().num_arrays() > 0 {
            writeln!(w, "CELL_DATA {}", data.total_cells())?;
            self.write_attributes(w, data.cell_data())?;
        }

        Ok(())
    }

    fn write_points<W: Write>(&self, w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        let n = data.points.len();
        let arr = data.points.as_data_array();
        let type_name = arr.scalar_type().vtk_name();
        writeln!(w, "POINTS {} {}", n, type_name)?;

        match self.file_type {
            FileType::Ascii => {
                let slice = arr.as_slice();
                for i in 0..n {
                    let base = i * 3;
                    writeln!(w, "{} {} {}", slice[base], slice[base + 1], slice[base + 2])?;
                }
            }
            FileType::Binary => {
                // Big-endian binary
                let slice = arr.as_slice();
                for &val in slice {
                    w.write_all(&val.to_be_bytes())?;
                }
                writeln!(w)?;
            }
        }

        Ok(())
    }

    fn write_cells<W: Write>(
        &self,
        w: &mut W,
        keyword: &str,
        cells: &CellArray,
    ) -> Result<(), VtkError> {
        let num_cells = cells.num_cells();
        // Legacy format size = num_cells (for npts values) + total connectivity entries
        let total_size = num_cells + cells.connectivity_len();
        writeln!(w, "{} {} {}", keyword, num_cells, total_size)?;

        match self.file_type {
            FileType::Ascii => {
                for cell in cells.iter() {
                    write!(w, "{}", cell.len())?;
                    for &id in cell {
                        write!(w, " {}", id)?;
                    }
                    writeln!(w)?;
                }
            }
            FileType::Binary => {
                for cell in cells.iter() {
                    let npts = cell.len() as i32;
                    w.write_all(&npts.to_be_bytes())?;
                    for &id in cell {
                        let id32 = id as i32;
                        w.write_all(&id32.to_be_bytes())?;
                    }
                }
                writeln!(w)?;
            }
        }

        Ok(())
    }

    fn write_attributes<W: Write>(
        &self,
        w: &mut W,
        attrs: &DataSetAttributes,
    ) -> Result<(), VtkError> {
        let vectors_name = attrs.vectors().map(|a| a.name().to_string());
        let normals_name = attrs.normals().map(|a| a.name().to_string());

        for i in 0..attrs.num_arrays() {
            if let Some(arr) = attrs.get_array_by_index(i) {
                if normals_name.as_deref() == Some(arr.name()) && arr.num_components() == 3 {
                    self.write_data_array_as_vectors(w, "NORMALS", arr)?;
                } else if vectors_name.as_deref() == Some(arr.name()) && arr.num_components() == 3 {
                    self.write_data_array_as_vectors(w, "VECTORS", arr)?;
                } else {
                    self.write_data_array_as_scalars(w, arr)?;
                }
            }
        }
        Ok(())
    }

    fn write_data_array_as_scalars<W: Write>(
        &self,
        w: &mut W,
        arr: &AnyDataArray,
    ) -> Result<(), VtkError> {
        let name = arr.name();
        let nc = arr.num_components();
        let nt = arr.num_tuples();
        let type_name = arr.scalar_type().vtk_name();

        if nc == 1 {
            writeln!(w, "SCALARS {} {}", name, type_name)?;
        } else {
            writeln!(w, "SCALARS {} {} {}", name, type_name, nc)?;
        }
        writeln!(w, "LOOKUP_TABLE default")?;

        match self.file_type {
            FileType::Ascii => {
                self.write_any_array_ascii(w, arr, nt, nc)?;
            }
            FileType::Binary => {
                self.write_any_array_binary(w, arr)?;
            }
        }

        Ok(())
    }

    fn write_data_array_as_vectors<W: Write>(
        &self,
        w: &mut W,
        keyword: &str,
        arr: &AnyDataArray,
    ) -> Result<(), VtkError> {
        writeln!(
            w,
            "{} {} {}",
            keyword,
            arr.name(),
            arr.scalar_type().vtk_name()
        )?;

        match self.file_type {
            FileType::Ascii => {
                self.write_any_array_ascii(w, arr, arr.num_tuples(), arr.num_components())?;
            }
            FileType::Binary => {
                self.write_any_array_binary(w, arr)?;
            }
        }

        Ok(())
    }

    fn write_any_array_ascii<W: Write>(
        &self,
        w: &mut W,
        arr: &AnyDataArray,
        num_tuples: usize,
        _num_components: usize,
    ) -> Result<(), VtkError> {
        macro_rules! write_typed {
            ($a:expr) => {
                for i in 0..num_tuples {
                    let t = $a.tuple(i);
                    for (j, v) in t.iter().enumerate() {
                        if j > 0 {
                            write!(w, " ")?;
                        }
                        write!(w, "{}", v)?;
                    }
                    writeln!(w)?;
                }
            };
        }
        match arr {
            AnyDataArray::F32(a) => write_typed!(a),
            AnyDataArray::F64(a) => write_typed!(a),
            AnyDataArray::I8(a) => write_typed!(a),
            AnyDataArray::I16(a) => write_typed!(a),
            AnyDataArray::I32(a) => write_typed!(a),
            AnyDataArray::I64(a) => write_typed!(a),
            AnyDataArray::U8(a) => write_typed!(a),
            AnyDataArray::U16(a) => write_typed!(a),
            AnyDataArray::U32(a) => write_typed!(a),
            AnyDataArray::U64(a) => write_typed!(a),
        }
        Ok(())
    }

    fn write_any_array_binary<W: Write>(
        &self,
        w: &mut W,
        arr: &AnyDataArray,
    ) -> Result<(), VtkError> {
        macro_rules! write_be {
            ($a:expr) => {
                for &val in $a.as_slice() {
                    w.write_all(&val.to_be_bytes())?;
                }
            };
        }
        match arr {
            AnyDataArray::F32(a) => write_be!(a),
            AnyDataArray::F64(a) => write_be!(a),
            AnyDataArray::I8(a) => {
                w.write_all(bytemuck::cast_slice(a.as_slice()))?;
            }
            AnyDataArray::I16(a) => write_be!(a),
            AnyDataArray::I32(a) => write_be!(a),
            AnyDataArray::I64(a) => write_be!(a),
            AnyDataArray::U8(a) => {
                w.write_all(a.as_slice())?;
            }
            AnyDataArray::U16(a) => write_be!(a),
            AnyDataArray::U32(a) => write_be!(a),
            AnyDataArray::U64(a) => write_be!(a),
        }
        writeln!(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataArray, PolyData};

    #[test]
    fn write_triangle_ascii() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let writer = LegacyWriter::ascii();
        let mut buf = Vec::new();
        writer.write_poly_data_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.starts_with("# vtk DataFile Version 4.2"));
        assert!(output.contains("ASCII"));
        assert!(output.contains("DATASET POLYDATA"));
        assert!(output.contains("POINTS 3 double"));
        assert!(output.contains("POLYGONS 1 4"));
        assert!(output.contains("3 0 1 2"));
    }

    #[test]
    fn write_active_vectors_as_vectors() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let vectors = DataArray::from_vec(
            "velocity",
            vec![1.0f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            3,
        );
        pd.point_data_mut().add_array(vectors.into());
        pd.point_data_mut().set_active_vectors("velocity");

        let writer = LegacyWriter::ascii();
        let mut buf = Vec::new();
        writer.write_poly_data_to(&mut buf, &pd).unwrap();
        let output = String::from_utf8(buf).unwrap();

        assert!(output.contains("VECTORS velocity double"));
        assert!(!output.contains("SCALARS velocity"));
    }
}
