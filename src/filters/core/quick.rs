//! Quick convenience functions that don't require parameter structs.
//!
//! ```
//! use crate::filters::core::quick::*;
//!
//! let s = sphere();
//! assert!(s.points.len() > 0);
//! let c = cube();
//! assert!(c.points.len() > 0);
//! ```

use crate::data::PolyData;

/// Generate a default sphere (radius 0.5, 16x16 resolution).
pub fn sphere() -> PolyData {
    crate::filters::core::sources::sphere::sphere(
        &crate::filters::core::sources::sphere::SphereParams::default(),
    )
}

/// Generate a sphere with custom resolution.
pub fn sphere_res(theta: usize, phi: usize) -> PolyData {
    crate::filters::core::sources::sphere::sphere(
        &crate::filters::core::sources::sphere::SphereParams {
            theta_resolution: theta,
            phi_resolution: phi,
            ..Default::default()
        },
    )
}

/// Generate a default cube.
pub fn cube() -> PolyData {
    crate::filters::core::sources::cube::cube(
        &crate::filters::core::sources::cube::CubeParams::default(),
    )
}

/// Generate a default cone.
pub fn cone() -> PolyData {
    crate::filters::core::sources::cone::cone(
        &crate::filters::core::sources::cone::ConeParams::default(),
    )
}

/// Generate a default cylinder.
pub fn cylinder() -> PolyData {
    crate::filters::core::sources::cylinder::cylinder(
        &crate::filters::core::sources::cylinder::CylinderParams::default(),
    )
}

/// Generate a default arrow.
pub fn arrow() -> PolyData {
    crate::filters::core::sources::arrow::arrow(
        &crate::filters::core::sources::arrow::ArrowParams::default(),
    )
}

/// Generate a default plane.
pub fn plane() -> PolyData {
    crate::filters::core::sources::plane::plane(
        &crate::filters::core::sources::plane::PlaneParams::default(),
    )
}

/// Generate a sphere with normals already computed.
pub fn sphere_with_normals() -> PolyData {
    crate::filters::core::normals::compute_normals(&sphere())
}

/// Generate a sphere with normals and elevation scalars.
pub fn sphere_with_elevation() -> PolyData {
    crate::filters::core::elevation::elevation_z(&crate::filters::core::normals::compute_normals(
        &sphere(),
    ))
}

/// Generate an isosurface from an implicit function.
///
/// Samples the function on a grid and extracts the zero-level isosurface.
pub fn isosurface_from_implicit(
    func: &dyn crate::types::ImplicitFunction,
    bounds: crate::types::BoundingBox,
    resolution: usize,
) -> PolyData {
    let dims = [resolution, resolution, resolution];
    let spacing = [
        bounds.size()[0] / (resolution - 1) as f64,
        bounds.size()[1] / (resolution - 1) as f64,
        bounds.size()[2] / (resolution - 1) as f64,
    ];
    let origin = [bounds.x_min, bounds.y_min, bounds.z_min];

    let img = crate::filters::points::sample_implicit::sample_implicit_function(
        dims, spacing, origin, "implicit", func,
    );

    let scalars: Vec<f64> = img.point_data().scalars().unwrap().to_f64_vec();
    crate::filters::core::marching_cubes::marching_cubes(&img, &scalars, 0.0)
}

/// Generate random points in a box.
pub fn random_points(n: usize, bounds: crate::types::BoundingBox) -> PolyData {
    let size = bounds.size();
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        // Simple deterministic pseudo-random using hash
        let h1 =
            ((i as u64).wrapping_mul(2654435761u64) & 0xFFFFFFFF) as f64 / 0xFFFFFFFFu64 as f64;
        let h2 = ((i as u64).wrapping_mul(2246822519u64).wrapping_add(1) & 0xFFFFFFFF) as f64
            / 0xFFFFFFFFu64 as f64;
        let h3 = ((i as u64).wrapping_mul(3266489917u64).wrapping_add(2) & 0xFFFFFFFF) as f64
            / 0xFFFFFFFFu64 as f64;
        pts.push([
            bounds.x_min + h1 * size[0],
            bounds.y_min + h2 * size[1],
            bounds.z_min + h3 * size[2],
        ]);
    }
    PolyData::from_vertices(pts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_sources() {
        assert!(sphere().points.len() > 0);
        assert!(cube().points.len() > 0);
        assert!(cone().points.len() > 0);
        assert!(cylinder().points.len() > 0);
        assert!(arrow().points.len() > 0);
        assert!(plane().points.len() > 0);
    }

    #[test]
    fn sphere_with_data() {
        let s = sphere_with_normals();
        assert!(s.point_data().normals().is_some());

        let s = sphere_with_elevation();
        assert!(s.point_data().scalars().is_some());
    }

    #[test]
    fn sphere_custom_res() {
        let s = sphere_res(8, 8);
        let s2 = sphere_res(32, 32);
        assert!(s2.points.len() > s.points.len());
    }

    #[test]
    fn isosurface() {
        let sphere_fn = crate::types::ImplicitSphere::new([0.0, 0.0, 0.0], 0.5);
        let bb = crate::types::BoundingBox::from_corners([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let iso = isosurface_from_implicit(&sphere_fn, bb, 16);
        assert!(iso.points.len() > 0);
        assert!(iso.polys.num_cells() > 0);
    }

    #[test]
    fn random_point_cloud() {
        let bb = crate::types::BoundingBox::from_corners([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let pd = random_points(100, bb);
        assert_eq!(pd.points.len(), 100);
        assert_eq!(pd.verts.num_cells(), 100);
    }
}
