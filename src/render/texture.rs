/// A 2D texture for mapping onto surfaces.
///
/// Stores RGBA pixel data. Used with `Coloring::Texture` to apply
/// image textures to actors that have UV (texture coordinate) data.
#[derive(Debug, Clone)]
pub struct Texture {
    /// RGBA pixel data (4 bytes per pixel, row-major, top-to-bottom).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Texture {
    /// Create a texture from RGBA data.
    pub fn from_rgba(data: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(data.len(), (width * height * 4) as usize);
        Self {
            data,
            width,
            height,
        }
    }

    /// Create a 1x1 solid color texture.
    pub fn solid(r: u8, g: u8, b: u8) -> Self {
        Self {
            data: vec![r, g, b, 255],
            width: 1,
            height: 1,
        }
    }

    /// Create a checkerboard texture for testing.
    pub fn checkerboard(size: u32, tile_size: u32) -> Self {
        let mut data = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let is_white = ((x / tile_size) + (y / tile_size)) % 2 == 0;
                let v = if is_white { 220u8 } else { 40u8 };
                data.extend_from_slice(&[v, v, v, 255]);
            }
        }
        Self {
            data,
            width: size,
            height: size,
        }
    }
}

/// A region within a texture atlas (normalized UV coordinates).
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    /// Left edge in atlas UV space (0.0 to 1.0).
    pub u_min: f32,
    /// Bottom edge in atlas UV space.
    pub v_min: f32,
    /// Right edge in atlas UV space.
    pub u_max: f32,
    /// Top edge in atlas UV space.
    pub v_max: f32,
}

impl AtlasRegion {
    /// Remap a local UV coordinate (0..1) to atlas UV space.
    pub fn remap(&self, u: f32, v: f32) -> (f32, f32) {
        (
            self.u_min + u * (self.u_max - self.u_min),
            self.v_min + v * (self.v_max - self.v_min),
        )
    }
}

/// A texture atlas that packs multiple textures into a single large texture.
///
/// Each sub-texture gets a named region with UV coordinates for lookup.
/// The atlas uses a simple shelf-packing algorithm.
///
/// # Example
///
/// ```
/// use crate::render::texture::{Texture, TextureAtlas};
///
/// let tex_a = Texture::checkerboard(64, 8);
/// let tex_b = Texture::solid(255, 0, 0);
///
/// let mut atlas = TextureAtlas::new(256);
/// atlas.add("checker", &tex_a);
/// atlas.add("red", &tex_b);
/// atlas.pack();
///
/// let region = atlas.region("checker").unwrap();
/// assert!(region.u_max > region.u_min);
/// ```
#[derive(Debug, Clone)]
pub struct TextureAtlas {
    /// Maximum atlas dimension (width = height).
    pub max_size: u32,
    /// Packed atlas texture (created after `pack()`).
    pub texture: Option<Texture>,
    /// Named texture entries before packing.
    entries: Vec<(String, Texture)>,
    /// Regions after packing, indexed by entry order.
    regions: Vec<(String, AtlasRegion)>,
}

impl TextureAtlas {
    /// Create a new empty atlas with the given max dimension.
    pub fn new(max_size: u32) -> Self {
        Self {
            max_size,
            texture: None,
            entries: Vec::new(),
            regions: Vec::new(),
        }
    }

    /// Add a texture to be packed. Call `pack()` after adding all textures.
    pub fn add(&mut self, name: &str, texture: &Texture) {
        self.entries.push((name.to_string(), texture.clone()));
        // Invalidate previous packing
        self.texture = None;
        self.regions.clear();
    }

    /// Pack all added textures into the atlas using shelf packing.
    ///
    /// Textures are sorted by height (tallest first) and placed left-to-right
    /// in rows (shelves). Returns true if all textures fit.
    pub fn pack(&mut self) -> bool {
        if self.entries.is_empty() {
            let data = vec![0u8; (self.max_size * self.max_size * 4) as usize];
            self.texture = Some(Texture::from_rgba(data, self.max_size, self.max_size));
            return true;
        }

        // Sort by height descending for better shelf packing
        let mut order: Vec<usize> = (0..self.entries.len()).collect();
        order.sort_by(|&a, &b| self.entries[b].1.height.cmp(&self.entries[a].1.height));

        let atlas_w = self.max_size;
        let atlas_h = self.max_size;
        let mut atlas_data = vec![0u8; (atlas_w * atlas_h * 4) as usize];

        // Shelf packing
        let mut shelf_x: u32 = 0;
        let mut shelf_y: u32 = 0;
        let mut shelf_height: u32 = 0;
        let padding = 1u32; // 1px padding to avoid bleeding

        let mut region_map: Vec<(usize, AtlasRegion)> = Vec::new();

        for &idx in &order {
            let tex = &self.entries[idx].1;
            let tw = tex.width;
            let th = tex.height;

            // Check if we need a new shelf
            if shelf_x + tw + padding > atlas_w {
                shelf_y += shelf_height + padding;
                shelf_x = 0;
                shelf_height = 0;
            }

            // Check if it fits vertically
            if shelf_y + th > atlas_h {
                return false; // doesn't fit
            }

            // Blit texture into atlas
            for row in 0..th {
                let src_start = (row * tw * 4) as usize;
                let src_end = src_start + (tw * 4) as usize;
                let dst_y = shelf_y + row;
                let dst_start = ((dst_y * atlas_w + shelf_x) * 4) as usize;
                if src_end <= tex.data.len() && dst_start + (tw * 4) as usize <= atlas_data.len() {
                    atlas_data[dst_start..dst_start + (tw * 4) as usize]
                        .copy_from_slice(&tex.data[src_start..src_end]);
                }
            }

            // Record region in normalized UV space
            let region = AtlasRegion {
                u_min: shelf_x as f32 / atlas_w as f32,
                v_min: shelf_y as f32 / atlas_h as f32,
                u_max: (shelf_x + tw) as f32 / atlas_w as f32,
                v_max: (shelf_y + th) as f32 / atlas_h as f32,
            };
            region_map.push((idx, region));

            shelf_x += tw + padding;
            shelf_height = shelf_height.max(th);
        }

        // Build regions in original insertion order
        self.regions.clear();
        let mut ordered_regions = vec![None; self.entries.len()];
        for (idx, region) in region_map {
            ordered_regions[idx] = Some(region);
        }
        for (i, entry) in self.entries.iter().enumerate() {
            if let Some(region) = ordered_regions[i] {
                self.regions.push((entry.0.clone(), region));
            }
        }

        self.texture = Some(Texture::from_rgba(atlas_data, atlas_w, atlas_h));
        true
    }

    /// Get the atlas region for a named texture.
    pub fn region(&self, name: &str) -> Option<AtlasRegion> {
        self.regions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
    }

    /// Get the atlas region by index.
    pub fn region_by_index(&self, idx: usize) -> Option<AtlasRegion> {
        self.regions.get(idx).map(|(_, r)| *r)
    }

    /// Number of textures in the atlas.
    pub fn num_textures(&self) -> usize {
        self.entries.len()
    }

    /// Get the packed atlas texture. Returns None if `pack()` hasn't been called.
    pub fn atlas_texture(&self) -> Option<&Texture> {
        self.texture.as_ref()
    }

    /// Remap UV coordinates from local texture space to atlas space.
    ///
    /// Given a texture name and local (u, v) in [0,1], returns atlas (u, v).
    pub fn remap_uv(&self, name: &str, u: f32, v: f32) -> Option<(f32, f32)> {
        self.region(name).map(|r| r.remap(u, v))
    }

    /// Create a `Texture` that can be used with `Coloring::TextureMap`,
    /// along with remapped UV coordinates for a specific sub-texture.
    ///
    /// This enables atlas usage with the existing single-texture rendering path:
    /// the actor uses the atlas as its texture, and its UVs are remapped to the
    /// sub-texture's region.
    pub fn texture_and_region(&self, name: &str) -> Option<(Texture, AtlasRegion)> {
        let tex = self.texture.as_ref()?;
        let region = self.region(name)?;
        Some((tex.clone(), region))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_texture() {
        let t = Texture::solid(255, 0, 0);
        assert_eq!(t.width, 1);
        assert_eq!(t.height, 1);
        assert_eq!(t.data, vec![255, 0, 0, 255]);
    }

    #[test]
    fn checkerboard_texture() {
        let t = Texture::checkerboard(8, 4);
        assert_eq!(t.width, 8);
        assert_eq!(t.height, 8);
        assert_eq!(t.data.len(), 8 * 8 * 4);
    }

    #[test]
    fn from_rgba() {
        let data = vec![0u8; 4 * 2 * 2];
        let t = Texture::from_rgba(data, 2, 2);
        assert_eq!(t.width, 2);
    }

    #[test]
    fn atlas_pack_two_textures() {
        let a = Texture::checkerboard(32, 8);
        let b = Texture::checkerboard(16, 4);

        let mut atlas = TextureAtlas::new(128);
        atlas.add("big", &a);
        atlas.add("small", &b);
        assert!(atlas.pack(), "should fit in 128x128");
        assert!(atlas.atlas_texture().is_some());
        assert_eq!(atlas.num_textures(), 2);

        let r1 = atlas.region("big").unwrap();
        let r2 = atlas.region("small").unwrap();
        // Regions should not overlap
        assert!(
            r1.u_max <= r2.u_min
                || r2.u_max <= r1.u_min
                || r1.v_max <= r2.v_min
                || r2.v_max <= r1.v_min,
            "regions should not overlap"
        );
    }

    #[test]
    fn atlas_remap_uv() {
        let a = Texture::solid(255, 0, 0);
        let mut atlas = TextureAtlas::new(64);
        atlas.add("red", &a);
        atlas.pack();

        let (u, v) = atlas.remap_uv("red", 0.5, 0.5).unwrap();
        let region = atlas.region("red").unwrap();
        assert!(u >= region.u_min && u <= region.u_max);
        assert!(v >= region.v_min && v <= region.v_max);
    }

    #[test]
    fn atlas_region_remap() {
        let region = AtlasRegion {
            u_min: 0.25,
            v_min: 0.0,
            u_max: 0.75,
            v_max: 0.5,
        };
        let (u, v) = region.remap(0.0, 0.0);
        assert!((u - 0.25).abs() < 1e-6);
        assert!((v - 0.0).abs() < 1e-6);
        let (u, v) = region.remap(1.0, 1.0);
        assert!((u - 0.75).abs() < 1e-6);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn atlas_overflow() {
        let big = Texture::checkerboard(128, 8);
        let mut atlas = TextureAtlas::new(64);
        atlas.add("too_big", &big);
        assert!(
            !atlas.pack(),
            "128x128 texture should not fit in 64x64 atlas"
        );
    }

    #[test]
    fn atlas_texture_and_region() {
        let a = Texture::checkerboard(16, 4);
        let b = Texture::solid(0, 255, 0);
        let mut atlas = TextureAtlas::new(64);
        atlas.add("checker", &a);
        atlas.add("green", &b);
        atlas.pack();

        let (tex, region) = atlas.texture_and_region("checker").unwrap();
        assert_eq!(tex.width, 64);
        assert!(region.u_max > region.u_min);
    }
}
