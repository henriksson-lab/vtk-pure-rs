use std::io::Write;

use crate::render::Scene;

/// Export scene configuration as JSON (camera, lights, background, fog, clip planes).
///
/// Does NOT include mesh data — only scene settings.
pub fn scene_to_json<W: Write>(w: &mut W, scene: &Scene) -> std::io::Result<()> {
    writeln!(w, "{{")?;

    // Camera
    let c = &scene.camera;
    writeln!(w, "  \"camera\": {{")?;
    writeln!(
        w,
        "    \"position\": [{}, {}, {}],",
        c.position.x, c.position.y, c.position.z
    )?;
    writeln!(
        w,
        "    \"focal_point\": [{}, {}, {}],",
        c.focal_point.x, c.focal_point.y, c.focal_point.z
    )?;
    writeln!(
        w,
        "    \"view_up\": [{}, {}, {}],",
        c.view_up.x, c.view_up.y, c.view_up.z
    )?;
    writeln!(w, "    \"fov\": {},", c.fov)?;
    writeln!(w, "    \"near_clip\": {},", c.near_clip)?;
    writeln!(w, "    \"far_clip\": {}", c.far_clip)?;
    writeln!(w, "  }},")?;

    // Background
    writeln!(
        w,
        "  \"background\": [{}, {}, {}, {}],",
        scene.background[0], scene.background[1], scene.background[2], scene.background[3]
    )?;

    // Actors summary
    writeln!(w, "  \"num_actors\": {},", scene.actors.len())?;

    // Lights
    writeln!(w, "  \"lights\": [")?;
    for (i, light) in scene.lights.iter().enumerate() {
        let lt = match light.light_type {
            crate::render::LightType::Directional => "directional",
            crate::render::LightType::Point => "point",
            crate::render::LightType::Spot { .. } => "spot",
            crate::render::LightType::Ambient => "ambient",
        };
        write!(
            w,
            "    {{\"type\": \"{lt}\", \"intensity\": {}, \"color\": [{}, {}, {}]}}",
            light.intensity, light.color[0], light.color[1], light.color[2]
        )?;
        if i < scene.lights.len() - 1 {
            write!(w, ",")?;
        }
        writeln!(w)?;
    }
    writeln!(w, "  ],")?;

    // Fog
    writeln!(w, "  \"fog\": {{")?;
    writeln!(w, "    \"enabled\": {},", scene.fog.enabled)?;
    writeln!(w, "    \"near\": {},", scene.fog.near)?;
    writeln!(w, "    \"far\": {},", scene.fog.far)?;
    writeln!(w, "    \"density\": {}", scene.fog.density)?;
    writeln!(w, "  }},")?;

    // Clip planes
    writeln!(w, "  \"clip_planes\": [")?;
    for (i, cp) in scene.clip_planes.iter().enumerate() {
        write!(
            w,
            "    {{\"normal\": [{}, {}, {}], \"distance\": {}, \"enabled\": {}}}",
            cp.normal[0], cp.normal[1], cp.normal[2], cp.distance, cp.enabled
        )?;
        if i < scene.clip_planes.len() - 1 {
            write!(w, ",")?;
        }
        writeln!(w)?;
    }
    writeln!(w, "  ]")?;

    writeln!(w, "}}")?;
    Ok(())
}

/// Export scene JSON to a string.
pub fn scene_to_json_string(scene: &Scene) -> String {
    let mut buf = Vec::new();
    scene_to_json(&mut buf, scene).unwrap();
    String::from_utf8(buf).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::PolyData;
    use crate::render::{Actor, ClipPlane, Fog, Scene};

    #[test]
    fn json_export() {
        let scene = Scene::new()
            .with_actor(Actor::new(PolyData::new()))
            .with_background(0.1, 0.2, 0.3)
            .with_fog(Fog::linear(5.0, 50.0));

        let json = scene_to_json_string(&scene);
        assert!(json.contains("\"camera\""));
        assert!(json.contains("\"background\""));
        assert!(json.contains("\"num_actors\": 1"));
        assert!(json.contains("\"fog\""));
        assert!(json.contains("\"enabled\": true"));
    }

    #[test]
    fn json_with_clip_planes() {
        let mut scene = Scene::new();
        scene.clip_planes.push(ClipPlane::x(1.0));

        let json = scene_to_json_string(&scene);
        assert!(json.contains("\"clip_planes\""));
        assert!(json.contains("\"normal\""));
    }
}
