use crate::data::{AnyDataArray, DataArray, ImageData};

/// Correlate an image with a second image used as the correlation kernel.
///
/// This follows `vtkImageCorrelation`: the output dimensions match the first
/// input, the second input is placed with its origin at each output pixel, and
/// only the overlapping part of the first input contributes near boundaries.
/// All scalar components are summed into one f64 output component.
pub fn image_correlation(
    input: &ImageData,
    kernel: &ImageData,
    input_scalars: &str,
    kernel_scalars: &str,
) -> ImageData {
    let in_array = match input.point_data().get_array(input_scalars) {
        Some(array) => array,
        None => return input.clone(),
    };
    let kernel_array = match kernel.point_data().get_array(kernel_scalars) {
        Some(array) => array,
        None => return input.clone(),
    };
    if in_array.num_components() != kernel_array.num_components() {
        return input.clone();
    }

    let dims = input.dimensions();
    let kernel_dims = kernel.dimensions();
    let n = dims[0].saturating_mul(dims[1]).saturating_mul(dims[2]);
    if n == 0 || kernel_dims.iter().any(|&d| d == 0) {
        return input.clone();
    }

    let num_components = in_array.num_components();
    let mut in_tuple = vec![0.0; num_components];
    let mut kernel_tuple = vec![0.0; num_components];
    let mut output = vec![0.0; n];

    let input_plane = dims[0] * dims[1];
    let kernel_plane = kernel_dims[0] * kernel_dims[1];

    for z in 0..dims[2] {
        let max_kz = (dims[2] - 1 - z).min(kernel_dims[2] - 1);
        for y in 0..dims[1] {
            let max_ky = (dims[1] - 1 - y).min(kernel_dims[1] - 1);
            for x in 0..dims[0] {
                let max_kx = (dims[0] - 1 - x).min(kernel_dims[0] - 1);
                let mut sum = 0.0;

                for kz in 0..=max_kz {
                    for ky in 0..=max_ky {
                        for kx in 0..=max_kx {
                            let in_idx = (z + kz) * input_plane + (y + ky) * dims[0] + x + kx;
                            let kernel_idx = kz * kernel_plane + ky * kernel_dims[0] + kx;
                            in_array.tuple_as_f64(in_idx, &mut in_tuple);
                            kernel_array.tuple_as_f64(kernel_idx, &mut kernel_tuple);
                            for component in 0..num_components {
                                sum += in_tuple[component] * kernel_tuple[component];
                            }
                        }
                    }
                }

                output[z * input_plane + y * dims[0] + x] = sum;
            }
        }
    }

    let mut result = ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(
            input_scalars,
            output,
            1,
        )));
    result.set_extent(input.extent());
    result
}

/// Compute Pearson correlation coefficient between two ImageData arrays.
pub fn image_pearson_correlation(
    a: &ImageData,
    b: &ImageData,
    a_scalars: &str,
    b_scalars: &str,
) -> f64 {
    let aa = match a.point_data().get_array(a_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(b_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = aa.num_tuples().min(ba.num_tuples());
    if n < 2 {
        return 0.0;
    }

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];
    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_ab = 0.0;
    let mut sum_a2 = 0.0;
    let mut sum_b2 = 0.0;

    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        sum_a += buf_a[0];
        sum_b += buf_b[0];
        sum_ab += buf_a[0] * buf_b[0];
        sum_a2 += buf_a[0] * buf_a[0];
        sum_b2 += buf_b[0] * buf_b[0];
    }

    let nf = n as f64;
    let num = nf * sum_ab - sum_a * sum_b;
    let den = ((nf * sum_a2 - sum_a * sum_a) * (nf * sum_b2 - sum_b * sum_b)).sqrt();
    if den > 1e-15 {
        num / den
    } else {
        0.0
    }
}

/// Compute covariance between two ImageData arrays.
pub fn image_covariance(a: &ImageData, b: &ImageData, a_scalars: &str, b_scalars: &str) -> f64 {
    let aa = match a.point_data().get_array(a_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let ba = match b.point_data().get_array(b_scalars) {
        Some(x) => x,
        None => return 0.0,
    };
    let n = aa.num_tuples().min(ba.num_tuples());
    if n < 2 {
        return 0.0;
    }

    let mut buf_a = [0.0f64];
    let mut buf_b = [0.0f64];
    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_ab = 0.0;
    for i in 0..n {
        aa.tuple_as_f64(i, &mut buf_a);
        ba.tuple_as_f64(i, &mut buf_b);
        sum_a += buf_a[0];
        sum_b += buf_b[0];
        sum_ab += buf_a[0] * buf_b[0];
    }
    let nf = n as f64;
    sum_ab / nf - (sum_a / nf) * (sum_b / nf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sliding_correlation_matches_vtk_kernel_placement() {
        let mut input = ImageData::with_dimensions(4, 1, 1);
        input
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));
        let mut kernel = ImageData::with_dimensions(2, 1, 1);
        kernel
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "k",
                vec![10.0, 1.0],
                1,
            )));

        let result = image_correlation(&input, &kernel, "v", "k");
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0; 1];
        let expected = [12.0, 23.0, 34.0, 40.0];
        for (i, value) in expected.iter().enumerate() {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - value).abs() < 1e-10);
        }
    }

    #[test]
    fn pearson_correlation_helper() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0, 4.0, 5.0],
                1,
            )));
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "b",
                vec![2.0, 4.0, 6.0, 8.0, 10.0],
                1,
            )));

        let r = image_pearson_correlation(&img, &img, "a", "b");
        assert!((r - 1.0).abs() < 1e-10);
    }

    #[test]
    fn covariance_basic() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "a",
                vec![1.0, 2.0, 3.0, 4.0],
                1,
            )));

        let cov = image_covariance(&img, &img, "a", "a");
        assert!(cov > 0.0);
    }
}
