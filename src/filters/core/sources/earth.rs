//! Earth globe geometry with latitude/longitude grid.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Parameters for earth globe generation.
pub struct EarthParams {
    /// Radius of the globe. Default: 1.0
    pub radius: f64,
    /// Number of latitude divisions. Default: 36
    pub theta_resolution: usize,
    /// Number of longitude divisions. Default: 72
    pub phi_resolution: usize,
}

impl Default for EarthParams {
    fn default() -> Self {
        Self {
            radius: 1.0,
            theta_resolution: 36,
            phi_resolution: 72,
        }
    }
}

/// Generate an earth globe sphere with latitude/longitude data arrays.
///
/// Creates a UV sphere with "Latitude" and "Longitude" point data arrays
/// in degrees, plus "TCoords" for equirectangular texture mapping.
pub fn earth(params: &EarthParams) -> PolyData {
    let n_theta = params.theta_resolution;
    let n_phi = params.phi_resolution;

    let mut points = Points::<f64>::new();
    let mut lat_data = Vec::new();
    let mut lon_data = Vec::new();
    let mut tcoords = Vec::new();

    // Generate points on sphere
    for j in 0..=n_theta {
        let theta = std::f64::consts::PI * j as f64 / n_theta as f64;
        let lat = 90.0 - theta.to_degrees(); // latitude: 90 at north pole, -90 at south

        for i in 0..=n_phi {
            let phi = 2.0 * std::f64::consts::PI * i as f64 / n_phi as f64;
            let lon = phi.to_degrees() - 180.0; // longitude: -180 to 180

            let x = params.radius * theta.sin() * phi.cos();
            let y = params.radius * theta.sin() * phi.sin();
            let z = params.radius * theta.cos();

            points.push([x, y, z]);
            lat_data.push(lat);
            lon_data.push(lon);

            // Equirectangular texture coordinates
            let u = i as f64 / n_phi as f64;
            let v = 1.0 - j as f64 / n_theta as f64;
            tcoords.push(u);
            tcoords.push(v);
        }
    }

    // Generate quads
    let mut polys = CellArray::new();
    let row_size = n_phi + 1;

    for j in 0..n_theta {
        for i in 0..n_phi {
            let p0 = (j * row_size + i) as i64;
            let p1 = (j * row_size + i + 1) as i64;
            let p2 = ((j + 1) * row_size + i + 1) as i64;
            let p3 = ((j + 1) * row_size + i) as i64;

            // Two triangles per quad
            polys.push_cell(&[p0, p1, p2]);
            polys.push_cell(&[p0, p2, p3]);
        }
    }

    let mut mesh = PolyData::new();
    mesh.points = points;
    mesh.polys = polys;
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Latitude", lat_data, 1,
        )));
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Longitude",
            lon_data,
            1,
        )));
    mesh.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "TCoords", tcoords, 2,
        )));
    mesh.point_data_mut().set_active_tcoords("TCoords");
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_earth() {
        let globe = earth(&EarthParams::default());
        assert!(globe.points.len() > 100);
        assert!(globe.polys.num_cells() > 100);
        assert!(globe.point_data().get_array("Latitude").is_some());
        assert!(globe.point_data().get_array("Longitude").is_some());
        assert!(globe.point_data().tcoords().is_some());
    }

    #[test]
    fn small_earth() {
        let globe = earth(&EarthParams {
            radius: 2.0,
            theta_resolution: 4,
            phi_resolution: 8,
            ..Default::default()
        });
        assert!(globe.points.len() > 0);

        // Check radius
        let p = globe.points.get(globe.points.len() / 2);
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 2.0).abs() < 0.1);
    }

    #[test]
    fn latitude_range() {
        let globe = earth(&EarthParams {
            theta_resolution: 10,
            phi_resolution: 10,
            ..Default::default()
        });
        let arr = globe.point_data().get_array("Latitude").unwrap();
        let mut min_lat = f64::MAX;
        let mut max_lat = f64::MIN;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            min_lat = min_lat.min(buf[0]);
            max_lat = max_lat.max(buf[0]);
        }
        assert!((max_lat - 90.0).abs() < 1.0);
        assert!((min_lat + 90.0).abs() < 1.0);
    }
}
