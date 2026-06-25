//! Draw geometric primitives on images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Draw a circle outline on the image.
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
    let mut data: Vec<f64> = (0..arr.num_tuples())
        .map(|idx| {
            arr.tuple_as_f64(idx, &mut buf);
            buf[0]
        })
        .collect();
    let radius = radius + 0.1;
    if radius > 0.0 {
        let number_of_steps = (2.0 * std::f64::consts::PI * radius).ceil() as usize;
        let theta_cos = (1.0 / radius).cos();
        let theta_sin = (1.0 / radius).sin();
        let mut x = radius;
        let mut y = 0.0;
        for _ in 0..number_of_steps {
            let p0 = cx as isize + x as isize;
            let p1 = cy as isize + y as isize;
            if p0 >= 0 && p0 < nx as isize && p1 >= 0 && p1 < ny as isize {
                data[p1 as usize * nx + p0 as usize] = value;
            }
            let temp = theta_cos * x + theta_sin * y;
            y = theta_cos * y - theta_sin * x;
            x = temp;
        }
    }
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
    let ny = dims[1];
    let min_x = x0.min(nx.saturating_sub(1));
    let max_x = x1.min(nx.saturating_sub(1));
    let min_y = y0.min(ny.saturating_sub(1));
    let max_y = y1.min(ny.saturating_sub(1));
    let mut buf = [0.0f64];
    let data: Vec<f64> = (0..arr.num_tuples())
        .map(|idx| {
            arr.tuple_as_f64(idx, &mut buf);
            let iy = idx / nx;
            let ix = idx % nx;
            if ix >= min_x && ix <= max_x && iy >= min_y && iy <= max_y {
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

/// Draw a line segment on the image.
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

    if let Some((mut a0, mut a1, b0, b1)) = clip_segment(x0, y0, x1, y1, nx as isize, ny as isize) {
        a0 -= b0;
        a1 -= b1;
        let mut inc0 = if a0 < 0 { -1 } else { 1 };
        let mut inc1 = if a1 < 0 { -(nx as isize) } else { nx as isize };
        let p0 = a0.abs();
        let p1 = a1.abs();
        if p0 == 0 {
            inc0 = 0;
        }
        if p1 == 0 {
            inc1 = 0;
        }
        let number_of_steps = p0.max(p1);
        let s0 = if number_of_steps == 0 {
            0.0
        } else {
            p0 as f64 / number_of_steps as f64
        };
        let s1 = if number_of_steps == 0 {
            0.0
        } else {
            p1 as f64 / number_of_steps as f64
        };
        let mut f0 = 0.5;
        let mut f1 = 0.5;
        let mut ptr = b1 * nx as isize + b0;
        data[ptr as usize] = value;
        for _ in 0..number_of_steps {
            f0 += s0;
            if f0 > 1.0 {
                ptr += inc0;
                f0 -= 1.0;
            }
            f1 += s1;
            if f1 > 1.0 {
                ptr += inc1;
                f1 -= 1.0;
            }
            data[ptr as usize] = value;
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

fn clip_segment(
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    nx: isize,
    ny: isize,
) -> Option<(isize, isize, isize, isize)> {
    let (mut a0, mut a1, mut b0, mut b1) = (x0, y0, x1, y1);
    let min0 = 0;
    let max0 = nx - 1;
    let min1 = 0;
    let max1 = ny - 1;

    if a0 < min0 && b0 < min0 {
        return None;
    }
    if a0 < min0 && b0 >= min0 {
        let fract = (b0 - min0) as f64 / (b0 - a0) as f64;
        a0 = min0;
        a1 = b1 + (fract * (a1 - b1) as f64) as isize;
    }
    if b0 < min0 && a0 >= min0 {
        let fract = (a0 - min0) as f64 / (a0 - b0) as f64;
        b0 = min0;
        b1 = a1 + (fract * (b1 - a1) as f64) as isize;
    }

    if a0 > max0 && b0 > max0 {
        return None;
    }
    if a0 > max0 && b0 <= max0 {
        let fract = (b0 - max0) as f64 / (b0 - a0) as f64;
        a0 = max0;
        a1 = b1 + (fract * (a1 - b1) as f64) as isize;
    }
    if b0 > max0 && a0 <= max0 {
        let fract = (a0 - max0) as f64 / (a0 - b0) as f64;
        b0 = max0;
        b1 = a1 + (fract * (b1 - a1) as f64) as isize;
    }

    if a1 < min1 && b1 < min1 {
        return None;
    }
    if a1 < min1 && b1 >= min1 {
        let fract = (b1 - min1) as f64 / (b1 - a1) as f64;
        a1 = min1;
        a0 = b0 + (fract * (a0 - b0) as f64) as isize;
    }
    if b1 < min1 && a1 >= min1 {
        let fract = (a1 - min1) as f64 / (a1 - b1) as f64;
        b1 = min1;
        b0 = a0 + (fract * (b0 - a0) as f64) as isize;
    }

    if a1 > max1 && b1 > max1 {
        return None;
    }
    if a1 > max1 && b1 <= max1 {
        let fract = (b1 - max1) as f64 / (b1 - a1) as f64;
        a1 = max1;
        a0 = b0 + (fract * (a0 - b0) as f64) as isize;
    }
    if b1 > max1 && a1 <= max1 {
        let fract = (a1 - max1) as f64 / (a1 - b1) as f64;
        b1 = max1;
        b0 = a0 + (fract * (b0 - a0) as f64) as isize;
    }

    Some((a0, a1, b0, b1))
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
        arr.tuple_as_f64(13 + 10 * 20, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(10 + 10 * 20, &mut buf);
        assert_eq!(buf[0], 0.0);
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
