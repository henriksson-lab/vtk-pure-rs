//! Sector (pie-slice) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Parameters for sector generation.
pub struct SectorParams {
    /// Inner radius. Default: 1.0
    pub inner_radius: f64,
    /// Outer radius. Default: 2.0
    pub outer_radius: f64,
    /// Start angle in degrees. Default: 0.0
    pub start_angle: f64,
    /// End angle in degrees. Default: 90.0
    pub end_angle: f64,
    /// Number of radial subdivisions. Default: 1
    pub radial_resolution: usize,
    /// Number of circumferential subdivisions. Default: 6
    pub resolution: usize,
    /// Z coordinate (sectors are in XY plane). Default: 0.0
    pub z: f64,
}

impl Default for SectorParams {
    fn default() -> Self {
        Self {
            inner_radius: 1.0,
            outer_radius: 2.0,
            start_angle: 0.0,
            end_angle: 90.0,
            radial_resolution: 1,
            resolution: 6,
            z: 0.0,
        }
    }
}

/// Generate a sector in the XY plane, matching `vtkSectorSource`.
pub fn sector(params: &SectorParams) -> PolyData {
    let radial_resolution = params.radial_resolution.max(1);
    let circumferential_resolution = params.resolution.max(3);
    let inner_radius = params.inner_radius.max(0.0);
    let outer_radius = params.outer_radius.max(0.0);
    let z_coord = params.z.max(0.0);
    let start_angle = params.start_angle.max(0.0);
    let end_angle = params.end_angle.max(0.0);
    let angle = (end_angle - start_angle).to_radians();

    let mut points = Points::<f64>::new();
    let mut strips = CellArray::new();

    let num_line_points = radial_resolution + 1;
    for j in 0..=circumferential_resolution {
        let theta = start_angle.to_radians() + angle * j as f64 / circumferential_resolution as f64;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();
        for i in 0..=radial_resolution {
            let t = i as f64 / radial_resolution as f64;
            let radius = inner_radius + (outer_radius - inner_radius) * t;
            points.push([radius * cos_theta, radius * sin_theta, z_coord]);
        }
    }

    for i in 0..radial_resolution {
        let mut strip = Vec::with_capacity(2 * (circumferential_resolution + 1));
        for j in 0..=circumferential_resolution {
            let base = (j * num_line_points) as i64;
            strip.push(base + i as i64 + 1);
            strip.push(base + i as i64);
        }
        strips.push_cell(&strip);
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.strips = strips;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pie() {
        let s = sector(&SectorParams {
            inner_radius: 0.0,
            outer_radius: 1.0,
            start_angle: 0.0,
            end_angle: 360.0,
            resolution: 16,
            ..Default::default()
        });
        assert!(s.points.len() > 10);
        assert_eq!(s.strips.num_cells(), 1);
    }

    #[test]
    fn quarter_pie() {
        let s = sector(&SectorParams::default());
        assert!(s.points.len() > 3);
        assert!(s.strips.num_cells() > 0);
    }

    #[test]
    fn annular_sector() {
        let s = sector(&SectorParams {
            inner_radius: 0.5,
            outer_radius: 1.0,
            start_angle: 0.0,
            end_angle: 180.0,
            resolution: 8,
            ..Default::default()
        });
        assert!(s.points.len() > 0);
        assert!(s.strips.num_cells() > 0);

        // No point at origin
        for i in 0..s.points.len() {
            let p = s.points.get(i);
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!(r >= 0.49, "point too close to origin: r={r}");
        }
    }

    #[test]
    fn custom_z() {
        let s = sector(&SectorParams {
            z: 5.0,
            ..Default::default()
        });
        let p = s.points.get(0);
        assert_eq!(p[2], 5.0);
    }

    #[test]
    fn negative_z_is_clamped_like_vtk() {
        let s = sector(&SectorParams {
            z: -5.0,
            ..Default::default()
        });
        let p = s.points.get(0);
        assert_eq!(p[2], 0.0);
    }

    #[test]
    fn radial_resolution_adds_strips() {
        let s = sector(&SectorParams {
            radial_resolution: 3,
            ..Default::default()
        });
        assert_eq!(s.strips.num_cells(), 3);
    }
}
