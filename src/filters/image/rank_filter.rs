//! Rank-based image filters (min, max, median, percentile).

use crate::data::{AnyDataArray, DataArray, ImageData};

/// Apply a rank filter with a given percentile (0.0 = min, 0.5 = median, 1.0 = max).
pub fn rank_filter(input: &ImageData, scalars: &str, radius: usize, percentile: f64) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let n = arr.num_tuples();
    let ncomp = arr.num_components();
    let mut buf = vec![0.0f64; ncomp];
    let vals: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.clone()
        })
        .collect();

    let r = radius as isize;
    let (nx, ny, nz) = (dims[0], dims[1], dims[2]);
    let p = percentile.clamp(0.0, 1.0);

    let mut data = Vec::with_capacity(n * ncomp);
    for idx in 0..n {
        let iz = idx / (nx * ny);
        let rem = idx % (nx * ny);
        let iy = rem / nx;
        let ix = rem % nx;
        for c in 0..ncomp {
            let mut neighborhood = Vec::new();
            for dz in -r..=r {
                for dy in -r..=r {
                    for dx in -r..=r {
                        let sx = ix as isize + dx;
                        let sy = iy as isize + dy;
                        let sz = iz as isize + dz;
                        if sx >= 0
                            && sx < nx as isize
                            && sy >= 0
                            && sy < ny as isize
                            && sz >= 0
                            && sz < nz as isize
                        {
                            let tuple = sx as usize + sy as usize * nx + sz as usize * nx * ny;
                            neighborhood.push(vals[tuple * ncomp + c]);
                        }
                    }
                }
            }
            neighborhood.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if neighborhood.is_empty() {
                data.push(vals[idx * ncomp + c]);
                continue;
            }
            if p == 0.5 {
                let mid = neighborhood.len() / 2;
                let value = if neighborhood.len().is_multiple_of(2) {
                    let low_mid = neighborhood[..mid]
                        .iter()
                        .copied()
                        .fold(f64::NEG_INFINITY, f64::max);
                    low_mid + (neighborhood[mid] - low_mid) / 2.0
                } else {
                    neighborhood[mid]
                };
                data.push(value);
            } else {
                let idx_p = ((neighborhood.len() - 1) as f64 * p) as usize;
                data.push(neighborhood[idx_p]);
            }
        }
    }

    let mut output = input.clone();
    let mut attrs = input.point_data().clone();
    attrs.remove_array(scalars);
    attrs.add_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, ncomp)));
    attrs.set_active_scalars(scalars);
    *output.point_data_mut() = attrs;
    output
}

/// Min filter (rank 0).
pub fn min_filter(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    rank_filter(input, scalars, radius, 0.0)
}

/// Max filter (rank 1).
pub fn max_filter(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    rank_filter(input, scalars, radius, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_min_max() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| (x + y) as f64,
        );
        let mn = min_filter(&img, "v", 1);
        let mx = max_filter(&img, "v", 1);
        let arr_mn = mn.point_data().get_array("v").unwrap();
        let arr_mx = mx.point_data().get_array("v").unwrap();
        let mut buf = [0.0];
        arr_mn.tuple_as_f64(2 + 2 * 5, &mut buf);
        assert_eq!(buf[0], 2.0); // min around center(2,2) is 1+1=2
        arr_mx.tuple_as_f64(2 + 2 * 5, &mut buf);
        assert_eq!(buf[0], 6.0); // max around center(2,2) is 3+3=6
    }
    #[test]
    fn test_rank_median() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x as f64,
        );
        let med = rank_filter(&img, "v", 1, 0.5);
        assert_eq!(med.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn rank_filter_preserves_other_point_arrays() {
        let mut img = ImageData::from_function(
            [3, 3, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, y, _| x + y,
        );
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "other",
                vec![1.0; 9],
                1,
            )));

        let filtered = rank_filter(&img, "v", 1, 0.5);
        assert!(filtered.point_data().get_array("other").is_some());
        assert!(filtered.point_data().scalars().is_some());
    }

    #[test]
    fn median_uses_vtk_even_neighborhood_rule_per_component() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 10.0, 2.0, 20.0],
                2,
            )));

        let filtered = rank_filter(&img, "v", 1, 0.5);
        let arr = filtered.point_data().get_array("v").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 15.0]);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf, [1.0, 15.0]);
    }
}
