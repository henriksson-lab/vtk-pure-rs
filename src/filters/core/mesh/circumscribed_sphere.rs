use crate::data::PolyData;

/// Compute an approximate bounding sphere of a mesh using VTK's
/// `vtkSphere::ComputeBoundingSphere` point algorithm.
///
/// Returns `(radius, center)`.
pub fn bounding_sphere(input: &PolyData) -> (f64, [f64; 3]) {
    let n: usize = input.points.len();
    if n == 0 {
        return (0.0, [0.0, 0.0, 0.0]);
    }
    if n == 1 {
        return (0.0, input.points.get(0));
    }

    let mut x_min = [f64::MAX; 3];
    let mut x_max = [-f64::MAX; 3];
    let mut y_min = [f64::MAX; 3];
    let mut y_max = [-f64::MAX; 3];
    let mut z_min = [f64::MAX; 3];
    let mut z_max = [-f64::MAX; 3];

    for i in 0..n {
        let p = input.points.get(i);
        if p[0] < x_min[0] {
            x_min = p;
        }
        if p[0] > x_max[0] {
            x_max = p;
        }
        if p[1] < y_min[1] {
            y_min = p;
        }
        if p[1] > y_max[1] {
            y_max = p;
        }
        if p[2] < z_min[2] {
            z_min = p;
        }
        if p[2] > z_max[2] {
            z_max = p;
        }
    }

    let x_span = dist_sq(&x_min, &x_max);
    let y_span = dist_sq(&y_min, &y_max);
    let z_span = dist_sq(&z_min, &z_max);

    let (a, b) = if x_span > y_span {
        if x_span > z_span {
            (x_min, x_max)
        } else {
            (z_min, z_max)
        }
    } else if y_span > z_span {
        (y_min, y_max)
    } else {
        (z_min, z_max)
    };

    let mut cx: f64 = (a[0] + b[0]) * 0.5;
    let mut cy: f64 = (a[1] + b[1]) * 0.5;
    let mut cz: f64 = (a[2] + b[2]) * 0.5;
    let mut radius: f64 = dist_sq(&a, &b).sqrt() * 0.5;
    let mut radius_sq = radius * radius;

    // Make a single VTK-style pass over the points and grow the sphere as needed.
    for i in 0..n {
        let p = input.points.get(i);
        let dx: f64 = p[0] - cx;
        let dy: f64 = p[1] - cy;
        let dz: f64 = p[2] - cz;
        let dist_sq = dx * dx + dy * dy + dz * dz;

        if dist_sq > radius_sq {
            let dist = dist_sq.sqrt();
            radius = (radius + dist) * 0.5;
            radius_sq = radius * radius;
            let delta = dist - radius;
            cx = (radius * cx + delta * p[0]) / dist;
            cy = (radius * cy + delta * p[1]) / dist;
            cz = (radius * cz + delta * p[2]) / dist;
        }
    }

    (radius, [cx, cy, cz])
}

fn dist_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx: f64 = a[0] - b[0];
    let dy: f64 = a[1] - b[1];
    let dz: f64 = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_point() {
        let mut pd = PolyData::new();
        pd.points.push([3.0, 4.0, 5.0]);
        let (r, c) = bounding_sphere(&pd);
        assert_eq!(r, 0.0);
        assert_eq!(c, [3.0, 4.0, 5.0]);
    }

    #[test]
    fn two_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        let (r, c) = bounding_sphere(&pd);
        assert!((r - 3.0).abs() < 1e-10, "r={}", r);
        assert!((c[0] - 3.0).abs() < 1e-10);
        assert!(c[1].abs() < 1e-10);
        assert!(c[2].abs() < 1e-10);
    }

    #[test]
    fn cube_vertices() {
        let mut pd = PolyData::new();
        for &x in &[-1.0f64, 1.0] {
            for &y in &[-1.0f64, 1.0] {
                for &z in &[-1.0f64, 1.0] {
                    pd.points.push([x, y, z]);
                }
            }
        }
        let (r, c) = bounding_sphere(&pd);
        // Circumscribed sphere of unit cube centered at origin: r = sqrt(3)
        let expected: f64 = 3.0f64.sqrt();
        assert!((r - expected).abs() < 0.1, "r={}, expected={}", r, expected);
        // Center should be near origin
        assert!(c[0].abs() < 0.1);
        assert!(c[1].abs() < 0.1);
        assert!(c[2].abs() < 0.1);
    }
}
