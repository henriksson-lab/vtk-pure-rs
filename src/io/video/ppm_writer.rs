//! Write frame sequences as numbered PPM image files (no dependencies).

use std::io::Write;
use std::path::Path;

use crate::io::video::FrameSequence;

/// Write a frame sequence as numbered PPM files.
///
/// Creates files like `{dir}/frame_0000.ppm`, `frame_0001.ppm`, etc.
pub fn write_ppm_sequence(seq: &FrameSequence, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create dir: {e}"))?;

    for (i, _frame) in seq.frames.iter().enumerate() {
        let path = dir.join(format!("frame_{:04}.ppm", i));
        let rgb = seq.frame_as_rgb(i);
        write_ppm(&path, seq.width, seq.height, &rgb)?;
    }

    Ok(())
}

/// Write a single PPM P6 binary file.
pub fn write_ppm(path: &Path, width: u32, height: u32, rgb: &[u8]) -> Result<(), String> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(3))
        .ok_or_else(|| "image dimensions are too large".to_string())?;
    if rgb.len() != expected {
        return Err(format!(
            "pixel data size mismatch: expected {expected}, got {}",
            rgb.len()
        ));
    }

    let mut file = std::fs::File::create(path).map_err(|e| format!("create file: {e}"))?;
    let header =
        format!("P6\n# ppm file written by the visualization toolkit\n{width} {height}\n255\n");
    file.write_all(header.as_bytes())
        .map_err(|e| format!("write header: {e}"))?;
    file.write_all(rgb)
        .map_err(|e| format!("write pixels: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_ppm_sequence_roundtrip() {
        let mut seq = FrameSequence::new(2, 2);
        seq.add_frame(vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 128, 128, 128, 255,
        ]);

        let dir = std::env::temp_dir().join("vtk_ppm_test");
        let _ = std::fs::remove_dir_all(&dir);
        write_ppm_sequence(&seq, &dir).unwrap();

        let ppm_path = dir.join("frame_0000.ppm");
        assert!(ppm_path.exists());

        let content = std::fs::read(&ppm_path).unwrap();
        // PPM header followed by 12 bytes RGB.
        let header = "P6\n# ppm file written by the visualization toolkit\n2 2\n255\n";
        assert!(content.starts_with(header.as_bytes()));
        assert_eq!(content.len(), header.len() + 12);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_ppm_rejects_wrong_pixel_count() {
        let path = std::env::temp_dir().join("vtk_bad_ppm_test.ppm");
        let err = write_ppm(&path, 2, 2, &[0; 3]).unwrap_err();
        assert!(err.contains("pixel data size mismatch"));
        let _ = std::fs::remove_file(&path);
    }
}
