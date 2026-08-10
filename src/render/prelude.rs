//! Convenience re-exports for common vtk-render types.
//!
//! ```
//! use vtk_pure_rs::render::prelude::*;
//!
//! let scene = Scene::new()
//!     .with_background(0.1, 0.1, 0.15)
//!     .with_actor(Actor::new(vtk_pure_rs::data::PolyData::new()));
//! assert_eq!(scene.num_actors(), 1);
//!
//! let cmap = ColorMap::viridis();
//! let camera = Camera::new();
//! let material = Material::matte();
//! assert_eq!(material.specular, 0.0);
//! assert!(cmap.map(0.5)[1] >= 0.0);
//! assert!(camera.distance() > 0.0);
//! ```

pub use crate::render::measurement::MeshMeasurements;
pub use crate::render::viewport::Viewport;
pub use crate::render::{
    Actor, AngleProtractor, Annotations, AtlasRegion, AxesWidget, BloomConfig, Camera,
    CameraAnimation, ClipPlane, ColorMap, Coloring, DistanceRuler, DofConfig, Easing,
    EnvironmentMap, Fog, FogMode, GlyphInstance, ImpostorConfig, InstancedGlyphs, Keyframe,
    Label3D, Light, LightType, LodLevel, LodSet, Material, PathTracer, PickResult, RayTracer,
    Renderer, Representation, ScalarBar, ScalarBarOrientation, Scene, ShadowConfig,
    SilhouetteConfig, Skybox, SsaoConfig, StereoConfig, StereoMode, SubdivisionConfig, Texture,
    TextureAtlas, Track, TransferFunction, VolumeActor, WebViewerConfig,
};
