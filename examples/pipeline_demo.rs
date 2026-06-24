//! Pipeline demo: demonstrates the filter pipeline, I/O, and offline processing.
//!
//! This example:
//! 1. Generates a sphere source
//! 2. Builds a processing chain (normals -> elevation -> decimate)
//! 3. Writes the result to multiple formats
//! 4. Prints mesh statistics

use std::path::Path;

use vtk_pure_rs::data::PolyData;
use vtk_pure_rs::filters::core::sources::{sphere, SphereParams};
use vtk_pure_rs::filters::core::{decimate, elevation, topology};
use vtk_pure_rs::filters::normals::normals;

fn main() {
    println!("vtk-rs Pipeline Demo");
    println!("====================\n");

    // 1. Generate a sphere
    let src = sphere(&SphereParams {
        theta_resolution: 32,
        phi_resolution: 32,
        ..Default::default()
    });
    println!("Source: {src}");

    // 2. Apply a VTK-style source -> filter -> filter -> filter chain.
    let with_normals = normals::compute_normals(&src);
    let with_elevation = elevation::elevation_z(&with_normals);
    let result = decimate::decimate(&with_elevation, 0.5);

    println!(
        "Pipeline stages: {:?}",
        ["normals", "elevation", "decimate"]
    );

    // 3. Get the output
    println!("Output: {result}");

    // 4. Topology analysis
    let topo = topology::analyze_topology(&result);
    println!("\nTopology:");
    println!("  Points:        {}", topo.num_points);
    println!("  Edges:         {}", topo.num_edges);
    println!("  Faces:         {}", topo.num_faces);
    println!("  Boundary edges:{}", topo.num_boundary_edges);
    println!("  Euler:         {}", topo.euler_characteristic);
    println!("  Components:    {}", topo.num_components);
    println!("  Manifold:      {}", topo.is_manifold);
    println!("  Triangle mesh: {}", topo.is_triangle_mesh);
    if let Some(g) = topo.genus {
        println!("  Genus:         {g}");
    }

    // 5. Data array statistics
    if let Some(scalars) = result.point_data().scalars() {
        if let Some(stats) = scalars.statistics() {
            println!("\nScalar statistics ({}):", scalars.name());
            println!("  Range: [{:.3}, {:.3}]", stats.min, stats.max);
            println!("  Mean:  {:.3}", stats.mean);
            println!("  Std:   {:.3}", stats.std_dev());
        }
    }

    // 6. Write to multiple formats
    let dir = std::env::temp_dir().join("vtk_pipeline_demo");
    let _ = std::fs::create_dir_all(&dir);

    let formats = ["vtk", "vtp", "stl", "obj", "ply", "glb"];
    println!("\nWriting to:");
    for ext in &formats {
        let path = dir.join(format!("sphere.{ext}"));
        match write_poly_data(&path, &result) {
            Ok(()) => println!("  {} ✓", path.display()),
            Err(e) => println!("  {} ✗ {e}", path.display()),
        }
    }

    // 7. Verify roundtrip
    let vtk_path = dir.join("sphere.vtk");
    match read_poly_data(&vtk_path) {
        Ok(loaded) => {
            println!("\nRoundtrip verification:");
            println!("  Original: {} points", result.points.len());
            println!("  Loaded:   {} points", loaded.points.len());
            println!("  Match:    {}", result.approx_eq(&loaded, 1e-6));
        }
        Err(e) => println!("\nRoundtrip failed: {e}"),
    }

    let _ = std::fs::remove_dir_all(&dir);
    println!("\nDone.");
}

fn read_poly_data(path: &Path) -> Result<PolyData, String> {
    match extension(path).as_str() {
        "vtk" => {
            vtk_pure_rs::io::legacy::LegacyReader::read_poly_data(path).map_err(|e| e.to_string())
        }
        "vtp" => vtk_pure_rs::io::xml::VtpReader::read(path).map_err(|e| e.to_string()),
        "stl" => vtk_pure_rs::io::stl::StlReader::read(path).map_err(|e| e.to_string()),
        "obj" => vtk_pure_rs::io::obj::ObjReader::read(path).map_err(|e| e.to_string()),
        "ply" => vtk_pure_rs::io::ply::PlyReader::read(path).map_err(|e| e.to_string()),
        "glb" => vtk_pure_rs::io::gltf::GlbReader::read(path).map_err(|e| e.to_string()),
        "off" => vtk_pure_rs::io::off::read_off_file(path),
        ext => Err(format!("unknown file extension: .{ext}")),
    }
}

fn write_poly_data(path: &Path, data: &PolyData) -> Result<(), String> {
    match extension(path).as_str() {
        "vtk" => vtk_pure_rs::io::legacy::LegacyWriter::ascii()
            .write_poly_data(path, data)
            .map_err(|e| e.to_string()),
        "vtp" => vtk_pure_rs::io::xml::VtpWriter::write(path, data).map_err(|e| e.to_string()),
        "stl" => vtk_pure_rs::io::stl::StlWriter::binary()
            .write(path, data)
            .map_err(|e| e.to_string()),
        "obj" => vtk_pure_rs::io::obj::ObjWriter::write(path, data).map_err(|e| e.to_string()),
        "ply" => vtk_pure_rs::io::ply::PlyWriter::write(path, data).map_err(|e| e.to_string()),
        "glb" => vtk_pure_rs::io::gltf::GlbWriter::write(path, data).map_err(|e| e.to_string()),
        "off" => vtk_pure_rs::io::off::write_off_file(data, path),
        ext => Err(format!("unknown file extension: .{ext}")),
    }
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default()
}
