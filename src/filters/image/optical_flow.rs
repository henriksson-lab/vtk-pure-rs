//! Simple optical flow estimation between two images (Lucas-Kanade).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute optical flow between two images using block matching.
/// Returns an image with 2-component vectors (dx, dy).
pub fn block_matching_flow(
    img_a: &ImageData,
    img_b: &ImageData,
    scalars: &str,
    block_size: usize,
    search_radius: usize,
) -> ImageData {
    let arr_a = match img_a.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return img_a.clone(),
    };
    let arr_b = match img_b.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return img_a.clone(),
    };
    let dims = img_a.dimensions();
    let (nx, ny) = (dims[0], dims[1]);
    let mut buf = [0.0f64];
    let va: Vec<f64> = (0..arr_a.num_tuples())
        .map(|i| {
            arr_a.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    let vb: Vec<f64> = (0..arr_b.num_tuples())
        .map(|i| {
            arr_b.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let half = block_size / 2;
    let sr = search_radius as isize;
    let mut flow = vec![0.0f64; nx * ny * 2];

    for iy in half..ny.saturating_sub(half) {
        for ix in half..nx.saturating_sub(half) {
            let mut best_dx = 0isize;
            let mut best_dy = 0isize;
            let mut best_sad = f64::INFINITY;

            for dy in -sr..=sr {
                for dx in -sr..=sr {
                    let mut sad = 0.0;
                    for by in 0..block_size {
                        for bx in 0..block_size {
                            let ax = ix - half + bx;
                            let ay = iy - half + by;
                            let bxx = (ix as isize - half as isize + bx as isize + dx) as usize;
                            let byy = (iy as isize - half as isize + by as isize + dy) as usize;
                            if bxx < nx && byy < ny {
                                sad += (va[ax + ay * nx] - vb[bxx + byy * nx]).abs();
                            } else {
                                sad += 255.0;
                            }
                        }
                    }
                    if sad < best_sad {
                        best_sad = sad;
                        best_dx = dx;
                        best_dy = dy;
                    }
                }
            }
            let idx = ix + iy * nx;
            flow[idx * 2] = best_dx as f64;
            flow[idx * 2 + 1] = best_dy as f64;
        }
    }

    ImageData::with_dimensions(nx, ny, 1)
        .with_spacing(img_a.spacing())
        .with_origin(img_a.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Flow", flow, 2)))
}

/// Compute flow magnitude image from a flow field.
pub fn flow_magnitude(flow_image: &ImageData) -> ImageData {
    let arr = match flow_image.point_data().get_array("Flow") {
        Some(a) if a.num_components() == 2 => a,
        _ => return flow_image.clone(),
    };
    let n = arr.num_tuples();
    let mut buf = [0.0f64; 2];
    let data: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            (buf[0] * buf[0] + buf[1] * buf[1]).sqrt()
        })
        .collect();
    let dims = flow_image.dimensions();
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(flow_image.spacing())
        .with_origin(flow_image.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            "FlowMagnitude",
            data,
            1,
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_flow() {
        let a = ImageData::from_function(
            [16, 16, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| {
                if (x - 8.0).abs() < 2.5 {
                    100.0
                } else {
                    0.0
                }
            },
        );
        let b = ImageData::from_function(
            [16, 16, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| {
                if (x - 10.0).abs() < 2.5 {
                    100.0
                } else {
                    0.0
                }
            },
        );
        let f = block_matching_flow(&a, &b, "v", 3, 3);
        assert_eq!(f.dimensions(), [16, 16, 1]);
        assert!(f.point_data().get_array("Flow").is_some());
    }
    #[test]
    fn test_magnitude() {
        let a = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x,
        );
        let b = ImageData::from_function(
            [8, 8, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let f = block_matching_flow(&a, &b, "v", 3, 2);
        let m = flow_magnitude(&f);
        assert_eq!(m.dimensions(), [8, 8, 1]);
    }
}
