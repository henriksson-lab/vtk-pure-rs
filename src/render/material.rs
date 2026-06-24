/// Surface material properties for Phong/PBR shading.
///
/// Analogous to VTK's `vtkProperty`. Controls how light interacts
/// with the surface.
///
/// # Examples
///
/// ```
/// use crate::render::Material;
///
/// let matte = Material::matte();
/// assert_eq!(matte.specular, 0.0);
///
/// let metal = Material::pbr_metal(0.3);
/// assert!(metal.pbr);
/// assert_eq!(metal.metallic, 1.0);
/// ```
#[derive(Debug, Clone)]
pub struct Material {
    /// Ambient light contribution. Default: 0.1
    pub ambient: f64,
    /// Diffuse light contribution. Default: 0.7
    pub diffuse: f64,
    /// Specular highlight contribution. Default: 0.3
    pub specular: f64,
    /// Specular power (shininess). Higher = tighter highlights. Default: 32.0
    pub specular_power: f64,
    /// Specular highlight color. Default: white [1, 1, 1]
    pub specular_color: [f32; 3],
    /// Edge color for wireframe/edge overlay. Default: black [0, 0, 0]
    pub edge_color: [f32; 3],
    /// Edge visibility (for edge overlay mode). Default: false
    pub edge_visibility: bool,
    /// Line width for wireframe/edge rendering. Default: 1.0
    pub line_width: f32,
    /// Point size for point rendering. Default: 3.0
    pub point_size: f32,
    /// Whether to use flat shading (per-face normals). Default: false
    pub flat_shading: bool,
    /// Backface culling. Default: false
    pub backface_culling: bool,
    /// Metallic factor for PBR (0.0 = dielectric, 1.0 = metal). Default: 0.0
    pub metallic: f64,
    /// Roughness factor for PBR (0.0 = mirror, 1.0 = fully rough). Default: 0.5
    pub roughness: f64,
    /// Whether to use PBR shading instead of Blinn-Phong. Default: false
    pub pbr: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ambient: 0.1,
            diffuse: 0.7,
            specular: 0.3,
            specular_power: 32.0,
            specular_color: [1.0, 1.0, 1.0],
            edge_color: [0.0, 0.0, 0.0],
            edge_visibility: false,
            line_width: 1.0,
            point_size: 3.0,
            flat_shading: false,
            backface_culling: false,
            metallic: 0.0,
            roughness: 0.5,
            pbr: false,
        }
    }
}

impl Material {
    /// Create a default material.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a matte (no specular) material.
    pub fn matte() -> Self {
        Self {
            specular: 0.0,
            ..Default::default()
        }
    }

    /// Create a shiny/metallic material.
    pub fn shiny() -> Self {
        Self {
            specular: 0.8,
            specular_power: 128.0,
            diffuse: 0.5,
            ..Default::default()
        }
    }

    /// Create a flat-shaded material (per-face coloring).
    pub fn flat() -> Self {
        Self {
            flat_shading: true,
            ..Default::default()
        }
    }

    /// Create a PBR metal material.
    pub fn pbr_metal(roughness: f64) -> Self {
        Self {
            pbr: true,
            metallic: 1.0,
            roughness,
            ..Default::default()
        }
    }

    /// Create a PBR dielectric (plastic/ceramic) material.
    pub fn pbr_dielectric(roughness: f64) -> Self {
        Self {
            pbr: true,
            metallic: 0.0,
            roughness,
            ..Default::default()
        }
    }

    /// Create a material with edge overlay.
    pub fn with_edges(mut self) -> Self {
        self.edge_visibility = true;
        self
    }
}

impl std::fmt::Display for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.pbr {
            write!(
                f,
                "Material(PBR: metallic={:.1}, roughness={:.1})",
                self.metallic, self.roughness
            )
        } else {
            write!(
                f,
                "Material(Phong: ambient={:.1}, diffuse={:.1}, specular={:.1})",
                self.ambient, self.diffuse, self.specular
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material() {
        let m = Material::default();
        assert_eq!(m.ambient, 0.1);
        assert_eq!(m.diffuse, 0.7);
        assert!(!m.flat_shading);
    }

    #[test]
    fn matte_no_specular() {
        let m = Material::matte();
        assert_eq!(m.specular, 0.0);
    }

    #[test]
    fn shiny_high_power() {
        let m = Material::shiny();
        assert_eq!(m.specular_power, 128.0);
    }

    #[test]
    fn with_edges_builder() {
        let m = Material::flat().with_edges();
        assert!(m.flat_shading);
        assert!(m.edge_visibility);
    }

    #[test]
    fn pbr_metal() {
        let m = Material::pbr_metal(0.3);
        assert!(m.pbr);
        assert_eq!(m.metallic, 1.0);
        assert_eq!(m.roughness, 0.3);
    }

    #[test]
    fn pbr_dielectric() {
        let m = Material::pbr_dielectric(0.8);
        assert!(m.pbr);
        assert_eq!(m.metallic, 0.0);
        assert_eq!(m.roughness, 0.8);
    }

    #[test]
    fn display_phong() {
        let m = Material::default();
        let s = format!("{m}");
        assert!(s.contains("Phong"));
    }

    #[test]
    fn display_pbr() {
        let m = Material::pbr_metal(0.3);
        let s = format!("{m}");
        assert!(s.contains("PBR"));
        assert!(s.contains("metallic"));
    }
}
