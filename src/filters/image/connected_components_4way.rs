//! 4-connected and 8-connected component labeling.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Label connected components using 8-connectivity.
pub fn label_components_8(input: &ImageData, scalars: &str) -> ImageData {
    label_impl(input, scalars, true)
}

/// Label connected components using 4-connectivity.
pub fn label_components_4(input: &ImageData, scalars: &str) -> ImageData {
    label_impl(input, scalars, false)
}

fn label_impl(input: &ImageData, scalars: &str, eight: bool) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let n = nx * ny;
    let mut buf = [0.0f64];
    let fg: Vec<bool> = (0..n)
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
            if !fg[idx] {
                continue;
            }
            // 4-connected: left, up
            if ix > 0 && fg[idx - 1] {
                union(&mut parent, &mut rank, idx, idx - 1);
            }
            if iy > 0 && fg[idx - nx] {
                union(&mut parent, &mut rank, idx, idx - nx);
            }
            if eight {
                if ix > 0 && iy > 0 && fg[idx - nx - 1] {
                    union(&mut parent, &mut rank, idx, idx - nx - 1);
                }
                if ix + 1 < nx && iy > 0 && fg[idx - nx + 1] {
                    union(&mut parent, &mut rank, idx, idx - nx + 1);
                }
            }
        }
    }

    let mut label_map: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    let mut next_label = 1.0f64;
    let data: Vec<f64> = (0..n)
        .map(|i| {
            if !fg[i] {
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

fn find(p: &mut [usize], mut i: usize) -> usize {
    while p[i] != i {
        p[i] = p[p[i]];
        i = p[i];
    }
    i
}
fn union(p: &mut [usize], r: &mut [u8], a: usize, b: usize) {
    let ra = find(p, a);
    let rb = find(p, b);
    if ra == rb {
        return;
    }
    if r[ra] < r[rb] {
        p[ra] = rb;
    } else if r[ra] > r[rb] {
        p[rb] = ra;
    } else {
        p[rb] = ra;
        r[ra] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_4way() {
        // Diagonal pixels: 4-connected = 2 components, 8-connected = 1
        let img = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if ((x - 0.0).abs() < 0.5 && (y - 0.0).abs() < 0.5)
                    || ((x - 2.0).abs() < 0.5 && (y - 2.0).abs() < 0.5)
                {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let l4 = label_components_4(&img, "v");
        let l8 = label_components_8(&img, "v");
        let count = |img: &ImageData| {
            let a = img.point_data().get_array("Labels").unwrap();
            let mut b = [0.0f64];
            let mut mx = 0.0f64;
            for i in 0..a.num_tuples() {
                a.tuple_as_f64(i, &mut b);
                mx = mx.max(b[0]);
            }
            mx as usize
        };
        assert_eq!(count(&l4), 2);
        assert_eq!(count(&l8), 2); // still 2 since not adjacent diagonally
    }
}
