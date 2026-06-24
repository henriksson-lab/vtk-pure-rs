use crate::data::{AnyDataArray, DataArray, ImageData, PolyData};

/// Compute the Euclidean distance from each voxel center to the nearest point
/// in a PolyData surface.
///
/// Uses brute force nearest-point search (O(num_voxels * num_surface_points)).
/// Adds a "DistanceToSurface" point data array to the returned ImageData.
pub fn compute_distance_to_surface(image: &ImageData, surface: &PolyData) -> ImageData {
    let dims = image.dimensions();
    let spacing = image.spacing();
    let origin = image.origin();

    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let num_voxels: usize = nx * ny * nz;

    // Collect surface points into a Vec for faster iteration
    let num_surf_pts: usize = surface.points.len();
    let mut surf_pts: Vec<[f64; 3]> = Vec::with_capacity(num_surf_pts);
    for i in 0..num_surf_pts {
        surf_pts.push(surface.points.get(i));
    }

    let mut distances: Vec<f64> = Vec::with_capacity(num_voxels);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let vx: f64 = origin[0] + (i as f64) * spacing[0];
                let vy: f64 = origin[1] + (j as f64) * spacing[1];
                let vz: f64 = origin[2] + (k as f64) * spacing[2];

                let mut min_dist_sq: f64 = f64::MAX;
                for sp in &surf_pts {
                    let dx: f64 = vx - sp[0];
                    let dy: f64 = vy - sp[1];
                    let dz: f64 = vz - sp[2];
                    let d2: f64 = dx * dx + dy * dy + dz * dz;
                    if d2 < min_dist_sq {
                        min_dist_sq = d2;
                    }
                }

                if min_dist_sq == f64::MAX {
                    distances.push(0.0);
                } else {
                    distances.push(min_dist_sq.sqrt());
                }
            }
        }
    }

    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "DistanceToSurface",
            distances,
            1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Points;

    fn make_single_point_surface(pt: [f64; 3]) -> PolyData {
        let mut pd = PolyData::default();
        pd.points.push(pt);
        pd
    }

    #[test]
    fn distance_to_origin_point() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.set_spacing([1.0, 1.0, 1.0]);
        img.set_origin([0.0, 0.0, 0.0]);

        let surface = make_single_point_surface([0.0, 0.0, 0.0]);
        let result = compute_distance_to_surface(&img, &surface);
        let arr = result.point_data().get_array("DistanceToSurface").unwrap();
        assert_eq!(arr.num_tuples(), 9);

        // Voxel at (0,0,0) should have distance 0
        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 0.0).abs() < 1e-10);

        // Voxel at (1,0,0) should have distance 1
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-10);

        // Voxel at (1,1,0) should have distance sqrt(2)
        arr.tuple_as_f64(4, &mut val);
        let expected: f64 = 2.0f64.sqrt();
        assert!((val[0] - expected).abs() < 1e-10);
    }

    #[test]
    fn empty_surface_returns_zero() {
        let img = ImageData::with_dimensions(2, 2, 1);
        let surface = PolyData::default();
        let result = compute_distance_to_surface(&img, &surface);
        let arr = result.point_data().get_array("DistanceToSurface").unwrap();
        let mut val = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut val);
            assert!((val[0] - 0.0).abs() < 1e-10);
        }
    }

    #[test]
    fn distance_with_spacing() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.set_spacing([2.0, 1.0, 1.0]);
        img.set_origin([0.0, 0.0, 0.0]);

        let surface = make_single_point_surface([0.0, 0.0, 0.0]);
        let result = compute_distance_to_surface(&img, &surface);
        let arr = result.point_data().get_array("DistanceToSurface").unwrap();

        let mut val = [0.0f64];
        arr.tuple_as_f64(0, &mut val);
        assert!((val[0] - 0.0).abs() < 1e-10);

        // Second voxel at x=2.0
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-10);
    }
}
