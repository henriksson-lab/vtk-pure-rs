use crate::data::PolyData;

use crate::render::{
    Annotations, AxesWidget, BloomConfig, Camera, ClipPlane, ColorMap, DofConfig, Fog, Light,
    LodSet, Material, ScalarBar, ShadowConfig, SilhouetteConfig, Skybox, StereoConfig, Texture,
    Viewport, VolumeActor,
};

/// How to render the surface.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Representation {
    #[default]
    Surface,
    Wireframe,
    Points,
}

/// How to determine vertex colors.
#[derive(Debug, Clone)]
pub enum Coloring {
    /// Uniform color for the entire actor.
    Solid([f32; 3]),
    /// Map active scalars from point data through a color map.
    ScalarMap {
        color_map: ColorMap,
        /// Scalar range [min, max]. If None, auto-computed from data.
        range: Option<[f64; 2]>,
    },
    /// Apply a 2D texture using texture coordinates from point data.
    TextureMap { texture: Texture },
}

impl Default for Coloring {
    fn default() -> Self {
        Coloring::Solid([1.0, 1.0, 1.0])
    }
}

/// A renderable entity in the scene.
#[derive(Debug, Clone)]
pub struct Actor {
    pub data: PolyData,
    pub coloring: Coloring,
    pub opacity: f32,
    pub representation: Representation,
    pub material: Material,
    /// Optional level-of-detail set. When present, the renderer selects
    /// the appropriate detail level based on camera distance.
    pub lod: Option<LodSet>,
    /// Per-actor position offset (translation).
    pub position: [f64; 3],
    /// Per-actor uniform scale factor.
    pub scale: f64,
    /// Whether this actor is visible.
    pub visible: bool,
}

impl Actor {
    /// Create an actor with a solid color (shorthand for `Actor::new(data).with_color(r, g, b)`).
    pub fn colored(data: PolyData, r: f32, g: f32, b: f32) -> Self {
        Self::new(data).with_color(r, g, b)
    }

    pub fn new(data: PolyData) -> Self {
        Self {
            data,
            coloring: Coloring::default(),
            opacity: 1.0,
            representation: Representation::Surface,
            material: Material::default(),
            lod: None,
            position: [0.0, 0.0, 0.0],
            scale: 1.0,
            visible: true,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.coloring = Coloring::Solid([r, g, b]);
        self
    }

    pub fn with_scalar_coloring(mut self, color_map: ColorMap, range: Option<[f64; 2]>) -> Self {
        self.coloring = Coloring::ScalarMap { color_map, range };
        self
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.coloring = Coloring::TextureMap { texture };
        self
    }

    /// Set the actor position (translation).
    pub fn with_position(mut self, x: f64, y: f64, z: f64) -> Self {
        self.position = [x, y, z];
        self
    }

    /// Set the actor scale.
    pub fn with_scale(mut self, s: f64) -> Self {
        self.scale = s;
        self
    }

    /// Set the actor opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set the actor material.
    pub fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }

    /// Set the actor representation mode.
    pub fn with_representation(mut self, repr: Representation) -> Self {
        self.representation = repr;
        self
    }

    /// Set visibility.
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// A 3D scene containing actors, lights, a camera, and background color.
///
/// # Examples
///
/// ```
/// use crate::data::PolyData;
/// use crate::render::{Actor, Scene};
///
/// let pd = PolyData::from_triangles(
///     vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
///     vec![[0, 1, 2]],
/// );
/// let scene = Scene::new()
///     .with_actor(Actor::new(pd).with_color(1.0, 0.0, 0.0))
///     .with_background(0.2, 0.2, 0.2)
///     .with_axes();
/// assert_eq!(scene.num_actors(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Scene {
    pub actors: Vec<Actor>,
    pub lights: Vec<Light>,
    pub scalar_bars: Vec<ScalarBar>,
    pub axes_widget: Option<AxesWidget>,
    pub silhouette: SilhouetteConfig,
    pub volumes: Vec<VolumeActor>,
    /// Up to 6 clip planes for section views.
    pub clip_planes: Vec<ClipPlane>,
    /// Distance fog configuration.
    pub fog: Fog,
    /// Shadow mapping configuration.
    pub shadows: ShadowConfig,
    /// Skybox / environment background.
    pub skybox: Skybox,
    /// Bloom post-processing.
    pub bloom: BloomConfig,
    /// 3D annotations (labels, rulers, protractors).
    pub annotations: Annotations,
    /// Stereo rendering configuration.
    pub stereo: StereoConfig,
    /// Screen-space ambient occlusion configuration.
    pub ssao: crate::render::ssao::SsaoConfig,
    /// Depth-of-field post-processing configuration.
    pub dof: DofConfig,
    /// Split-screen viewports with per-viewport cameras.
    /// When non-empty, each entry is rendered in its viewport region.
    /// When empty, the main `camera` renders full-screen.
    pub viewports: Vec<(Viewport, Camera)>,
    pub camera: Camera,
    pub background: [f32; 4],
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            actors: Vec::new(),
            lights: vec![Light::headlight()],
            scalar_bars: Vec::new(),
            axes_widget: None,
            silhouette: SilhouetteConfig::default(),
            volumes: Vec::new(),
            clip_planes: Vec::new(),
            fog: Fog::default(),
            shadows: ShadowConfig::default(),
            skybox: Skybox::default(),
            bloom: BloomConfig::default(),
            annotations: Annotations::default(),
            stereo: StereoConfig::default(),
            ssao: crate::render::ssao::SsaoConfig::default(),
            dof: DofConfig::default(),
            viewports: Vec::new(),
            camera: Camera::default(),
            background: [0.1, 0.1, 0.1, 1.0],
        }
    }
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    /// Quick-start: create a scene from a single PolyData with auto camera.
    pub fn from_poly_data(data: PolyData) -> Self {
        let mut scene = Self::new();
        scene.add_actor(Actor::new(data));
        scene.reset_camera();
        scene
    }

    /// Quick-start: create a scene from a colored PolyData with auto camera.
    pub fn from_poly_data_colored(data: PolyData, r: f32, g: f32, b: f32) -> Self {
        let mut scene = Self::new();
        scene.add_actor(Actor::new(data).with_color(r, g, b));
        scene.reset_camera();
        scene
    }

    /// Quick-start: create a scene from multiple PolyData with different colors.
    pub fn from_meshes(meshes: Vec<(PolyData, [f32; 3])>) -> Self {
        let mut scene = Self::new();
        for (data, color) in meshes {
            scene.add_actor(Actor::new(data).with_color(color[0], color[1], color[2]));
        }
        scene.reset_camera();
        scene
    }

    pub fn add_actor(&mut self, actor: Actor) {
        self.actors.push(actor);
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Add a scalar bar (color legend) overlay.
    pub fn add_scalar_bar(&mut self, bar: ScalarBar) {
        self.scalar_bars.push(bar);
    }

    /// Remove all lights and start fresh.
    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }

    /// Remove all actors.
    pub fn clear_actors(&mut self) {
        self.actors.clear();
    }

    /// Number of actors.
    pub fn num_actors(&self) -> usize {
        self.actors.len()
    }

    /// Add a clip plane for section views.
    pub fn add_clip_plane(&mut self, plane: crate::render::ClipPlane) {
        self.clip_planes.push(plane);
    }

    /// Reset the camera to view all actors.
    pub fn reset_camera(&mut self) {
        use crate::types::BoundingBox;
        if self.actors.is_empty() {
            return;
        }
        let mut bb = BoundingBox::empty();
        for actor in &self.actors {
            let ab = actor.data.points.bounds();
            bb.expand([ab.x_min, ab.y_min, ab.z_min]);
            bb.expand([ab.x_max, ab.y_max, ab.z_max]);
        }
        let center = [
            (bb.x_min + bb.x_max) / 2.0,
            (bb.y_min + bb.y_max) / 2.0,
            (bb.z_min + bb.z_max) / 2.0,
        ];
        let dx = bb.x_max - bb.x_min;
        let dy = bb.y_max - bb.y_min;
        let dz = bb.z_max - bb.z_min;
        let diag = (dx * dx + dy * dy + dz * dz).sqrt();
        self.camera.reset_to_bounds(center, diag);
    }

    /// Builder: add an actor.
    pub fn with_actor(mut self, actor: Actor) -> Self {
        self.actors.push(actor);
        self
    }

    /// Builder: set background color.
    pub fn with_background(mut self, r: f32, g: f32, b: f32) -> Self {
        self.background = [r, g, b, 1.0];
        self
    }

    /// Builder: add a light.
    pub fn with_light(mut self, light: Light) -> Self {
        self.lights.push(light);
        self
    }

    /// Builder: enable axes widget.
    pub fn with_axes(mut self) -> Self {
        self.axes_widget = Some(AxesWidget::default());
        self
    }

    /// Builder: add a scalar bar.
    pub fn with_scalar_bar(mut self, bar: ScalarBar) -> Self {
        self.scalar_bars.push(bar);
        self
    }

    /// Builder: enable distance fog.
    pub fn with_fog(mut self, fog: Fog) -> Self {
        self.fog = fog;
        self
    }

    /// Builder: set skybox.
    pub fn with_skybox(mut self, skybox: Skybox) -> Self {
        self.skybox = skybox;
        self
    }

    /// Builder: enable shadows.
    pub fn with_shadows(mut self) -> Self {
        self.shadows = ShadowConfig::new();
        self
    }

    /// Builder: enable bloom.
    pub fn with_bloom(mut self) -> Self {
        self.bloom = BloomConfig::new();
        self
    }

    /// Builder: enable silhouette rendering.
    pub fn with_silhouette(mut self) -> Self {
        self.silhouette = SilhouetteConfig::new();
        self
    }
    /// Total number of points across all actors.
    pub fn total_points(&self) -> usize {
        self.actors.iter().map(|a| a.data.points.len()).sum()
    }

    /// Total number of polygon cells across all actors.
    pub fn total_cells(&self) -> usize {
        self.actors.iter().map(|a| a.data.total_cells()).sum()
    }

    /// Get a summary string of the scene.
    pub fn summary(&self) -> String {
        format!(
            "Scene: {} actors ({} points, {} cells), {} lights, {} clip planes",
            self.actors.len(),
            self.total_points(),
            self.total_cells(),
            self.lights.len(),
            self.clip_planes.len()
        )
    }

    /// Print detailed scene information to stdout.
    pub fn print_info(&self) {
        println!("{}", self.summary());
        println!("  Camera: {}", self.camera);
        for (i, actor) in self.actors.iter().enumerate() {
            println!("  Actor {i}: {actor}");
            println!("    Material: {}", actor.material);
        }
        for (i, light) in self.lights.iter().enumerate() {
            println!("  Light {i}: {light}");
        }
        if self.fog.enabled {
            println!(
                "  Fog: {:?} near={:.1} far={:.1}",
                self.fog.mode, self.fog.near, self.fog.far
            );
        }
        if !self.clip_planes.is_empty() {
            println!("  Clip planes: {}", self.clip_planes.len());
        }
        if !self.volumes.is_empty() {
            println!("  Volumes: {}", self.volumes.len());
        }
    }

    /// Pick the closest actor at a screen coordinate.
    ///
    /// Convenience wrapper around `crate::render::pick()`. Returns the actor index
    /// and world position, or None if nothing was hit.
    pub fn find_actor_at(
        &self,
        screen_x: f64,
        screen_y: f64,
        width: u32,
        height: u32,
    ) -> Option<(usize, [f64; 3])> {
        let result = crate::render::pick(self, screen_x, screen_y, width, height)?;
        Some((result.actor_index, result.position))
    }
}

impl std::fmt::Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Scene: {} actors, {} lights, {} clip planes",
            self.actors.len(),
            self.lights.len(),
            self.clip_planes.len()
        )
    }
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Actor: {} points, {:?}, opacity={:.1}",
            self.data.points.len(),
            self.representation,
            self.opacity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scene() {
        let s = Scene::new();
        assert_eq!(s.num_actors(), 0);
        assert_eq!(s.lights.len(), 1); // default headlight
        assert!(s.scalar_bars.is_empty());
        assert!(s.clip_planes.is_empty());
    }

    #[test]
    fn add_remove_actors() {
        let mut s = Scene::new();
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        s.add_actor(Actor::new(pd));
        assert_eq!(s.num_actors(), 1);
        s.clear_actors();
        assert_eq!(s.num_actors(), 0);
    }

    #[test]
    fn actor_builders() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let actor = Actor::new(pd).with_color(1.0, 0.0, 0.0);
        match actor.coloring {
            Coloring::Solid(c) => assert_eq!(c, [1.0, 0.0, 0.0]),
            _ => panic!("expected Solid"),
        }
    }

    #[test]
    fn reset_camera() {
        let mut s = Scene::new();
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        s.add_actor(Actor::new(pd));
        s.reset_camera();
        assert!(s.camera.distance() > 5.0);
    }

    #[test]
    fn clip_planes() {
        let mut s = Scene::new();
        s.add_clip_plane(crate::render::ClipPlane::x(0.0));
        assert_eq!(s.clip_planes.len(), 1);
    }

    #[test]
    fn scene_builder() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scene = Scene::new()
            .with_actor(Actor::new(pd).with_color(1.0, 0.0, 0.0))
            .with_background(0.2, 0.3, 0.4)
            .with_axes();
        assert_eq!(scene.num_actors(), 1);
        assert!((scene.background[0] - 0.2).abs() < 0.01);
        assert!(scene.axes_widget.is_some());
    }

    #[test]
    fn actor_position_scale() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let actor = Actor::new(pd)
            .with_position(5.0, 0.0, 0.0)
            .with_scale(2.0)
            .with_opacity(0.5);
        assert_eq!(actor.position, [5.0, 0.0, 0.0]);
        assert_eq!(actor.scale, 2.0);
        assert_eq!(actor.opacity, 0.5);
    }

    #[test]
    fn actor_visibility() {
        let pd = PolyData::new();
        let actor = Actor::new(pd).with_visible(false);
        assert!(!actor.visible);
    }

    #[test]
    fn display_scene() {
        let scene = Scene::new().with_actor(Actor::new(PolyData::new()));
        let s = format!("{scene}");
        assert!(s.contains("1 actors"));
    }

    #[test]
    fn display_actor() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let actor = Actor::new(pd);
        let s = format!("{actor}");
        assert!(s.contains("3 points"));
        assert!(s.contains("Surface"));
    }

    #[test]
    fn from_poly_data() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scene = Scene::from_poly_data(pd);
        assert_eq!(scene.num_actors(), 1);
        assert!(scene.camera.distance() > 0.1);
    }

    #[test]
    fn from_meshes() {
        let pd1 = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let pd2 = PolyData::from_triangles(
            vec![[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scene = Scene::from_meshes(vec![(pd1, [1.0, 0.0, 0.0]), (pd2, [0.0, 0.0, 1.0])]);
        assert_eq!(scene.num_actors(), 2);
    }

    #[test]
    fn total_points_cells() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scene = Scene::new()
            .with_actor(Actor::new(pd.clone()))
            .with_actor(Actor::new(pd));
        assert_eq!(scene.total_points(), 6);
        assert_eq!(scene.total_cells(), 2);
    }

    #[test]
    fn scene_summary() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scene = Scene::new().with_actor(Actor::new(pd));
        let s = scene.summary();
        assert!(s.contains("1 actors"));
        assert!(s.contains("3 points"));
    }

    #[test]
    fn actor_colored() {
        let pd = PolyData::new();
        let actor = Actor::colored(pd, 1.0, 0.0, 0.0);
        match actor.coloring {
            Coloring::Solid(c) => assert_eq!(c, [1.0, 0.0, 0.0]),
            _ => panic!("expected Solid"),
        }
    }

    #[test]
    fn points_partial_eq() {
        let a = crate::data::Points::from_vec(vec![[1.0, 2.0, 3.0]]);
        let b = crate::data::Points::from_vec(vec![[1.0, 2.0, 3.0]]);
        let c = crate::data::Points::from_vec(vec![[4.0, 5.0, 6.0]]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn advanced_builders() {
        let scene = Scene::new()
            .with_skybox(crate::render::Skybox::sky())
            .with_fog(crate::render::Fog::linear(10.0, 100.0))
            .with_bloom()
            .with_shadows();
        assert!(scene.fog.enabled);
        assert!(scene.bloom.enabled);
        assert!(scene.shadows.enabled);
        assert!(matches!(
            scene.skybox,
            crate::render::Skybox::ThreeStop { .. }
        ));
    }
}
