use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract a single component from a multi-component ImageData array.
///
/// For a 3-component "RGB" array, component 0=R, 1=G, 2=B.
pub fn image_extract_component(
    input: &ImageData,
    array_name: &str,
    component: usize,
    output: &str,
) -> ImageData {
    image_extract_components(input, array_name, &[component], output)
}

/// Extract one to three components from a multi-component ImageData array.
///
/// This mirrors VTK's `vtkImageExtractComponents`: output image geometry is
/// preserved, and the output scalar array contains only the selected
/// components in the requested order.
pub fn image_extract_components(
    input: &ImageData,
    array_name: &str,
    components: &[usize],
    output: &str,
) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let nc = arr.num_components();
    if components.is_empty() || components.len() > 3 || components.iter().any(|&c| c >= nc) {
        return input.clone();
    }

    let n = arr.num_tuples();
    let mut buf = vec![0.0f64; nc];
    let mut values = Vec::with_capacity(n * components.len());
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        for &component in components {
            values.push(buf[component]);
        }
    }

    let mut output_image = ImageData::with_dimensions(0, 0, 0);
    output_image.set_extent(input.extent());
    output_image.set_spacing(input.spacing());
    output_image.set_origin(input.origin());
    output_image.with_point_array(AnyDataArray::F64(DataArray::from_vec(
        output,
        values,
        components.len(),
    )))
}

/// Merge multiple scalar arrays into a single multi-component array.
pub fn image_merge_components(input: &ImageData, names: &[&str], output: &str) -> ImageData {
    if names.is_empty() {
        return input.clone();
    }

    let arrays: Vec<_> = names
        .iter()
        .filter_map(|n| input.point_data().get_array(n))
        .collect();
    if arrays.is_empty() {
        return input.clone();
    }

    let nc = arrays.len();
    let n = arrays[0].num_tuples();
    if arrays
        .iter()
        .any(|array| array.num_components() != 1 || array.num_tuples() != n)
    {
        return input.clone();
    }
    let mut buf = [0.0f64];
    let mut values = Vec::with_capacity(n * nc);

    for i in 0..n {
        for arr in &arrays {
            arr.tuple_as_f64(i, &mut buf);
            values.push(buf[0]);
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(output, values, nc)));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_r_from_rgb() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                3,
            )));

        let result = image_extract_component(&img, "rgb", 0, "red");
        let arr = result.point_data().get_array("red").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0);
        assert!(result.point_data().get_array("rgb").is_none());
    }

    #[test]
    fn extract_two_components() {
        let mut img = ImageData::with_dimensions(1, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0, 2.0, 3.0],
                3,
            )));

        let result = image_extract_components(&img, "rgb", &[2, 0], "br");
        let arr = result.point_data().get_array("br").unwrap();
        assert_eq!(arr.num_components(), 2);
        let mut buf = [0.0f64; 2];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [3.0, 1.0]);
    }

    #[test]
    fn merge_channels() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "r",
                vec![1.0, 0.5],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "g",
                vec![0.0, 0.5],
                1,
            )));

        let result = image_merge_components(&img, &["r", "g"], "rg");
        let arr = result.point_data().get_array("rg").unwrap();
        assert_eq!(arr.num_components(), 2);
    }

    #[test]
    fn invalid_component() {
        let mut img = ImageData::with_dimensions(2, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0],
                1,
            )));

        let result = image_extract_component(&img, "v", 5, "out"); // only 1 component
        assert!(result.point_data().get_array("out").is_none());
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(2, 1, 1);
        let r = image_extract_component(&img, "nope", 0, "out");
        assert!(r.point_data().get_array("out").is_none());
    }
}
