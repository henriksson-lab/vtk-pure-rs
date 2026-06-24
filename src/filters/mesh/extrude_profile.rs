//! Extrude a 2D profile along a path to create a sweep surface.

use crate::data::{CellArray, Points, PolyData};

/// Sweep a 2D profile along a 3D polyline path.
///
/// The profile is defined as 2D points in a local frame perpendicular to the path.
pub fn sweep_profile_along_path(
    profile: &[[f64; 2]],
    path: &[[f64; 3]],
    close_profile: bool,
) -> PolyData {
    let np = profile.len();
    let nl = path.len();
    if np < 2 || nl < 2 {
        return PolyData::new();
    }

    let mut points = Points::<f64>::new();
    let mut polys = CellArray::new();

    for i in 0..nl {
        // Compute local frame at path point
        let tangent = if i + 1 < nl {
            let d = [
                path[i + 1][0] - path[i][0],
                path[i + 1][1] - path[i][1],
                path[i + 1][2] - path[i][2],
            ];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
            [d[0] / len, d[1] / len, d[2] / len]
        } else {
            let d = [
                path[i][0] - path[i - 1][0],
                path[i][1] - path[i - 1][1],
                path[i][2] - path[i - 1][2],
            ];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-15);
            [d[0] / len, d[1] / len, d[2] / len]
        };

        let up = if tangent[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = cross(tangent, up);
        let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt().max(1e-15);
        let u = [u[0] / ul, u[1] / ul, u[2] / ul];
        let v = cross(tangent, u);

        for j in 0..np {
            let px = path[i][0] + profile[j][0] * u[0] + profile[j][1] * v[0];
            let py = path[i][1] + profile[j][0] * u[1] + profile[j][1] * v[1];
            let pz = path[i][2] + profile[j][0] * u[2] + profile[j][1] * v[2];
            points.push([px, py, pz]);
        }
    }

    // Connect adjacent profile rings
    let ring_size = if close_profile { np } else { np - 1 };
    for i in 0..nl - 1 {
        for j in 0..ring_size {
            let j_next = (j + 1) % np;
            let p0 = (i * np + j) as i64;
            let p1 = (i * np + j_next) as i64;
            let p2 = ((i + 1) * np + j_next) as i64;
            let p3 = ((i + 1) * np + j) as i64;
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p3]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh
}

/// Sweep a circle along a path (tube).
pub fn sweep_circle_along_path(path: &[[f64; 3]], radius: f64, sides: usize) -> PolyData {
    let n = sides.max(3);
    let profile: Vec<[f64; 2]> = (0..n)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect();
    sweep_profile_along_path(&profile, path, true)
}

/// Sweep a rectangle along a path (ribbon).
pub fn sweep_rect_along_path(path: &[[f64; 3]], width: f64, height: f64) -> PolyData {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let profile = [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]];
    sweep_profile_along_path(&profile, path, true)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn circle_sweep() {
        let path: Vec<[f64; 3]> = (0..10).map(|i| [0.0, 0.0, i as f64]).collect();
        let result = sweep_circle_along_path(&path, 0.5, 8);
        assert!(result.points.len() > 50);
        assert!(result.polys.num_cells() > 50);
    }
    #[test]
    fn rect_sweep() {
        let path = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]];
        let result = sweep_rect_along_path(&path, 0.2, 0.1);
        assert!(result.polys.num_cells() > 0);
    }
    #[test]
    fn custom_profile() {
        let profile = vec![[0.0, 0.0], [0.5, 0.0], [0.5, 0.3], [0.0, 0.3]]; // L-shape
        let path: Vec<[f64; 3]> = (0..5).map(|i| [0.0, 0.0, i as f64 * 0.5]).collect();
        let result = sweep_profile_along_path(&profile, &path, true);
        assert!(result.polys.num_cells() > 0);
    }
}
