use crate::data::PolyData;

/// Computed mass properties of a closed triangular surface.
#[derive(Debug, Clone)]
pub struct MassProperties {
    /// Total surface area.
    pub surface_area: f64,
    /// Enclosed volume (only meaningful for closed surfaces).
    pub volume: f64,
    /// Center of mass (centroid) of the enclosed volume.
    pub center: [f64; 3],
}

/// Compute surface area, volume, and centroid of a PolyData.
///
/// Uses the divergence theorem to compute volume from a closed triangular
/// surface. The mesh should be a closed, consistently-wound triangle mesh
/// for accurate volume and centroid results. Surface area works for any
/// triangle mesh.
pub fn mass_properties(input: &PolyData) -> MassProperties {
    let mut total_area = 0.0f64;
    let mut total_volume = 0.0f64;
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    let mut cz = 0.0f64;

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        // Fan triangulate for non-triangles
        let p0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let p1 = input.points.get(cell[i] as usize);
            let p2 = input.points.get(cell[i + 1] as usize);

            // Triangle area via cross product
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            total_area += area;

            // Signed volume contribution via divergence theorem:
            // V = (1/6) * sum_triangles (p0 . (p1 x p2))
            let vol_contrib = p0[0] * (p1[1] * p2[2] - p1[2] * p2[1])
                + p0[1] * (p1[2] * p2[0] - p1[0] * p2[2])
                + p0[2] * (p1[0] * p2[1] - p1[1] * p2[0]);
            total_volume += vol_contrib;

            // Centroid contribution
            // Using the formula from "Geometric Tools for Computer Graphics"
            let tri_center = [
                (p0[0] + p1[0] + p2[0]) / 3.0,
                (p0[1] + p1[1] + p2[1]) / 3.0,
                (p0[2] + p1[2] + p2[2]) / 3.0,
            ];
            cx += vol_contrib * tri_center[0];
            cy += vol_contrib * tri_center[1];
            cz += vol_contrib * tri_center[2];
        }
    }

    total_volume /= 6.0;
    let abs_vol = total_volume.abs();

    let center = if abs_vol > 1e-30 {
        let inv = 1.0 / (6.0 * total_volume * 4.0);
        [cx * inv, cy * inv, cz * inv]
    } else {
        [0.0, 0.0, 0.0]
    };

    MassProperties {
        surface_area: total_area,
        volume: abs_vol,
        center,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::core::sources;

    #[test]
    fn cube_properties() {
        let cube = sources::cube::cube(&Default::default());
        // Triangulate the cube first
        let tri_cube = crate::filters::core::triangulate::triangulate(&cube);
        let props = mass_properties(&tri_cube);

        // Unit cube: area = 6, volume = 1
        assert!(
            (props.surface_area - 6.0).abs() < 0.1,
            "area = {}",
            props.surface_area
        );
        assert!(
            (props.volume - 1.0).abs() < 0.1,
            "volume = {}",
            props.volume
        );
    }

    #[test]
    fn sphere_area() {
        let sphere = sources::sphere::sphere(&sources::sphere::SphereParams {
            radius: 1.0,
            theta_resolution: 32,
            phi_resolution: 32,
            ..Default::default()
        });
        let props = mass_properties(&sphere);

        // Sphere: area ≈ 4π ≈ 12.566, volume ≈ 4π/3 ≈ 4.189
        assert!(
            (props.surface_area - 4.0 * std::f64::consts::PI).abs() < 0.5,
            "sphere area = {}",
            props.surface_area
        );
        assert!(
            (props.volume - 4.0 * std::f64::consts::PI / 3.0).abs() < 0.5,
            "sphere volume = {}",
            props.volume
        );
    }

    #[test]
    fn single_triangle_area() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let props = mass_properties(&pd);
        assert!((props.surface_area - 0.5).abs() < 1e-10);
    }
}
