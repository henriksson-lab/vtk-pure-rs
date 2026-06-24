//! Simple Whitted-style CPU ray tracer for PolyData scenes.
//!
//! Supports triangle intersection (Moller-Trumbore), Blinn-Phong shading,
//! shadows, and one-bounce reflections.

use crate::render::{Light, LightType, Scene};

/// A ray with origin and direction.
#[derive(Debug, Clone, Copy)]
struct Ray {
    origin: [f64; 3],
    direction: [f64; 3],
}

/// Hit record from a ray-triangle intersection.
#[derive(Debug, Clone, Copy)]
struct Hit {
    t: f64,
    point: [f64; 3],
    normal: [f64; 3],
    color: [f32; 3],
    specular: f64,
    specular_power: f64,
}

/// A pre-extracted triangle for ray tracing.
struct Triangle {
    v0: [f64; 3],
    v1: [f64; 3],
    v2: [f64; 3],
    normal: [f64; 3],
    color: [f32; 3],
    specular: f64,
    specular_power: f64,
}

/// Simple CPU ray tracer for PolyData scenes.
pub struct RayTracer {
    pub width: u32,
    pub height: u32,
}

impl RayTracer {
    /// Create a new ray tracer with the given image dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Render the scene and return RGBA pixel data.
    pub fn render(&self, scene: &Scene) -> Vec<u8> {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut pixels = vec![0u8; w * h * 4];

        // Extract all triangles from scene actors
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

        for y in 0..h {
            for x in 0..w {
                // Map pixel to normalized device coords [-1, 1]
                let u = (2.0 * (x as f64 + 0.5) / w as f64 - 1.0) * half_w;
                let v = (1.0 - 2.0 * (y as f64 + 0.5) / h as f64) * half_h;

                let dir = [
                    cam_fwd.x + u * cam_right.x + v * cam_up_corrected.x,
                    cam_fwd.y + u * cam_right.y + v * cam_up_corrected.y,
                    cam_fwd.z + u * cam_right.z + v * cam_up_corrected.z,
                ];
                let dir = normalize(dir);

                let ray = Ray {
                    origin: cam_pos,
                    direction: dir,
                };
                let color = trace_ray(&ray, &triangles, &scene.lights, &scene.background, 1);

                let idx = (y * w + x) * 4;
                pixels[idx] = (color[0].clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 1] = (color[1].clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 2] = (color[2].clamp(0.0, 1.0) * 255.0) as u8;
                pixels[idx + 3] = 255;
            }
        }

        pixels
    }
}

fn trace_ray(
    ray: &Ray,
    triangles: &[Triangle],
    lights: &[Light],
    bg: &[f32; 4],
    bounces: u32,
) -> [f32; 3] {
    if let Some(hit) = closest_hit(ray, triangles) {
        let mut color = [0.0f32; 3];

        for light in lights {
            match light.light_type {
                LightType::Ambient => {
                    color[0] += hit.color[0] * light.color[0] * light.intensity as f32;
                    color[1] += hit.color[1] * light.color[1] * light.intensity as f32;
                    color[2] += hit.color[2] * light.color[2] * light.intensity as f32;
                }
                LightType::Directional => {
                    let light_dir = normalize(neg(light.direction));
                    // Shadow test
                    let shadow_origin = offset_point(hit.point, hit.normal);
                    let shadow_ray = Ray {
                        origin: shadow_origin,
                        direction: light_dir,
                    };
                    if closest_hit(&shadow_ray, triangles).is_some() {
                        continue;
                    }

                    // Diffuse
                    let ndl = dot(hit.normal, light_dir).max(0.0) as f32;
                    color[0] += hit.color[0] * light.color[0] * ndl * light.intensity as f32;
                    color[1] += hit.color[1] * light.color[1] * ndl * light.intensity as f32;
                    color[2] += hit.color[2] * light.color[2] * ndl * light.intensity as f32;

                    // Blinn-Phong specular
                    let view_dir = normalize(sub(ray.origin, hit.point));
                    let half_vec = normalize(add(light_dir, view_dir));
                    let ndh = dot(hit.normal, half_vec).max(0.0);
                    let spec = ndh.powf(hit.specular_power) as f32 * hit.specular as f32;
                    color[0] += light.color[0] * spec * light.intensity as f32;
                    color[1] += light.color[1] * spec * light.intensity as f32;
                    color[2] += light.color[2] * spec * light.intensity as f32;
                }
                LightType::Point => {
                    let to_light = sub(light.position, hit.point);
                    let dist = length(to_light);
                    let light_dir = scale(to_light, 1.0 / dist);

                    let shadow_origin = offset_point(hit.point, hit.normal);
                    let shadow_ray = Ray {
                        origin: shadow_origin,
                        direction: light_dir,
                    };
                    if let Some(sh) = closest_hit(&shadow_ray, triangles) {
                        if sh.t < dist {
                            continue;
                        }
                    }

                    let ndl = dot(hit.normal, light_dir).max(0.0) as f32;
                    let atten = (1.0 / (dist * dist)) as f32;
                    color[0] +=
                        hit.color[0] * light.color[0] * ndl * atten * light.intensity as f32;
                    color[1] +=
                        hit.color[1] * light.color[1] * ndl * atten * light.intensity as f32;
                    color[2] +=
                        hit.color[2] * light.color[2] * ndl * atten * light.intensity as f32;
                }
                _ => {}
            }
        }

        // Reflection (1 bounce)
        if bounces > 0 && hit.specular > 0.01 {
            let refl_dir = reflect(ray.direction, hit.normal);
            let refl_origin = offset_point(hit.point, hit.normal);
            let refl_ray = Ray {
                origin: refl_origin,
                direction: refl_dir,
            };
            let refl_color = trace_ray(&refl_ray, triangles, lights, bg, bounces - 1);
            let s = hit.specular as f32 * 0.3;
            color[0] += refl_color[0] * s;
            color[1] += refl_color[1] * s;
            color[2] += refl_color[2] * s;
        }

        color
    } else {
        [bg[0], bg[1], bg[2]]
    }
}

fn closest_hit(ray: &Ray, triangles: &[Triangle]) -> Option<Hit> {
    let mut best: Option<Hit> = None;
    let mut best_t = f64::MAX;

    for tri in triangles {
        if let Some((t, u, v)) = moller_trumbore(ray, tri) {
            if t > 1e-6 && t < best_t {
                best_t = t;
                let point = [
                    ray.origin[0] + t * ray.direction[0],
                    ray.origin[1] + t * ray.direction[1],
                    ray.origin[2] + t * ray.direction[2],
                ];
                let _ = (u, v);
                best = Some(Hit {
                    t,
                    point,
                    normal: tri.normal,
                    color: tri.color,
                    specular: tri.specular,
                    specular_power: tri.specular_power,
                });
            }
        }
    }

    best
}

/// Moller-Trumbore ray-triangle intersection. Returns (t, u, v).
fn moller_trumbore(ray: &Ray, tri: &Triangle) -> Option<(f64, f64, f64)> {
    let edge1 = sub(tri.v1, tri.v0);
    let edge2 = sub(tri.v2, tri.v0);
    let h = cross(ray.direction, edge2);
    let a = dot(edge1, h);

    if a.abs() < 1e-12 {
        return None;
    }

    let f = 1.0 / a;
    let s = sub(ray.origin, tri.v0);
    let u = f * dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = cross(s, edge1);
    let v = f * dot(ray.direction, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * dot(edge2, q);
    if t > 1e-6 {
        Some((t, u, v))
    } else {
        None
    }
}

fn extract_triangles(scene: &Scene) -> Vec<Triangle> {
    let mut triangles = Vec::new();

    for actor in &scene.actors {
        if !actor.visible {
            continue;
        }
        let color = match &actor.coloring {
            crate::render::Coloring::Solid(c) => *c,
            _ => [0.8, 0.8, 0.8],
        };
        let specular = actor.material.specular;
        let specular_power = actor.material.specular_power;
        let pd = &actor.data;
        let pos = actor.position;
        let s = actor.scale;

        for cell in pd.polys.iter() {
            if cell.len() >= 3 {
                // Fan triangulation for polygons
                let v0_idx = cell[0] as usize;
                let p0 = pd.points.get(v0_idx);
                let v0 = [p0[0] * s + pos[0], p0[1] * s + pos[1], p0[2] * s + pos[2]];

                for i in 1..cell.len() - 1 {
                    let v1_idx = cell[i] as usize;
                    let v2_idx = cell[i + 1] as usize;
                    let p1 = pd.points.get(v1_idx);
                    let p2 = pd.points.get(v2_idx);
                    let v1 = [p1[0] * s + pos[0], p1[1] * s + pos[1], p1[2] * s + pos[2]];
                    let v2 = [p2[0] * s + pos[0], p2[1] * s + pos[1], p2[2] * s + pos[2]];

                    let edge1 = sub(v1, v0);
                    let edge2 = sub(v2, v0);
                    let normal = normalize(cross(edge1, edge2));

                    triangles.push(Triangle {
                        v0,
                        v1,
                        v2,
                        normal,
                        color,
                        specular,
                        specular_power,
                    });
                }
            }
        }
    }

    triangles
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
fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn neg(a: [f64; 3]) -> [f64; 3] {
    [-a[0], -a[1], -a[2]]
}
fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let l = length(a);
    if l < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    scale(a, 1.0 / l)
}
fn reflect(v: [f64; 3], n: [f64; 3]) -> [f64; 3] {
    let d = 2.0 * dot(v, n);
    sub(v, scale(n, d))
}
fn offset_point(p: [f64; 3], n: [f64; 3]) -> [f64; 3] {
    add(p, scale(n, 1e-4))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;
    use crate::render::{Actor, Camera, Scene};

    fn make_triangle_scene() -> Scene {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let mut scene = Scene::new().with_actor(Actor::new(mesh).with_color(0.8, 0.2, 0.2));
        scene.camera.position = glam::DVec3::new(0.5, 0.4, 2.0);
        scene.camera.focal_point = glam::DVec3::new(0.5, 0.4, 0.0);
        scene
    }

    #[test]
    fn test_render_triangle() {
        let scene = make_triangle_scene();
        let rt = RayTracer::new(16, 16);
        let pixels = rt.render(&scene);
        assert_eq!(pixels.len(), 16 * 16 * 4);

        // Check that at least some pixels are non-black (triangle is visible)
        let non_black = pixels
            .chunks(4)
            .any(|px| px[0] > 0 || px[1] > 0 || px[2] > 0);
        assert!(non_black, "Should have non-black pixels from the triangle");
    }

    #[test]
    fn test_render_center_pixel_is_colored() {
        let scene = make_triangle_scene();
        let rt = RayTracer::new(32, 32);
        let pixels = rt.render(&scene);

        // Center pixel should hit the triangle
        let cx = 16;
        let cy = 16;
        let idx = (cy * 32 + cx) * 4;
        let r = pixels[idx];
        let g = pixels[idx + 1];
        let b = pixels[idx + 2];
        // The triangle is red-ish, so R should be > 0
        assert!(
            r > 0 || g > 0 || b > 0,
            "Center pixel should be non-black (got {r},{g},{b})"
        );
    }
}
