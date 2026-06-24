//! Monte Carlo path tracer for photorealistic rendering.
//!
//! A simple unbiased path tracer with cosine-weighted hemisphere sampling,
//! Reinhard tone mapping, and gamma correction.

use crate::render::Scene;

/// Monte Carlo path tracer.
pub struct PathTracer {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub max_bounces: u32,
}

impl PathTracer {
    /// Create a new path tracer.
    pub fn new(width: u32, height: u32, samples_per_pixel: u32, max_bounces: u32) -> Self {
        Self {
            width,
            height,
            samples_per_pixel,
            max_bounces,
        }
    }

    /// Render the scene and return RGBA pixel data.
    pub fn render(&self, scene: &Scene) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut pixels = vec![0u8; w * h * 4];

        // Extract triangles from scene (reuse logic similar to ray_tracer)
        let triangles = extract_triangles(scene);

        let cam_pos = [
            scene.camera.position.x,
            scene.camera.position.y,
            scene.camera.position.z,
        ];
        let cam_fwd = scene.camera.direction();
        let cam_up = scene.camera.view_up.normalize();
        let cam_right = cam_fwd.cross(cam_up).normalize();
        let cam_up_corrected = cam_right.cross(cam_fwd).normalize();

        let aspect = w as f64 / h as f64;
        let fov_rad = scene.camera.fov.to_radians();
        let half_h = (fov_rad * 0.5).tan();
        let half_w = half_h * aspect;

        // Simple pseudo-random number generator (xorshift64)
        let mut rng_state: u64 = 0x12345678_9ABCDEF0;

        for y in 0..h {
            for x in 0..w {
                let mut accum = [0.0f64; 3];

                for _s in 0..self.samples_per_pixel {
                    // Jitter within pixel
                    let jx = next_f64(&mut rng_state);
                    let jy = next_f64(&mut rng_state);
                    let u = (2.0 * (x as f64 + jx) / w as f64 - 1.0) * half_w;
                    let v = (1.0 - 2.0 * (y as f64 + jy) / h as f64) * half_h;

                    let dir = normalize([
                        cam_fwd.x + u * cam_right.x + v * cam_up_corrected.x,
                        cam_fwd.y + u * cam_right.y + v * cam_up_corrected.y,
                        cam_fwd.z + u * cam_right.z + v * cam_up_corrected.z,
                    ]);

                    let color = trace_path(
                        cam_pos,
                        dir,
                        &triangles,
                        &scene.background,
                        self.max_bounces,
                        &mut rng_state,
                    );
                    accum[0] += color[0];
                    accum[1] += color[1];
                    accum[2] += color[2];
                }

                let inv_spp = 1.0 / self.samples_per_pixel as f64;
                let mut hdr = [accum[0] * inv_spp, accum[1] * inv_spp, accum[2] * inv_spp];

                // Reinhard tone mapping
                hdr[0] = hdr[0] / (1.0 + hdr[0]);
                hdr[1] = hdr[1] / (1.0 + hdr[1]);
                hdr[2] = hdr[2] / (1.0 + hdr[2]);

                // Gamma correction (gamma 2.2)
                let gamma = 1.0 / 2.2;
                let idx = (y * w + x) * 4;
                pixels[idx] = (hdr[0].powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 1] = (hdr[1].powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 2] = (hdr[2].powf(gamma).clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 3] = 255;
            }
        }

        pixels
    }
}

struct Triangle {
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
    normal: [f64; 3],
    color: [f64; 3],
}

fn extract_triangles(scene: &Scene) -> Vec<Triangle> {
    let mut tris = Vec::new();
    for actor in &scene.actors {
        if !actor.visible {
            continue;
        }
        let color = match &actor.coloring {
            crate::render::Coloring::Solid(c) => [c[0] as f64, c[1] as f64, c[2] as f64],
            _ => [0.8, 0.8, 0.8],
        };
        let pos = actor.position;
        let s = actor.scale;
        for cell in actor.data.polys.iter() {
            if cell.len() >= 3 {
                let p0 = actor.data.points.get(cell[0] as usize);
                let v0 = [p0[0] * s + pos[0], p0[1] * s + pos[1], p0[2] * s + pos[2]];
                for i in 1..cell.len() - 1 {
                    let p1 = actor.data.points.get(cell[i] as usize);
                    let p2 = actor.data.points.get(cell[i + 1] as usize);
                    let v1 = [p1[0] * s + pos[0], p1[1] * s + pos[1], p1[2] * s + pos[2]];
                    let v2 = [p2[0] * s + pos[0], p2[1] * s + pos[1], p2[2] * s + pos[2]];
                    let e1 = sub(v1, v0);
                    let e2 = sub(v2, v0);
                    let n = normalize(cross(e1, e2));
                    tris.push(Triangle {
                        v0,
                        v1,
                        v2,
                        normal: n,
                        color,
                    });
                }
            }
        }
    }
    tris
}

fn trace_path(
    origin: [f64; 3],
    direction: [f64; 3],
    triangles: &[Triangle],
    bg: &[f32; 4],
    max_bounces: u32,
    rng: &mut u64,
) -> [f64; 3] {
    let mut throughput = [1.0f64; 3];
    let mut radiance = [0.0f64; 3];
    let mut o = origin;
    let mut d = direction;

    for _bounce in 0..=max_bounces {
        if let Some((t, tri_idx)) = closest_hit(o, d, triangles) {
            let tri = &triangles[tri_idx];
            let hit_pt = [o[0] + t * d[0], o[1] + t * d[1], o[2] + t * d[2]];
            let mut normal = tri.normal;

            // Ensure normal faces the ray
            if dot(normal, d) > 0.0 {
                normal = neg(normal);
            }

            // Accumulate emission (none for diffuse surfaces, but ambient light contribution)
            // Simple: no emissive surfaces, just bounce

            // Cosine-weighted hemisphere sampling for next bounce
            let (new_dir, cos_theta) = cosine_hemisphere(normal, rng);

            // BRDF for Lambertian: albedo / pi
            // PDF for cosine sampling: cos_theta / pi
            // weight = (albedo / pi) * cos_theta / (cos_theta / pi) = albedo
            throughput[0] *= tri.color[0];
            throughput[1] *= tri.color[1];
            throughput[2] *= tri.color[2];

            let _ = cos_theta;

            o = [
                hit_pt[0] + normal[0] * 1e-4,
                hit_pt[1] + normal[1] * 1e-4,
                hit_pt[2] + normal[2] * 1e-4,
            ];
            d = new_dir;
        } else {
            // Hit background (sky light)
            let sky = [bg[0] as f64, bg[1] as f64, bg[2] as f64];
            radiance[0] += throughput[0] * sky[0];
            radiance[1] += throughput[1] * sky[1];
            radiance[2] += throughput[2] * sky[2];
            break;
        }
    }

    radiance
}

fn closest_hit(o: [f64; 3], d: [f64; 3], tris: &[Triangle]) -> Option<(f64, usize)> {
    let mut best_t = f64::MAX;
    let mut best_idx = None;
    for (i, tri) in tris.iter().enumerate() {
        if let Some(t) = moller_trumbore(o, d, tri) {
            if t > 1e-6 && t < best_t {
                best_t = t;
                best_idx = Some(i);
            }
        }
    }
    best_idx.map(|i| (best_t, i))
}

fn moller_trumbore(o: [f64; 3], d: [f64; 3], tri: &Triangle) -> Option<f64> {
    let e1 = sub(tri.v1, tri.v0);
    let e2 = sub(tri.v2, tri.v0);
    let h = cross(d, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-12 {
        return None;
    }
    let f = 1.0 / a;
    let s = sub(o, tri.v0);
    let u = f * dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = f * dot(d, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot(e2, q);
    if t > 1e-6 {
        Some(t)
    } else {
        None
    }
}

fn cosine_hemisphere(normal: [f64; 3], rng: &mut u64) -> ([f64; 3], f64) {
    let r1 = next_f64(rng);
    let r2 = next_f64(rng);
    let sin_theta = (1.0 - r1).sqrt();
    let cos_theta = r1.sqrt();
    let phi = 2.0 * std::f64::consts::PI * r2;

    // Build local coordinate frame
    let w = normal;
    let a = if w[0].abs() > 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let u = normalize(cross(a, w));
    let v = cross(w, u);

    let dir = normalize([
        u[0] * phi.cos() * sin_theta + v[0] * phi.sin() * sin_theta + w[0] * cos_theta,
        u[1] * phi.cos() * sin_theta + v[1] * phi.sin() * sin_theta + w[1] * cos_theta,
        u[2] * phi.cos() * sin_theta + v[2] * phi.sin() * sin_theta + w[2] * cos_theta,
    ]);

    (dir, cos_theta)
}

// xorshift64 PRNG
fn next_u64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn next_f64(state: &mut u64) -> f64 {
    (next_u64(state) >> 11) as f64 / (1u64 << 53) as f64
}

// Vector math helpers
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn neg(a: [f64; 3]) -> [f64; 3] {
    [-a[0], -a[1], -a[2]]
}
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let l = dot(a, a).sqrt();
    if l < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [a[0] / l, a[1] / l, a[2] / l]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;
    use crate::render::{Actor, Scene};

    #[test]
    fn test_path_tracer_basic() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut scene = Scene::new()
            .with_actor(Actor::new(mesh).with_color(0.8, 0.2, 0.2))
            .with_background(0.5, 0.5, 0.8);
        scene.camera.position = glam::DVec3::new(0.5, 0.4, 2.0);
        scene.camera.focal_point = glam::DVec3::new(0.5, 0.4, 0.0);

        let pt = PathTracer::new(8, 8, 4, 2);
        let pixels = pt.render(&scene);
        assert_eq!(pixels.len(), 8 * 8 * 4);

        // With a bright background, most pixels should be non-black
        let non_black = pixels
            .chunks(4)
            .filter(|px| px[0] > 0 || px[1] > 0 || px[2] > 0)
            .count();
        assert!(non_black > 0, "Should have some non-black pixels");
    }
}
