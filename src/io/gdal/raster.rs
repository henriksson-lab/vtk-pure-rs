//! Read geospatial raster data (GeoTIFF, DEM, etc.) as ImageData.

use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::VtkError;
use std::path::Path;

use crate::types::{RasterBandInfo, RasterInfo, SpatialRef};

/// Read a raster file (GeoTIFF, etc.) as ImageData.
///
/// Each band becomes a scalar array in point data.
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

    let pixel_w = geo_transform[1].abs();
    let pixel_h = geo_transform[5].abs();

    let mut img = ImageData::with_dimensions(width, height, 1);
    img.set_spacing([pixel_w, pixel_h, 1.0]);
    img.set_origin([
        geo_transform[0],
        geo_transform[3] - height as f64 * pixel_h,
        0.0,
    ]);

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

        let data: Vec<f64> = buf.data().to_vec();

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

        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(&band_name, data, 1)));
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
    let origin_x = info.origin_x();
    let origin_y = info.origin_y();
    let dx = info.pixel_width();
    let dy = info.pixel_height();

    let scalars = img
        .point_data()
        .get_array_by_index(0)
        .ok_or_else(|| VtkError::Parse("no band data".into()))?;

    let mut points = crate::data::Points::<f64>::new();
    for j in 0..h {
        for i in 0..w {
            let x = origin_x + i as f64 * dx;
            let y = origin_y + j as f64 * dy;
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
