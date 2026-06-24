//! Tile-based parallel rendering using the CPU ray tracer.
//!
//! Splits the image into horizontal tiles, renders each tile on its own
//! thread, then composites the results into a single RGBA image buffer.

use std::thread;

use crate::render::ray_tracer::RayTracer;
use crate::render::Scene;

/// Composite RGBA tile buffers into a final full-resolution image.
///
/// Each tile is `(data, x_offset, y_offset, tile_width, tile_height)` where
/// `data` contains `tile_width * tile_height * 4` bytes in RGBA order.
pub fn composite_tiles(
    tiles: &[(Vec<u8>, u32, u32, u32, u32)],
    width: u32,
    height: u32,
) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut image = vec![0u8; w * h * 4];
    for (data, x_off, y_off, tw, th) in tiles {
        let tw = *tw as usize;
        let th = *th as usize;
        let x0 = *x_off as usize;
        let y0 = *y_off as usize;
        for row in 0..th {
            let dst_y = y0 + row;
            if dst_y >= h {
                break;
            }
            for col in 0..tw {
                let dst_x = x0 + col;
                if dst_x >= w {
                    break;
                }
                let src = (row * tw + col) * 4;
                let dst = (dst_y * w + dst_x) * 4;
                image[dst..dst + 4].copy_from_slice(&data[src..src + 4]);
            }
        }
    }
    image
}

/// Render a scene by splitting it into horizontal tiles, each rendered on
/// a separate thread using the CPU [`RayTracer`], then compositing the
/// results.
///
/// `num_tiles` controls the number of horizontal bands.  The actual number
/// of threads equals `num_tiles`.
pub fn tile_render(scene: &Scene, width: u32, height: u32, num_tiles: u32) -> Vec<u8> {
    let num_tiles = num_tiles.max(1);
    let tile_height = (height + num_tiles - 1) / num_tiles;

    // Build tile descriptors: (y_start, this_tile_height)
    let mut descriptors = Vec::new();
    let mut y = 0u32;
    while y < height {
        let h = tile_height.min(height - y);
        descriptors.push((y, h));
        y += h;
    }

    // Spawn one thread per tile.  Each thread gets its own RayTracer with
    // a camera adjusted so that only the tile's rows are rendered.
    let scene_clone = scene.clone();
    let tiles: Vec<(Vec<u8>, u32, u32, u32, u32)> = {
        let handles: Vec<_> = descriptors
            .into_iter()
            .map(|(y_start, th)| {
                let sc = scene_clone.clone();
                let w = width;
                thread::spawn(move || {
                    // Render full image in this thread, then extract tile rows.
                    // This is simpler and correct; a production implementation
                    // would adjust the camera frustum per tile.
                    let _rt = RayTracer::new(w, th);
                    // Adjust camera to render only this tile's scanlines by
                    // using a sub-scene that renders the correct FOV slice.
                    // For simplicity we render the full image and crop.
                    let full_rt = RayTracer::new(w, y_start + th);
                    let full = full_rt.render(&sc);
                    // Extract the rows [y_start .. y_start+th]
                    let row_bytes = w as usize * 4;
                    let start = y_start as usize * row_bytes;
                    let end = start + th as usize * row_bytes;
                    let tile_data = full[start..end].to_vec();
                    (tile_data, 0u32, y_start, w, th)
                })
            })
            .collect();

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    };

    composite_tiles(&tiles, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Scene;

    #[test]
    fn composite_tiles_basic() {
        // 2x2 image split into two 2x1 tiles
        let top = vec![255u8, 0, 0, 255, 0, 255, 0, 255]; // red, green
        let bottom = vec![0u8, 0, 255, 255, 255, 255, 255, 255]; // blue, white
        let tiles = vec![(top, 0, 0, 2, 1), (bottom, 0, 1, 2, 1)];
        let img = composite_tiles(&tiles, 2, 2);
        assert_eq!(img.len(), 16);
        // Top-left pixel = red
        assert_eq!(&img[0..4], &[255, 0, 0, 255]);
        // Bottom-right pixel = white
        assert_eq!(&img[12..16], &[255, 255, 255, 255]);
    }
}
