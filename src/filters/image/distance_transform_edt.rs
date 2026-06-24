//! Euclidean distance transform for binary images.

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute Euclidean distance transform of a binary image.
/// Each foreground pixel gets the distance to the nearest background pixel.
pub fn distance_transform_edt(input: &ImageData, scalars: &str) -> ImageData {
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

    // Brute-force for small images; separable pass for larger
    let mut dist = vec![f64::INFINITY; n];

    // Pass 1: horizontal
    let mut row_dist = vec![f64::INFINITY; nx];
    for iy in 0..ny {
        // Forward
        let mut d = f64::INFINITY;
        for ix in 0..nx {
            let idx = ix + iy * nx;
            if !fg[idx] {
                d = 0.0;
            } else {
                d += 1.0;
            }
            row_dist[ix] = d;
        }
        // Backward
        d = f64::INFINITY;
        for ix in (0..nx).rev() {
            let idx = ix + iy * nx;
            if !fg[idx] {
                d = 0.0;
            } else {
                d += 1.0;
            }
            row_dist[ix] = row_dist[ix].min(d);
            dist[idx] = row_dist[ix];
        }
    }

    // Pass 2: vertical (approximate EDT using squared distances)
    let dist_h = dist.clone();
    for ix in 0..nx {
        for iy in 0..ny {
            let idx = ix + iy * nx;
            let mut best = dist_h[idx] * dist_h[idx];
            for oy in 0..ny {
                let dy = (iy as f64 - oy as f64).abs();
                let dh = dist_h[ix + oy * nx];
                let d2 = dy * dy + dh * dh;
                if d2 < best {
                    best = d2;
                }
            }
            dist[idx] = best.sqrt();
        }
    }

    ImageData::with_dimensions(nx, ny, dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec("Distance", dist, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_edt() {
        let img = ImageData::from_function(
            [11, 11, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| {
                if (x - 5.0).abs() < 3.5 && (y - 5.0).abs() < 3.5 {
                    1.0
                } else {
                    0.0
                }
            },
        );
        let r = distance_transform_edt(&img, "v");
        let arr = r.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(5 + 5 * 11, &mut buf);
        assert!(buf[0] > 2.0); // center is far from boundary
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 1e-10); // background = 0
    }
    #[test]
    fn test_all_bg() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |_, _, _| 0.0,
        );
        let r = distance_transform_edt(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }
}
