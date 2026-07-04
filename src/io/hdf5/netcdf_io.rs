//! NetCDF (.nc) reader via the netcdf crate.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::path::Path;

use super::types::NetcdfVarInfo;

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

    // Try to read coordinate variables for origin and spacing.  VTK's
    // NetCDF CF reader maps netCDF dimensions in reverse order to image axes.
    let ndims = shape.len();
    let x_axis = read_axis_coordinate(&file, &var, ndims - 1);
    let y_axis = read_axis_coordinate(&file, &var, ndims - 2);
    let z_axis = if ndims == 3 {
        read_axis_coordinate(&file, &var, 0)
    } else {
        None
    };

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    img.set_origin([
        x_axis.map(|axis| axis.0).unwrap_or(0.0),
        y_axis.map(|axis| axis.0).unwrap_or(0.0),
        z_axis.map(|axis| axis.0).unwrap_or(0.0),
    ]);
    img.set_spacing([
        x_axis.map(|axis| axis.1).unwrap_or(1.0),
        y_axis.map(|axis| axis.1).unwrap_or(1.0),
        z_axis.map(|axis| axis.1).unwrap_or(1.0),
    ]);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(&var_name, data, 1)));

    Ok((img, var_infos))
}

fn read_axis_coordinate(
    file: &netcdf_rs::File,
    var: &netcdf_rs::Variable,
    dim_idx: usize,
) -> Option<(f64, f64)> {
    let dim_name = var.dimensions().get(dim_idx)?.name().to_string();
    let coord_var = file.variable(&dim_name)?;
    let vals: Vec<f64> = coord_var.get_values(..).ok()?;
    let origin = *vals.first()?;
    let spacing = if vals.len() >= 2 {
        (vals[vals.len() - 1] - origin) / (vals.len() - 1) as f64
    } else {
        1.0
    };
    Some((origin, spacing))
}

#[cfg(test)]
mod tests {
    #[test]
    fn axis_spacing_keeps_regular_coordinate_sign() {
        let vals = [10.0, 7.5, 5.0];
        let origin = vals[0];
        let spacing = (vals[vals.len() - 1] - origin) / (vals.len() - 1) as f64;
        assert_eq!(origin, 10.0);
        assert_eq!(spacing, -2.5);
    }
}
