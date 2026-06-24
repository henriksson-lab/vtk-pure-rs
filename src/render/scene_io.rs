use std::io::{BufRead, Write};

use crate::render::{ClipPlane, Light, LightType, Scene};

/// Save scene configuration (camera, lights, background, clip planes) to a text format.
///
/// Does NOT save mesh data — only the scene settings. Load mesh data separately
/// and apply the saved scene configuration.
pub fn save_scene_config<W: Write>(w: &mut W, scene: &Scene) -> std::io::Result<()> {
    writeln!(w, "# vtk-rs scene configuration")?;
    writeln!(w, "version 1")?;
    writeln!(w)?;

    // Camera
    let c = &scene.camera;
    writeln!(
        w,
        "camera.position {} {} {}",
        c.position.x, c.position.y, c.position.z
    )?;
    writeln!(
        w,
        "camera.focal_point {} {} {}",
        c.focal_point.x, c.focal_point.y, c.focal_point.z
    )?;
    writeln!(
        w,
        "camera.view_up {} {} {}",
        c.view_up.x, c.view_up.y, c.view_up.z
    )?;
    writeln!(w, "camera.fov {}", c.fov)?;
    writeln!(w, "camera.near_clip {}", c.near_clip)?;
    writeln!(w, "camera.far_clip {}", c.far_clip)?;
    writeln!(w)?;

    // Background
    writeln!(
        w,
        "background {} {} {} {}",
        scene.background[0], scene.background[1], scene.background[2], scene.background[3]
    )?;
    writeln!(w)?;

    // Lights
    for (i, light) in scene.lights.iter().enumerate() {
        if !light.enabled {
            continue;
        }
        let lt = match light.light_type {
            LightType::Directional => "directional",
            LightType::Point => "point",
            LightType::Spot { .. } => "spot",
            LightType::Ambient => "ambient",
        };
        writeln!(w, "light.{i}.type {lt}")?;
        writeln!(
            w,
            "light.{i}.position {} {} {}",
            light.position[0], light.position[1], light.position[2]
        )?;
        writeln!(
            w,
            "light.{i}.direction {} {} {}",
            light.direction[0], light.direction[1], light.direction[2]
        )?;
        writeln!(
            w,
            "light.{i}.color {} {} {}",
            light.color[0], light.color[1], light.color[2]
        )?;
        writeln!(w, "light.{i}.intensity {}", light.intensity)?;
    }
    writeln!(w)?;

    // Clip planes
    for (i, cp) in scene.clip_planes.iter().enumerate() {
        if !cp.enabled {
            continue;
        }
        writeln!(
            w,
            "clip_plane.{i} {} {} {} {}",
            cp.normal[0], cp.normal[1], cp.normal[2], cp.distance
        )?;
    }

    Ok(())
}

/// Load scene configuration from the text format.
///
/// Applies camera, lights, background, and clip planes to the given scene.
pub fn load_scene_config<R: BufRead>(r: R, scene: &mut Scene) -> std::io::Result<()> {
    scene.lights.clear();
    scene.clip_planes.clear();

    for line in r.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        match parts[0] {
            "camera.position" if parts.len() >= 4 => {
                scene.camera.position = glam::DVec3::new(
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                );
            }
            "camera.focal_point" if parts.len() >= 4 => {
                scene.camera.focal_point = glam::DVec3::new(
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(0.0),
                    parts[3].parse().unwrap_or(0.0),
                );
            }
            "camera.view_up" if parts.len() >= 4 => {
                scene.camera.view_up = glam::DVec3::new(
                    parts[1].parse().unwrap_or(0.0),
                    parts[2].parse().unwrap_or(1.0),
                    parts[3].parse().unwrap_or(0.0),
                );
            }
            "camera.fov" => {
                scene.camera.fov = parts[1].parse().unwrap_or(30.0);
            }
            "camera.near_clip" => {
                scene.camera.near_clip = parts[1].parse().unwrap_or(0.01);
            }
            "camera.far_clip" => {
                scene.camera.far_clip = parts[1].parse().unwrap_or(1000.0);
            }
            "background" if parts.len() >= 5 => {
                scene.background = [
                    parts[1].parse().unwrap_or(0.1),
                    parts[2].parse().unwrap_or(0.1),
                    parts[3].parse().unwrap_or(0.1),
                    parts[4].parse().unwrap_or(1.0),
                ];
            }
            key if key.starts_with("clip_plane.") && parts.len() >= 5 => {
                scene.clip_planes.push(ClipPlane {
                    normal: [
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(1.0),
                    ],
                    distance: parts[4].parse().unwrap_or(0.0),
                    enabled: true,
                });
            }
            key if key.starts_with("light.") && key.ends_with(".type") => {
                let lt = match parts[1] {
                    "directional" => LightType::Directional,
                    "point" => LightType::Point,
                    "ambient" => LightType::Ambient,
                    _ => LightType::Directional,
                };
                scene.lights.push(Light {
                    light_type: lt,
                    ..Light::default()
                });
            }
            key if key.starts_with("light.") && key.ends_with(".intensity") => {
                if let Some(light) = scene.lights.last_mut() {
                    light.intensity = parts[1].parse().unwrap_or(1.0);
                }
            }
            key if key.starts_with("light.") && key.ends_with(".color") && parts.len() >= 4 => {
                if let Some(light) = scene.lights.last_mut() {
                    light.color = [
                        parts[1].parse().unwrap_or(1.0),
                        parts[2].parse().unwrap_or(1.0),
                        parts[3].parse().unwrap_or(1.0),
                    ];
                }
            }
            key if key.starts_with("light.") && key.ends_with(".direction") && parts.len() >= 4 => {
                if let Some(light) = scene.lights.last_mut() {
                    light.direction = [
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(-1.0),
                    ];
                }
            }
            key if key.starts_with("light.") && key.ends_with(".position") && parts.len() >= 4 => {
                if let Some(light) = scene.lights.last_mut() {
                    light.position = [
                        parts[1].parse().unwrap_or(0.0),
                        parts[2].parse().unwrap_or(0.0),
                        parts[3].parse().unwrap_or(0.0),
                    ];
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_scene_config() {
        let mut scene = Scene::new();
        scene.camera.position = glam::DVec3::new(1.0, 2.0, 3.0);
        scene.camera.focal_point = glam::DVec3::new(0.5, 0.5, 0.0);
        scene.camera.fov = 45.0;
        scene.background = [0.2, 0.3, 0.4, 1.0];
        scene.clip_planes.push(ClipPlane::x(1.0));

        let mut buf = Vec::new();
        save_scene_config(&mut buf, &scene).unwrap();

        let config_str = String::from_utf8(buf.clone()).unwrap();
        assert!(config_str.contains("camera.position"));
        assert!(config_str.contains("clip_plane"));

        let mut loaded = Scene::new();
        load_scene_config(std::io::BufReader::new(&buf[..]), &mut loaded).unwrap();

        assert!((loaded.camera.position.x - 1.0).abs() < 1e-6);
        assert!((loaded.camera.position.y - 2.0).abs() < 1e-6);
        assert!((loaded.camera.fov - 45.0).abs() < 1e-6);
        assert!((loaded.background[0] - 0.2).abs() < 0.01);
        assert_eq!(loaded.clip_planes.len(), 1);
    }

    #[test]
    fn roundtrip_lights() {
        let mut scene = Scene::new();
        scene.clear_lights();
        scene.add_light(Light::directional([0.0, -1.0, 0.0], [1.0, 0.9, 0.8], 0.8));

        let mut buf = Vec::new();
        save_scene_config(&mut buf, &scene).unwrap();

        let mut loaded = Scene::new();
        load_scene_config(std::io::BufReader::new(&buf[..]), &mut loaded).unwrap();

        assert_eq!(loaded.lights.len(), 1);
        assert!((loaded.lights[0].intensity - 0.8).abs() < 0.01);
    }
}
