use std::io::Write;
use std::path::Path;

use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::types::VtkError;

/// Writer for Stanford PLY format (binary little-endian).
pub struct PlyBinaryWriter;

impl PlyBinaryWriter {
    pub fn write(path: &Path, data: &PolyData) -> Result<(), VtkError> {
        let file = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::new(file);
        Self::write_to(&mut w, data)
    }

    pub fn write_to<W: Write>(w: &mut W, data: &PolyData) -> Result<(), VtkError> {
        let n_verts = data.points.len();
        let n_faces = data.polys.num_cells();

        let point_normals = point_normals(data);
        let point_colors = point_colors(data);
        let point_tcoords = point_tcoords(data);
        let cell_colors = cell_colors(data);
        let has_normals = point_normals.is_some();
        let has_colors = point_colors.is_some();
        let has_tcoords = point_tcoords.is_some();
        let has_cell_colors = cell_colors.is_some();

        // Header (always ASCII)
        writeln!(w, "ply")?;
        writeln!(w, "format binary_little_endian 1.0")?;
        writeln!(w, "comment VTK generated PLY File")?;
        writeln!(w, "obj_info vtkPolyData points and polygons: vtk4.0")?;
        writeln!(w, "element vertex {}", n_verts)?;
        writeln!(w, "property float x")?;
        writeln!(w, "property float y")?;
        writeln!(w, "property float z")?;
        if has_normals {
            writeln!(w, "property float nx")?;
            writeln!(w, "property float ny")?;
            writeln!(w, "property float nz")?;
        }
        if has_colors {
            writeln!(w, "property uchar red")?;
            writeln!(w, "property uchar green")?;
            writeln!(w, "property uchar blue")?;
        }
        if has_tcoords {
            writeln!(w, "property float u")?;
            writeln!(w, "property float v")?;
        }
        writeln!(w, "element face {}", n_faces)?;
        writeln!(w, "property list uchar int vertex_indices")?;
        if has_cell_colors {
            writeln!(w, "property uchar red")?;
            writeln!(w, "property uchar green")?;
            writeln!(w, "property uchar blue")?;
        }
        writeln!(w, "end_header")?;

        // Vertices (binary f32 little-endian)
        for i in 0..n_verts {
            let p = data.points.get(i);
            w.write_all(&(p[0] as f32).to_le_bytes())?;
            w.write_all(&(p[1] as f32).to_le_bytes())?;
            w.write_all(&(p[2] as f32).to_le_bytes())?;
            if let Some(normals) = point_normals {
                let mut tuple = [0.0; 3];
                normals.tuple_as_f64(i, &mut tuple);
                w.write_all(&(tuple[0] as f32).to_le_bytes())?;
                w.write_all(&(tuple[1] as f32).to_le_bytes())?;
                w.write_all(&(tuple[2] as f32).to_le_bytes())?;
            }
            if let Some(colors) = point_colors {
                let tuple = colors.tuple(i);
                w.write_all(&[tuple[0], tuple[1], tuple[2]])?;
            }
            if let Some(tcoords) = point_tcoords {
                let mut tuple = [0.0; 2];
                tcoords.tuple_as_f64(i, &mut tuple);
                w.write_all(&(tuple[0] as f32).to_le_bytes())?;
                w.write_all(&(tuple[1] as f32).to_le_bytes())?;
            }
        }

        // Faces (binary)
        for (cell_idx, cell) in data.polys.iter().enumerate() {
            if cell.len() > u8::MAX as usize {
                return Err(VtkError::Unsupported(
                    "PLY face vertex count exceeds uchar list count".into(),
                ));
            }
            w.write_all(&[cell.len() as u8])?;
            for &id in cell {
                if id < i32::MIN as i64 || id > i32::MAX as i64 {
                    return Err(VtkError::Unsupported(
                        "PLY face vertex index exceeds int range".into(),
                    ));
                }
                w.write_all(&(id as i32).to_le_bytes())?;
            }
            if let Some(colors) = cell_colors {
                let tuple = colors.tuple(cell_idx);
                w.write_all(&[tuple[0], tuple[1], tuple[2]])?;
            }
        }

        Ok(())
    }
}

fn point_normals(data: &PolyData) -> Option<&AnyDataArray> {
    let normals = data.point_data().normals()?;
    (normals.num_components() == 3 && normals.num_tuples() == data.points.len()).then_some(normals)
}

fn point_colors(data: &PolyData) -> Option<&DataArray<u8>> {
    match data.point_data().scalars() {
        Some(AnyDataArray::U8(colors))
            if (colors.num_components() == 3 || colors.num_components() == 4)
                && colors.num_tuples() == data.points.len() =>
        {
            Some(colors)
        }
        _ => None,
    }
}

fn point_tcoords(data: &PolyData) -> Option<&AnyDataArray> {
    let tcoords = data.point_data().tcoords()?;
    (tcoords.num_components() == 2 && tcoords.num_tuples() == data.points.len()).then_some(tcoords)
}

fn cell_colors(data: &PolyData) -> Option<&DataArray<u8>> {
    match data.cell_data().scalars() {
        Some(AnyDataArray::U8(colors))
            if (colors.num_components() == 3 || colors.num_components() == 4)
                && colors.num_tuples() == data.polys.num_cells() =>
        {
            Some(colors)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::AnyDataArray;
    use crate::io::ply::PlyBinaryReader;

    #[test]
    fn write_binary_ply() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        // Header should be ASCII
        let header_end = buf.windows(11).position(|w| w == b"end_header\n").unwrap() + 11;
        let header = std::str::from_utf8(&buf[..header_end]).unwrap();
        assert!(header.contains("binary_little_endian"));
        // Binary data: 3 vertices * 12 bytes + 1 face * (1 + 12) bytes
        assert_eq!(buf.len() - header_end, 3 * 12 + 1 * 13);
    }

    #[test]
    fn write_binary_ply_ignores_non_u8_vertex_colors_without_lookup_table() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 1.0],
                3,
            )));
        pd.point_data_mut().set_active_scalars("rgb");

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        let header_end = buf.windows(11).position(|w| w == b"end_header\n").unwrap() + 11;
        let header = std::str::from_utf8(&buf[..header_end]).unwrap();
        assert!(!header.contains("property uchar red"));

        let result = PlyBinaryReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
        assert!(result.point_data().scalars().is_none());
    }

    #[test]
    fn write_binary_ply_preserves_u8_vertex_colors() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "RGB",
                vec![255, 0, 0, 0, 127, 0, 0, 0, 255],
                3,
            )));
        pd.point_data_mut().set_active_scalars("RGB");

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        let result = PlyBinaryReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
        let scalars = result.point_data().scalars().unwrap();
        let mut tuple = [0.0; 3];
        scalars.tuple_as_f64(0, &mut tuple);
        assert_eq!(tuple, [255.0, 0.0, 0.0]);
        scalars.tuple_as_f64(1, &mut tuple);
        assert_eq!(tuple, [0.0, 127.0, 0.0]);
        scalars.tuple_as_f64(2, &mut tuple);
        assert_eq!(tuple, [0.0, 0.0, 255.0]);
    }

    #[test]
    fn write_binary_ply_drops_u8_vertex_alpha_by_default() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "RGBA",
                vec![255, 0, 0, 9, 0, 127, 0, 8, 0, 0, 255, 7],
                4,
            )));
        pd.point_data_mut().set_active_scalars("RGBA");

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        let result = PlyBinaryReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
        let scalars = result.point_data().scalars().unwrap();
        assert_eq!(scalars.num_components(), 3);
        let mut tuple = [0.0; 3];
        scalars.tuple_as_f64(0, &mut tuple);
        assert_eq!(tuple, [255.0, 0.0, 0.0]);
        scalars.tuple_as_f64(1, &mut tuple);
        assert_eq!(tuple, [0.0, 127.0, 0.0]);
        scalars.tuple_as_f64(2, &mut tuple);
        assert_eq!(tuple, [0.0, 0.0, 255.0]);
    }

    #[test]
    fn write_binary_ply_preserves_normals_and_tcoords() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Normals",
                vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0],
                3,
            )));
        pd.point_data_mut().set_active_normals("Normals");
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "TCoords",
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                2,
            )));
        pd.point_data_mut().set_active_tcoords("TCoords");

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        let result = PlyBinaryReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
        assert!(result.point_data().normals().is_some());
        assert!(result.point_data().tcoords().is_some());
    }

    #[test]
    fn write_binary_ply_preserves_cell_colors() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.cell_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "RGB",
                vec![10, 20, 30],
                3,
            )));
        pd.cell_data_mut().set_active_scalars("RGB");

        let mut buf = Vec::new();
        PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
        let result = PlyBinaryReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
        let colors = result.cell_data().scalars().unwrap();
        let mut tuple = [0.0; 3];
        colors.tuple_as_f64(0, &mut tuple);
        assert_eq!(tuple, [10.0, 20.0, 30.0]);
    }
}
