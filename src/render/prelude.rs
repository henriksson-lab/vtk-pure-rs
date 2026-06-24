//! Convenience re-exports for common vtk-render types.
//!
//! ```
//! use crate::render::prelude::*;
//! ```

pub use crate::render::measurement::MeshMeasurements;
pub use crate::render::viewport::Viewport;
pub use crate::render::{
    Actor, AngleProtractor, Annotations, AxesWidget, BloomConfig, Camera, CameraAnimation,
    ClipPlane, ColorMap, Coloring, DistanceRuler, Easing, Fog, FogMode, GlyphInstance,
    InstancedGlyphs, Keyframe, Label3D, Light, LightType, LodLevel, LodSet, Material, PickResult,
    Renderer, Representation, ScalarBar, ScalarBarOrientation, Scene, ShadowConfig,
    SilhouetteConfig, Skybox, StereoConfig, StereoMode, Texture, Track, TransferFunction,
    VolumeActor,
};
