use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a spherical region-of-interest mask to ImageData.
///
/// Voxels inside the sphere keep their values; outside voxels are set to `outside_value`.
pub fn image_sphere_roi(
    input: &ImageData,
    scalars: &str,
    center: [f64; 3],
    radius: f64,
    outside_value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    if arr.num_tuples() < n {
        return input.clone();
    }
    let num_comp = arr.num_components();
    let r2 = radius * radius;

    let mut buf = vec![0.0f64; num_comp];
    let mut values = Vec::with_capacity(nx * ny * nz * num_comp);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = input.point_from_ijk(i, j, k);
                let x = p[0] - center[0];
                let y = p[1] - center[1];
                let z = p[2] - center[2];
                arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                if x * x + y * y + z * z <= r2 {
                    values.extend_from_slice(&buf);
                } else {
                    values.extend(std::iter::repeat(outside_value).take(num_comp));
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .field_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, num_comp,
        )));
    img
}

/// Apply a box ROI mask to ImageData.
pub fn image_box_roi(
    input: &ImageData,
    scalars: &str,
    bounds: [f64; 6],
    outside_value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    if arr.num_tuples() < n {
        return input.clone();
    }
    let num_comp = arr.num_components();

    let mut buf = vec![0.0f64; num_comp];
    let mut values = Vec::with_capacity(nx * ny * nz * num_comp);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let [x, y, z] = input.point_from_ijk(i, j, k);
                arr.tuple_as_f64(k * ny * nx + j * nx + i, &mut buf);
                let inside = x >= bounds[0]
                    && x <= bounds[1]
                    && y >= bounds[2]
                    && y <= bounds[3]
                    && z >= bounds[4]
                    && z <= bounds[5];
                if inside {
                    values.extend_from_slice(&buf);
                } else {
                    values.extend(std::iter::repeat(outside_value).take(num_comp));
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .field_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars, values, num_comp,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_roi() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 25],
                1,
            )));

        let result = image_sphere_roi(&img, "v", [2.0, 2.0, 0.0], 1.5, 0.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0); // center inside
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // corner outside
    }

    #[test]
    fn box_roi() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 25],
                1,
            )));

        let result = image_box_roi(&img, "v", [1.0, 3.0, 1.0, 3.0, -1.0, 1.0], 0.0);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0); // (2,2) inside
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // (0,0) outside
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_sphere_roi(&img, "nope", [0.0; 3], 1.0, 0.0);
        assert_eq!(r.dimensions(), [3, 3, 1]);
    }
}
