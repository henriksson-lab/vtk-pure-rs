//! Headless batch renderer: generate a mesh, render offscreen on the GPU, save as image.
//!
//! No window and no display server are needed. `WgpuRenderer::new_headless()` creates
//! a wgpu device with no surface attached; `render_to_image()` then draws into a
//! renderer-owned texture and reads the pixels back to the CPU.
//!
//! Usage: cargo run --features render-wgpu --example headless_render

#[cfg(feature = "render-wgpu")]
mod app {
    use vtk_pure_rs::filters::core::{elevation, sources};
    use vtk_pure_rs::filters::normals::normals;
    use vtk_pure_rs::render::*;
    use vtk_pure_rs::render_wgpu::WgpuRenderer;

    /// Build the demo scene (a shaded, elevation-coloured sphere on a dark background).
    pub fn build_scene() -> Scene {
        let sphere = sources::sphere(&sources::SphereParams {
            theta_resolution: 32,
            phi_resolution: 32,
            ..Default::default()
        });
        let mesh = normals::compute_normals(&elevation::elevation_z(&sphere));

        let mut scene = Scene::new()
            .with_actor(
                Actor::new(mesh)
                    .with_scalar_coloring(ColorMap::viridis(), None)
                    .with_material(Material::pbr_dielectric(0.4)),
            )
            .with_background(0.05, 0.05, 0.1)
            .with_fog(Fog::linear(2.0, 8.0).with_color(0.05, 0.05, 0.1));

        scene.add_scalar_bar(ScalarBar::new(
            "Elevation",
            ColorMap::viridis(),
            [-1.0, 1.0],
        ));
        scene.axes_widget = Some(AxesWidget::default());
        scene.camera.look_at([0.0, 0.5, 3.0], [0.0, 0.0, 0.0]);
        scene
    }

    /// Number of distinct RGB colours in an RGBA buffer — a cheap "is this a real
    /// render, or a flat fill?" check.
    pub fn distinct_colors(rgba: &[u8]) -> usize {
        let mut seen = std::collections::HashSet::new();
        for px in rgba.chunks_exact(4) {
            seen.insert((px[0], px[1], px[2]));
        }
        seen.len()
    }

    pub fn run() {
        println!("vtk-rs headless batch renderer");
        println!("==============================\n");

        let scene = build_scene();
        println!("Scene: {}", scene.summary());
        scene.print_info();
        println!();

        let width = 640u32;
        let height = 480u32;

        println!("Creating headless GPU context (no window, no surface)...");
        let mut renderer = match WgpuRenderer::new_headless_blocking(width, height) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Could not create a headless GPU context: {e}");
                eprintln!("(A working wgpu backend — Vulkan, Metal, DX12 or GL — is required.)");
                std::process::exit(1);
            }
        };
        println!(
            "GPU context ready, colour format {:?}",
            renderer.color_format()
        );

        println!("Rendering {width}x{height} offscreen...");
        let rgba = renderer
            .render_to_image(&scene, width, height)
            .expect("offscreen render failed");
        assert_eq!(rgba.len(), (width * height * 4) as usize);

        let n_colors = distinct_colors(&rgba);
        println!(
            "Read back {} bytes, {} distinct colours",
            rgba.len(),
            n_colors
        );
        assert!(
            n_colors > 16,
            "image looks like a flat fill ({n_colors} colours) — nothing was drawn"
        );

        // Save in multiple formats
        let dir = std::env::temp_dir().join("vtk_headless");
        let _ = std::fs::create_dir_all(&dir);

        let ppm_path = dir.join("render.ppm");
        screenshot::save_ppm(&ppm_path, &rgba, width, height).unwrap();
        println!("Saved: {}", ppm_path.display());

        let bmp_path = dir.join("render.bmp");
        screenshot::save_bmp(&bmp_path, &rgba, width, height).unwrap();
        println!("Saved: {}", bmp_path.display());

        let tga_path = dir.join("render.tga");
        screenshot::save_tga(&tga_path, &rgba, width, height).unwrap();
        println!("Saved: {}", tga_path.display());

        println!("\nDone. Images saved to {}", dir.display());
    }
}

#[cfg(feature = "render-wgpu")]
fn main() {
    app::run();
}

#[cfg(not(feature = "render-wgpu"))]
fn main() {
    println!("headless_render example requires --features render-wgpu");
}
