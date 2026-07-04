//! SEG-Y seismic data format reader for vtk-rs.
//!
//! Reads SEG-Y rev 1 files: 3200-byte EBCDIC header, 400-byte binary header,
//! then trace headers + trace data. Outputs as ImageData (2D) or PolyData points.

use crate::data::{AnyDataArray, DataArray, ImageData, Points, PolyData};
use std::io::{Read, Seek, SeekFrom};

/// Read SEG-Y trace data as a 2D ImageData (traces × samples).
pub fn read_segy_as_image<R: Read + Seek>(r: &mut R) -> Result<ImageData, String> {
    // Skip EBCDIC header (3200 bytes)
    r.seek(SeekFrom::Start(3200)).map_err(|e| e.to_string())?;

    // Read binary header (400 bytes)
    let mut bin_hdr = [0u8; 400];
    r.read_exact(&mut bin_hdr).map_err(|e| e.to_string())?;

    let sample_interval = i16::from_be_bytes([bin_hdr[16], bin_hdr[17]]) as f64 / 1000.0;
    let n_samples = i16::from_be_bytes([bin_hdr[20], bin_hdr[21]]) as usize;
    let format_code = i16::from_be_bytes([bin_hdr[24], bin_hdr[25]]);

    if n_samples == 0 {
        return Err("zero samples per trace".into());
    }

    let bytes_per_sample = match format_code {
        1 => 4, // IBM float
        2 => 4, // 4-byte int
        3 => 2, // 2-byte int
        5 => 4, // IEEE float
        8 => 1, // 1-byte int
        _ => 4, // default to 4
    };

    let trace_data_size = n_samples * bytes_per_sample;
    let trace_header_size = 240;

    // Count traces by reading until EOF
    let current = r.seek(SeekFrom::Current(0)).map_err(|e| e.to_string())?;
    let end = r.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
    let data_bytes = end - current;
    let trace_size = trace_header_size + trace_data_size as u64;
    let n_traces = if trace_size > 0 {
        (data_bytes / trace_size) as usize
    } else {
        0
    };

    if n_traces == 0 {
        return Err("no traces found".into());
    }

    r.seek(SeekFrom::Start(3600)).map_err(|e| e.to_string())?;

    let mut all_data = vec![0.0f64; n_traces * n_samples];
    let mut trace_buf = vec![0u8; trace_data_size];
    let mut hdr_buf = [0u8; 240];

    for ti in 0..n_traces {
        if r.read_exact(&mut hdr_buf).is_err() {
            break;
        }
        if r.read_exact(&mut trace_buf).is_err() {
            break;
        }

        for si in 0..n_samples {
            let offset = si * bytes_per_sample;
            let val = match format_code {
                5 => {
                    // IEEE float
                    if offset + 4 <= trace_buf.len() {
                        f32::from_be_bytes(trace_buf[offset..offset + 4].try_into().unwrap()) as f64
                    } else {
                        0.0
                    }
                }
                2 => {
                    // 4-byte int
                    if offset + 4 <= trace_buf.len() {
                        i32::from_be_bytes(trace_buf[offset..offset + 4].try_into().unwrap()) as f64
                    } else {
                        0.0
                    }
                }
                3 => {
                    // 2-byte int
                    if offset + 2 <= trace_buf.len() {
                        i16::from_be_bytes(trace_buf[offset..offset + 2].try_into().unwrap()) as f64
                    } else {
                        0.0
                    }
                }
                1 => {
                    if offset + 4 <= trace_buf.len() {
                        read_ibm_float(&trace_buf[offset..offset + 4]) as f64
                    } else {
                        0.0
                    }
                }
                8 => {
                    if offset < trace_buf.len() {
                        trace_buf[offset] as i8 as f64
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            };
            all_data[si * n_traces + ti] = val;
        }
    }

    let img = ImageData::with_dimensions(n_traces, n_samples, 1)
        .with_spacing([1.0, sample_interval, 1.0])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("trace", all_data, 1)));

    Ok(img)
}

/// Read SEG-Y as a PolyData point cloud (trace positions from headers).
pub fn read_segy_as_points<R: Read + Seek>(r: &mut R) -> Result<PolyData, String> {
    r.seek(SeekFrom::Start(3200)).map_err(|e| e.to_string())?;
    let mut bin_hdr = [0u8; 400];
    r.read_exact(&mut bin_hdr).map_err(|e| e.to_string())?;

    let n_samples = i16::from_be_bytes([bin_hdr[20], bin_hdr[21]]) as usize;
    let format_code = i16::from_be_bytes([bin_hdr[24], bin_hdr[25]]);
    let bps = match format_code {
        1 | 2 | 5 => 4,
        3 => 2,
        8 => 1,
        _ => 4,
    };
    let trace_data_size = n_samples * bps;

    let mut points = Points::<f64>::new();
    let mut hdr_buf = [0u8; 240];
    let mut skip_buf = vec![0u8; trace_data_size];

    loop {
        if r.read_exact(&mut hdr_buf).is_err() {
            break;
        }
        // CDP X at bytes 180-184, CDP Y at bytes 184-188 (big-endian i32)
        let coordinate_multiplier =
            decode_multiplier(i16::from_be_bytes(hdr_buf[70..72].try_into().unwrap()));
        let x = coordinate_multiplier
            * i32::from_be_bytes(hdr_buf[180..184].try_into().unwrap()) as f64;
        let y = coordinate_multiplier
            * i32::from_be_bytes(hdr_buf[184..188].try_into().unwrap()) as f64;
        points.push([x, y, 0.0]);
        if r.read_exact(&mut skip_buf).is_err() {
            break;
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    Ok(mesh)
}

fn read_ibm_float(bytes: &[u8]) -> f32 {
    let word = u32::from_be_bytes(bytes.try_into().unwrap());
    let sign = if (word >> 31) & 0x01 == 0 { 1.0 } else { -1.0 };
    let exponent = ((word >> 24) & 0x7f) as i32;
    let fraction = (word & 0x00ff_ffff) as f32 / 2.0_f32.powi(24);
    if fraction == 0.0 {
        0.0
    } else {
        sign * fraction * 16.0_f32.powi(exponent - 64)
    }
}

fn decode_multiplier(multiplier: i16) -> f64 {
    if multiplier < 0 {
        -1.0 / multiplier as f64
    } else if multiplier > 0 {
        multiplier as f64
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // SEG-Y requires actual binary files to test meaningfully;
    // we test the error path here.
    #[test]
    fn empty_input() {
        let data: &[u8] = &[];
        let mut cursor = std::io::Cursor::new(data);
        assert!(read_segy_as_image(&mut cursor).is_err());
    }

    #[test]
    fn ibm_float_matches_segy_encoding() {
        assert!((read_ibm_float(&[0x41, 0x10, 0x00, 0x00]) - 1.0).abs() < 1e-6);
        assert!((read_ibm_float(&[0xc1, 0x10, 0x00, 0x00]) + 1.0).abs() < 1e-6);
        assert_eq!(read_ibm_float(&[0x00, 0x00, 0x00, 0x00]), 0.0);
    }

    #[test]
    fn reads_one_byte_integer_samples_and_raw_spacing() {
        let mut data = vec![0u8; 3600];
        data[3216..3218].copy_from_slice(&500i16.to_be_bytes());
        data[3220..3222].copy_from_slice(&2i16.to_be_bytes());
        data[3224..3226].copy_from_slice(&8i16.to_be_bytes());

        data.extend_from_slice(&[0u8; 240]);
        data.extend_from_slice(&[255u8, 2u8]);
        data.extend_from_slice(&[0u8; 240]);
        data.extend_from_slice(&[3u8, 4u8]);

        let mut cursor = std::io::Cursor::new(data);
        let img = read_segy_as_image(&mut cursor).unwrap();
        assert_eq!(img.dimensions(), [2, 2, 1]);
        assert_eq!(img.spacing(), [1.0, 0.5, 1.0]);

        let array = img.point_data().get_array("trace").unwrap();
        let AnyDataArray::F64(values) = array else {
            panic!("expected f64 amplitudes");
        };
        assert_eq!(values.as_slice(), &[-1.0, 3.0, 2.0, 4.0]);
    }

    #[test]
    fn points_apply_coordinate_multiplier() {
        let mut data = vec![0u8; 3600];
        data[3220..3222].copy_from_slice(&1i16.to_be_bytes());
        data[3224..3226].copy_from_slice(&8i16.to_be_bytes());

        let mut hdr = [0u8; 240];
        hdr[70..72].copy_from_slice(&(-10i16).to_be_bytes());
        hdr[180..184].copy_from_slice(&100i32.to_be_bytes());
        hdr[184..188].copy_from_slice(&(-50i32).to_be_bytes());
        data.extend_from_slice(&hdr);
        data.push(1u8);

        let mut cursor = std::io::Cursor::new(data);
        let points = read_segy_as_points(&mut cursor).unwrap();
        assert_eq!(points.points.get(0), [10.0, -5.0, 0.0]);
    }
}
