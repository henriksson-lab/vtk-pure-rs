//! Region growing segmentation for images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Region growing from seed point with tolerance threshold.
pub fn region_grow(
    input: &ImageData,
    scalars: &str,
    seed_x: usize,
    seed_y: usize,
    tolerance: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = nx * ny;
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let seed_val = vals[seed_x + seed_y * nx];
    let mut mask = vec![0.0f64; n];
    let mut visited = vec![false; n];
    let mut queue = std::collections::VecDeque::new();
    let seed_idx = seed_x + seed_y * nx;
    queue.push_back(seed_idx);
    visited[seed_idx] = true;

    while let Some(idx) = queue.pop_front() {
        let v = vals[idx];
        if (v - seed_val).abs() > tolerance {
            continue;
        }
        mask[idx] = 1.0;
        let iy = idx / nx;
        let ix = idx % nx;
        let neighbors = [
            (ix.wrapping_sub(1), iy),
            (ix + 1, iy),
            (ix, iy.wrapping_sub(1)),
            (ix, iy + 1),
        ];
        for (sx, sy) in neighbors {
            if sx < nx && sy < ny {
                let ni = sx + sy * nx;
                if !visited[ni] {
                    visited[ni] = true;
                    queue.push_back(ni);
                }
            }
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Region", mask, 1)))
}

/// Multi-seed region growing.
pub fn multi_seed_grow(
    input: &ImageData,
    scalars: &str,
    seeds: &[(usize, usize)],
    tolerance: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = nx * ny;
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut labels = vec![0.0f64; n];
    let mut visited = vec![false; n];

    for (label, &(sx, sy)) in seeds.iter().enumerate() {
        let seed_val = vals[sx + sy * nx];
        let mut queue = std::collections::VecDeque::new();
        let si = sx + sy * nx;
        if visited[si] {
            continue;
        }
        queue.push_back(si);
        visited[si] = true;
        while let Some(idx) = queue.pop_front() {
            if (vals[idx] - seed_val).abs() > tolerance {
                continue;
            }
            labels[idx] = (label + 1) as f64;
            let iy = idx / nx;
            let ix = idx % nx;
            for (nx2, ny2) in [
                (ix.wrapping_sub(1), iy),
                (ix + 1, iy),
                (ix, iy.wrapping_sub(1)),
                (ix, iy + 1),
            ] {
                if nx2 < nx && ny2 < ny {
                    let ni = nx2 + ny2 * nx;
                    if !visited[ni] {
                        visited[ni] = true;
                        queue.push_back(ni);
                    }
                }
            }
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Labels", labels, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_grow() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if x < 5.0 {
                    10.0
                } else {
                    100.0
                }
            },
        );
        let r = region_grow(&img, "v", 2, 2, 5.0);
        let arr = r.point_data().get_array("Region").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(2 + 2 * 10, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(8 + 2 * 10, &mut buf);
        assert_eq!(buf[0], 0.0); // other side not reached
    }
    #[test]
    fn test_multi() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| {
                if x < 4.0 {
                    10.0
                } else if x > 6.0 {
                    90.0
                } else {
                    50.0
                }
            },
        );
        let r = multi_seed_grow(&img, "v", &[(1, 5), (8, 5)], 15.0);
        assert_eq!(r.dimensions(), [10, 10, 1]);
    }
}
