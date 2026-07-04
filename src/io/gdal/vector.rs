//! Read geospatial vector data (Shapefile, GeoPackage, KML) as PolyData.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use crate::types::VtkError;
use std::path::Path;

use crate::types::{SpatialRef, VectorLayerInfo};

/// Read the first vector layer from a file as PolyData.
///
/// Points become vertex cells, lines become line cells, and polygon rings become line cells.
pub fn read_vector(path: &Path) -> Result<(PolyData, VectorLayerInfo), VtkError> {
    let dataset = gdal::Dataset::open(path).map_err(|e| {
        VtkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{e}"),
        ))
    })?;

    read_layer(&dataset, 0)
}

fn read_layer(
    dataset: &gdal::Dataset,
    layer_index: usize,
) -> Result<(PolyData, VectorLayerInfo), VtkError> {
    let layer = dataset
        .layer(layer_index)
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

    let mut points = Points::<f64>::new();
    let polys = CellArray::new();
    let mut lines = CellArray::new();
    let mut verts = CellArray::new();
    let mut num_features = 0;

    let num_fields = field_names.len();
    let mut field_values: Vec<Vec<f64>> = vec![Vec::new(); num_fields];
    let mut geom_type_priority = 0;
    let mut geom_type_str = "Unknown";

    for feature in layer.features() {
        num_features += 1;

        let Some(geom) = feature.geometry() else {
            continue;
        };

        let (priority, name) = geometry_type_name(geom.geometry_type());
        if priority > geom_type_priority {
            geom_type_priority = priority;
            geom_type_str = name;
        }

        let n_cells = insert_geometry_recursive(geom, &mut points, &mut lines, &mut verts);
        if n_cells == 0 {
            continue;
        }

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
            for _ in 0..n_cells {
                field_values[fi].push(val);
            }
        }
    }

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

fn geometry_type_name(geom_type: gdal::vector::OGRwkbGeometryType) -> (u8, &'static str) {
    use gdal::vector::OGRwkbGeometryType::*;

    match geom_type {
        wkbPoint | wkbPoint25D | wkbMultiPoint | wkbMultiPoint25D => (1, "Point"),
        wkbLinearRing
        | wkbLineString
        | wkbLineString25D
        | wkbMultiLineString
        | wkbMultiLineString25D => (2, "LineString"),
        wkbPolygon | wkbPolygon25D | wkbMultiPolygon | wkbMultiPolygon25D => (3, "Polygon"),
        wkbGeometryCollection | wkbGeometryCollection25D => (4, "GeometryCollection"),
        _ => (0, "Unknown"),
    }
}

fn insert_geometry_recursive(
    geom: &gdal::vector::Geometry,
    points: &mut Points<f64>,
    lines: &mut CellArray,
    verts: &mut CellArray,
) -> usize {
    use gdal::vector::OGRwkbGeometryType::*;

    match geom.geometry_type() {
        wkbPoint | wkbPoint25D => {
            let (x, y, z) = geom.get_point(0);
            let idx = points.len() as i64;
            points.push([x, y, z]);
            verts.push_cell(&[idx]);
            1
        }
        wkbLineString | wkbLineString25D | wkbLinearRing => {
            let n = geom.point_count();
            let base = points.len() as i64;
            let mut cell = Vec::with_capacity(n);
            for i in 0..n {
                let (x, y, z) = geom.get_point(i as i32);
                points.push([x, y, z]);
                cell.push(base + i as i64);
            }
            if cell.is_empty() {
                0
            } else {
                lines.push_cell(&cell);
                1
            }
        }
        wkbPolygon | wkbPolygon25D => {
            let mut n_cells = 0;
            for i in 0..geom.geometry_count() {
                if let Some(ring) = geom.geometry_ref(i) {
                    n_cells += insert_geometry_recursive(&ring, points, lines, verts);
                }
            }
            n_cells
        }
        wkbMultiPoint
        | wkbMultiPoint25D
        | wkbMultiLineString
        | wkbMultiLineString25D
        | wkbMultiPolygon
        | wkbMultiPolygon25D
        | wkbGeometryCollection
        | wkbGeometryCollection25D => {
            let mut n_cells = 0;
            for i in 0..geom.geometry_count() {
                if let Some(child) = geom.geometry_ref(i) {
                    n_cells += insert_geometry_recursive(&child, points, lines, verts);
                }
            }
            n_cells
        }
        _ => 0,
    }
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
        if let Ok(result) = read_layer(&dataset, i) {
            results.push(result);
        }
    }

    Ok(results)
}
