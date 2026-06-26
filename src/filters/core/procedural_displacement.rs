//! Procedural noise-based vertex displacement.

use crate::data::{Points, PolyData};

/// Displace vertices along normals by multi-octave noise.
pub fn fbm_displace(
    mesh: &PolyData,
    amplitude: f64,
    frequency: f64,
    octaves: usize,
    seed: u64,
) -> PolyData {
    let n = mesh.points.len();
    if n == 0 {
        return mesh.clone();
    }
    let nm = normals(mesh);
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        let mut val = 0.0;
        let mut amp = amplitude;
        let mut freq = frequency;
        for oct in 0..octaves {
            val += amp * noise3(p[0] * freq, p[1] * freq, p[2] * freq, seed + oct as u64);
            amp *= 0.5;
            freq *= 2.0;
        }
        pts.push([
            p[0] + nm[i][0] * val,
            p[1] + nm[i][1] * val,
            p[2] + nm[i][2] * val,
        ]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}

/// Jitter vertices randomly.
pub fn jitter(mesh: &PolyData, mag: f64, seed: u64) -> PolyData {
    let n = mesh.points.len();
    let mut rng = seed.wrapping_add(1);
    let nxt = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s >> 33) as f64 / (1u64 << 31) as f64 - 0.5
    };
    let mut pts = Points::<f64>::new();
    for i in 0..n {
        let p = mesh.points.get(i);
        pts.push([
            p[0] + nxt(&mut rng) * mag,
            p[1] + nxt(&mut rng) * mag,
            p[2] + nxt(&mut rng) * mag,
        ]);
    }
    let mut r = mesh.clone();
    r.points = pts;
    r
}

fn normals(m: &PolyData) -> Vec<[f64; 3]> {
    let n = m.points.len();
    let mut nm = vec![[0.0; 3]; n];
    for c in m.polys.iter() {
        if c.len() < 3 {
            continue;
        }
        let a = m.points.get(c[0] as usize);
        let b = m.points.get(c[1] as usize);
        let cc = m.points.get(c[2] as usize);
        let f = [
            (b[1] - a[1]) * (cc[2] - a[2]) - (b[2] - a[2]) * (cc[1] - a[1]),
            (b[2] - a[2]) * (cc[0] - a[0]) - (b[0] - a[0]) * (cc[2] - a[2]),
            (b[0] - a[0]) * (cc[1] - a[1]) - (b[1] - a[1]) * (cc[0] - a[0]),
        ];
        for &p in c {
            let i = p as usize;
            for j in 0..3 {
                nm[i][j] += f[j];
            }
        }
    }
    for n in &mut nm {
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 1e-15 {
            for j in 0..3 {
                n[j] /= l;
            }
        }
    }
    nm
}

fn noise3(x: f64, y: f64, z: f64, seed: u64) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let iz = z.floor() as i64;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let fz = z - z.floor();
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);
    let sz = fz * fz * (3.0 - 2.0 * fz);
    let h = |x: i64, y: i64, z: i64| -> f64 {
        let h = ((x.wrapping_mul(374761393)
            ^ y.wrapping_mul(668265263)
            ^ z.wrapping_mul(1013904223)) as u64)
            .wrapping_add(seed);
        let h = h.wrapping_mul(6364136223846793005).wrapping_add(1);
        2.0 * ((h >> 33) as f64 / (1u64 << 31) as f64) - 1.0
    };
    let mut r = 0.0;
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                r += (if dx == 0 { 1.0 - sx } else { sx })
                    * (if dy == 0 { 1.0 - sy } else { sy })
                    * (if dz == 0 { 1.0 - sz } else { sz })
                    * h(ix + dx, iy + dy, iz + dz);
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fbm() {
        let m = crate::filters::core::sources::sphere::sphere(
            &crate::filters::core::sources::sphere::SphereParams::default(),
        );
        let r = fbm_displace(&m, 0.05, 3.0, 3, 42);
        assert_eq!(r.points.len(), m.points.len());
    }
    #[test]
    fn jit() {
        let m = PolyData::from_points(vec![[0.0; 3]]);
        let r = jitter(&m, 0.01, 1);
        let p = r.points.get(0);
        assert!(p[0].abs() < 0.02);
    }

    #[test]
    fn noise_is_signed() {
        let a = noise3(0.25, 0.5, 0.75, 7);
        let b = noise3(3.25, 1.5, 4.75, 11);
        assert!((-1.0..=1.0).contains(&a));
        assert!((-1.0..=1.0).contains(&b));
        assert!(a > 0.0 || b > 0.0);
    }
}
