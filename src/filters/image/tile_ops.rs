//! Image tiling and mosaic operations.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Tile an image NxM times by repeating the content.
pub fn tile_image(image: &ImageData, array_name: &str, nx: usize, ny: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    if dims.contains(&0) || nx == 0 || ny == 0 {
        return image.clone();
    }
    if arr.num_tuples() < dims[0] * dims[1] * dims[2] {
        return image.clone();
    }
    let sp = image.spacing();
    let new_dims = [dims[0] * nx, dims[1] * ny, dims[2]];
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let vals: Vec<f64> = (0..dims[0] * dims[1] * dims[2])
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.clone()
        })
        .flatten()
        .collect();

    let n = new_dims[0] * new_dims[1] * new_dims[2];
    let mut data: Vec<f64> = Vec::with_capacity(n * nc);
    for idx in 0..n {
        let iz = idx / (new_dims[0] * new_dims[1]);
        let rem = idx % (new_dims[0] * new_dims[1]);
        let iy = rem / new_dims[0];
        let ix = rem % new_dims[0];
        let ox = ix % dims[0];
        let oy = iy % dims[1];
        let oz = iz % dims[2];
        let src = (ox + oy * dims[0] + oz * dims[0] * dims[1]) * nc;
        data.extend_from_slice(&vals[src..src + nc]);
    }

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, nc)))
}

/// Mirror-tile: tile with alternating flips for seamless tiling.
pub fn mirror_tile(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    if dims.contains(&0) {
        return image.clone();
    }
    if arr.num_tuples() < dims[0] * dims[1] * dims[2] {
        return image.clone();
    }
    let sp = image.spacing();
    let new_dims = [dims[0] * 2, dims[1] * 2, dims[2]];
    let nc = arr.num_components();
    let mut buf = vec![0.0f64; nc];
    let vals: Vec<f64> = (0..dims[0] * dims[1] * dims[2])
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.clone()
        })
        .flatten()
        .collect();

    let n = new_dims[0] * new_dims[1] * new_dims[2];
    let mut data: Vec<f64> = Vec::with_capacity(n * nc);
    for idx in 0..n {
        let iz = idx / (new_dims[0] * new_dims[1]);
        let rem = idx % (new_dims[0] * new_dims[1]);
        let iy = rem / new_dims[0];
        let ix = rem % new_dims[0];
        let tile_x = ix / dims[0];
        let tile_y = iy / dims[1];
        let mut ox = ix % dims[0];
        let mut oy = iy % dims[1];
        let oz = iz % dims[2];
        if tile_x % 2 == 1 {
            ox = dims[0] - 1 - ox;
        } // flip
        if tile_y % 2 == 1 {
            oy = dims[1] - 1 - oy;
        }
        let src = (ox + oy * dims[0] + oz * dims[0] * dims[1]) * nc;
        data.extend_from_slice(&vals[src..src + nc]);
    }

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, nc)))
}

/// Stitch two images side by side along X.
pub fn stitch_x(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    let a_arr = match a.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let b_arr = match b.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let ad = a.dimensions();
    let bd = b.dimensions();
    if ad.contains(&0) || bd.contains(&0) {
        return a.clone();
    }
    if ad[1] != bd[1] || ad[2] != bd[2] {
        return a.clone();
    }
    let new_dims = [ad[0] + bd[0], ad[1], ad[2]];
    let sp = a.spacing();
    let nc = a_arr.num_components();
    if b_arr.num_components() != nc {
        return a.clone();
    }
    let mut buf = vec![0.0f64; nc];
    let mut data = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2] * nc);
    for iz in 0..new_dims[2] {
        for iy in 0..new_dims[1] {
            for ix in 0..new_dims[0] {
                if ix < ad[0] {
                    let idx = ix + iy * ad[0] + iz * ad[0] * ad[1];
                    if idx < a_arr.num_tuples() {
                        a_arr.tuple_as_f64(idx, &mut buf);
                        data.extend_from_slice(&buf);
                    } else {
                        data.extend(std::iter::repeat(0.0).take(nc));
                    }
                } else {
                    let bx = ix - ad[0];
                    let idx = bx + iy * bd[0] + iz * bd[0] * bd[1];
                    if idx < b_arr.num_tuples() {
                        b_arr.tuple_as_f64(idx, &mut buf);
                        data.extend_from_slice(&buf);
                    } else {
                        data.extend(std::iter::repeat(0.0).take(nc));
                    }
                }
            }
        }
    }

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin(a.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, nc)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tile() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = tile_image(&img, "v", 2, 2);
        assert_eq!(result.dimensions(), [10, 10, 1]);
    }
    #[test]
    fn mirror() {
        let img = ImageData::from_function(
            [4, 4, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let result = mirror_tile(&img, "v");
        assert_eq!(result.dimensions(), [8, 8, 1]);
    }
    #[test]
    fn stitch() {
        let a = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        let b = ImageData::from_function(
            [3, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 2.0,
        );
        let result = stitch_x(&a, &b, "v");
        assert_eq!(result.dimensions(), [8, 5, 1]);
    }

    #[test]
    fn tile_preserves_components_and_rejects_zero_counts() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                2,
            )));

        let result = tile_image(&img, "v", 0, 2);
        assert_eq!(result.dimensions(), [2, 1, 1]);
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 2.0]);
    }

    #[test]
    fn stitch_preserves_components() {
        let mut a = ImageData::with_dimensions(1, 1, 1);
        a.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0],
                2,
            )));
        let mut b = ImageData::with_dimensions(1, 1, 1);
        b.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![3.0, 4.0],
                2,
            )));

        let result = stitch_x(&a, &b, "v");
        let arr = result.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [3.0, 4.0]);
    }
}
