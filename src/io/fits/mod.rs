//! FITS (Flexible Image Transport System) reader for vtk-rs.
//!
//! Reads simple 2D/3D FITS image data (primary HDU) into ImageData.
//! Supports BITPIX -32 (f32), -64 (f64), 16 (i16), 32 (i32).

use crate::data::{AnyDataArray, DataArray, ImageData};
use std::io::{Read, Seek};

/// Read a FITS primary image into ImageData.
pub fn read_fits<R: Read + Seek>(r: &mut R) -> Result<ImageData, String> {
    // Read header blocks (2880-byte blocks of 80-char keyword records)
    let mut naxis = 0usize;
    let mut naxis_vals = [1usize; 3];
    let mut bitpix: i32 = -32;
    let mut bzero = 0.0f64;
    let mut bscale = 1.0f64;

    let mut header_done = false;
    while !header_done {
        let mut block = [0u8; 2880];
        r.read_exact(&mut block).map_err(|e| e.to_string())?;

        for i in 0..36 {
            let record = std::str::from_utf8(&block[i * 80..(i + 1) * 80]).unwrap_or("");
            let key = record[..8].trim();
            let val_str = if record.len() > 10 {
                record[10..].trim()
            } else {
                ""
            };
            let val_str = val_str.split('/').next().unwrap_or("").trim();

            match key {
                "BITPIX" => bitpix = val_str.parse().unwrap_or(-32),
                "NAXIS" => naxis = val_str.parse().unwrap_or(0),
                "NAXIS1" => naxis_vals[0] = val_str.parse().unwrap_or(1),
                "NAXIS2" => naxis_vals[1] = val_str.parse().unwrap_or(1),
                "NAXIS3" => naxis_vals[2] = val_str.parse().unwrap_or(1),
                "BZERO" => bzero = val_str.parse().unwrap_or(0.0),
                "BSCALE" => bscale = val_str.parse().unwrap_or(1.0),
                "END" => {
                    header_done = true;
                    break;
                }
                _ => {}
            }
        }
    }

    if naxis < 1 {
        return Err("NAXIS < 1".into());
    }
    let dims = [
        naxis_vals[0],
        if naxis >= 2 { naxis_vals[1] } else { 1 },
        if naxis >= 3 { naxis_vals[2] } else { 1 },
    ];
    let total = dims[0] * dims[1] * dims[2];

    let bytes_per_pixel = match bitpix {
        8 => 1,
        16 => 2,
        32 => 4,
        -32 => 4,
        -64 => 8,
        _ => 4,
    };
    let mut raw = vec![0u8; total * bytes_per_pixel];
    r.read_exact(&mut raw).map_err(|e| e.to_string())?;

    let mut data = Vec::with_capacity(total);
    for i in 0..total {
        let off = i * bytes_per_pixel;
        let val = match bitpix {
            -32 => f32::from_be_bytes(raw[off..off + 4].try_into().unwrap()) as f64,
            -64 => f64::from_be_bytes(raw[off..off + 8].try_into().unwrap()),
            16 => i16::from_be_bytes(raw[off..off + 2].try_into().unwrap()) as f64,
            32 => i32::from_be_bytes(raw[off..off + 4].try_into().unwrap()) as f64,
            8 => raw[off] as f64,
            _ => 0.0,
        };
        data.push(val * bscale + bzero);
    }

    Ok(ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Data", data, 1))))
}

pub fn read_fits_file(path: &std::path::Path) -> Result<ImageData, String> {
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    read_fits(&mut f)
}

/// Write a minimal FITS file from ImageData.
pub fn write_fits<W: std::io::Write>(
    w: &mut W,
    image: &ImageData,
    array_name: &str,
) -> Result<(), String> {
    let dims = image.dimensions();
    let arr = image
        .point_data()
        .get_array(array_name)
        .ok_or_else(|| format!("array '{array_name}' not found"))?;

    // Build header
    let mut header = Vec::new();
    let add = |h: &mut Vec<u8>, s: &str| {
        let mut rec = [b' '; 80];
        rec[..s.len().min(80)].copy_from_slice(&s.as_bytes()[..s.len().min(80)]);
        h.extend_from_slice(&rec);
    };
    add(&mut header, "SIMPLE  =                    T");
    add(&mut header, "BITPIX  =                  -64");
    let naxis = if dims[2] > 1 {
        3
    } else if dims[1] > 1 {
        2
    } else {
        1
    };
    add(
        &mut header,
        &format!("NAXIS   =                    {naxis}"),
    );
    add(&mut header, &format!("NAXIS1  = {:>20}", dims[0]));
    if naxis >= 2 {
        add(&mut header, &format!("NAXIS2  = {:>20}", dims[1]));
    }
    if naxis >= 3 {
        add(&mut header, &format!("NAXIS3  = {:>20}", dims[2]));
    }
    add(&mut header, "END");

    // Pad to 2880 bytes
    while header.len() % 2880 != 0 {
        header.push(b' ');
    }

    w.write_all(&header).map_err(|e| e.to_string())?;

    // Write data as big-endian f64
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        w.write_all(&buf[0].to_be_bytes())
            .map_err(|e| e.to_string())?;
    }

    // Pad data to 2880 bytes
    let data_bytes = n * 8;
    let pad = (2880 - (data_bytes % 2880)) % 2880;
    w.write_all(&vec![0u8; pad]).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let image = ImageData::from_function(
            [4, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "flux",
            |x, y, _z| x * 10.0 + y,
        );
        let mut buf = Vec::new();
        write_fits(&mut buf, &image, "flux").unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let loaded = read_fits(&mut cursor).unwrap();
        assert_eq!(loaded.dimensions(), [4, 3, 1]);
        let arr = loaded.point_data().get_array("Data").unwrap();
        let mut b = [0.0f64];
        arr.tuple_as_f64(0, &mut b);
        assert!((b[0] - 0.0).abs() < 0.01); // x=0, y=0
    }

    #[test]
    fn empty_err() {
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        assert!(read_fits(&mut cursor).is_err());
    }
}
