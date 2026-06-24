use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute chamfer distance transform using 3-4-5 weights.
///
/// More accurate than the simple forward-backward pass. Uses the
/// 3-4-5 Borgefors chamfer mask for better Euclidean approximation.
/// Adds "ChamferDistance" array.
pub fn image_chamfer_distance(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;
    let big = 1e8f64;

    let mut buf = [0.0f64];
    let mut dist: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                0.0
            } else {
                big
            }
        })
        .collect();

    // Forward pass (top-left to bottom-right)
    for j in 0..ny {
        for i in 0..nx {
            let idx = j * nx + i;
            if i > 0 {
                dist[idx] = dist[idx].min(dist[j * nx + i - 1] + 3.0);
            }
            if j > 0 {
                dist[idx] = dist[idx].min(dist[(j - 1) * nx + i] + 3.0);
            }
            if i > 0 && j > 0 {
                dist[idx] = dist[idx].min(dist[(j - 1) * nx + i - 1] + 4.0);
            }
            if i + 1 < nx && j > 0 {
                dist[idx] = dist[idx].min(dist[(j - 1) * nx + i + 1] + 4.0);
            }
        }
    }

    // Backward pass (bottom-right to top-left)
    for j in (0..ny).rev() {
        for i in (0..nx).rev() {
            let idx = j * nx + i;
            if i + 1 < nx {
                dist[idx] = dist[idx].min(dist[j * nx + i + 1] + 3.0);
            }
            if j + 1 < ny {
                dist[idx] = dist[idx].min(dist[(j + 1) * nx + i] + 3.0);
            }
            if i + 1 < nx && j + 1 < ny {
                dist[idx] = dist[idx].min(dist[(j + 1) * nx + i + 1] + 4.0);
            }
            if i > 0 && j + 1 < ny {
                dist[idx] = dist[idx].min(dist[(j + 1) * nx + i - 1] + 4.0);
            }
        }
    }

    // Scale to approximate Euclidean (divide by 3)
    for d in &mut dist {
        *d /= 3.0;
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ChamferDistance",
            dist,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_seed() {
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values = vec![0.0; 49];
        values[24] = 1.0; // center seed
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_chamfer_distance(&img, "v", 0.5);
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(24, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(25, &mut buf);
        assert!((buf[0] - 1.0).abs() < 0.5);
    }

    #[test]
    fn all_foreground() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![1.0; 9], 1)));

        let result = image_chamfer_distance(&img, "v", 0.5);
        let arr = result.point_data().get_array("ChamferDistance").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_chamfer_distance(&img, "nope", 0.5);
        assert!(r.point_data().get_array("ChamferDistance").is_none());
    }
}
