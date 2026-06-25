//! Advanced morphological operations: top-hat, hit-or-miss, skeleton.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// White top-hat: original - opening (highlights bright features smaller than SE).
pub fn white_top_hat(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let opened = crate::filters::core::image::morphology_3d::open_3d(image, array_name, radius);
    subtract_images(image, &opened, array_name)
}

/// Black top-hat: closing - original (highlights dark features smaller than SE).
pub fn black_top_hat(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let closed = crate::filters::core::image::morphology_3d::close_3d(image, array_name, radius);
    subtract_images(&closed, image, array_name)
}

/// Morphological skeleton via iterative thinning (2D).
///
/// This follows the erosion rules from `vtkImageSkeleton2D` on each XY slice.
pub fn morphological_skeleton_2d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let n = nx * ny * nz;
    let mut buf = [0.0f64];
    let mut grid: Vec<f64> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                if buf[0] > 0.5 {
                    2.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        })
        .collect();

    while skeleton_2d_vtk_pass(&mut grid, nx, ny, nz) {}

    let output: Vec<f64> = grid
        .iter()
        .map(|&v| if v > 1.0 { 1.0 } else { 0.0 })
        .collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

fn skeleton_2d_vtk_pass(grid: &mut [f64], nx: usize, ny: usize, nz: usize) -> bool {
    let mut changed = false;
    if nx == 0 || ny == 0 || nz == 0 {
        return false;
    }

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

                if vtk_skeleton_erodes(n, 0) {
                    grid[idx] = 1.0;
                    changed = true;
                }
            }
        }
    }

    for v in grid {
        if *v <= 1.0 {
            *v = 0.0;
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

fn subtract_images(a: &ImageData, b: &ImageData, array_name: &str) -> ImageData {
    let a_arr = match a.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let b_arr = match b.point_data().get_array(array_name) {
        Some(x) => x,
        None => return a.clone(),
    };
    let n = a_arr.num_tuples().min(b_arr.num_tuples());
    let mut output = Vec::with_capacity(n);
    let mut ab = [0.0f64];
    let mut bb = [0.0f64];
    for i in 0..n {
        a_arr.tuple_as_f64(i, &mut ab);
        b_arr.tuple_as_f64(i, &mut bb);
        output.push((ab[0] - bb[0]).max(0.0));
    }
    let mut result = a.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn top_hat() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 5.0).powi(2) + (y - 5.0).powi(2) < 4.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = white_top_hat(&img, "v", 1);
        assert!(result.point_data().get_array("v").is_some());
    }
    #[test]
    fn skeleton() {
        let img = ImageData::from_function(
            [20, 20, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if x > 3.0 && x < 17.0 && y > 8.0 && y < 12.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let result = morphological_skeleton_2d(&img, "v");
        // Skeleton of a rectangle should be thinner
        let arr = result.point_data().get_array("v").unwrap();
        let mut count = 0;
        let mut buf = [0.0f64];
        for i in 0..arr.num_tuples() {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        let orig = img.point_data().get_array("v").unwrap();
        let mut orig_count = 0;
        for i in 0..orig.num_tuples() {
            orig.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                orig_count += 1;
            }
        }
        assert!(count < orig_count, "skeleton should have fewer pixels");
    }
}
