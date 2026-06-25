use crate::data::{AnyDataArray, DataArray, ImageData};

/// Remove small 2D islands from an image, following vtkImageIslandRemoval2D.
///
/// Pixels equal to `island_value` are treated as island pixels.  Islands with
/// fewer than `area_threshold` pixels are replaced with `replace_value`; all
/// other pixels are copied from the input.  Connectivity is 4-neighbor by
/// default, or 8-neighbor when `square_neighborhood` is true.
pub fn image_island_remove_2d(
    input: &ImageData,
    scalars: &str,
    area_threshold: usize,
    island_value: f64,
    replace_value: f64,
    square_neighborhood: bool,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let num_components = arr.num_components();
    let num_tuples = arr.num_tuples();
    let num_points = nx.saturating_mul(ny).saturating_mul(nz);
    if num_points == 0 || num_tuples < num_points {
        return input.clone();
    }

    let mut buf = vec![0.0f64; num_components];
    let mut input_values = vec![0.0f64; num_tuples * num_components];
    for tuple_idx in 0..num_tuples {
        arr.tuple_as_f64(tuple_idx, &mut buf);
        for component in 0..num_components {
            input_values[tuple_idx * num_components + component] = buf[component];
        }
    }

    let mut output_values = input_values.clone();
    let mut visited = vec![0u8; num_tuples * num_components];
    let mut pixels = Vec::with_capacity(area_threshold.saturating_add(8).max(1));

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;
    let offsets: &[(isize, isize)] = if square_neighborhood {
        &[
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ]
    } else {
        &[(-1, 0), (1, 0), (0, -1), (0, 1)]
    };

    for component in 0..num_components {
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let scalar_idx = idx(i, j, k) * num_components + component;
                    if visited[scalar_idx] != 0 {
                        continue;
                    }
                    if input_values[scalar_idx] != island_value {
                        visited[scalar_idx] = 2;
                        continue;
                    }

                    pixels.clear();
                    pixels.push((i, j, k));
                    visited[scalar_idx] = 1;
                    let mut next_pixel_idx = 0usize;
                    let mut keep_value = 1u8;

                    while keep_value == 1 {
                        let (ci, cj, ck) = pixels[next_pixel_idx];
                        for &(di, dj) in offsets {
                            let ni = ci as isize + di;
                            let nj = cj as isize + dj;
                            if ni < 0 || ni >= nx as isize || nj < 0 || nj >= ny as isize {
                                continue;
                            }
                            let neighbor_tuple = idx(ni as usize, nj as usize, ck);
                            let neighbor_idx = neighbor_tuple * num_components + component;
                            if input_values[neighbor_idx] == island_value {
                                match visited[neighbor_idx] {
                                    2 => keep_value = 2,
                                    0 => {
                                        pixels.push((ni as usize, nj as usize, ck));
                                        visited[neighbor_idx] = 1;
                                    }
                                    _ => {}
                                }
                            }
                        }

                        next_pixel_idx += 1;
                        if keep_value == 1 && pixels.len() >= area_threshold {
                            keep_value = 2;
                        }
                        if keep_value == 1 && next_pixel_idx >= pixels.len() {
                            keep_value = 3;
                        }
                    }

                    for &(pi, pj, pk) in &pixels {
                        let pixel_idx = idx(pi, pj, pk) * num_components + component;
                        visited[pixel_idx] = keep_value;
                        if keep_value == 3 {
                            output_values[pixel_idx] = replace_value;
                        }
                    }
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            scalars,
            output_values,
            num_components,
        )));
    img
}

/// Compatibility helper for the earlier binary-mask API.
pub fn image_island_remove(
    input: &ImageData,
    scalars: &str,
    threshold: f64,
    min_size: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let mut buf = [0.0f64];
    let mask: Vec<f64> = (0..arr.num_tuples())
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if buf[0] >= threshold {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut binary = input.clone();
    binary
        .point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, mask, 1)));
    image_island_remove_2d(&binary, scalars, min_size, 1.0, 0.0, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_small_island_value() {
        let mut img = ImageData::with_dimensions(7, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![0.0, 0.0, 0.0, 0.0, 5.0, 0.0, 5.0],
                1,
            )));

        let result = image_island_remove_2d(&img, "m", 3, 0.0, 255.0, false);
        let arr = result.point_data().get_array("m").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
        arr.tuple_as_f64(5, &mut buf);
        assert_eq!(buf[0], 255.0);
    }

    #[test]
    fn cross_neighborhood_keeps_diagonal_islands_separate() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 0.0, 0.0, 1.0],
                1,
            )));

        let result = image_island_remove_2d(&img, "m", 2, 1.0, 9.0, false);
        let arr = result.point_data().get_array("m").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 9.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 9.0);
    }

    #[test]
    fn square_neighborhood_connects_diagonal_islands() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 0.0, 0.0, 1.0],
                1,
            )));

        let result = image_island_remove_2d(&img, "m", 2, 1.0, 9.0, true);
        let arr = result.point_data().get_array("m").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn slices_are_processed_independently() {
        let mut img = ImageData::with_dimensions(2, 1, 2);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 1.0, 1.0, 0.0],
                1,
            )));

        let result = image_island_remove_2d(&img, "m", 2, 1.0, 9.0, false);
        let arr = result.point_data().get_array("m").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 9.0);
    }

    #[test]
    fn compatibility_helper_returns_binary_mask() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "m",
                vec![1.0, 1.0, 1.0, 0.0, 1.0],
                1,
            )));

        let result = image_island_remove(&img, "m", 0.5, 3);
        let arr = result.point_data().get_array("m").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(4, &mut buf);
        assert_eq!(buf[0], 0.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let result = image_island_remove_2d(&img, "nope", 1, 0.0, 255.0, false);
        assert!(result.point_data().get_array("nope").is_none());
    }
}
