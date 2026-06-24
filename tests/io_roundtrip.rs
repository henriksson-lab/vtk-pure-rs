//! Integration tests for I/O format roundtrips.
//!
//! Each test writes a PolyData to a format and reads it back,
//! verifying that the geometry is preserved.

use vtk_data::{AnyDataArray, DataArray, PolyData};

fn make_triangle() -> PolyData {
    PolyData::from_triangles(
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0, 1, 2]],
    )
}

fn make_two_triangles() -> PolyData {
    PolyData::from_triangles(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ],
        vec![[0, 1, 2], [1, 3, 2]],
    )
}

fn make_triangle_with_scalars() -> PolyData {
    let mut pd = make_triangle();
    let s = DataArray::from_vec("temperature", vec![10.0f64, 20.0, 30.0], 1);
    pd.point_data_mut().add_array(AnyDataArray::F64(s));
    pd.point_data_mut().set_active_scalars("temperature");
    pd
}

// === OBJ ===

#[test]
fn obj_roundtrip_triangle() {
    let pd = make_triangle();
    let mut buf = Vec::new();
    vtk_io_obj::ObjWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_obj::ObjReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(result.points.len(), 3);
    assert_eq!(result.polys.num_cells(), 1);
}

#[test]
fn obj_roundtrip_two_triangles() {
    let pd = make_two_triangles();
    let mut buf = Vec::new();
    vtk_io_obj::ObjWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_obj::ObjReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(result.points.len(), 4);
    assert_eq!(result.polys.num_cells(), 2);
}

// === STL ===

#[test]
fn stl_ascii_roundtrip() {
    let pd = make_two_triangles();
    let mut buf = Vec::new();
    vtk_io_stl::StlWriter::ascii()
        .write_to(&mut buf, &pd)
        .unwrap();
    let result = vtk_io_stl::StlReader::read_from(&buf).unwrap();
    // STL duplicates vertices per triangle
    assert_eq!(result.polys.num_cells(), 2);
}

#[test]
fn stl_binary_roundtrip() {
    let pd = make_two_triangles();
    let mut buf = Vec::new();
    vtk_io_stl::StlWriter::binary()
        .write_to(&mut buf, &pd)
        .unwrap();
    let result = vtk_io_stl::StlReader::read_from(&buf).unwrap();
    assert_eq!(result.polys.num_cells(), 2);
}

// === VTK Legacy ===

#[test]
fn legacy_ascii_roundtrip() {
    let pd = make_triangle();
    let mut buf = Vec::new();
    vtk_io_legacy::LegacyWriter::ascii()
        .write_poly_data_to(&mut buf, &pd)
        .unwrap();
    let result =
        vtk_io_legacy::LegacyReader::read_poly_data_from(std::io::BufReader::new(&buf[..]))
            .unwrap();
    assert_eq!(result.points.len(), 3);
    assert_eq!(result.polys.num_cells(), 1);
}

#[test]
fn legacy_binary_roundtrip() {
    let pd = make_two_triangles();
    let mut buf = Vec::new();
    vtk_io_legacy::LegacyWriter::binary()
        .write_poly_data_to(&mut buf, &pd)
        .unwrap();
    let result =
        vtk_io_legacy::LegacyReader::read_poly_data_from(std::io::BufReader::new(&buf[..]))
            .unwrap();
    assert_eq!(result.points.len(), 4);
    assert_eq!(result.polys.num_cells(), 2);
}

#[test]
fn legacy_with_scalars() {
    let pd = make_triangle_with_scalars();
    let mut buf = Vec::new();
    vtk_io_legacy::LegacyWriter::ascii()
        .write_poly_data_to(&mut buf, &pd)
        .unwrap();
    let result =
        vtk_io_legacy::LegacyReader::read_poly_data_from(std::io::BufReader::new(&buf[..]))
            .unwrap();
    assert!(result.point_data().scalars().is_some());
}

// === PLY ===

#[test]
fn ply_ascii_roundtrip() {
    let pd = make_triangle();
    let mut buf = Vec::new();
    vtk_io_ply::PlyWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_ply::PlyReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(result.points.len(), 3);
    assert_eq!(result.polys.num_cells(), 1);
}

#[test]
fn ply_binary_roundtrip() {
    let pd = make_two_triangles();
    let mut buf = Vec::new();
    vtk_io_ply::PlyBinaryWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_ply::PlyBinaryReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(result.points.len(), 4);
    assert_eq!(result.polys.num_cells(), 2);
}

// === VTP XML ===

#[test]
fn vtp_ascii_roundtrip() {
    let pd = make_triangle_with_scalars();
    let mut buf = Vec::new();
    vtk_io_xml::VtpWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_xml::VtpReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(result.points.len(), 3);
    assert!(result.point_data().scalars().is_some());
}

#[test]
fn vtp_binary_roundtrip() {
    let pd = make_triangle_with_scalars();
    let mut buf = Vec::new();
    vtk_io_xml::VtpBinaryWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_xml::VtpReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(result.points.len(), 3);
    assert!(result.point_data().scalars().is_some());
}

// === GLB ===

#[test]
fn glb_roundtrip() {
    let pd = make_two_triangles();
    let mut buf = Vec::new();
    vtk_io_gltf::GlbWriter::write_to(&mut buf, &pd).unwrap();
    let result = vtk_io_gltf::GlbReader::read_from(&buf).unwrap();
    assert_eq!(result.points.len(), 4);
    assert_eq!(result.polys.num_cells(), 2);
}

// === Cross-format: write in one, read point count ===

#[test]
fn cross_format_point_preservation() {
    let pd = make_two_triangles();
    let n = pd.points.len();

    // VTK Legacy
    let mut buf = Vec::new();
    vtk_io_legacy::LegacyWriter::ascii()
        .write_poly_data_to(&mut buf, &pd)
        .unwrap();
    let r = vtk_io_legacy::LegacyReader::read_poly_data_from(std::io::BufReader::new(&buf[..]))
        .unwrap();
    assert_eq!(r.points.len(), n, "VTK Legacy");

    // OBJ
    let mut buf = Vec::new();
    vtk_io_obj::ObjWriter::write_to(&mut buf, &pd).unwrap();
    let r = vtk_io_obj::ObjReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(r.points.len(), n, "OBJ");

    // VTP
    let mut buf = Vec::new();
    vtk_io_xml::VtpWriter::write_to(&mut buf, &pd).unwrap();
    let r = vtk_io_xml::VtpReader::read_from(std::io::BufReader::new(&buf[..])).unwrap();
    assert_eq!(r.points.len(), n, "VTP");

    // GLB
    let mut buf = Vec::new();
    vtk_io_gltf::GlbWriter::write_to(&mut buf, &pd).unwrap();
    let r = vtk_io_gltf::GlbReader::read_from(&buf).unwrap();
    assert_eq!(r.points.len(), n, "GLB");
}
