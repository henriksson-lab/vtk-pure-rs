use std::hint::black_box;

use vtk_pure_rs::data::ImageData;
use vtk_pure_rs::data::PolyData;
use vtk_pure_rs::filters::core::sources::sphere::{sphere, SphereParams};
use vtk_pure_rs::filters::geometry::clean::{clean, CleanParams};

fn make_sphere(resolution: usize) -> PolyData {
    sphere(&SphereParams {
        theta_resolution: resolution,
        phi_resolution: resolution,
        ..SphereParams::default()
    })
}

fn make_image_data(size: usize) -> (ImageData, Vec<f64>) {
    let mut img = ImageData::with_dimensions(size, size, size);
    let sp = 1.0 / size as f64;
    img.set_spacing([sp, sp, sp]);
    let n = size * size * size;
    let mut scalars = Vec::with_capacity(n);
    for k in 0..size {
        for j in 0..size {
            for i in 0..size {
                let x = i as f64 / size as f64 - 0.5;
                let y = j as f64 / size as f64 - 0.5;
                let z = k as f64 / size as f64 - 0.5;
                scalars.push((x * x + y * y + z * z).sqrt() - 0.3);
            }
        }
    }
    (img, scalars)
}

fn main() {
    println!("vtk-rs filter benchmarks");
    println!("========================\n");

    bench("sphere(16) generation", 100, || {
        black_box(make_sphere(16));
    });

    bench("sphere(64) generation", 10, || {
        black_box(make_sphere(64));
    });

    let pd16 = make_sphere(16);
    let pd32 = make_sphere(32);
    let pd64 = make_sphere(64);

    bench("normals (sphere 16)", 100, || {
        black_box(vtk_pure_rs::filters::normals::normals::compute_normals(
            &pd16,
        ));
    });

    bench("normals (sphere 64)", 10, || {
        black_box(vtk_pure_rs::filters::normals::normals::compute_normals(
            &pd64,
        ));
    });

    bench("normals_par (sphere 64)", 10, || {
        black_box(vtk_pure_rs::filters::normals::normals::compute_normals_par(
            &pd64,
        ));
    });

    bench("elevation (sphere 64)", 20, || {
        black_box(vtk_pure_rs::filters::geometry::elevation::elevation(
            &pd64,
            [0.0, 0.0, -1.0],
            [0.0, 0.0, 1.0],
        ));
    });

    bench("elevation_par (sphere 64)", 20, || {
        black_box(vtk_pure_rs::filters::geometry::elevation::elevation_par(
            &pd64,
            [0.0, 0.0, -1.0],
            [0.0, 0.0, 1.0],
        ));
    });

    let (img32, scalars32) = make_image_data(32);
    bench("marching_cubes (32^3)", 10, || {
        black_box(vtk_pure_rs::filters::core::marching_cubes::marching_cubes(
            &img32, &scalars32, 0.0,
        ));
    });

    bench("triangulate (sphere 32)", 50, || {
        black_box(vtk_pure_rs::filters::geometry::triangulate::triangulate(
            &pd32,
        ));
    });

    bench("clean (sphere 32)", 20, || {
        black_box(clean(&pd32, &CleanParams::default()));
    });

    bench("decimate 50% (sphere 32)", 10, || {
        black_box(vtk_pure_rs::filters::core::decimate::decimate(&pd32, 0.5));
    });

    bench("smooth 20 iters (sphere 32)", 10, || {
        black_box(vtk_pure_rs::filters::smooth::smooth::smooth(
            &pd32, 20, 1.0, true,
        ));
    });
}

fn bench<F: FnMut()>(name: &str, iterations: u32, mut f: F) {
    // Warm up
    f();

    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;

    let per_str = if per_iter.as_millis() > 0 {
        format!("{:.2} ms", per_iter.as_secs_f64() * 1000.0)
    } else {
        format!("{:.1} us", per_iter.as_secs_f64() * 1_000_000.0)
    };

    println!("  {:<35} {:>10} ({} iters)", name, per_str, iterations);
}
