//! TrueType font rendering via fontdue.
//!
//! Provides high-quality text rasterization at any size, producing
//! either RGBA textures or coverage bitmaps for overlay rendering.

use fontdue::{Font, FontSettings};

/// A loaded TrueType font ready for rasterization.
#[derive(Clone)]
pub struct TrueTypeFont {
    font: Font,
}

impl std::fmt::Debug for TrueTypeFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrueTypeFont").finish()
    }
}

/// A rasterized glyph with position and coverage bitmap.
#[derive(Debug, Clone)]
pub struct RasterizedGlyph {
    /// Character this glyph represents.
    pub character: char,
    /// Width of the glyph bitmap in pixels.
    pub width: usize,
    /// Height of the glyph bitmap in pixels.
    pub height: usize,
    /// Coverage bitmap (0..255 per pixel, row-major).
    pub bitmap: Vec<u8>,
    /// Horizontal advance to next glyph in pixels.
    pub advance_width: f32,
    /// X offset from cursor to bitmap left edge.
    pub x_offset: f32,
    /// Y offset from baseline to bitmap top edge.
    pub y_offset: f32,
}

/// A laid-out text string with positioned glyphs.
#[derive(Debug, Clone)]
pub struct TextLayout {
    /// Individual glyph positions and bitmaps.
    pub glyphs: Vec<PositionedGlyph>,
    /// Total width of the laid-out text in pixels.
    pub total_width: f32,
    /// Line height in pixels.
    pub line_height: f32,
}

/// A glyph with its position in the layout.
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// Rasterized glyph data.
    pub glyph: RasterizedGlyph,
    /// X position of bitmap left edge in layout space.
    pub x: f32,
    /// Y position of bitmap top edge in layout space.
    pub y: f32,
}

impl TrueTypeFont {
    /// Load a TrueType font from raw .ttf/.otf bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        let font = Font::from_bytes(data, FontSettings::default())
            .map_err(|e| format!("failed to load font: {}", e))?;
        Ok(Self { font })
    }

    /// Load the built-in fallback font (Liberation Sans subset).
    /// Returns None if no built-in font is available.
    pub fn builtin() -> Option<Self> {
        // We don't bundle a font file — users must provide one.
        // This is a placeholder for when a default font is embedded.
        None
    }

    /// Rasterize a single character at the given pixel size.
    pub fn rasterize(&self, ch: char, px_size: f32) -> RasterizedGlyph {
        let (metrics, bitmap) = self.font.rasterize(ch, px_size);
        RasterizedGlyph {
            character: ch,
            width: metrics.width,
            height: metrics.height,
            bitmap,
            advance_width: metrics.advance_width,
            x_offset: metrics.xmin as f32,
            y_offset: metrics.ymin as f32,
        }
    }

    /// Lay out a text string at the given pixel size.
    ///
    /// Returns positioned glyphs and total dimensions.
    pub fn layout(&self, text: &str, px_size: f32) -> TextLayout {
        let mut glyphs = Vec::new();
        let mut cursor_x: f32 = 0.0;

        let metrics_a = self.font.rasterize('A', px_size).0;
        let line_height = px_size;

        for ch in text.chars() {
            if ch == ' ' {
                let (m, _) = self.font.rasterize(' ', px_size);
                cursor_x += m.advance_width.max(px_size * 0.25);
                continue;
            }
            if ch == '\n' {
                // Simple: skip newlines for single-line layout
                continue;
            }

            let glyph = self.rasterize(ch, px_size);
            let x = cursor_x + glyph.x_offset;
            let y = glyph.y_offset;

            cursor_x += glyph.advance_width;

            glyphs.push(PositionedGlyph { glyph, x, y });
        }

        TextLayout {
            glyphs,
            total_width: cursor_x,
            line_height,
        }
    }

    /// Render text to an RGBA image buffer.
    ///
    /// Returns (pixels, width, height) where pixels is RGBA u8 data.
    pub fn render_to_rgba(&self, text: &str, px_size: f32, color: [u8; 3]) -> (Vec<u8>, u32, u32) {
        let layout = self.layout(text, px_size);

        let img_w = (layout.total_width.ceil() as u32).max(1);
        let img_h = (layout.line_height.ceil() as u32).max(1);
        let mut pixels = vec![0u8; (img_w * img_h * 4) as usize];

        for pg in &layout.glyphs {
            let gx = pg.x.round() as i32;
            // Align glyph to baseline
            let gy = (img_h as f32 - layout.line_height + pg.y).round() as i32;

            for row in 0..pg.glyph.height {
                for col in 0..pg.glyph.width {
                    let coverage = pg.glyph.bitmap[row * pg.glyph.width + col];
                    if coverage == 0 {
                        continue;
                    }

                    let px = gx + col as i32;
                    let py = gy + row as i32;
                    if px < 0 || py < 0 || px >= img_w as i32 || py >= img_h as i32 {
                        continue;
                    }

                    let idx = ((py as u32 * img_w + px as u32) * 4) as usize;
                    if idx + 3 < pixels.len() {
                        // Alpha-blend coverage onto buffer
                        let alpha = coverage as f32 / 255.0;
                        let inv = 1.0 - alpha;
                        pixels[idx] = (color[0] as f32 * alpha + pixels[idx] as f32 * inv) as u8;
                        pixels[idx + 1] =
                            (color[1] as f32 * alpha + pixels[idx + 1] as f32 * inv) as u8;
                        pixels[idx + 2] =
                            (color[2] as f32 * alpha + pixels[idx + 2] as f32 * inv) as u8;
                        pixels[idx + 3] = ((alpha * 255.0) as u8).max(pixels[idx + 3]);
                    }
                }
            }
        }

        (pixels, img_w, img_h)
    }

    /// Render text to a `Texture` for use with the rendering system.
    pub fn render_to_texture(
        &self,
        text: &str,
        px_size: f32,
        color: [u8; 3],
    ) -> crate::render::Texture {
        let (pixels, w, h) = self.render_to_rgba(text, px_size, color);
        crate::render::Texture::from_rgba(pixels, w, h)
    }

    /// Generate overlay-compatible vertex data for text rendering.
    ///
    /// Returns (positions, colors, indices) where positions are in
    /// normalized [0,1] screen coordinates. Each glyph pixel becomes
    /// a small quad.
    pub fn render_to_overlay_quads(
        &self,
        text: &str,
        px_size: f32,
        screen_x: f32,
        screen_y: f32,
        screen_height: f32,
        color: [f32; 4],
        screen_width_px: f32,
        screen_height_px: f32,
    ) -> (Vec<[f32; 2]>, Vec<[f32; 4]>, Vec<u32>) {
        let layout = self.layout(text, px_size);

        let mut positions = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        let scale_x = screen_height / screen_width_px;
        let scale_y = screen_height / screen_height_px;

        for pg in &layout.glyphs {
            for row in 0..pg.glyph.height {
                for col in 0..pg.glyph.width {
                    let coverage = pg.glyph.bitmap[row * pg.glyph.width + col];
                    if coverage < 32 {
                        continue;
                    } // skip near-zero

                    let px = pg.x + col as f32;
                    let py = pg.y + row as f32;

                    let nx = screen_x + px * scale_x;
                    let ny = screen_y - py * scale_y;
                    let qw = scale_x;
                    let qh = scale_y;

                    let alpha = coverage as f32 / 255.0 * color[3];
                    let c = [color[0], color[1], color[2], alpha];

                    let base = positions.len() as u32;
                    positions.push([nx, ny]);
                    positions.push([nx + qw, ny]);
                    positions.push([nx + qw, ny + qh]);
                    positions.push([nx, ny + qh]);
                    colors.push(c);
                    colors.push(c);
                    colors.push(c);
                    colors.push(c);
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }

        (positions, colors, indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // We can't test with a real font file in unit tests without bundling one,
    // so test the API structure and glyph types.

    #[test]
    fn rasterized_glyph_struct() {
        let g = RasterizedGlyph {
            character: 'A',
            width: 10,
            height: 12,
            bitmap: vec![128; 120],
            advance_width: 11.0,
            x_offset: 0.0,
            y_offset: -10.0,
        };
        assert_eq!(g.character, 'A');
        assert_eq!(g.bitmap.len(), 120);
    }

    #[test]
    fn text_layout_struct() {
        let layout = TextLayout {
            glyphs: vec![],
            total_width: 100.0,
            line_height: 16.0,
        };
        assert_eq!(layout.total_width, 100.0);
    }

    #[test]
    fn builtin_not_available() {
        // No built-in font bundled yet
        assert!(TrueTypeFont::builtin().is_none());
    }

    #[test]
    fn load_system_font_and_rasterize() {
        // Try to load a system font if available
        let paths = [
            "/usr/share/fonts/truetype/freefont/FreeMono.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
        ];
        let font = paths.iter().find_map(|p| {
            std::fs::read(p)
                .ok()
                .and_then(|data| TrueTypeFont::from_bytes(&data).ok())
        });
        let Some(font) = font else {
            // No system font found — skip test
            return;
        };

        // Rasterize a single character
        let glyph = font.rasterize('A', 24.0);
        assert!(glyph.width > 0, "glyph should have nonzero width");
        assert!(glyph.height > 0, "glyph should have nonzero height");
        assert!(!glyph.bitmap.is_empty());

        // Layout a string
        let layout = font.layout("Hello", 24.0);
        assert!(layout.total_width > 0.0);
        assert!(!layout.glyphs.is_empty());

        // Render to RGBA
        let (pixels, w, h) = font.render_to_rgba("Hi", 32.0, [255, 255, 255]);
        assert!(w > 0);
        assert!(h > 0);
        // Should have some non-zero alpha pixels
        let has_content = pixels.chunks(4).any(|p| p[3] > 0);
        assert!(has_content, "rendered text should have visible pixels");

        // Render to texture
        let tex = font.render_to_texture("VTK", 16.0, [200, 200, 200]);
        assert!(tex.width > 0);
        assert!(tex.height > 0);
    }
}
