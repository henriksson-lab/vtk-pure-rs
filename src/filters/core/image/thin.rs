use crate::data::{AnyDataArray, DataArray, ImageData};

/// Morphological thinning (skeletonization) of a 2D binary ImageData.
///
/// Iteratively removes boundary pixels that don't disconnect the
/// foreground until only a 1-pixel-wide skeleton remains.
/// Works on XY slices (nz=1). Adds "Skeleton" array.
pub fn image_thin(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;
    if n == 0 || arr.num_tuples() < n {
        return input.clone();
    }
    if nx < 3 || ny < 3 {
        let mut buf = [0.0f64];
        let skeleton: Vec<f64> = (0..n)
            .map(|i| {
                arr.tuple_as_f64(i, &mut buf);
                if buf[0] >= threshold {
                    1.0
                } else {
                    0.0
                }
            })
            .collect();
        let mut out = input.clone();
        out.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Skeleton", skeleton, 1,
            )));
        return out;
    }

    let mut buf = [0.0f64];
    let mut img: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                2.0
            } else {
                0.0
            }
        })
        .collect();

    while vtk_skeleton_2d_pass(&mut img, nx, ny, nz, 0) {}

    let skeleton: Vec<f64> = img
        .iter()
        .map(|&v| if v > 1.0 { 1.0 } else { 0.0 })
        .collect();
    let mut out = input.clone();
    out.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Skeleton", skeleton, 1,
        )));
    out
}

fn vtk_skeleton_2d_pass(grid: &mut [f64], nx: usize, ny: usize, nz: usize, prune: i32) -> bool {
    let mut changed = false;

    for z in 0..nz {
        let slice = z * nx * ny;
        for y in 0..ny {
            for x in 0..nx {
                let idx = slice + y * nx + x;
                if grid[idx] == 0.0 {
                    continue;
                }

                let n = [
                    neighbor(grid, nx, ny, slice, x, y, -1, 0),
                    neighbor(grid, nx, ny, slice, x, y, -1, -1),
                    neighbor(grid, nx, ny, slice, x, y, 0, -1),
                    neighbor(grid, nx, ny, slice, x, y, 1, -1),
                    neighbor(grid, nx, ny, slice, x, y, 1, 0),
                    neighbor(grid, nx, ny, slice, x, y, 1, 1),
                    neighbor(grid, nx, ny, slice, x, y, 0, 1),
                    neighbor(grid, nx, ny, slice, x, y, -1, 1),
                ];

                if vtk_skeleton_erodes(n, prune) {
                    grid[idx] = 1.0;
                    changed = true;
                }
            }
        }
    }

    for value in grid {
        if *value <= 1.0 {
            *value = 0.0;
        }
    }

    changed
}

fn neighbor(
    grid: &[f64],
    nx: usize,
    ny: usize,
    slice: usize,
    x: usize,
    y: usize,
    dx: isize,
    dy: isize,
) -> f64 {
    let Some(xx) = x.checked_add_signed(dx) else {
        return 0.0;
    };
    let Some(yy) = y.checked_add_signed(dy) else {
        return 0.0;
    };
    if xx >= nx || yy >= ny {
        0.0
    } else {
        grid[slice + yy * nx + xx]
    }
}

fn vtk_skeleton_erodes(n: [f64; 8], prune: i32) -> bool {
    let mut erode_case = 0;
    for idx in (0..8).rev() {
        if n[idx] > 0.0 {
            erode_case += 1;
        }
        if idx != 0 {
            erode_case *= 2;
        }
    }

    if erode_case == 54 || erode_case == 216 {
        return true;
    }
    if erode_case == 99 || erode_case == 141 {
        return false;
    }

    let count_faces =
        (n[0] > 0.0) as i32 + (n[2] > 0.0) as i32 + (n[4] > 0.0) as i32 + (n[6] > 0.0) as i32;
    let count_corners =
        (n[1] > 0.0) as i32 + (n[3] > 0.0) as i32 + (n[5] > 0.0) as i32 + (n[7] > 0.0) as i32;

    if count_faces == 2 && count_corners == 0 && n[2] > 0.0 && n[4] > 0.0 {
        return true;
    }
    if prune > 1 && count_faces + count_corners <= 1 {
        return true;
    }

    (n[0] == 0.0 || n[2] == 0.0 || n[4] == 0.0 || n[6] == 0.0)
        && (prune > 1
            || count_faces != 1
            || count_corners != 2
            || ((n[1] == 0.0 || n[2] == 0.0 || n[3] == 0.0)
                && (n[3] == 0.0 || n[4] == 0.0 || n[5] == 0.0)
                && (n[5] == 0.0 || n[6] == 0.0 || n[7] == 0.0)
                && (n[7] == 0.0 || n[0] == 0.0 || n[1] == 0.0)))
        && (prune != 0
            || count_faces != 2
            || count_corners != 2
            || ((n[1] == 0.0 || n[2] == 0.0 || n[3] == 0.0 || n[4] != 0.0)
                && (n[0] == 0.0 || n[1] == 0.0 || n[2] == 0.0 || n[3] != 0.0)
                && (n[7] == 0.0 || n[0] == 0.0 || n[1] == 0.0 || n[2] != 0.0)
                && (n[6] == 0.0 || n[7] == 0.0 || n[0] == 0.0 || n[1] != 0.0)
                && (n[5] == 0.0 || n[6] == 0.0 || n[7] == 0.0 || n[0] != 0.0)
                && (n[4] == 0.0 || n[5] == 0.0 || n[6] == 0.0 || n[7] != 0.0)
                && (n[3] == 0.0 || n[4] == 0.0 || n[5] == 0.0 || n[6] != 0.0)
                && (n[2] == 0.0 || n[3] == 0.0 || n[4] == 0.0 || n[5] != 0.0)))
        && (n[1] == 0.0 || n[0] > 1.0 || n[2] > 1.0)
        && (n[3] == 0.0 || n[2] > 1.0 || n[4] > 1.0)
        && (n[5] == 0.0 || n[4] > 1.0 || n[6] > 1.0)
        && (n[7] == 0.0 || n[6] > 1.0 || n[0] > 1.0)
        && (n[0] == 0.0 || n[4] == 0.0 || n[2] > 1.0 || n[6] > 1.0)
        && (n[2] == 0.0 || n[6] == 0.0 || n[0] > 1.0 || n[4] > 1.0)
        && (prune > 1 || count_faces > 2 || (count_faces == 2 && count_corners > 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thin_rectangle() {
        let mut img = ImageData::with_dimensions(10, 5, 1);
        let mut values = vec![0.0; 50];
        for j in 1..4 {
            for i in 1..9 {
                values[j * 10 + i] = 1.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_thin(&img, "v", 0.5);
        let arr = result.point_data().get_array("Skeleton").unwrap();
        let mut buf = [0.0f64];
        let mut count = 0;
        for i in 0..50 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        // Skeleton should have fewer pixels than the rectangle
        assert!(count < 24); // original has 24 foreground pixels
        assert!(count > 0);
    }

    #[test]
    fn single_pixel_preserved() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0; 25];
        values[12] = 1.0; // single pixel
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_thin(&img, "v", 0.5);
        let arr = result.point_data().get_array("Skeleton").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(5, 5, 1);
        let r = image_thin(&img, "nope", 0.5);
        assert!(r.point_data().get_array("Skeleton").is_none());
    }

    #[test]
    fn tiny_image_does_not_underflow() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 1.0, 0.0],
                1,
            )));

        let result = image_thin(&img, "v", 0.5);
        let arr = result.point_data().get_array("Skeleton").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }
}
