//! Read geospatial vector data (Shapefile, GeoPackage, KML) as PolyData.

use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::types::VtkError;
use std::path::Path;

use crate::types::{GeomType, SpatialRef, VectorLayerInfo};

/// Read the first vector layer from a file as PolyData.
///
/// Points become vertex cells, lines become line cells, polygons become polygon cells.
pub fn read_vector(path: &Path) -> Result<(PolyData, VectorLayerInfo), VtkError> {
    let dataset = gdal::Dataset::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let layer = dataset
        .layer(0)
        .map_err(|e| VtkError::Parse(format!("layer: {e}")))?;

    let spatial_ref = layer
        .spatial_ref()
        .map(|sr| SpatialRef {
            epsg: sr.auth_code().ok().map(|c| c as u32),
            wkt: sr.to_wkt().unwrap_or_default(),
            proj4: sr.to_proj4().unwrap_or_default(),
        })
        .unwrap_or_default();

    let field_names: Vec<String> = layer
        .defn()
        .fields()
        .map(|f| f.name().to_string())
        .collect();

    let mut points = crate::data::Points::<f64>::new();
    let mut polys = crate::data::CellArray::new();
    let mut lines = crate::data::CellArray::new();
    let mut verts = crate::data::CellArray::new();
    let mut num_features = 0;

    // Collect field values
    let num_fields = field_names.len();
    let mut field_values: Vec<Vec<f64>> = vec![Vec::new(); num_fields];

    for feature in layer.features() {
        num_features += 1;

        // Read field values as f64 where possible
        for (fi, fname) in field_names.iter().enumerate() {
            let val = feature
                .field(fname)
                .ok()
                .flatten()
                .and_then(|v| match v {
                    gdal::vector::FieldValue::IntegerValue(i) => Some(i as f64),
                    gdal::vector::FieldValue::Integer64Value(i) => Some(i as f64),
                    gdal::vector::FieldValue::RealValue(f) => Some(f),
                    _ => None,
                })
                .unwrap_or(0.0);
            field_values[fi].push(val);
        }

        let Some(geom) = feature.geometry() else {
            continue;
        };

        match geom.geometry_type() {
            gdal::vector::OGRwkbGeometryType::wkbPoint
            | gdal::vector::OGRwkbGeometryType::wkbPoint25D => {
                let (x, y, z) = geom.get_point(0);
                let idx = points.len() as i64;
                points.push([x, y, z]);
                verts.push_cell(&[idx]);
            }
            gdal::vector::OGRwkbGeometryType::wkbLineString
            | gdal::vector::OGRwkbGeometryType::wkbLineString25D => {
                let n = geom.point_count();
                let base = points.len() as i64;
                let mut cell = Vec::with_capacity(n);
                for i in 0..n {
                    let (x, y, z) = geom.get_point(i as i32);
                    points.push([x, y, z]);
                    cell.push(base + i as i64);
                }
                lines.push_cell(&cell);
            }
            gdal::vector::OGRwkbGeometryType::wkbPolygon
            | gdal::vector::OGRwkbGeometryType::wkbPolygon25D => {
                // Read outer ring
                if let Some(ring) = geom.geometry_ref(0) {
                    let n = ring.point_count();
                    let base = points.len() as i64;
                    let mut cell = Vec::with_capacity(n.saturating_sub(1));
                    for i in 0..n.saturating_sub(1) {
                        // skip closing vertex
                        let (x, y, z) = ring.get_point(i as i32);
                        points.push([x, y, z]);
                        cell.push(base + i as i64);
                    }
                    if !cell.is_empty() {
                        polys.push_cell(&cell);
                    }
                }
            }
            _ => {}
        }
    }

    let geom_type_str = if polys.num_cells() > 0 {
        "Polygon"
    } else if lines.num_cells() > 0 {
        "LineString"
    } else {
        "Point"
    };

    let info = VectorLayerInfo {
        name: layer.name().to_string(),
        num_features,
        geometry_type: geom_type_str.to_string(),
        spatial_ref,
        field_names: field_names.clone(),
    };

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd.lines = lines;
    pd.verts = verts;

    // Add numeric fields as cell data
    for (fi, fname) in field_names.iter().enumerate() {
        if !field_values[fi].is_empty() {
            pd.cell_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(
                    fname,
                    field_values[fi].clone(),
                    1,
                )));
        }
    }

    Ok((pd, info))
}

/// Read all layers from a vector dataset.
pub fn read_all_layers(path: &Path) -> Result<Vec<(PolyData, VectorLayerInfo)>, VtkError> {
    let dataset = gdal::Dataset::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    let mut results = Vec::new();
    let num_layers = dataset.layer_count();

    for i in 0..num_layers {
        // Re-open to get each layer (GDAL layer iteration quirk)
        if let Ok(result) = read_vector(path) {
            results.push(result);
            break; // simplified: just first layer for now
        }
    }

    Ok(results)
}
