//! Image tiling and mosaic operations.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Tile an image NxM times by repeating the content.
pub fn tile_image(image: &ImageData, array_name: &str, nx: usize, ny: usize) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let new_dims = [dims[0] * nx, dims[1] * ny, dims[2]];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..dims[0] * dims[1] * dims[2])
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();

    let n = new_dims[0] * new_dims[1] * new_dims[2];
    let data: Vec<f64> = (0..n)
        .map(|idx| {
            let iz = idx / (new_dims[0] * new_dims[1]);
            let rem = idx % (new_dims[0] * new_dims[1]);
            let iy = rem / new_dims[0];
            let ix = rem % new_dims[0];
            let ox = ix % dims[0];
            let oy = iy % dims[1];
            let oz = iz % dims[2];
            vals[ox + oy * dims[0] + oz * dims[0] * dims[1]]
        })
        .collect();

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
}

/// Mirror-tile: tile with alternating flips for seamless tiling.
pub fn mirror_tile(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let sp = image.spacing();
    let new_dims = [dims[0] * 2, dims[1] * 2, dims[2]];
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..dims[0] * dims[1] * dims[2])
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0]
            } else {
                0.0
            }
        })
        .collect();

    let n = new_dims[0] * new_dims[1] * new_dims[2];
    let data: Vec<f64> = (0..n)
        .map(|idx| {
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
            vals[ox + oy * dims[0] + oz * dims[0] * dims[1]]
        })
        .collect();

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin(image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
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
    if ad[1] != bd[1] || ad[2] != bd[2] {
        return a.clone();
    }
    let new_dims = [ad[0] + bd[0], ad[1], ad[2]];
    let sp = a.spacing();
    let mut buf = [0.0f64];
    let mut data = Vec::with_capacity(new_dims[0] * new_dims[1] * new_dims[2]);
    for iz in 0..new_dims[2] {
        for iy in 0..new_dims[1] {
            for ix in 0..new_dims[0] {
                if ix < ad[0] {
                    let idx = ix + iy * ad[0] + iz * ad[0] * ad[1];
                    if idx < a_arr.num_tuples() {
                        a_arr.tuple_as_f64(idx, &mut buf);
                        data.push(buf[0]);
                    } else {
                        data.push(0.0);
                    }
                } else {
                    let bx = ix - ad[0];
                    let idx = bx + iy * bd[0] + iz * bd[0] * bd[1];
                    if idx < b_arr.num_tuples() {
                        b_arr.tuple_as_f64(idx, &mut buf);
                        data.push(buf[0]);
                    } else {
                        data.push(0.0);
                    }
                }
            }
        }
    }

    ImageData::with_dimensions(new_dims[0], new_dims[1], new_dims[2])
        .with_spacing(sp)
        .with_origin(a.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(array_name, data, 1)))
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
}
