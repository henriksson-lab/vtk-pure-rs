//! NetCDF (.nc) reader via the netcdf crate.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::path::Path;

use crate::types::NetcdfVarInfo;

/// Read a NetCDF file as ImageData (for gridded data) with variable metadata.
///
/// Reads the first 3D or 2D variable as the scalar field.
pub fn read_netcdf(path: &Path) -> Result<(ImageData, Vec<NetcdfVarInfo>), VtkError> {
    let file = netcdf_rs::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let mut var_infos = Vec::new();
    let mut first_3d: Option<String> = None;

    for var in file.variables() {
        let dims: Vec<String> = var
            .dimensions()
            .iter()
            .map(|d| d.name().to_string())
            .collect();
        let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
        let info = NetcdfVarInfo {
            name: var.name().to_string(),
            dimensions: dims,
            shape: shape.clone(),
            dtype: format!("{:?}", var.vartype()),
        };
        if first_3d.is_none() && (shape.len() == 3 || shape.len() == 2) {
            first_3d = Some(var.name().to_string());
        }
        var_infos.push(info);
    }

    let var_name = first_3d.ok_or_else(|| VtkError::Parse("no 2D/3D variable found".into()))?;
    let var = file
        .variable(&var_name)
        .ok_or_else(|| VtkError::Parse(format!("variable '{var_name}' not found")))?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let (nx, ny, nz) = match shape.len() {
        2 => (shape[1], shape[0], 1),
        3 => (shape[2], shape[1], shape[0]),
        _ => return Err(VtkError::Parse("unsupported variable dimensions".into())),
    };

    let total = nx * ny * nz;
    let data: Vec<f64> = var
        .get_values::<f64, _>(..)
        .map_err(|e| VtkError::Parse(format!("read data: {e}")))?;

    if data.len() != total {
        return Err(VtkError::Parse(format!(
            "data size mismatch: expected {total}, got {}",
            data.len()
        )));
    }

    // Try to read coordinate variables for spacing
    let dx = read_spacing(&file, &var, 2).unwrap_or(1.0);
    let dy = read_spacing(&file, &var, 1).unwrap_or(1.0);
    let dz = if shape.len() == 3 {
        read_spacing(&file, &var, 0).unwrap_or(1.0)
    } else {
        1.0
    };

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    img.set_spacing([dx, dy, dz]);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&var_name, data, 1)));

    Ok((img, var_infos))
}

fn read_spacing(file: &netcdf_rs::File, var: &netcdf_rs::Variable, dim_idx: usize) -> Option<f64> {
    let dim_name = var.dimensions().get(dim_idx)?.name().to_string();
    let coord_var = file.variable(&dim_name)?;
    let vals: Vec<f64> = coord_var.get_values(..).ok()?;
    if vals.len() >= 2 {
        Some((vals[1] - vals[0]).abs())
    } else {
        None
    }
}
