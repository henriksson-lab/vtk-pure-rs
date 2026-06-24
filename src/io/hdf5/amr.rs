//! AMR (BoxLib/Chombo) reader via HDF5.
//!
//! Reads block-structured AMR data from HDF5 files as used by
//! BoxLib (AMReX) and Chombo frameworks.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::path::Path;

use crate::types::AmrLevelInfo;

/// AMR dataset read from HDF5.
#[derive(Debug)]
pub struct AmrData {
    pub levels: Vec<AmrLevel>,
    pub num_components: usize,
    pub component_names: Vec<String>,
}

/// A single AMR refinement level.
#[derive(Debug)]
pub struct AmrLevel {
    pub info: AmrLevelInfo,
    pub boxes: Vec<ImageData>,
}

/// Read a BoxLib/AMReX plotfile from HDF5.
pub fn read_amr(path: &Path) -> Result<AmrData, VtkError> {
    let file = hdf5::File::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let num_levels = file
        .attr("num_levels")
        .and_then(|a| a.read_scalar::<i64>())
        .unwrap_or(1) as usize;

    let num_components = file
        .attr("num_components")
        .and_then(|a| a.read_scalar::<i64>())
        .unwrap_or(1) as usize;

    let component_names: Vec<String> = (0..num_components)
        .map(|i| {
            file.attr(&format!("component_{i}"))
                .and_then(|a| a.read_scalar::<hdf5::types::VarLenAscii>())
                .map(|s| s.to_string())
                .unwrap_or_else(|_| format!("var{i}"))
        })
        .collect();

    let mut levels = Vec::with_capacity(num_levels);

    for lev in 0..num_levels {
        let level_group = file
            .group(&format!("level_{lev}"))
            .map_err(|e| VtkError::Parse(format!("level_{lev}: {e}")))?;

        let ref_ratio = level_group
            .attr("ref_ratio")
            .and_then(|a| a.read_scalar::<i64>())
            .unwrap_or(2) as usize;

        let dx: Vec<f64> = level_group
            .attr("dx")
            .and_then(|a| a.read_raw::<f64>())
            .unwrap_or_else(|_| vec![1.0; 3]);
        let dx_arr = [
            dx.first().copied().unwrap_or(1.0),
            dx.get(1).copied().unwrap_or(1.0),
            dx.get(2).copied().unwrap_or(1.0),
        ];

        let info = AmrLevelInfo {
            level: lev,
            num_boxes: 0, // filled below
            refinement_ratio: ref_ratio,
            dx: dx_arr,
        };

        // Read boxes (each box is stored as a dataset with box extent metadata)
        let box_names: Vec<String> = level_group
            .member_names()
            .unwrap_or_default()
            .into_iter()
            .filter(|n| n.starts_with("data:"))
            .collect();

        let mut boxes = Vec::new();
        for bname in &box_names {
            if let Ok(ds) = level_group.dataset(bname) {
                let shape = ds.shape();
                if shape.len() >= 3 {
                    let nx = shape[0];
                    let ny = shape[1];
                    let nz = shape[2];

                    let data: Vec<f64> = ds
                        .read_raw()
                        .map_err(|e| VtkError::Parse(format!("{bname}: {e}")))?;

                    let mut img = ImageData::with_dimensions(nx, ny, nz);
                    img.set_spacing(dx_arr);
                    if data.len() == nx * ny * nz {
                        img.point_data_mut()
                            .add_array(AnyDataArray::F64(DataArray::from_vec("data", data, 1)));
                    }
                    boxes.push(img);
                }
            }
        }

        let mut level_info = info;
        level_info.num_boxes = boxes.len();
        levels.push(AmrLevel {
            info: level_info,
            boxes,
        });
    }

    Ok(AmrData {
        levels,
        num_components,
        component_names,
    })
}
