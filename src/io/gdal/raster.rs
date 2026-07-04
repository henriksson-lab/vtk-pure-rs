//! Read geospatial raster data (GeoTIFF, DEM, etc.) as ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::path::Path;

use super::types::{RasterBandInfo, RasterInfo, SpatialRef};

/// Read a raster file (GeoTIFF, etc.) as ImageData.
///
/// Each band becomes a scalar array in cell data.
pub fn read_raster(path: &Path) -> Result<(ImageData, RasterInfo), VtkError> {
    let dataset = gdal::Dataset::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let (width, height) = dataset.raster_size();
    let num_bands = dataset.raster_count();
    let geo_transform = dataset
        .geo_transform()
        .map_err(|e| VtkError::Parse(format!("geo_transform: {e}")))?;

    let spatial_ref = dataset
        .spatial_ref()
        .map(|sr| SpatialRef {
            epsg: sr.auth_code().ok().map(|c| c as u32),
            wkt: sr.to_wkt().unwrap_or_default(),
            proj4: sr.to_proj4().unwrap_or_default(),
        })
        .unwrap_or_default();

    let (origin, spacing, flip_axis) = raster_geometry(width, height, geo_transform);

    let mut img = ImageData::with_dimensions(width + 1, height + 1, 1);
    img.set_spacing(spacing);
    img.set_origin(origin);

    let mut band_infos = Vec::new();

    for band_idx in 1..=num_bands {
        let band = dataset
            .rasterband(band_idx)
            .map_err(|e| VtkError::Parse(format!("band {band_idx}: {e}")))?;

        let no_data = band.no_data_value();

        // Read as f64
        let buf = band
            .read_as::<f64>((0, 0), (width, height), (width, height), None)
            .map_err(|e| VtkError::Parse(format!("read band {band_idx}: {e}")))?;

        let data = orient_raster_data(buf.data(), width, height, flip_axis);

        // Compute min/max
        let (mut min, mut max) = (f64::MAX, f64::MIN);
        for &v in &data {
            if let Some(nd) = no_data {
                if (v - nd).abs() < 1e-10 {
                    continue;
                }
            }
            min = min.min(v);
            max = max.max(v);
        }

        let band_name = if num_bands == 1 {
            "Elevation".to_string()
        } else {
            format!("Band{band_idx}")
        };

        band_infos.push(RasterBandInfo {
            index: band_idx,
            name: band_name.clone(),
            data_type: format!("{:?}", band.band_type()),
            no_data_value: no_data,
            min,
            max,
        });

        img.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&band_name, data, 1)));
        if img.cell_data().scalars().is_none() {
            img.cell_data_mut().set_active_scalars(&band_name);
        }
    }

    let info = RasterInfo {
        width,
        height,
        num_bands,
        spatial_ref,
        geo_transform,
        driver: dataset.driver().short_name().to_string(),
        bands: band_infos,
    };

    Ok((img, info))
}

/// Read a raster as elevation mesh (PolyData with quads).
///
/// Each pixel becomes a vertex at (geo_x, geo_y, elevation).
pub fn read_raster_as_mesh(path: &Path) -> Result<crate::data::PolyData, VtkError> {
    let (img, info) = read_raster(path)?;

    let w = info.width;
    let h = info.height;
    let origin = img.origin();
    let spacing = img.spacing();

    let scalars = img
        .cell_data()
        .get_array_by_index(0)
        .ok_or_else(|| VtkError::Parse("no band data".into()))?;

    let mut points = crate::data::Points::<f64>::new();
    for j in 0..h {
        for i in 0..w {
            let x = origin[0] + i as f64 * spacing[0];
            let y = origin[1] + j as f64 * spacing[1];
            let mut z = [0.0f64];
            scalars.tuple_as_f64(j * w + i, &mut z);
            points.push([x, y, z[0]]);
        }
    }

    let mut polys = crate::data::CellArray::new();
    for j in 0..h - 1 {
        for i in 0..w - 1 {
            let v00 = (j * w + i) as i64;
            let v10 = v00 + 1;
            let v01 = v00 + w as i64;
            let v11 = v01 + 1;
            polys.push_cell(&[v00, v10, v11, v01]);
        }
    }

    let mut pd = crate::data::PolyData::new();
    pd.points = points;
    pd.polys = polys;
    Ok(pd)
}

fn raster_geometry(
    width: usize,
    height: usize,
    geo_transform: [f64; 6],
) -> ([f64; 3], [f64; 3], [bool; 2]) {
    let upper_left = geo_transform_point(geo_transform, 0.0, 0.0);
    let lower_right = geo_transform_point(geo_transform, width as f64, height as f64);
    let geo_spacing = [
        (lower_right[0] - upper_left[0]) / width as f64,
        (lower_right[1] - upper_left[1]) / height as f64,
    ];

    (
        [
            upper_left[0].min(lower_right[0]),
            upper_left[1].min(lower_right[1]),
            0.0,
        ],
        [geo_spacing[0].abs(), geo_spacing[1].abs(), 1.0],
        [geo_spacing[0] < 0.0, geo_spacing[1] < 0.0],
    )
}

fn geo_transform_point(geo_transform: [f64; 6], x: f64, y: f64) -> [f64; 2] {
    [
        geo_transform[0] + geo_transform[1] * x + geo_transform[2] * y,
        geo_transform[3] + geo_transform[4] * x + geo_transform[5] * y,
    ]
}

fn orient_raster_data(data: &[f64], width: usize, height: usize, flip_axis: [bool; 2]) -> Vec<f64> {
    let mut oriented = vec![0.0; data.len()];
    for j in 0..height {
        let source_j = if flip_axis[1] { height - 1 - j } else { j };
        for i in 0..width {
            let source_i = if flip_axis[0] { width - 1 - i } else { i };
            oriented[j * width + i] = data[source_j * width + source_i];
        }
    }
    oriented
}
