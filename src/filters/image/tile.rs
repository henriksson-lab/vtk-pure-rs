use crate::data::{AnyDataArray, DataArray, ImageData};

/// Tile (repeat) an ImageData in X and Y.
///
/// Creates a new ImageData that repeats the input `nx_tiles × ny_tiles` times.
pub fn image_tile(input: &ImageData, scalars: &str, nx_tiles: usize, ny_tiles: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let sx = dims[0] as usize;
    let sy = dims[1] as usize;
    let sz = dims[2] as usize;
    let spacing = input.spacing();
    let origin = input.origin();
    let ntx = nx_tiles.max(1);
    let nty = ny_tiles.max(1);
    let out_nx = sx * ntx;
    let out_ny = sy * nty;

    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let mut values = Vec::with_capacity(out_nx * out_ny * sz * nc);

    for k in 0..sz {
        for j in 0..out_ny {
            for i in 0..out_nx {
                let si = i % sx;
                let sj = j % sy;
                arr.tuple_as_f64(k * sy * sx + sj * sx + si, &mut buf);
                values.extend_from_slice(&buf);
            }
        }
    }

    let mut img = ImageData::with_dimensions(out_nx, out_ny, sz);
    img.set_origin(origin);
    img.set_spacing(spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, values, nc)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_2x2() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let result = image_tile(&img, "v", 2, 2);
        assert_eq!(result.dimensions(), [4, 4, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0); // tiled
    }

    #[test]
    fn tile_1x1_noop() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));
        let result = image_tile(&img, "v", 1, 1);
        assert_eq!(result.dimensions(), [3, 3, 1]);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(2, 2, 1);
        let r = image_tile(&img, "nope", 2, 2);
        assert_eq!(r.dimensions(), [2, 2, 1]);
    }

    #[test]
    fn tile_preserves_components() {
        let mut img = ImageData::with_dimensions(1, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                2,
            )));

        let result = image_tile(&img, "v", 2, 1);
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [1.0, 2.0]);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf, [3.0, 4.0]);
    }
}
