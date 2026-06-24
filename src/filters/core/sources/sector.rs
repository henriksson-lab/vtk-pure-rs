//! Sector (pie-slice) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Parameters for sector generation.
pub struct SectorParams {
    /// Inner radius. Default: 0.0 (filled sector)
    pub inner_radius: f64,
    /// Outer radius. Default: 1.0
    pub outer_radius: f64,
    /// Start angle in degrees. Default: 0.0
    pub start_angle: f64,
    /// End angle in degrees. Default: 90.0
    pub end_angle: f64,
    /// Number of angular subdivisions. Default: 32
    pub resolution: usize,
    /// Z coordinate (sectors are in XY plane). Default: 0.0
    pub z: f64,
}

impl Default for SectorParams {
    fn default() -> Self {
        Self {
            inner_radius: 0.0,
            outer_radius: 1.0,
            start_angle: 0.0,
            end_angle: 90.0,
            resolution: 32,
            z: 0.0,
        }
    }
}

/// Generate a sector (pie-slice or annular sector) in the XY plane.
///
/// If `inner_radius` is 0, generates a filled pie slice.
/// If `inner_radius` > 0, generates an annular sector (ring segment).
pub fn sector(params: &SectorParams) -> PolyData {
    let n = params.resolution.max(2);
    let a0 = params.start_angle.to_radians();
    let a1 = params.end_angle.to_radians();

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    if params.inner_radius < 1e-15 {
        // Filled sector: center + outer arc
        points.push([0.0, 0.0, params.z]); // center = index 0

        for i in 0..=n {
            let t = i as f64 / n as f64;
            let angle = a0 + t * (a1 - a0);
            let x = params.outer_radius * angle.cos();
            let y = params.outer_radius * angle.sin();
            points.push([x, y, params.z]);
        }

        // Fan triangulation from center
        for i in 0..n {
            let p1 = (i + 1) as i64;
            let p2 = (i + 2) as i64;
            polys.push_cell(&[0, p1, p2]);
        }
    } else {
        // Annular sector: inner arc + outer arc
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let angle = a0 + t * (a1 - a0);
            let xi = params.inner_radius * angle.cos();
            let yi = params.inner_radius * angle.sin();
            let xo = params.outer_radius * angle.cos();
            let yo = params.outer_radius * angle.sin();
            points.push([xi, yi, params.z]); // inner point
            points.push([xo, yo, params.z]); // outer point
        }

        // Quad strip between inner and outer arcs
        for i in 0..n {
            let i0 = (i * 2) as i64; // inner current
            let o0 = (i * 2 + 1) as i64; // outer current
            let i1 = (i * 2 + 2) as i64; // inner next
            let o1 = (i * 2 + 3) as i64; // outer next
            polys.push_cell(&[i0, o0, o1]);
            polys.push_cell(&[i0, o1, i1]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_pie() {
        let s = sector(&SectorParams {
            start_angle: 0.0,
            end_angle: 360.0,
            resolution: 16,
            ..Default::default()
        });
        assert!(s.points.len() > 10);
        assert_eq!(s.polys.num_cells(), 16);
    }

    #[test]
    fn quarter_pie() {
        let s = sector(&SectorParams::default());
        assert!(s.points.len() > 3);
        assert!(s.polys.num_cells() > 0);
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
        assert!(s.polys.num_cells() > 0);

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
}
