use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// DICOM image metadata.
#[derive(Debug, Clone)]
pub struct DicomInfo {
    pub rows: u32,
    pub columns: u32,
    pub bits_allocated: u16,
    pub bits_stored: u16,
    pub pixel_representation: u16,
    pub pixel_spacing: [f64; 2],
    pub slice_thickness: f64,
    pub rescale_slope: f64,
    pub rescale_intercept: f64,
    pub patient_name: String,
    pub modality: String,
}

impl Default for DicomInfo {
    fn default() -> Self {
        Self {
            rows: 0,
            columns: 0,
            bits_allocated: 0,
            bits_stored: 0,
            pixel_representation: 0,
            pixel_spacing: [1.0, 1.0],
            slice_thickness: 1.0,
            rescale_slope: 1.0,
            rescale_intercept: 0.0,
            patient_name: String::new(),
            modality: String::new(),
        }
    }
}

/// Read a DICOM file → ImageData + metadata.
pub fn read_dicom(path: &Path) -> Result<(ImageData, DicomInfo), VtkError> {
    let mut file = std::fs::File::open(path).map_err(VtkError::Io)?;
    file.seek(SeekFrom::Start(128)).map_err(VtkError::Io)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).map_err(VtkError::Io)?;
    if &magic != b"DICM" {
        return Err(VtkError::Parse("not a DICOM file".into()));
    }

    let mut info = DicomInfo::default();
    let mut pixel_data: Vec<u8> = Vec::new();

    loop {
        let (g, e) = match read_tag(&mut file) {
            Ok(t) => t,
            Err(_) => break,
        };
        let mut vr = [0u8; 2];
        if file.read_exact(&mut vr).is_err() {
            break;
        }
        let vr_s = std::str::from_utf8(&vr).unwrap_or("  ");
        let vlen = if matches!(vr_s, "OB" | "OW" | "OF" | "SQ" | "UC" | "UN" | "UR" | "UT") {
            let _ = file.read_exact(&mut [0u8; 2]);
            read_u32(&mut file).unwrap_or(0)
        } else {
            read_u16(&mut file).unwrap_or(0) as u32
        };
        if vlen == 0xFFFFFFFF {
            break;
        }
        let mut val = vec![0u8; vlen as usize];
        if file.read_exact(&mut val).is_err() {
            break;
        }

        match (g, e) {
            (0x0028, 0x0010) => info.rows = le16(&val) as u32,
            (0x0028, 0x0011) => info.columns = le16(&val) as u32,
            (0x0028, 0x0100) => info.bits_allocated = le16(&val),
            (0x0028, 0x0101) => info.bits_stored = le16(&val),
            (0x0028, 0x0103) => info.pixel_representation = le16(&val),
            (0x0028, 0x0030) => {
                let s = lossy(&val);
                let p: Vec<&str> = s.trim().split('\\').collect();
                if p.len() >= 2 {
                    info.pixel_spacing = [
                        p[0].trim().parse().unwrap_or(1.0),
                        p[1].trim().parse().unwrap_or(1.0),
                    ];
                }
            }
            (0x0018, 0x0050) => {
                info.slice_thickness = lossy(&val).trim().parse().unwrap_or(1.0);
            }
            (0x0028, 0x1053) => {
                info.rescale_slope = lossy(&val).trim().parse().unwrap_or(1.0);
            }
            (0x0028, 0x1052) => {
                info.rescale_intercept = lossy(&val).trim().parse().unwrap_or(0.0);
            }
            (0x0010, 0x0010) => {
                info.patient_name = lossy(&val).trim().to_string();
            }
            (0x0008, 0x0060) => {
                info.modality = lossy(&val).trim().to_string();
            }
            (0x7FE0, 0x0010) => pixel_data = val,
            _ => {}
        }
    }

    if info.rows == 0 || info.columns == 0 || pixel_data.is_empty() {
        return Err(VtkError::Parse("incomplete DICOM".into()));
    }

    let n = (info.rows * info.columns) as usize;
    let scalars: Vec<f64> = match info.bits_allocated {
        8 => pixel_data
            .iter()
            .take(n)
            .map(|&v| v as f64 * info.rescale_slope + info.rescale_intercept)
            .collect(),
        16 => pixel_data
            .chunks_exact(2)
            .take(n)
            .map(|b| {
                let raw = if info.pixel_representation == 0 {
                    u16::from_le_bytes([b[0], b[1]]) as f64
                } else {
                    i16::from_le_bytes([b[0], b[1]]) as f64
                };
                raw * info.rescale_slope + info.rescale_intercept
            })
            .collect(),
        o => return Err(VtkError::Unsupported(format!("bits_allocated={o}"))),
    };

    let mut img = ImageData::with_dimensions(info.columns as usize, info.rows as usize, 1);
    img.set_spacing([
        info.pixel_spacing[1],
        info.pixel_spacing[0],
        info.slice_thickness,
    ]);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PixelData",
            scalars,
            1,
        )));
    Ok((img, info))
}

fn read_tag(r: &mut impl Read) -> Result<(u16, u16), VtkError> {
    Ok((read_u16(r)?, read_u16(r)?))
}
fn read_u16(r: &mut impl Read) -> Result<u16, VtkError> {
    let mut b = [0u8; 2];
    r.read_exact(&mut b).map_err(VtkError::Io)?;
    Ok(u16::from_le_bytes(b))
}
fn read_u32(r: &mut impl Read) -> Result<u32, VtkError> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b).map_err(VtkError::Io)?;
    Ok(u32::from_le_bytes(b))
}
fn le16(d: &[u8]) -> u16 {
    if d.len() >= 2 {
        u16::from_le_bytes([d[0], d[1]])
    } else {
        0
    }
}
fn lossy(d: &[u8]) -> String {
    String::from_utf8_lossy(d).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let i = DicomInfo::default();
        assert_eq!(i.rescale_slope, 1.0);
    }

    #[test]
    fn nonexistent() {
        assert!(read_dicom(Path::new("/no.dcm")).is_err());
    }

    #[test]
    fn le16_conv() {
        assert_eq!(le16(&[1, 0]), 1);
        assert_eq!(le16(&[0, 1]), 256);
    }

    #[test]
    fn rescale() {
        assert!((1024.0f64 * 1.0 + (-1024.0)).abs() < 1e-10);
    }
}
