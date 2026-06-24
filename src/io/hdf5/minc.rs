//! MINC (.mnc) reader via NetCDF.
//!
//! MINC is a medical imaging format based on NetCDF/HDF5,
//! used primarily in neuroimaging research.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::path::Path;

/// MINC file metadata.
#[derive(Debug, Clone, Default)]
pub struct MincInfo {
    pub dimensions: Vec<String>,
    pub space_type: String,
    pub step: [f64; 3],
    pub start: [f64; 3],
}

/// Read a MINC file as ImageData.
///
/// MINC files store 3D medical images in NetCDF format with specific
/// dimension naming: xspace, yspace, zspace.
pub fn read_minc(path: &Path) -> Result<(ImageData, MincInfo), VtkError> {
    let file = netcdf_rs::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let mut info = MincInfo {
        step: [1.0, 1.0, 1.0],
        start: [0.0, 0.0, 0.0],
        ..Default::default()
    };

    // Read dimension info (xspace, yspace, zspace)
    let dim_names = ["xspace", "yspace", "zspace"];
    let mut dim_sizes = [1usize; 3];

    for (i, &dname) in dim_names.iter().enumerate() {
        if let Some(dim) = file.dimension(dname) {
            dim_sizes[i] = dim.len();
            info.dimensions.push(dname.to_string());
        }
        // Read step (spacing) and start (origin) from dimension variables
        if let Some(var) = file.variable(dname) {
            if let Ok(attr) = var.attribute("step").ok_or(()) {
                if let Ok(v) = attr.value() {
                    if let netcdf_rs::AttrValue::Double(vals) = v {
                        if !vals.is_empty() {
                            info.step[i] = vals[0];
                        }
                    }
                }
            }
            if let Ok(attr) = var.attribute("start").ok_or(()) {
                if let Ok(v) = attr.value() {
                    if let netcdf_rs::AttrValue::Double(vals) = v {
                        if !vals.is_empty() {
                            info.start[i] = vals[0];
                        }
                    }
                }
            }
        }
    }

    // Find the image variable (usually "image" or "image-min"/"image-max")
    let image_var = file
        .variable("image")
        .ok_or_else(|| VtkError::Parse("no 'image' variable found in MINC file".into()))?;

    let data: Vec<f64> = image_var
        .get_values(..)
        .map_err(|e| VtkError::Parse(format!("read image: {e}")))?;

    let total = dim_sizes[0] * dim_sizes[1] * dim_sizes[2];
    if data.len() != total {
        return Err(VtkError::Parse(format!(
            "data size mismatch: expected {total}, got {}",
            data.len()
        )));
    }

    let mut img = ImageData::with_dimensions(dim_sizes[0], dim_sizes[1], dim_sizes[2]);
    img.set_spacing([info.step[0].abs(), info.step[1].abs(), info.step[2].abs()]);
    img.set_origin(info.start);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("image", data, 1)));

    Ok((img, info))
}
