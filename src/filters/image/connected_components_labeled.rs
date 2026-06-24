//! Connected component labeling for 2D binary images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Label connected components in a binary image using union-find.
/// Returns image with each pixel labeled by component ID (0 = background).
pub fn label_components(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = nx * ny;
    let mut buf = [0.0f64];
    let vals: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();

    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0u8; n];

    for iy in 0..ny {
        for ix in 0..nx {
            let idx = ix + iy * nx;
            if !vals[idx] {
                continue;
            }
            if ix > 0 && vals[idx - 1] {
                union(&mut parent, &mut rank, idx, idx - 1);
            }
            if iy > 0 && vals[idx - nx] {
                union(&mut parent, &mut rank, idx, idx - nx);
            }
        }
    }

    // Relabel components sequentially
    let mut label_map: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    let mut next_label = 1.0f64;
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if !vals[i] {
                return 0.0;
            }
            let root = find(&mut parent, i);
            *label_map.entry(root).or_insert_with(|| {
                let l = next_label;
                next_label += 1.0;
                l
            })
        })
        .collect();

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Labels", data, 1)))
}

/// Count number of connected components.
pub fn count_components(input: &ImageData, scalars: &str) -> usize {
    let labeled = label_components(input, scalars);
    let arr = labeled.point_data().get_array("Labels").unwrap();
    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let mut max_label = 0.0f64;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        max_label = max_label.max(buf[0]);
    }
    max_label as usize
}

fn find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn union(parent: &mut [usize], rank: &mut [u8], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        parent[ra] = rb;
    } else if rank[ra] > rank[rb] {
        parent[rb] = ra;
    } else {
        parent[rb] = ra;
        rank[ra] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_two_blobs() {
        let img = ImageData::from_function(
            [10, 10, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x < 3.0 && y < 3.0) || (x > 6.0 && y > 6.0) {
                    1.0
                } else {
                    0.0
                }
            },
        );
        assert_eq!(count_components(&img, "v"), 2);
    }
    #[test]
    fn test_single() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 1.0,
        );
        assert_eq!(count_components(&img, "v"), 1);
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
        assert_eq!(count_components(&img, "v"), 0);
    }
}
