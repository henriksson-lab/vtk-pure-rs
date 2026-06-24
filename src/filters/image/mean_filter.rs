use crate::data::{AnyDataArray, DataArray, DataSetAttributes, ImageData};

/// Box mean filter on ImageData: replace each voxel with the mean of its
/// NxNxN neighborhood, where N = 2*radius + 1.
///
/// The `scalars` parameter names the point data array to filter. Other arrays
/// are passed through unchanged.
pub fn mean_filter(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx: usize = dims[0] as usize;
    let ny: usize = dims[1] as usize;
    let nz: usize = dims[2] as usize;
    let n: usize = nx * ny * nz;

    let mut values: Vec<f64> = vec![0.0; n];
    let mut buf: [f64; 1] = [0.0];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        values[i] = buf[0];
    }

    let r: i64 = radius as i64;
    let mut result: Vec<f64> = vec![0.0; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let mut sum: f64 = 0.0;
                let mut count: f64 = 0.0;

                for dk in -r..=r {
                    let kk: i64 = k as i64 + dk;
                    if kk < 0 || kk >= nz as i64 {
                        continue;
                    }
                    for dj in -r..=r {
                        let jj: i64 = j as i64 + dj;
                        if jj < 0 || jj >= ny as i64 {
                            continue;
                        }
                        for di in -r..=r {
                            let ii: i64 = i as i64 + di;
                            if ii < 0 || ii >= nx as i64 {
                                continue;
                            }
                            let idx: usize = kk as usize * ny * nx + jj as usize * nx + ii as usize;
                            sum += values[idx];
                            count += 1.0;
                        }
                    }
                }

                let out_idx: usize = k * ny * nx + j * nx + i;
                result[out_idx] = if count > 0.0 { sum / count } else { 0.0 };
            }
        }
    }

    let mut img = input.clone();
    let mut new_attrs = DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            new_attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            new_attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = new_attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spike_image() -> ImageData {
        let mut img = ImageData::with_dimensions(5, 5, 5);
        let n: usize = 125;
        let mut values: Vec<f64> = vec![0.0; n];
        // Spike at center (2,2,2) -> index = 2*25 + 2*5 + 2 = 62
        values[62] = 125.0;
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("val", values, 1)));
        img
    }

    #[test]
    fn mean_filter_reduces_spike() {
        let img = make_spike_image();
        let result = mean_filter(&img, "val", 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf: [f64; 1] = [0.0];
        arr.tuple_as_f64(62, &mut buf);
        // With radius 1, the 3x3x3 = 27 neighborhood averages 125/27 ~ 4.63
        assert!(buf[0] < 125.0, "center should be reduced, got {}", buf[0]);
        assert!(buf[0] > 0.0, "center should be positive, got {}", buf[0]);
    }

    #[test]
    fn mean_filter_spreads_value() {
        let img = make_spike_image();
        let result = mean_filter(&img, "val", 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf: [f64; 1] = [0.0];
        // Neighbor at (3,2,2) -> index 63
        arr.tuple_as_f64(63, &mut buf);
        assert!(buf[0] > 0.0, "neighbor should pick up value");
    }

    #[test]
    fn mean_filter_preserves_uniform() {
        let mut img = ImageData::with_dimensions(3, 3, 3);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "val",
                vec![7.0; 27],
                1,
            )));
        let result = mean_filter(&img, "val", 1);
        let arr = result.point_data().get_array("val").unwrap();
        let mut buf: [f64; 1] = [0.0];
        for i in 0..27 {
            arr.tuple_as_f64(i, &mut buf);
            assert!(
                (buf[0] - 7.0).abs() < 1e-10,
                "uniform field should be preserved, voxel {} = {}",
                i,
                buf[0]
            );
        }
    }
}
