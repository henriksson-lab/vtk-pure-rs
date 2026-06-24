//! Draw geometric primitives on images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Draw a filled circle on the image.
pub fn draw_circle(
    input: &ImageData,
    scalars: &str,
    cx: f64,
    cy: f64,
    radius: f64,
    value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let r2 = radius * radius;
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|idx| {
            arr.tuple_as_f64(idx, &mut buf);
            let iy = idx / nx;
            let ix = idx % nx;
            let dx = ix as f64 - cx;
            let dy = iy as f64 - cy;
            if dx * dx + dy * dy <= r2 {
                value
            } else {
                buf[0]
            }
        })
        .collect();
    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Draw a filled rectangle on the image.
pub fn draw_rect(
    input: &ImageData,
    scalars: &str,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0];
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|idx| {
            arr.tuple_as_f64(idx, &mut buf);
            let iy = idx / nx;
            let ix = idx % nx;
            if ix >= x0 && ix <= x1 && iy >= y0 && iy <= y1 {
                value
            } else {
                buf[0]
            }
        })
        .collect();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

/// Draw a line using Bresenham's algorithm.
pub fn draw_line(
    input: &ImageData,
    scalars: &str,
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    value: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let mut data: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: isize = if x0 < x1 { 1 } else { -1 };
    let sy: isize = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0;
    let mut cy = y0;
    loop {
        if cx >= 0 && cx < nx as isize && cy >= 0 && cy < ny as isize {
            data[cx as usize + cy as usize * nx] = value;
        }
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn blank(n: usize) -> ImageData {
        ImageData::from_function(
            [n, n, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        )
    }
    #[test]
    fn test_circle() {
        let r = draw_circle(&blank(20), "v", 10.0, 10.0, 3.0, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(10 + 10 * 20, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
    #[test]
    fn test_rect() {
        let r = draw_rect(&blank(10), "v", 2, 2, 5, 5, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(3 + 3 * 10, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
    #[test]
    fn test_line() {
        let r = draw_line(&blank(10), "v", 0, 0, 9, 9, 1.0);
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5 + 5 * 10, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
