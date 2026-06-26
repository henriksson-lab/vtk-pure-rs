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
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let n = nx * ny * nz;
    let spacing = input.spacing();
    let mut buf = [0.0f64];
    let fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] > 0.5
        })
        .collect();

    let index = |x: usize, y: usize, z: usize| -> usize { z * ny * nx + y * nx + x };

    let transform_line = |line: &[f64], step: f64| -> Vec<f64> {
        let mut out = vec![f64::INFINITY; line.len()];
        let seeds: Vec<usize> = line
            .iter()
            .enumerate()
            .filter_map(|(i, &v)| if v.is_finite() { Some(i) } else { None })
            .collect();
        for (i, out_value) in out.iter_mut().enumerate() {
            let mut best = f64::INFINITY;
            for &seed in &seeds {
                let delta = (i as f64 - seed as f64) * step;
                best = best.min(line[seed] + delta * delta);
            }
            *out_value = best;
        }
        out
    };

    let mut dist: Vec<f64> = fg
        .iter()
        .map(
            |&is_foreground| {
                if is_foreground {
                    f64::INFINITY
                } else {
                    0.0
                }
            },
        )
        .collect();

    for z in 0..nz {
        for y in 0..ny {
            let line: Vec<f64> = (0..nx).map(|x| dist[index(x, y, z)]).collect();
            let transformed = transform_line(&line, spacing[0]);
            for x in 0..nx {
                dist[index(x, y, z)] = transformed[x];
            }
        }
    }

    for z in 0..nz {
        for x in 0..nx {
            let line: Vec<f64> = (0..ny).map(|y| dist[index(x, y, z)]).collect();
            let transformed = transform_line(&line, spacing[1]);
            for y in 0..ny {
                dist[index(x, y, z)] = transformed[y];
            }
        }
    }

    for y in 0..ny {
        for x in 0..nx {
            let line: Vec<f64> = (0..nz).map(|z| dist[index(x, y, z)]).collect();
            let transformed = transform_line(&line, spacing[2]);
            for z in 0..nz {
                dist[index(x, y, z)] = transformed[z];
            }
        }
    }

    for value in &mut dist {
        *value = value.sqrt();
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Distance", dist, 1)));
    img
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

    #[test]
    fn test_3d_spacing() {
        let mut img = ImageData::with_dimensions(1, 1, 3);
        img.set_spacing([1.0, 1.0, 2.0]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 1.0],
                1,
            )));

        let r = distance_transform_edt(&img, "v");
        let arr = r.point_data().get_array("Distance").unwrap();
        let mut buf = [0.0];
        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 4.0).abs() < 1e-10);
    }
}
