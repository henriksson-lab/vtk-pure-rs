use crate::data::{AnyDataArray, DataArray, ImageData};
use crate::types::ImplicitFunction;

/// Clip ImageData by an implicit function.
///
/// For each voxel, evaluates the implicit function at its world-space position.
/// Voxels where `f(x,y,z) < 0` are masked (set to 0.0). Voxels where
/// `f(x,y,z) >= 0` retain their original scalar value.
///
/// Returns a new ImageData with a "Clipped" scalar array.
pub fn clip_volume(input: &ImageData, scalars: &str, func: &dyn ImplicitFunction) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let mut output_values = Vec::with_capacity(n);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = input.point_from_ijk(i, j, k);
                let fval = func.evaluate(p[0], p[1], p[2]);
                let idx = k * ny * nx + j * nx + i;
                arr.tuple_as_f64(idx, &mut buf);

                if fval >= 0.0 {
                    output_values.push(buf[0]);
                } else {
                    output_values.push(0.0);
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Clipped",
            output_values,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ImplicitPlane;

    #[test]
    fn clip_by_plane() {
        let img = ImageData::from_function(
            [5, 1, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "val",
            |x, _y, _z| x + 1.0, // values: 1, 2, 3, 4, 5
        );

        // Plane at x=2, normal +X: f(x,y,z) = x - 2
        // Points at x=0,1 are outside (f<0), x=2,3,4 are inside (f>=0)
        let plane = ImplicitPlane::new([2.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let result = clip_volume(&img, "val", &plane);

        let arr = result.point_data().get_array("Clipped").unwrap();
        let mut buf = [0.0f64];

        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // x=0, clipped

        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 0.0); // x=1, clipped

        arr.tuple_as_f64(2, &mut buf);
        assert!((buf[0] - 3.0).abs() < 1e-10); // x=2, kept

        arr.tuple_as_f64(4, &mut buf);
        assert!((buf[0] - 5.0).abs() < 1e-10); // x=4, kept
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let plane = ImplicitPlane::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let result = clip_volume(&img, "nope", &plane);
        assert!(result.point_data().get_array("Clipped").is_none());
    }
}
