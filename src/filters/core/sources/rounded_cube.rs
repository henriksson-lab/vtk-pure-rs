//! Rounded cube (superellipsoid approximation) geometry source.

use crate::data::{CellArray, Points, PolyData};

/// Create a rounded cube by projecting a sphere onto a rounded-corner shape.
pub fn rounded_cube(size: f64, corner_radius: f64, resolution: usize) -> PolyData {
    let res = resolution.max(3);
    let half = size.abs() * 0.5;
    let r = corner_radius.clamp(0.0, half);
    let inner = half - r;

    let mut pts = Points::<f64>::new();
    let mut polys = CellArray::new();

    // Use spherical coordinates and project
    for iv in 0..=res {
        let v = std::f64::consts::PI * iv as f64 / res as f64;
        for iu in 0..res {
            let u = 2.0 * std::f64::consts::PI * iu as f64 / res as f64;
            let sx = v.sin() * u.cos();
            let sy = v.sin() * u.sin();
            let sz = v.cos();
            let max_component = sx.abs().max(sy.abs()).max(sz.abs());
            let scale = if max_component > 0.0 {
                inner / max_component + r
            } else {
                r
            };
            let x = sx * scale;
            let y = sy * scale;
            let z = sz * scale;
            pts.push([x, y, z]);
        }
    }

    for iv in 0..res {
        for iu in 0..res {
            let i00 = iv * res + iu;
            let i10 = iv * res + (iu + 1) % res;
            let i01 = (iv + 1) * res + iu;
            let i11 = (iv + 1) * res + (iu + 1) % res;
            polys.push_cell(&[i00 as i64, i10 as i64, i11 as i64, i01 as i64]);
        }
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.polys = polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rounded() {
        let rc = rounded_cube(2.0, 0.3, 8);
        assert_eq!(rc.points.len(), 72); // 9 * 8
        assert_eq!(rc.polys.num_cells(), 64); // 8 * 8
    }
    #[test]
    fn test_sphere_like() {
        // When corner_radius == half_size, should be sphere-like
        let rc = rounded_cube(2.0, 1.0, 16);
        assert_eq!(rc.points.len(), 272);
    }
}
