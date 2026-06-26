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

/// Compute surface area, volume, and centroid of a triangular PolyData.
///
/// Uses the divergence theorem to compute volume from a closed triangular
/// surface. The mesh should be a closed, consistently-wound triangle mesh
/// for accurate volume and centroid results. Non-triangle polygon cells are
/// ignored, matching vtkMassProperties.
pub fn mass_properties(input: &PolyData) -> MassProperties {
    let mut total_area = 0.0f64;
    let mut vol = [0.0f64; 3];
    let mut munc = [0.0f64; 3];
    let mut wxyz = 0.0f64;
    let mut wxy = 0.0f64;
    let mut wxz = 0.0f64;
    let mut wyz = 0.0f64;
    let mut centroid_volume = 0.0f64;
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    let mut cz = 0.0f64;
    let num_cells = input.polys.num_cells();

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }

        let p0 = input.points.get(cell[0] as usize);
        let p1 = input.points.get(cell[1] as usize);
        let p2 = input.points.get(cell[2] as usize);

        let x = [p0[0], p1[0], p2[0]];
        let y = [p0[1], p1[1], p2[1]];
        let z = [p0[2], p1[2], p2[2]];

        let i = [x[1] - x[0], x[2] - x[0], x[2] - x[1]];
        let j = [y[1] - y[0], y[2] - y[0], y[2] - y[1]];
        let k = [z[1] - z[0], z[2] - z[0], z[2] - z[1]];

        let mut u = [
            j[0] * k[1] - k[0] * j[1],
            k[0] * i[1] - i[0] * k[1],
            i[0] * j[1] - j[0] * i[1],
        ];
        let length = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
        if length != 0.0 {
            u[0] /= length;
            u[1] /= length;
            u[2] /= length;
        } else {
            u = [0.0; 3];
        }

        let absu = [u[0].abs(), u[1].abs(), u[2].abs()];
        if absu[0] > absu[1] && absu[0] > absu[2] {
            munc[0] += 1.0;
        } else if absu[1] > absu[0] && absu[1] > absu[2] {
            munc[1] += 1.0;
        } else if absu[2] > absu[0] && absu[2] > absu[1] {
            munc[2] += 1.0;
        } else if absu[0] == absu[1] && absu[0] == absu[2] {
            wxyz += 1.0;
        } else if absu[0] == absu[1] && absu[0] > absu[2] {
            wxy += 1.0;
        } else if absu[0] == absu[2] && absu[0] > absu[1] {
            wxz += 1.0;
        } else if absu[1] == absu[2] && absu[0] < absu[2] {
            wyz += 1.0;
        }

        let a = (i[1] * i[1] + j[1] * j[1] + k[1] * k[1]).sqrt();
        let b = (i[0] * i[0] + j[0] * j[0] + k[0] * k[0]).sqrt();
        let c = (i[2] * i[2] + j[2] * j[2] + k[2] * k[2]).sqrt();
        let s = 0.5 * (a + b + c);
        let area = (s * (s - a) * (s - b) * (s - c)).abs().sqrt();
        total_area += area;

        let xavg = (x[0] + x[1] + x[2]) / 3.0;
        let yavg = (y[0] + y[1] + y[2]) / 3.0;
        let zavg = (z[0] + z[1] + z[2]) / 3.0;
        vol[0] += area * u[0] * xavg;
        vol[1] += area * u[1] * yavg;
        vol[2] += area * u[2] * zavg;

        let vol_contrib = p0[0] * (p1[1] * p2[2] - p1[2] * p2[1])
            + p0[1] * (p1[2] * p2[0] - p1[0] * p2[2])
            + p0[2] * (p1[0] * p2[1] - p1[1] * p2[0]);
        centroid_volume += vol_contrib;

        let tet_center = [
            (p0[0] + p1[0] + p2[0]) / 4.0,
            (p0[1] + p1[1] + p2[1]) / 4.0,
            (p0[2] + p1[2] + p2[2]) / 4.0,
        ];
        cx += vol_contrib * tet_center[0];
        cy += vol_contrib * tet_center[1];
        cz += vol_contrib * tet_center[2];
    }

    let kxyz = if num_cells > 0 {
        let n = num_cells as f64;
        [
            (munc[0] + (wxyz / 3.0) + ((wxy + wxz) / 2.0)) / n,
            (munc[1] + (wxyz / 3.0) + ((wxy + wyz) / 2.0)) / n,
            (munc[2] + (wxyz / 3.0) + ((wxz + wyz) / 2.0)) / n,
        ]
    } else {
        [0.0; 3]
    };
    let abs_vol = (kxyz[0] * vol[0] + kxyz[1] * vol[1] + kxyz[2] * vol[2]).abs();
    let centroid_volume = centroid_volume / 6.0;

    let center = if centroid_volume.abs() > 1e-30 {
        let inv = 1.0 / (6.0 * centroid_volume);
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
