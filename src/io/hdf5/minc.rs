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

    let image_var = file
        .variable("image")
        .ok_or_else(|| VtkError::Parse("no 'image' variable found in MINC file".into()))?;

    for dim in image_var.dimensions() {
        info.dimensions.push(dim.name().to_string());
    }

    // Read spatial dimension info by MINC dimension name, matching VTK's
    // name-based axis mapping instead of assuming file dimension order.
    let dim_names = ["xspace", "yspace", "zspace"];
    let mut dim_sizes = [1usize; 3];

    for (i, &dname) in dim_names.iter().enumerate() {
        if let Some(dim) = image_var
            .dimensions()
            .iter()
            .find(|dim| dim.name() == dname)
        {
            dim_sizes[i] = dim.len();
        }
        // Read step (spacing) and start (origin) from dimension variables
        if let Some(var) = file.variable(dname) {
            if let Some(attr) = var.attribute("spacetype") {
                if let Ok(v) = attr.value() {
                    if let netcdf_rs::AttrValue::Str(s) = v {
                        info.space_type = s;
                    }
                }
            }
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
    let data = reorder_image_data(&data, image_var.dimensions(), dim_sizes)?;

    let mut img = ImageData::with_dimensions(dim_sizes[0], dim_sizes[1], dim_sizes[2]);
    img.set_spacing(info.step);
    img.set_origin(info.start);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("image", data, 1)));

    Ok((img, info))
}

fn reorder_image_data(
    data: &[f64],
    dimensions: &[netcdf_rs::Dimension],
    dim_sizes: [usize; 3],
) -> Result<Vec<f64>, VtkError> {
    let mut dim_lengths = Vec::with_capacity(dimensions.len());
    let mut dim_axes = Vec::with_capacity(dimensions.len());
    for dim in dimensions {
        dim_lengths.push(dim.len());
        dim_axes.push(axis_from_dimension_name(dim.name()));
    }

    let mut out = vec![0.0; data.len()];
    let mut index = vec![0usize; dimensions.len()];
    for (src_idx, &value) in data.iter().enumerate() {
        let mut remainder = src_idx;
        for dim in (0..dimensions.len()).rev() {
            let len = dim_lengths[dim];
            index[dim] = remainder % len;
            remainder /= len;
        }

        let mut ijk = [0usize; 3];
        for (dim, axis) in dim_axes.iter().enumerate() {
            match axis {
                Some(axis) => ijk[*axis] = index[dim],
                None if dim_lengths[dim] == 1 => {}
                None => {
                    return Err(VtkError::Parse(format!(
                        "unsupported non-spatial MINC dimension '{}'",
                        dimensions[dim].name()
                    )));
                }
            }
        }

        let dst_idx = ijk[0] + dim_sizes[0] * (ijk[1] + dim_sizes[1] * ijk[2]);
        out[dst_idx] = value;
    }
    Ok(out)
}

fn axis_from_dimension_name(name: &str) -> Option<usize> {
    match name {
        "xspace" => Some(0),
        "yspace" => Some(1),
        "zspace" => Some(2),
        _ => None,
    }
}
