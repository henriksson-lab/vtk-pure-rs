//! Lightweight 3D math utilities (no external dependency).

/// Dot product of two 3D vectors.
pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product of two 3D vectors.
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Vector length (magnitude).
pub fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Normalize a vector to unit length.
pub fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = length(v);
    if len < 1e-15 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Distance between two points.
pub fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    length(sub(b, a))
}

/// Add two vectors.
pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Subtract b from a.
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Scale a vector by a scalar.
pub fn scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Linear interpolation between two points.
pub fn lerp(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

/// Angle between two vectors in radians.
pub fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = dot(a, b);
    let la = length(a);
    let lb = length(b);
    if la < 1e-15 || lb < 1e-15 {
        return 0.0;
    }
    (d / (la * lb)).clamp(-1.0, 1.0).acos()
}

/// Project vector a onto vector b.
pub fn project(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    let b_len2 = dot(b, b);
    if b_len2 < 1e-15 {
        return [0.0; 3];
    }
    scale(b, dot(a, b) / b_len2)
}

/// Reflect vector v about normal n.
pub fn reflect(v: [f64; 3], n: [f64; 3]) -> [f64; 3] {
    let d = 2.0 * dot(v, n);
    sub(v, scale(n, d))
}

/// Clamp a value to [min, max].
pub fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.clamp(min, max)
}

// --- Coordinate transforms ---

/// Cartesian (x, y, z) to spherical (r, theta, phi).
/// theta = polar angle from +Z [0, pi], phi = azimuthal angle from +X [0, 2pi].
pub fn cartesian_to_spherical(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let r = (x * x + y * y + z * z).sqrt();
    if r < 1e-15 {
        return (0.0, 0.0, 0.0);
    }
    let theta = (z / r).clamp(-1.0, 1.0).acos();
    let phi = y.atan2(x);
    let phi = if phi < 0.0 {
        phi + 2.0 * std::f64::consts::PI
    } else {
        phi
    };
    (r, theta, phi)
}

/// Spherical (r, theta, phi) to cartesian (x, y, z).
pub fn spherical_to_cartesian(r: f64, theta: f64, phi: f64) -> (f64, f64, f64) {
    (
        r * theta.sin() * phi.cos(),
        r * theta.sin() * phi.sin(),
        r * theta.cos(),
    )
}

/// Cartesian (x, y, z) to cylindrical (r, theta, z).
pub fn cartesian_to_cylindrical(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let r = (x * x + y * y).sqrt();
    let theta = y.atan2(x);
    let theta = if theta < 0.0 {
        theta + 2.0 * std::f64::consts::PI
    } else {
        theta
    };
    (r, theta, z)
}

/// Cylindrical (r, theta, z) to cartesian (x, y, z).
pub fn cylindrical_to_cartesian(r: f64, theta: f64, z: f64) -> (f64, f64, f64) {
    (r * theta.cos(), r * theta.sin(), z)
}

// --- Interpolation ---

/// Bilinear interpolation on a 2D grid.
/// Values at corners: v00 (bottom-left), v10 (bottom-right), v01 (top-left), v11 (top-right).
pub fn bilinear(v00: f64, v10: f64, v01: f64, v11: f64, u: f64, v: f64) -> f64 {
    let a = v00 * (1.0 - u) + v10 * u;
    let b = v01 * (1.0 - u) + v11 * u;
    a * (1.0 - v) + b * v
}

/// Trilinear interpolation on a 3D grid.
pub fn trilinear(
    v000: f64,
    v100: f64,
    v010: f64,
    v110: f64,
    v001: f64,
    v101: f64,
    v011: f64,
    v111: f64,
    u: f64,
    v: f64,
    w: f64,
) -> f64 {
    let a = bilinear(v000, v100, v010, v110, u, v);
    let b = bilinear(v001, v101, v011, v111, u, v);
    a * (1.0 - w) + b * w
}

/// Smooth Hermite interpolation (smoothstep).
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

// --- Noise ---

/// Simple hash-based value noise in 3D (deterministic, no external deps).
pub fn value_noise_3d(x: f64, y: f64, z: f64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let fz = z - z.floor();

    let sx = smoothstep(0.0, 1.0, fx);
    let sy = smoothstep(0.0, 1.0, fy);
    let sz = smoothstep(0.0, 1.0, fz);

    trilinear(
        hash3(ix, iy, iz),
        hash3(ix + 1, iy, iz),
        hash3(ix, iy + 1, iz),
        hash3(ix + 1, iy + 1, iz),
        hash3(ix, iy, iz + 1),
        hash3(ix + 1, iy, iz + 1),
        hash3(ix, iy + 1, iz + 1),
        hash3(ix + 1, iy + 1, iz + 1),
        sx,
        sy,
        sz,
    )
}

/// Fractional Brownian motion (multi-octave noise).
pub fn fbm_3d(x: f64, y: f64, z: f64, octaves: usize) -> f64 {
    let mut val = 0.0;
    let mut amp = 0.5;
    let mut freq = 1.0;
    for _ in 0..octaves {
        val += amp * value_noise_3d(x * freq, y * freq, z * freq);
        amp *= 0.5;
        freq *= 2.0;
    }
    val
}

fn hash3(x: i64, y: i64, z: i64) -> f64 {
    // Simple integer hash → [0, 1]
    let n = x.wrapping_mul(73856093) ^ y.wrapping_mul(19349663) ^ z.wrapping_mul(83492791);
    let n = ((n
        .wrapping_mul(n)
        .wrapping_mul(n.wrapping_mul(60493))
        .wrapping_add(19990303))
        & 0x7FFFFFFF) as f64;
    n / 0x7FFFFFFF as f64
}

/// Remap a value from [a_min, a_max] to [b_min, b_max].
pub fn remap(v: f64, a_min: f64, a_max: f64, b_min: f64, b_max: f64) -> f64 {
    let t = if (a_max - a_min).abs() > 1e-15 {
        (v - a_min) / (a_max - a_min)
    } else {
        0.0
    };
    b_min + t * (b_max - b_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_product() {
        assert!((dot([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])).abs() < 1e-15);
        assert!((dot([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn cross_product() {
        let c = cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn normalize_test() {
        let n = normalize([3.0, 4.0, 0.0]);
        assert!((length(n) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn distance_test() {
        assert!((distance([0.0; 3], [3.0, 4.0, 0.0]) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn lerp_test() {
        let m = lerp([0.0; 3], [10.0, 0.0, 0.0], 0.5);
        assert!((m[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn angle_test() {
        let a = angle_between([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn project_test() {
        let p = project([1.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((p[0] - 1.0).abs() < 1e-10);
        assert!(p[1].abs() < 1e-10);
    }

    #[test]
    fn remap_test() {
        assert!((remap(0.5, 0.0, 1.0, 0.0, 100.0) - 50.0).abs() < 1e-10);
    }

    #[test]
    fn spherical_roundtrip() {
        let (r, theta, phi) = cartesian_to_spherical(1.0, 1.0, 1.0);
        let (x, y, z) = spherical_to_cartesian(r, theta, phi);
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 1.0).abs() < 1e-10);
        assert!((z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cylindrical_roundtrip() {
        let (r, theta, z) = cartesian_to_cylindrical(3.0, 4.0, 5.0);
        assert!((r - 5.0).abs() < 1e-10);
        assert!((z - 5.0).abs() < 1e-10);
        let (x, y, z2) = cylindrical_to_cartesian(r, theta, z);
        assert!((x - 3.0).abs() < 1e-10);
        assert!((y - 4.0).abs() < 1e-10);
        assert!((z2 - 5.0).abs() < 1e-10);
    }

    #[test]
    fn bilinear_test() {
        assert!((bilinear(0.0, 1.0, 0.0, 1.0, 0.5, 0.5) - 0.5).abs() < 1e-10);
        assert!((bilinear(0.0, 1.0, 0.0, 1.0, 0.0, 0.0)).abs() < 1e-10);
        assert!((bilinear(0.0, 1.0, 0.0, 1.0, 1.0, 1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn smoothstep_test() {
        assert!((smoothstep(0.0, 1.0, 0.0)).abs() < 1e-10);
        assert!((smoothstep(0.0, 1.0, 1.0) - 1.0).abs() < 1e-10);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn noise_range() {
        for i in 0..100 {
            let v = value_noise_3d(i as f64 * 0.1, 0.0, 0.0);
            assert!(v >= 0.0 && v <= 1.0, "noise out of range: {v}");
        }
    }

    #[test]
    fn fbm_range() {
        let v = fbm_3d(1.23, 4.56, 7.89, 4);
        assert!(v >= 0.0 && v <= 1.0, "fbm out of range: {v}");
    }
}
