use crate::data::{AnyDataArray, DataArray, ImageData};

/// Iterative binary erosion followed by counting remaining foreground.
///
/// Returns the number of foreground voxels remaining after each erosion
/// step. Useful for granulometry (particle size distribution analysis).
pub fn image_granulometry(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
    max_radius: usize,
) -> Vec<usize> {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return vec![],
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let mut fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= threshold
        })
        .collect();

    let mut counts = Vec::with_capacity(max_radius + 1);
    counts.push(fg.iter().filter(|&&b| b).count());

    for r in 1..=max_radius {
        // Erode: a voxel survives only if all neighbors within radius are also foreground
        let mut new_fg = vec![false; n];
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    if !fg[k * ny * nx + j * nx + i] {
                        continue;
                    }
                    let mut all_fg = true;
                    'outer: for dk in -(r as i64)..=(r as i64) {
                        let kk = (k as i64 + dk).clamp(0, nz as i64 - 1) as usize;
                        for dj in -(r as i64)..=(r as i64) {
                            let jj = (j as i64 + dj).clamp(0, ny as i64 - 1) as usize;
                            for di in -(r as i64)..=(r as i64) {
                                let ii = (i as i64 + di).clamp(0, nx as i64 - 1) as usize;
                                if !fg[kk * ny * nx + jj * nx + ii] {
                                    all_fg = false;
                                    break 'outer;
                                }
                            }
                        }
                    }
                    new_fg[k * ny * nx + j * nx + i] = all_fg;
                }
            }
        }
        fg = new_fg;
        counts.push(fg.iter().filter(|&&b| b).count());
    }

    counts
}

/// Compute distance to nearest background voxel for each foreground voxel.
/// Equivalent to the erosion distance (how many erosion steps until removed).
pub fn erosion_thickness(input: &ImageData, scalars: &str, threshold: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let fg: Vec<bool> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] >= threshold
        })
        .collect();

    // Multi-pass erosion counting
    let mut thickness = vec![0.0f64; n];
    let mut current = fg.clone();
    let mut radius = 0;

    loop {
        radius += 1;
        let mut eroded = vec![false; n];
        let mut any_change = false;

        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    if !current[k * ny * nx + j * nx + i] {
                        continue;
                    }
                    // Check 6-connected neighbors
                    let has_bg = [
                        if i > 0 {
                            !current[k * ny * nx + j * nx + i - 1]
                        } else {
                            true
                        },
                        if i + 1 < nx {
                            !current[k * ny * nx + j * nx + i + 1]
                        } else {
                            true
                        },
                        if j > 0 {
                            !current[k * ny * nx + (j - 1) * nx + i]
                        } else {
                            true
                        },
                        if j + 1 < ny {
                            !current[k * ny * nx + (j + 1) * nx + i]
                        } else {
                            true
                        },
                        if k > 0 {
                            !current[(k - 1) * ny * nx + j * nx + i]
                        } else {
                            true
                        },
                        if k + 1 < nz {
                            !current[(k + 1) * ny * nx + j * nx + i]
                        } else {
                            true
                        },
                    ]
                    .iter()
                    .any(|&b| b);

                    if has_bg {
                        // This voxel is on the boundary -> mark with current radius
                        thickness[k * ny * nx + j * nx + i] = radius as f64;
                        any_change = true;
                    } else {
                        eroded[k * ny * nx + j * nx + i] = true;
                    }
                }
            }
        }

        if !any_change {
            break;
        }
        // Mark remaining interior
        for i in 0..n {
            if eroded[i] {
                thickness[i] = (radius + 1) as f64;
            }
        }
        current = eroded;
        if radius > 100 {
            break;
        } // safety
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "ErosionThickness",
            thickness,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn granulometry_decreasing() {
        let mut img = ImageData::with_dimensions(7, 7, 1);
        let mut values = vec![0.0; 49];
        for j in 1..6 {
            for i in 1..6 {
                values[j * 7 + i] = 1.0;
            }
        } // 5x5 block
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let counts = image_granulometry(&img, "v", 0.5, 3);
        assert!(counts.len() >= 2);
        assert!(counts[0] > counts[1]); // decreasing
    }

    #[test]
    fn erosion_thickness_test() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        let mut values = vec![0.0; 25];
        for j in 0..5 {
            for i in 0..5 {
                values[j * 5 + i] = 1.0;
            }
        } // all foreground
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = erosion_thickness(&img, "v", 0.5);
        let arr = result.point_data().get_array("ErosionThickness").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(12, &mut buf); // center
        assert!(buf[0] >= 1.0, "center thickness = {}", buf[0]); // center at least as thick as boundary
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        assert!(image_granulometry(&img, "nope", 0.5, 3).is_empty());
    }
}
