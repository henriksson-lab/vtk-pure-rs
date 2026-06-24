use crate::data::{AnyDataArray, DataArray, ImageData};

/// Create a checkerboard comparison of two ImageData scalar fields.
///
/// Alternate tiles from image A and image B. The tile size is given in
/// number of voxels per tile in each dimension.
///
/// Both images must have the same dimensions and the same named scalar array.
/// Returns a new ImageData with a "Checkerboard" scalar array.
pub fn image_checkerboard(
    a: &ImageData,
    b: &ImageData,
    scalars: &str,
    tile_size: [usize; 3],
) -> ImageData {
    let arr_a = match a.point_data().get_array(scalars) {
        Some(x) => x,
        None => return a.clone(),
    };
    let arr_b = match b.point_data().get_array(scalars) {
        Some(x) => x,
        None => return a.clone(),
    };

    let dims = a.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;

    let tx = tile_size[0].max(1);
    let ty = tile_size[1].max(1);
    let tz = tile_size[2].max(1);

    let mut buf = [0.0f64];
    let mut values = Vec::with_capacity(n);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let idx = k * ny * nx + j * nx + i;
                let parity = (i / tx + j / ty + k / tz) % 2;
                if parity == 0 {
                    arr_a.tuple_as_f64(idx, &mut buf);
                } else {
                    arr_b.tuple_as_f64(idx, &mut buf);
                }
                values.push(buf[0]);
            }
        }
    }

    let mut img = a.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Checkerboard",
            values,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_img(val: f64, nx: usize) -> ImageData {
        let mut img = ImageData::with_dimensions(nx, nx, 1);
        let vals = vec![val; nx * nx];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 1)));
        img
    }

    #[test]
    fn checkerboard_alternates() {
        let a = make_img(1.0, 4);
        let b = make_img(2.0, 4);
        let result = image_checkerboard(&a, &b, "v", [2, 2, 1]);

        let arr = result.point_data().get_array("Checkerboard").unwrap();
        let mut buf = [0.0f64];

        // (0,0) -> tile (0,0) -> parity 0 -> from A = 1.0
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);

        // (2,0) -> tile (1,0) -> parity 1 -> from B = 2.0
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 2.0);

        // (0,2) -> tile (0,1) -> parity 1 -> from B = 2.0
        arr.tuple_as_f64(8, &mut buf); // index = 2*4+0 = 8
        assert_eq!(buf[0], 2.0);

        // (2,2) -> tile (1,1) -> parity 0 -> from A = 1.0
        arr.tuple_as_f64(10, &mut buf); // index = 2*4+2 = 10
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let a = ImageData::with_dimensions(3, 3, 1);
        let b = ImageData::with_dimensions(3, 3, 1);
        let result = image_checkerboard(&a, &b, "nope", [1, 1, 1]);
        assert!(result.point_data().get_array("Checkerboard").is_none());
    }
}
