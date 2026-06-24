use crate::data::{AnyDataArray, DataArray, ImageData};

/// Approximate Canny edge detection on 2D ImageData.
///
/// Steps: 1) Gaussian blur, 2) Sobel gradient magnitude,
/// 3) double threshold, 4) hysteresis. Adds "CannyEdges" binary array.
pub fn image_canny(
    input: &ImageData,
    scalars: &str,
    low_threshold: f64,
    high_threshold: f64,
    sigma: f64,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let n = nx * ny;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let get = |v: &[f64], i: i64, j: i64| -> f64 {
        v[(j.clamp(0, ny as i64 - 1) as usize) * nx + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    // Step 1: Gaussian blur
    let r = ((sigma * 3.0).ceil() as usize).max(1);
    let mut blurred = vec![0.0f64; n];
    for j in 0..ny {
        for i in 0..nx {
            let mut sum = 0.0;
            let mut sw = 0.0;
            for dj in -(r as i64)..=(r as i64) {
                for di in -(r as i64)..=(r as i64) {
                    let w = (-(di * di + dj * dj) as f64 / (2.0 * sigma * sigma)).exp();
                    sum += w * get(&values, i as i64 + di, j as i64 + dj);
                    sw += w;
                }
            }
            blurred[j * nx + i] = sum / sw;
        }
    }

    // Step 2: Sobel gradient magnitude
    let mut mag = vec![0.0f64; n];
    for j in 0..ny {
        for i in 0..nx {
            let ii = i as i64;
            let jj = j as i64;
            let gx = -get(&blurred, ii - 1, jj - 1) + get(&blurred, ii + 1, jj - 1)
                - 2.0 * get(&blurred, ii - 1, jj)
                + 2.0 * get(&blurred, ii + 1, jj)
                - get(&blurred, ii - 1, jj + 1)
                + get(&blurred, ii + 1, jj + 1);
            let gy = -get(&blurred, ii - 1, jj - 1)
                - 2.0 * get(&blurred, ii, jj - 1)
                - get(&blurred, ii + 1, jj - 1)
                + get(&blurred, ii - 1, jj + 1)
                + 2.0 * get(&blurred, ii, jj + 1)
                + get(&blurred, ii + 1, jj + 1);
            mag[j * nx + i] = (gx * gx + gy * gy).sqrt();
        }
    }

    // Step 3+4: Double threshold + hysteresis via BFS
    let mut edges = vec![0.0f64; n];
    let mut strong = std::collections::VecDeque::new();

    for i in 0..n {
        if mag[i] >= high_threshold {
            edges[i] = 1.0;
            strong.push_back(i);
        }
    }

    // Hysteresis: propagate from strong edges to weak neighbors
    while let Some(pi) = strong.pop_front() {
        let px = pi % nx;
        let py = pi / nx;
        for dj in -1i64..=1 {
            for di in -1i64..=1 {
                if di == 0 && dj == 0 {
                    continue;
                }
                let ni = (px as i64 + di);
                let nj = (py as i64 + dj);
                if ni >= 0 && ni < nx as i64 && nj >= 0 && nj < ny as i64 {
                    let idx = nj as usize * nx + ni as usize;
                    if edges[idx] == 0.0 && mag[idx] >= low_threshold {
                        edges[idx] = 1.0;
                        strong.push_back(idx);
                    }
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CannyEdges",
            edges,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_edge() {
        let mut img = ImageData::with_dimensions(9, 9, 1);
        let mut values = vec![0.0; 81];
        for j in 0..9 {
            for i in 5..9 {
                values[j * 9 + i] = 100.0;
            }
        } // sharp edge at col 5
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_canny(&img, "v", 10.0, 50.0, 1.0);
        let arr = result.point_data().get_array("CannyEdges").unwrap();
        let mut buf = [0.0f64];
        // Should have some edge pixels near column 4-5
        let mut edge_count = 0;
        for i in 0..81 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                edge_count += 1;
            }
        }
        assert!(edge_count > 0);
    }

    #[test]
    fn uniform_no_edges() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![50.0; 25],
                1,
            )));

        let result = image_canny(&img, "v", 10.0, 50.0, 1.0);
        let arr = result.point_data().get_array("CannyEdges").unwrap();
        let mut buf = [0.0f64];
        let mut edge_count = 0;
        for i in 0..25 {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] > 0.5 {
                edge_count += 1;
            }
        }
        assert_eq!(edge_count, 0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_canny(&img, "nope", 10.0, 50.0, 1.0);
        assert!(r.point_data().get_array("CannyEdges").is_none());
    }
}
