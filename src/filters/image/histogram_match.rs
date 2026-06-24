use crate::data::{AnyDataArray, DataArray, ImageData};

/// Match the histogram of one image to another (histogram specification).
///
/// Remaps values in `input` so its cumulative histogram matches `reference`.
pub fn image_histogram_match(input: &ImageData, reference: &ImageData, scalars: &str) -> ImageData {
    let ia = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };
    let ra = match reference.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let ni = ia.num_tuples();
    let nr = ra.num_tuples();
    let mut buf = [0.0f64];

    // Sort input values with indices
    let mut ival: Vec<(f64, usize)> = (0..ni)
        .map(|i| {
            ia.tuple_as_f64(i, &mut buf);
            (buf[0], i)
        })
        .collect();
    ival.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Sort reference values
    let mut rval: Vec<f64> = (0..nr)
        .map(|i| {
            ra.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();
    rval.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Map each input rank to corresponding reference value
    let mut result = vec![0.0f64; ni];
    for (rank, &(_, orig_idx)) in ival.iter().enumerate() {
        let ref_rank = (rank as f64 / (ni - 1).max(1) as f64 * (nr - 1) as f64).round() as usize;
        result[orig_idx] = rval[ref_rank.min(nr - 1)];
    }

    let mut img = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                result.clone(),
                1,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = attrs;
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_to_reference() {
        let mut input = ImageData::with_dimensions(4, 1, 1);
        input
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 1.0, 2.0, 3.0],
                1,
            )));

        let mut reference = ImageData::with_dimensions(4, 1, 1);
        reference
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 20.0, 30.0, 40.0],
                1,
            )));

        let result = image_histogram_match(&input, &reference, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 10.0); // lowest maps to lowest
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 40.0); // highest maps to highest
    }

    #[test]
    fn same_image_identity() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        let values = vec![5.0, 3.0, 1.0, 4.0, 2.0];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                values.clone(),
                1,
            )));

        let result = image_histogram_match(&img, &img, "v");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // Values should be the same (matched to itself)
        for i in 0..5 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], values[i]);
        }
    }

    #[test]
    fn missing_array() {
        let a = ImageData::with_dimensions(3, 1, 1);
        let b = ImageData::with_dimensions(3, 1, 1);
        let r = image_histogram_match(&a, &b, "nope");
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
