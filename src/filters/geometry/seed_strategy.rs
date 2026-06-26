/// Seed point generation strategies for streamline tracing.

/// Generate seed points along a line segment.
pub fn seed_line(start: [f64; 3], end: [f64; 3], num_seeds: usize) -> Vec<[f64; 3]> {
    if num_seeds == 0 {
        return Vec::new();
    }
    if num_seeds == 1 {
        return vec![[
            (start[0] + end[0]) / 2.0,
            (start[1] + end[1]) / 2.0,
            (start[2] + end[2]) / 2.0,
        ]];
    }
    (0..num_seeds)
        .map(|i| {
            let t = i as f64 / (num_seeds - 1) as f64;
            [
                start[0] + t * (end[0] - start[0]),
                start[1] + t * (end[1] - start[1]),
                start[2] + t * (end[2] - start[2]),
            ]
        })
        .collect()
}

/// Generate seed points on a plane (grid of points).
pub fn seed_plane(
    origin: [f64; 3],
    axis1: [f64; 3],
    axis2: [f64; 3],
    n1: usize,
    n2: usize,
) -> Vec<[f64; 3]> {
    if n1 == 0 || n2 == 0 {
        return Vec::new();
    }

    let mut seeds = Vec::with_capacity(n1 * n2);
    for j in 0..n2 {
        for i in 0..n1 {
            let u = if n1 > 1 {
                i as f64 / (n1 - 1) as f64
            } else {
                0.5
            };
            let v = if n2 > 1 {
                j as f64 / (n2 - 1) as f64
            } else {
                0.5
            };
            seeds.push([
                origin[0] + u * axis1[0] + v * axis2[0],
                origin[1] + u * axis1[1] + v * axis2[1],
                origin[2] + u * axis1[2] + v * axis2[2],
            ]);
        }
    }
    seeds
}

/// Generate seed points on a sphere surface.
pub fn seed_sphere(center: [f64; 3], radius: f64, n_theta: usize, n_phi: usize) -> Vec<[f64; 3]> {
    if n_theta == 0 || n_phi == 0 {
        return Vec::new();
    }

    let mut seeds = Vec::with_capacity(n_theta * n_phi);
    for j in 0..n_phi {
        let phi = std::f64::consts::PI * (j as f64 + 0.5) / n_phi as f64;
        for i in 0..n_theta {
            let theta = 2.0 * std::f64::consts::PI * i as f64 / n_theta as f64;
            seeds.push([
                center[0] + radius * phi.sin() * theta.cos(),
                center[1] + radius * phi.sin() * theta.sin(),
                center[2] + radius * phi.cos(),
            ]);
        }
    }
    seeds
}

/// Generate seed points in a circle (ring).
pub fn seed_circle(
    center: [f64; 3],
    radius: f64,
    normal: [f64; 3],
    num_seeds: usize,
) -> Vec<[f64; 3]> {
    if num_seeds == 0 {
        return Vec::new();
    }

    // Build orthonormal basis
    let n = normalize(normal);
    let up = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let u = normalize(cross(n, up));
    let v = cross(n, u);

    (0..num_seeds)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / num_seeds as f64;
            [
                center[0] + radius * (angle.cos() * u[0] + angle.sin() * v[0]),
                center[1] + radius * (angle.cos() * u[1] + angle.sin() * v[1]),
                center[2] + radius * (angle.cos() * u[2] + angle.sin() * v[2]),
            ]
        })
        .collect()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-15 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
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
    fn line_seeds() {
        let seeds = seed_line([0.0; 3], [10.0, 0.0, 0.0], 5);
        assert_eq!(seeds.len(), 5);
        assert!((seeds[0][0]).abs() < 1e-10);
        assert!((seeds[4][0] - 10.0).abs() < 1e-10);
        assert!((seeds[2][0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn plane_seeds() {
        let seeds = seed_plane([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3, 3);
        assert_eq!(seeds.len(), 9);
    }

    #[test]
    fn sphere_seeds() {
        let seeds = seed_sphere([0.0; 3], 1.0, 8, 4);
        assert_eq!(seeds.len(), 32);
        // All should be approximately on the unit sphere
        for s in &seeds {
            let r = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn circle_seeds() {
        let seeds = seed_circle([0.0; 3], 2.0, [0.0, 0.0, 1.0], 8);
        assert_eq!(seeds.len(), 8);
        for s in &seeds {
            let r = (s[0] * s[0] + s[1] * s[1]).sqrt();
            assert!((r - 2.0).abs() < 1e-6);
            assert!(s[2].abs() < 1e-10);
        }
    }
}
