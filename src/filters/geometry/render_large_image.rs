//! Tile-based large image rendering utilities.
//!
//! Splits a viewport into tiles for rendering images larger than GPU
//! texture limits, then composites the tiles into a final image.

/// A tile specification for large image rendering.
#[derive(Debug, Clone)]
pub struct RenderTile {
    /// Tile column index.
    pub col: usize,
    /// Tile row index.
    pub row: usize,
    /// Pixel x start in the final image.
    pub x: usize,
    /// Pixel y start in the final image.
    pub y: usize,
    /// Tile width in pixels.
    pub width: usize,
    /// Tile height in pixels.
    pub height: usize,
    /// Camera viewport fraction: [left, right, bottom, top] in [0,1].
    pub viewport: [f64; 4],
}

/// Compute tiles for a large image.
///
/// Splits a `total_width x total_height` image into tiles of at most
/// `tile_size x tile_size` pixels.
pub fn compute_render_tiles(
    total_width: usize,
    total_height: usize,
    tile_size: usize,
) -> Vec<RenderTile> {
    let n_cols = (total_width + tile_size - 1) / tile_size;
    let n_rows = (total_height + tile_size - 1) / tile_size;
    let mut tiles = Vec::with_capacity(n_cols * n_rows);

    for row in 0..n_rows {
        for col in 0..n_cols {
            let x = col * tile_size;
            let y = row * tile_size;
            let w = (total_width - x).min(tile_size);
            let h = (total_height - y).min(tile_size);

            let left = x as f64 / total_width as f64;
            let right = (x + w) as f64 / total_width as f64;
            let bottom = y as f64 / total_height as f64;
            let top = (y + h) as f64 / total_height as f64;

            tiles.push(RenderTile {
                col,
                row,
                x,
                y,
                width: w,
                height: h,
                viewport: [left, right, bottom, top],
            });
        }
    }

    tiles
}

/// Composite tile RGBA buffers into a final image buffer.
///
/// `tile_buffers` is a list of (tile, rgba_data) pairs.
/// Returns the composited RGBA buffer for the full image.
pub fn composite_tiles(
    total_width: usize,
    total_height: usize,
    tile_buffers: &[(RenderTile, Vec<u8>)],
) -> Vec<u8> {
    let mut result = vec![0u8; total_width * total_height * 4];

    for (tile, rgba) in tile_buffers {
        for ty in 0..tile.height {
            for tx in 0..tile.width {
                let src_idx = (ty * tile.width + tx) * 4;
                let dst_x = tile.x + tx;
                let dst_y = tile.y + ty;
                let dst_idx = (dst_y * total_width + dst_x) * 4;

                if src_idx + 3 < rgba.len() && dst_idx + 3 < result.len() {
                    result[dst_idx] = rgba[src_idx];
                    result[dst_idx + 1] = rgba[src_idx + 1];
                    result[dst_idx + 2] = rgba[src_idx + 2];
                    result[dst_idx + 3] = rgba[src_idx + 3];
                }
            }
        }
    }

    result
}

/// Get the camera frustum adjustment for a tile.
///
/// Returns (x_offset, y_offset, x_scale, y_scale) to apply to the
/// projection matrix to render only this tile's portion.
pub fn tile_frustum_params(
    tile: &RenderTile,
    total_width: usize,
    total_height: usize,
) -> (f64, f64, f64, f64) {
    let tw = total_width as f64;
    let th = total_height as f64;
    let x_scale = tw / tile.width as f64;
    let y_scale = th / tile.height as f64;
    let x_offset = -(2.0 * tile.x as f64 + tile.width as f64 - tw) / tile.width as f64;
    let y_offset = -(2.0 * tile.y as f64 + tile.height as f64 - th) / tile.height as f64;
    (x_offset, y_offset, x_scale, y_scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_tile() {
        let tiles = compute_render_tiles(512, 512, 1024);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].width, 512);
        assert_eq!(tiles[0].height, 512);
    }

    #[test]
    fn four_tiles() {
        let tiles = compute_render_tiles(1000, 1000, 512);
        assert_eq!(tiles.len(), 4); // 2x2
        assert_eq!(tiles[0].width, 512);
        assert_eq!(tiles[1].width, 488); // remainder
    }

    #[test]
    fn composite() {
        let tiles = compute_render_tiles(4, 4, 2);
        assert_eq!(tiles.len(), 4);

        let mut buffers: Vec<(RenderTile, Vec<u8>)> = Vec::new();
        for tile in &tiles {
            let rgba = vec![255u8; tile.width * tile.height * 4];
            buffers.push((tile.clone(), rgba));
        }

        let result = composite_tiles(4, 4, &buffers);
        assert_eq!(result.len(), 64); // 4*4*4
        assert!(result.iter().all(|&b| b == 255));
    }

    #[test]
    fn frustum_params() {
        let tile = RenderTile {
            col: 0,
            row: 0,
            x: 0,
            y: 0,
            width: 512,
            height: 512,
            viewport: [0.0, 0.5, 0.0, 0.5],
        };
        let (xo, yo, xs, ys) = tile_frustum_params(&tile, 1024, 1024);
        assert!((xs - 2.0).abs() < 1e-10); // rendering half width → 2x scale
    }

    #[test]
    fn viewport_coverage() {
        let tiles = compute_render_tiles(800, 600, 256);
        // Verify all pixels are covered
        let mut covered = vec![false; 800 * 600];
        for tile in &tiles {
            for y in tile.y..tile.y + tile.height {
                for x in tile.x..tile.x + tile.width {
                    covered[y * 800 + x] = true;
                }
            }
        }
        assert!(covered.iter().all(|&c| c));
    }
}
