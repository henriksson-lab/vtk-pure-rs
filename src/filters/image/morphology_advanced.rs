//! Advanced morphological operations: top-hat, hit-or-miss, skeleton.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// White top-hat: original - opening (highlights bright features smaller than SE).
pub fn white_top_hat(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let opened = crate::filters::image::morphology_3d::open_3d(image, array_name, radius);
    subtract_images(image, &opened, array_name)
}

/// Black top-hat: closing - original (highlights dark features smaller than SE).
pub fn black_top_hat(image: &ImageData, array_name: &str, radius: usize) -> ImageData {
    let closed = crate::filters::image::morphology_3d::close_3d(image, array_name, radius);
    subtract_images(&closed, image, array_name)
}

/// Morphological skeleton via iterative thinning (2D).
pub fn morphological_skeleton_2d(image: &ImageData, array_name: &str) -> ImageData {
    let arr = match image.point_data().get_array(array_name) {
        Some(a) if a.num_components() == 1 => a,
        _ => return image.clone(),
    };
    let dims = image.dimensions();
    let n = dims[0] * dims[1];
    let mut buf = [0.0f64];
    let mut grid: Vec<bool> = (0..n)
        .map(|i| {
            if i < arr.num_tuples() {
                arr.tuple_as_f64(i, &mut buf);
                buf[0] > 0.5
            } else {
                false
            }
        })
        .collect();

    // Zhang-Suen thinning (simplified)
    let mut changed = true;
    while changed {
        changed = false;
        for pass in 0..2 {
            let mut to_remove = Vec::new();
            for y in 1..dims[1].saturating_sub(1) {
                for x in 1..dims[0].saturating_sub(1) {
                    let idx = x + y * dims[0];
                    if !grid[idx] {
                        continue;
                    }
                    let p = [
                        grid[idx - dims[0]] as u8,     // N
                        grid[idx - dims[0] + 1] as u8, // NE
                        grid[idx + 1] as u8,           // E
                        grid[idx + dims[0] + 1] as u8, // SE
                        grid[idx + dims[0]] as u8,     // S
                        grid[idx + dims[0] - 1] as u8, // SW
                        grid[idx - 1] as u8,           // W
                        grid[idx - dims[0] - 1] as u8, // NW
                    ];
                    let b = p.iter().sum::<u8>();
                    if b < 2 || b > 6 {
                        continue;
                    }
                    let mut transitions = 0u8;
                    for i in 0..8 {
                        if p[i] == 0 && p[(i + 1) % 8] == 1 {
                            transitions += 1;
                        }
                    }
                    if transitions != 1 {
                        continue;
                    }
                    if pass == 0 {
                        if p[0] * p[2] * p[4] != 0 || p[2] * p[4] * p[6] != 0 {
                            continue;
                        }
                    } else {
                        if p[0] * p[2] * p[6] != 0 || p[0] * p[4] * p[6] != 0 {
                            continue;
                        }
                    }
                    to_remove.push(idx);
                }
            }
            for idx in &to_remove {
                grid[*idx] = false;
                changed = true;
            }
        }
    }

    let output: Vec<f64> = grid.iter().map(|&b| if b { 1.0 } else { 0.0 }).collect();
    let mut result = image.clone();
    result
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, output, 1,
        )));
    result
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
