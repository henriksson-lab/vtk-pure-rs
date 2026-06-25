use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a single component from a multi-component ImageData array.
///
/// Creates a new ImageData with the same dimensions, spacing, and origin,
/// containing a new single-component array named "{name}_Component{component}".
pub fn extract_component(input: &ImageData, array_name: &str, component: usize) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };
    let nc: usize = arr.num_components();
    if component >= nc {
        return input.clone();
    }

    let n: usize = arr.num_tuples();
    let mut data: Vec<f64> = Vec::with_capacity(n);
    let mut buf: Vec<f64> = vec![0.0; nc];

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        data.push(buf[component]);
    }

    let out_name: String = format!("{}_Component{}", array_name, component);
    let mut result = ImageData::with_dimensions(0, 0, 0);
    result.set_extent(input.extent());
    result.set_spacing(input.spacing());
    result.set_origin(input.origin());
    result.with_point_array(AnyDataArray::F64(DataArray::from_vec(&out_name, data, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSet;

    #[test]
    fn extract_first_component() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        let n: usize = img.num_points();
        // 3-component array: (i, i*10, i*100)
        let mut vals: Vec<f64> = Vec::new();
        for i in 0..n {
            vals.push(i as f64);
            vals.push((i * 10) as f64);
            vals.push((i * 100) as f64);
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("rgb", vals, 3)));

        let result = extract_component(&img, "rgb", 0);
        let out_arr = result.point_data().get_array("rgb_Component0").unwrap();
        assert_eq!(out_arr.num_tuples(), n);
        assert_eq!(out_arr.num_components(), 1);
        assert!(result.point_data().get_array("rgb").is_none());
        let mut val = [0.0f64];
        out_arr.tuple_as_f64(2, &mut val);
        assert!((val[0] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn extract_second_component() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        let n: usize = img.num_points();
        let mut vals: Vec<f64> = Vec::new();
        for i in 0..n {
            vals.push(i as f64);
            vals.push((i * 10) as f64);
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("data", vals, 2)));

        let result = extract_component(&img, "data", 1);
        let out_arr = result.point_data().get_array("data_Component1").unwrap();
        let mut val = [0.0f64];
        out_arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 10.0).abs() < 1e-12);
    }

    #[test]
    fn out_of_range_component() {
        let mut img = ImageData::with_dimensions(2, 2, 1);
        let n: usize = img.num_points();
        let vals: Vec<f64> = vec![0.0; n * 2];
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vals, 2)));
        let result = extract_component(&img, "v", 3);
        assert!(result.point_data().get_array("v_Component3").is_none());
    }
}
