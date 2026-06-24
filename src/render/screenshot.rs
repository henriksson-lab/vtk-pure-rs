use std::io::Write;
use std::path::Path;

/// Save raw RGBA pixel data as a PPM image file (simple, no dependencies).
///
/// PPM is a simple uncompressed image format readable by most image viewers.
pub fn save_ppm(path: &Path, rgba: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;
    writeln!(f, "P6")?;
    writeln!(f, "{width} {height}")?;
    writeln!(f, "255")?;
    for pixel in rgba.chunks_exact(4) {
        f.write_all(&[pixel[0], pixel[1], pixel[2]])?; // RGB only, skip A
    }
    Ok(())
}

/// Save raw RGBA pixel data as a BMP image file (no dependencies).
pub fn save_bmp(path: &Path, rgba: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;

    let row_size = width * 3;
    let padding = (4 - (row_size % 4)) % 4;
    let padded_row = row_size + padding;
    let pixel_data_size = padded_row * height;
    let file_size = 54 + pixel_data_size;

    // BMP header
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&[0u8; 4])?; // reserved
    f.write_all(&54u32.to_le_bytes())?; // pixel data offset

    // DIB header (BITMAPINFOHEADER)
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(width as i32).to_le_bytes())?;
    f.write_all(&(height as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // planes
    f.write_all(&24u16.to_le_bytes())?; // bits per pixel
    f.write_all(&[0u8; 24])?; // compression, sizes, etc.

    // Pixel data (bottom-to-top, BGR)
    for row in (0..height).rev() {
        let row_start = (row * width * 4) as usize;
        for col in 0..width as usize {
            let idx = row_start + col * 4;
            f.write_all(&[rgba[idx + 2], rgba[idx + 1], rgba[idx]])?; // BGR
        }
        for _ in 0..padding {
            f.write_all(&[0u8])?;
        }
    }

    Ok(())
}

/// Save raw RGBA pixel data as a TGA image file (no dependencies).
pub fn save_tga(path: &Path, rgba: &[u8], width: u32, height: u32) -> std::io::Result<()> {
    let mut f = std::fs::File::create(path)?;

    // TGA header
    let header: [u8; 18] = [
        0, // ID length
        0, // color map type
        2, // image type (uncompressed true-color)
        0,
        0,
        0,
        0,
        0, // color map spec
        0,
        0, // x origin
        0,
        0, // y origin
        (width & 0xFF) as u8,
        (width >> 8) as u8,
        (height & 0xFF) as u8,
        (height >> 8) as u8,
        32,   // bits per pixel (RGBA)
        0x28, // image descriptor (top-left origin, 8 alpha bits)
    ];
    f.write_all(&header)?;

    // Pixel data (BGRA)
    for pixel in rgba.chunks_exact(4) {
        f.write_all(&[pixel[2], pixel[1], pixel[0], pixel[3]])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_ppm_test() {
        let rgba = vec![
            255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = std::env::temp_dir().join("vtk_test.ppm");
        save_ppm(&path, &rgba, 2, 2).unwrap();
        let data = std::fs::read(&path).unwrap();
        assert!(data.starts_with(b"P6"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_bmp_test() {
        let rgba = vec![
            255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = std::env::temp_dir().join("vtk_test.bmp");
        save_bmp(&path, &rgba, 2, 2).unwrap();
        let data = std::fs::read(&path).unwrap();
        assert!(data.starts_with(b"BM"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn save_tga_test() {
        let rgba = vec![
            255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let path = std::env::temp_dir().join("vtk_test.tga");
        save_tga(&path, &rgba, 2, 2).unwrap();
        let data = std::fs::read(&path).unwrap();
        assert_eq!(data.len(), 18 + 16); // header + 4 pixels * 4 bytes
        let _ = std::fs::remove_file(&path);
    }
}
