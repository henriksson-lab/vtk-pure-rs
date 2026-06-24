use crate::data::{AnyDataArray, DataArray, DataSet, ImageData, KdTree, PolyData};

/// Resample a PolyData scalar field onto an ImageData grid.
///
/// For each grid point, finds the nearest point in the PolyData and
/// copies its scalar value. Uses a k-d tree for efficient lookups.
pub fn resample_to_image(input: &PolyData, array_name: &str, dimensions: [usize; 3]) -> ImageData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]),
    };

    let n = input.points.len();
    if n == 0 {
        return ImageData::with_dimensions(dimensions[0], dimensions[1], dimensions[2]);
    }

    let bb = input.bounds();
    let padding = bb.diagonal_length() * 0.05;
    let origin = [bb.x_min - padding, bb.y_min - padding, bb.z_min - padding];
    let extent = [
        bb.x_max - bb.x_min + 2.0 * padding,
        bb.y_max - bb.y_min + 2.0 * padding,
        bb.z_max - bb.z_min + 2.0 * padding,
    ];
    let spacing = [
        extent[0] / (dimensions[0] as f64 - 1.0).max(1.0),
        extent[1] / (dimensions[1] as f64 - 1.0).max(1.0),
        extent[2] / (dimensions[2] as f64 - 1.0).max(1.0),
    ];

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let tree = KdTree::build(&pts);

    let nx = dimensions[0];
    let ny = dimensions[1];
    let nz = dimensions[2];
    let total = nx * ny * nz;
    let mut values = vec![0.0f64; total];
    let mut buf = [0.0f64];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [
                    origin[0] + i as f64 * spacing[0],
                    origin[1] + j as f64 * spacing[1],
                    origin[2] + k as f64 * spacing[2],
                ];
                if let Some((idx, _)) = tree.nearest(p) {
                    arr.tuple_as_f64(idx, &mut buf);
                    values[k * ny * nx + j * nx + i] = buf[0];
                }
            }
        }
    }

    let mut img = ImageData::with_dimensions(nx, ny, nz);
    img.set_origin(origin);
    img.set_spacing(spacing);
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            array_name, values, 1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_points() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![10.0, 20.0, 30.0],
                1,
            )));

        let img = resample_to_image(&pd, "temp", [5, 5, 1]);
        assert_eq!(img.dimensions(), [5, 5, 1]);
        assert!(img.point_data().get_array("temp").is_some());
    }

    #[test]
    fn missing_array() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        let img = resample_to_image(&pd, "nope", [3, 3, 3]);
        assert_eq!(img.dimensions(), [3, 3, 3]);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let img = resample_to_image(&pd, "val", [3, 3, 3]);
        assert_eq!(img.dimensions(), [3, 3, 3]);
    }
}
