//! Morphological skeletonization (thinning) of binary images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Skeletonize a binary image using iterative thinning.
pub fn skeletonize(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = nx * ny;
    let mut buf = [0.0f64];
    let mut img: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();

    loop {
        let mut changed = false;
        // Sub-iteration 1
        let to_remove = thinning_pass(&img, nx, ny, true);
        for &idx in &to_remove {
            img[idx] = false;
            changed = true;
        }
        // Sub-iteration 2
        let to_remove = thinning_pass(&img, nx, ny, false);
        for &idx in &to_remove {
            img[idx] = false;
            changed = true;
        }
        if !changed {
            break;
        }
    }

    let data: Vec<f64> = img.iter().map(|&v| if v { 1.0 } else { 0.0 }).collect();
    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}

fn thinning_pass(img: &[bool], nx: usize, ny: usize, first: bool) -> Vec<usize> {
    let mut remove = Vec::new();
    for iy in 1..ny.saturating_sub(1) {
        for ix in 1..nx.saturating_sub(1) {
            let idx = ix + iy * nx;
            if !img[idx] {
                continue;
            }
            let p = neighbors(img, ix, iy, nx);
            let b = p.iter().filter(|&&v| v).count();
            if b < 2 || b > 6 {
                continue;
            }
            let a = transitions(&p);
            if a != 1 {
                continue;
            }
            if first {
                if p[0] && p[2] && p[4] {
                    continue;
                }
                if p[2] && p[4] && p[6] {
                    continue;
                }
            } else {
                if p[0] && p[2] && p[6] {
                    continue;
                }
                if p[0] && p[4] && p[6] {
                    continue;
                }
            }
            remove.push(idx);
        }
    }
    remove
}

fn neighbors(img: &[bool], x: usize, y: usize, nx: usize) -> [bool; 8] {
    [
        img[x + (y - 1) * nx],                 // P2 (N)
        img[x + 1 + (y - 1) * nx],             // P3 (NE)
        img[x + 1 + y * nx],                   // P4 (E)
        img[x + 1 + (y + 1) * nx],             // P5 (SE)
        img[x + (y + 1) * nx],                 // P6 (S)
        img[x.wrapping_sub(1) + (y + 1) * nx], // P7 (SW)
        img[x.wrapping_sub(1) + y * nx],       // P8 (W)
        img[x.wrapping_sub(1) + (y - 1) * nx], // P9 (NW)
    ]
}

fn transitions(p: &[bool; 8]) -> usize {
    let mut count = 0;
    for i in 0..8 {
        if !p[i] && p[(i + 1) % 8] {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_line() {
        // Thick horizontal line should thin to 1-pixel line
        let img = ImageData::from_function(
            [20, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if y > 2.0 && y < 7.0 && x > 1.0 && x < 18.0 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let r = skeletonize(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        // Some pixel should remain
        let mut has_fg = false;
        for i in 0..200 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                has_fg = true;
                break;
            }
        }
        assert!(has_fg);
    }
    #[test]
    fn test_empty() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let r = skeletonize(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
