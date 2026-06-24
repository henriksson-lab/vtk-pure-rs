//! Surface of revolution from a 2D profile curve.

use crate::data::{CellArray, Points, PolyData};

/// Create a surface of revolution by rotating a profile curve around Z axis.
pub fn surface_of_revolution(profile: &[[f64; 2]], resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let n = profile.len();
    if n < 2 {
        return PolyData::new();
    }

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    for iu in 0..res {
        let angle = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
        let c = angle.cos();
        let s = angle.sin();
        for p in profile {
            pts.push([p[0] * c, p[0] * s, p[1]]);
        }
    }

    for iu in 0..res {
        let iu1 = (iu + 1) % res;
        for iv in 0..n - 1 {
            let i00 = (iu * n + iv) as i64;
            let i10 = (iu * n + iv + 1) as i64;
            let i01 = (iu1 * n + iv) as i64;
            let i11 = (iu1 * n + iv + 1) as i64;
            polys.push_cell(&[i00, i10, i11, i01]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

/// Create a vase/goblet shape from a simple profile.
pub fn vase(
    height: f64,
    base_radius: f64,
    rim_radius: f64,
    waist_radius: f64,
    resolution: usize,
) -> PolyData {
    let steps = 20;
    let profile: Vec<[f64; 2]> = (0..=steps)
        .map(|i| {
            let t = i as f64 / steps as f64;
            let z = t * height;
            let r = if t < 0.3 {
                base_radius + (waist_radius - base_radius) * (t / 0.3)
            } else if t < 0.7 {
                let s = (t - 0.3) / 0.4;
                waist_radius + (rim_radius - waist_radius) * s * s
            } else {
                rim_radius
            };
            [r, z]
        })
        .collect();
    surface_of_revolution(&profile, resolution)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cylinder_profile() {
        let profile = vec![[1.0, 0.0], [1.0, 1.0], [1.0, 2.0]];
        let s = surface_of_revolution(&profile, 12);
        assert_eq!(s.points.len(), 36); // 12 * 3
        assert_eq!(s.polys.num_cells(), 24); // 12 * 2
    }
    #[test]
    fn test_vase() {
        let v = vase(5.0, 1.0, 1.5, 0.5, 16);
        assert!(v.points.len() > 0);
        assert!(v.polys.num_cells() > 0);
    }
    #[test]
    fn test_cone_profile() {
        let profile = vec![[1.0, 0.0], [0.0, 2.0]];
        let s = surface_of_revolution(&profile, 8);
        assert_eq!(s.points.len(), 16);
        assert_eq!(s.polys.num_cells(), 8);
    }
}
