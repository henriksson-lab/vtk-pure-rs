use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Morphological skeletonization of a binary image.
///
/// Follows the erosion rules from `vtkImageSkeleton2D` independently on each
/// XY slice. The result has a "Skeleton" scalar array in point data with
/// values 0.0 or 1.0.
pub fn morphological_skeleton(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        None => return input.clone(),
        _ => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    let mut grid: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        grid[i] = if buf[0] > 0.5 { 2.0 } else { 0.0 };
    }

    while skeleton_2d_vtk_pass(&mut grid, nx, ny, nz) {}
    let skeleton: Vec<f64> = grid
        .iter()
        .map(|&v| if v > 1.0 { 1.0 } else { 0.0 })
        .collect();

    // Build output
    let mut output: ImageData = input.clone();
    let mut new_attrs: DataSetAttributes = DataSetAttributes::new();
    // Copy existing arrays
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        new_attrs.add_array(a.clone());
    }
    // Add skeleton array
    new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
        "Skeleton", skeleton, 1,
    )));
    *output.point_data_mut() = new_attrs;
    output
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cross_image() -> ImageData {
        // 7x7x1 image with a cross pattern
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values: Vec<f64> = vec![0.0; 49];
        // Horizontal bar at row 3
        for i in 1..6 {
            values[3 * 7 + i] = 1.0;
        }
        // Vertical bar at col 3
        for j in 1..6 {
            values[j * 7 + 3] = 1.0;
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("Binary", values, 1)));
        img
    }

    #[test]
    fn skeleton_is_subset_of_original() {
        let img = make_cross_image();
        let result = morphological_skeleton(&img, "Binary");
        let skel = result.point_data().get_array("Skeleton").unwrap();
        let orig = img.point_data().get_array("Binary").unwrap();
        let mut buf_s: [f64; 1] = [0.0];
        let mut buf_o: [f64; 1] = [0.0];
        for i in 0..49 {
            skel.tuple_as_f64(i, &mut buf_s);
            orig.tuple_as_f64(i, &mut buf_o);
            if buf_s[0] > 0.5 {
                assert!(
                    buf_o[0] > 0.5,
                    "skeleton pixel {} is set but original is not",
                    i
                );
            }
        }
    }

    #[test]
    fn skeleton_of_filled_block() {
        // A 5x5x1 filled block should have a skeleton at the center
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = vec![1.0; 25];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        let result = morphological_skeleton(&img, "mask");
        let skel = result.point_data().get_array("Skeleton").unwrap();
        // The center pixel (2,2) should be in the skeleton
        let mut buf: [f64; 1] = [0.0];
        skel.tuple_as_f64(2 * 5 + 2, &mut buf);
        assert!(buf[0] > 0.5, "center should be in skeleton");

        let mut count = 0;
        for i in 0..25 {
            skel.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                count += 1;
            }
        }
        assert!(count < 25, "skeleton should thin the filled block");
    }

    #[test]
    fn empty_image_produces_empty_skeleton() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let values: Vec<f64> = vec![0.0; 25];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("mask", values, 1)));
        let result = morphological_skeleton(&img, "mask");
        let skel = result.point_data().get_array("Skeleton").unwrap();
        let mut buf: [f64; 1] = [0.0];
        for i in 0..25 {
            skel.tuple_as_f64(i, &mut buf);
            assert!(buf[0] < 0.5, "empty image should have empty skeleton");
        }
    }
}
