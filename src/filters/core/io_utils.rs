use std::path::Path;

use crate::data::PolyData;
use crate::types::VtkError;

/// Read a PolyData from a file, auto-detecting the format by extension.
///
/// Supported extensions: `.vtk`, `.vtp`, `.stl`, `.obj`, `.ply`, `.glb`, `.off`
pub fn read_poly_data(path: &Path) -> Result<PolyData, VtkError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "vtk" => crate::io::legacy::LegacyReader::read_poly_data(path),
        "vtp" => crate::io::xml::VtpReader::read(path),
        "stl" => crate::io::stl::StlReader::read(path),
        "obj" => crate::io::obj::ObjReader::read(path),
        "ply" => crate::io::ply::PlyReader::read(path),
        "glb" => crate::io::gltf::GlbReader::read(path),
        "off" => crate::io::off::read_off_file(path).map_err(|e| VtkError::Parse(e)),
        _ => Err(VtkError::Unsupported(format!(
            "unknown file extension: .{ext}"
        ))),
    }
}

/// Write a PolyData to a file, auto-detecting the format by extension.
///
/// Supported extensions: `.vtk`, `.vtp`, `.stl`, `.obj`, `.ply`, `.glb`, `.off`
pub fn write_poly_data(path: &Path, data: &PolyData) -> Result<(), VtkError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "vtk" => crate::io::legacy::LegacyWriter::ascii().write_poly_data(path, data),
        "vtp" => crate::io::xml::VtpWriter::write(path, data),
        "stl" => crate::io::stl::StlWriter::binary().write(path, data),
        "obj" => crate::io::obj::ObjWriter::write(path, data),
        "ply" => crate::io::ply::PlyWriter::write(path, data),
        "glb" => crate::io::gltf::GlbWriter::write(path, data),
        "off" => crate::io::off::write_off_file(data, path)
            .map_err(|e| VtkError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))),
        _ => Err(VtkError::Unsupported(format!(
            "unknown file extension: .{ext}"
        ))),
    }
}

/// List of supported file extensions for reading.
pub fn supported_read_extensions() -> &'static [&'static str] {
    &["vtk", "vtp", "stl", "obj", "ply", "glb", "off"]
}

/// List of supported file extensions for writing.
pub fn supported_write_extensions() -> &'static [&'static str] {
    &["vtk", "vtp", "stl", "obj", "ply", "glb", "off"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_auto_vtk() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let dir = std::env::temp_dir().join("vtk_auto_io_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.vtk");
        write_poly_data(&path, &pd).unwrap();
        let result = read_poly_data(&path).unwrap();
        assert_eq!(result.points.len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn roundtrip_auto_stl() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let dir = std::env::temp_dir().join("vtk_auto_stl_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("test.stl");
        write_poly_data(&path, &pd).unwrap();
        let result = read_poly_data(&path).unwrap();
        assert_eq!(result.polys.num_cells(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unsupported_extension() {
        let result = read_poly_data(Path::new("test.xyz"));
        assert!(result.is_err());
    }

    #[test]
    fn supported_extensions() {
        assert!(supported_read_extensions().contains(&"vtk"));
        assert!(supported_read_extensions().contains(&"stl"));
        assert!(supported_read_extensions().contains(&"glb"));
    }
}
