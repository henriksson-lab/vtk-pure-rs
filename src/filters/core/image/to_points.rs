//! Convert ImageData to point cloud PolyData.
//!
//! Creates a point at each grid location of the ImageData, optionally
//! filtering by scalar threshold and transferring all point data arrays.

use crate::data::{AnyDataArray, DataArray, ImageData, Points, PolyData};

/// Convert an ImageData to a PolyData point cloud.
///
/// Creates one point per voxel with all point data arrays transferred.
pub fn image_to_points(image: &ImageData) -> PolyData {
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];

    let mut points = Points::<f64>::new();
    for iz in 0..dims[2] {
        for iy in 0..dims[1] {
            for ix in 0..dims[0] {
                points.push(image.point_from_ijk(ix, iy, iz));
            }
        }
    }

    let mut result = PolyData::new();
    result.points = points;
    for i in 0..n {
        result.verts.push_cell(&[i as i64]);
    }

    // Transfer point data
    let pd = image.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let name = arr.name().to_string();
            let mut data = Vec::with_capacity(n * nc);
            let mut buf = vec![0.0f64; nc];
            for i in 0..n {
                arr.tuple_as_f64(i, &mut buf);
                data.extend_from_slice(&buf);
            }
            result
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, nc)));
        }
    }

    result
}

/// Convert an ImageData to a point cloud, keeping only points where
/// a scalar array meets or exceeds a threshold.
pub fn image_to_points_threshold(image: &ImageData, array_name: &str, threshold: f64) -> PolyData {
    let dims = image.dimensions();
    let n = dims[0] * dims[1] * dims[2];

    let scalar_arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        None => return image_to_points(image), // fallback to all points
    };

    let mut indices: Vec<usize> = Vec::new();
    let mut buf = [0.0f64];
    for i in 0..n {
        scalar_arr.tuple_as_f64(i, &mut buf);
        if buf[0] >= threshold {
            indices.push(i);
        }
    }

    let mut points = Points::<f64>::new();
    for &idx in &indices {
        let iz = idx / (dims[0] * dims[1]);
        let rem = idx % (dims[0] * dims[1]);
        let iy = rem / dims[0];
        let ix = rem % dims[0];
        points.push(image.point_from_ijk(ix, iy, iz));
    }

    let mut result = PolyData::new();
    result.points = points;
    for i in 0..indices.len() {
        result.verts.push_cell(&[i as i64]);
    }

    // Transfer arrays for selected points
    let pd = image.point_data();
    for ai in 0..pd.num_arrays() {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let name = arr.name().to_string();
            let mut data = Vec::with_capacity(indices.len() * nc);
            let mut buf = vec![0.0f64; nc];
            for &idx in &indices {
                arr.tuple_as_f64(idx, &mut buf);
                data.extend_from_slice(&buf);
            }
            result
                .point_data_mut()
                .add_array(AnyDataArray::F64(DataArray::from_vec(&name, data, nc)));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_conversion() {
        let image = ImageData::from_function(
            [3, 3, 3],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "density",
            |x, y, z| x + y + z,
        );
        let points = image_to_points(&image);
        assert_eq!(points.points.len(), 27);
        assert_eq!(points.verts.num_cells(), 27);
        assert!(points.point_data().get_array("density").is_some());
    }

    #[test]
    fn threshold_filter() {
        let image = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "value",
            |x, _y, _z| x,
        );
        let points = image_to_points_threshold(&image, "value", 3.0);
        // Only points with x >= 3.0 should remain
        assert!(points.points.len() < 25);
        assert!(points.points.len() > 0);
        assert_eq!(points.verts.num_cells(), points.points.len());
    }

    #[test]
    fn preserves_coordinates() {
        let image = ImageData::with_dimensions(2, 2, 1)
            .with_spacing([0.5, 0.5, 1.0])
            .with_origin([1.0, 2.0, 0.0]);
        let points = image_to_points(&image);
        assert_eq!(points.points.len(), 4);
        let p0 = points.points.get(0);
        assert!((p0[0] - 1.0).abs() < 1e-10);
        assert!((p0[1] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn single_point_image() {
        // ImageData::new() has extent [0,0,0,0,0,0] → dims [1,1,1] → 1 point
        let image = ImageData::new();
        let points = image_to_points(&image);
        assert_eq!(points.points.len(), 1);
        assert_eq!(points.verts.num_cells(), 1);
    }
}
