//! Region growing segmentation for images.

use crate::data::{AnyDataArray, DataArray, ImageData};
use std::collections::VecDeque;

fn image_point_count(dims: [usize; 3]) -> Option<usize> {
    dims[0].checked_mul(dims[1])?.checked_mul(dims[2])
}

fn point_index(x: usize, y: usize, z: usize, dims: [usize; 3]) -> usize {
    x + y * dims[0] + z * dims[0] * dims[1]
}

fn push_unvisited_neighbors(
    idx: usize,
    dims: [usize; 3],
    visited: &mut [bool],
    queue: &mut VecDeque<usize>,
) {
    let nx = dims[0];
    let ny = dims[1];
    let plane = nx * ny;
    let z = idx / plane;
    let rem = idx % plane;
    let y = rem / nx;
    let x = rem % nx;

    let mut push = |neighbor: usize| {
        if !visited[neighbor] {
            visited[neighbor] = true;
            queue.push_back(neighbor);
        }
    };

    if x > 0 {
        push(idx - 1);
    }
    if x + 1 < nx {
        push(idx + 1);
    }
    if y > 0 {
        push(idx - nx);
    }
    if y + 1 < ny {
        push(idx + nx);
    }
    if z > 0 {
        push(idx - plane);
    }
    if z + 1 < dims[2] {
        push(idx + plane);
    }
}

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
    let n = match image_point_count(dims) {
        Some(n)
            if n > 0
                && seed_x < dims[0]
                && seed_y < dims[1]
                && dims[2] > 0
                && arr.num_tuples() >= n =>
        {
            n
        }
        Some(n) => {
            return ImageData::with_dimensions(dims[0], dims[1], dims[2])
                .with_spacing(input.spacing())
                .with_origin(input.origin())
                .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                    "Region",
                    vec![0.0; n],
                    1,
                )));
        }
        None => return input.clone(),
    };
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let seed_idx = point_index(seed_x, seed_y, 0, dims);
    let seed_val = vals[seed_idx];
    let mut mask = vec![0.0f64; n];
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
    queue.push_back(seed_idx);
    visited[seed_idx] = true;

    while let Some(idx) = queue.pop_front() {
        let v = vals[idx];
        if (v - seed_val).abs() > tolerance {
            continue;
        }
        mask[idx] = 1.0;
        push_unvisited_neighbors(idx, dims, &mut visited, &mut queue);
    }

    ImageData::with_dimensions(dims[0], dims[1], dims[2])
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
    let n = match image_point_count(dims) {
        Some(n) if n > 0 && dims[2] > 0 && arr.num_tuples() >= n => n,
        Some(n) => {
            return ImageData::with_dimensions(dims[0], dims[1], dims[2])
                .with_spacing(input.spacing())
                .with_origin(input.origin())
                .with_point_array(AnyDataArray::F64(DataArray::from_vec(
                    "Labels",
                    vec![0.0; n],
                    1,
                )));
        }
        None => return input.clone(),
    };
    let mut buf = [0.0f64];
    let vals: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut labels = vec![0.0f64; n];

    for (label, &(sx, sy)) in seeds.iter().enumerate() {
        if sx >= dims[0] || sy >= dims[1] {
            continue;
        }

        let seed_val = vals[point_index(sx, sy, 0, dims)];
        let mut queue = VecDeque::new();
        let mut visited = vec![false; n];
        let si = point_index(sx, sy, 0, dims);
        if labels[si] != 0.0 {
            continue;
        }
        queue.push_back(si);
        visited[si] = true;
        while let Some(idx) = queue.pop_front() {
            if labels[idx] != 0.0 {
                continue;
            }
            if (vals[idx] - seed_val).abs() > tolerance {
                continue;
            }
            labels[idx] = (label + 1) as f64;
            push_unvisited_neighbors(idx, dims, &mut visited, &mut queue);
        }
    }

    ImageData::with_dimensions(dims[0], dims[1], dims[2])
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

    #[test]
    fn grow_follows_6_connected_volume() {
        let img = ImageData::from_function(
            [3, 3, 2],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, z| {
                if x == 1.0 && y == 1.0 {
                    10.0 + z
                } else {
                    100.0
                }
            },
        );

        let r = region_grow(&img, "v", 1, 1, 2.0);
        let arr = r.point_data().get_array("Region").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(1 + 3 + 9, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn out_of_bounds_seed_returns_empty_mask() {
        let img = ImageData::from_function(
            [2, 2, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );

        let r = region_grow(&img, "v", 3, 0, 0.0);
        let arr = r.point_data().get_array("Region").unwrap();
        let mut buf = [0.0];
        for i in 0..4 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn later_seed_can_claim_voxel_rejected_by_earlier_seed() {
        let img = ImageData::from_function(
            [3, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| if x == 0.0 { 0.0 } else { 10.0 },
        );

        let r = multi_seed_grow(&img, "v", &[(0, 0), (2, 0)], 0.0);
        let arr = r.point_data().get_array("Labels").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 2.0);
    }
}
