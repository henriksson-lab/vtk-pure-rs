//! Geospatial types (always available, no GDAL needed).

/// Spatial reference / coordinate system info.
#[derive(Debug, Clone, Default)]
pub struct SpatialRef {
    /// EPSG code (e.g. 4326 for WGS84, 32632 for UTM zone 32N).
    pub epsg: Option<u32>,
    /// WKT projection string.
    pub wkt: String,
    /// PROJ string.
    pub proj4: String,
}

/// Raster band metadata.
#[derive(Debug, Clone)]
pub struct RasterBandInfo {
    pub index: usize,
    pub name: String,
    pub data_type: String,
    pub no_data_value: Option<f64>,
    pub min: f64,
    pub max: f64,
}

/// Geospatial raster metadata.
#[derive(Debug, Clone, Default)]
pub struct RasterInfo {
    pub width: usize,
    pub height: usize,
    pub num_bands: usize,
    pub spatial_ref: SpatialRef,
    /// GeoTransform: [origin_x, pixel_width, rotation_x, origin_y, rotation_y, pixel_height]
    pub geo_transform: [f64; 6],
    pub driver: String,
    pub bands: Vec<RasterBandInfo>,
}

impl RasterInfo {
    /// Pixel size in X direction.
    pub fn pixel_width(&self) -> f64 {
        self.geo_transform[1]
    }
    /// Pixel size in Y direction (usually negative).
    pub fn pixel_height(&self) -> f64 {
        self.geo_transform[5]
    }
    /// Origin X coordinate.
    pub fn origin_x(&self) -> f64 {
        self.geo_transform[0]
    }
    /// Origin Y coordinate.
    pub fn origin_y(&self) -> f64 {
        self.geo_transform[3]
    }
}

/// Vector layer metadata.
#[derive(Debug, Clone, Default)]
pub struct VectorLayerInfo {
    pub name: String,
    pub num_features: usize,
    pub geometry_type: String,
    pub spatial_ref: SpatialRef,
    pub field_names: Vec<String>,
}

/// Geometry type for vector features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeomType {
    Point,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_info_defaults() {
        let info = RasterInfo::default();
        assert_eq!(info.width, 0);
        assert_eq!(info.pixel_width(), 0.0);
    }

    #[test]
    fn spatial_ref_default() {
        let sr = SpatialRef::default();
        assert!(sr.epsg.is_none());
    }

    #[test]
    fn geom_types() {
        assert_ne!(GeomType::Point, GeomType::Polygon);
    }
}
