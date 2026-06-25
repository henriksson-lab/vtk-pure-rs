use crate::data::{AnyDataArray, DataArray, ImageData};

/// Percentile filter: replace each voxel with the Nth percentile of its neighborhood.
///
/// Generalizes median (50th percentile), min (0th), max (100th).
pub fn image_percentile_filter(
    input: &ImageData,
    scalars: &str,
    percentile: f64,
    radius: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    if n == 0 || arr.num_tuples() != n {
        return input.clone();
    }

    let nc = arr.num_components();
    let p = (percentile / 100.0).clamp(0.0, 1.0);

    let mut values = vec![0.0f64; n * nc];
    let mut buf = vec![0.0f64; nc];
    for tuple in 0..n {
        arr.tuple_as_f64(tuple, &mut buf);
        let start = tuple * nc;
        values[start..start + nc].copy_from_slice(&buf);
    }

    let mut result = vec![0.0f64; n * nc];
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let i_min = i.saturating_sub(radius);
                let j_min = j.saturating_sub(radius);
                let k_min = k.saturating_sub(radius);
                let i_max = (i + radius).min(nx - 1);
                let j_max = (j + radius).min(ny - 1);
                let k_max = (k + radius).min(nz - 1);

                for comp in 0..nc {
                    let mut nbhood = Vec::new();
                    for kk in k_min..=k_max {
                        for jj in j_min..=j_max {
                            for ii in i_min..=i_max {
                                let idx = (kk * ny * nx + jj * nx + ii) * nc + comp;
                                nbhood.push(values[idx]);
                            }
                        }
                    }
                    nbhood.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    result[(k * ny * nx + j * nx + i) * nc + comp] = percentile_value(&nbhood, p);
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(scalars, result, nc)));
    img
}

fn percentile_value(sorted: &[f64], p: f64) -> f64 {
    debug_assert!(!sorted.is_empty());
    let rank = (sorted.len() - 1) as f64 * p;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let t = rank - lo as f64;
        sorted[lo] * (1.0 - t) + sorted[hi] * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_filter() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 0.0, 0.0],
                1,
            )));
        let result = image_percentile_filter(&img, "v", 50.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 0.0); // median removes spike
    }

    #[test]
    fn max_filter() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 5.0, 3.0],
                1,
            )));
        let result = image_percentile_filter(&img, "v", 100.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0);
    }

    #[test]
    fn min_filter() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 5.0, 3.0],
                1,
            )));
        let result = image_percentile_filter(&img, "v", 0.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 1.0);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_percentile_filter(&img, "nope", 50.0, 1);
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }

    #[test]
    fn clips_boundary_neighborhood_like_vtk_median() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0, 100.0],
                1,
            )));
        let result = image_percentile_filter(&img, "v", 50.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 5.0);
    }

    #[test]
    fn filters_each_component() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 9.0, 5.0, 3.0, 7.0, 11.0],
                2,
            )));
        let result = image_percentile_filter(&img, "v", 50.0, 1);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [5.0, 7.0]);
        assert_eq!(arr.num_components(), 2);
    }
}
