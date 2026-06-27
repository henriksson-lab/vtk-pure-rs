//! Envelope detection (abs + smooth)
use crate::data::{AnyDataArray, DataArray, ImageData};
pub fn image_envelope_detect(input: &ImageData, scalars: &str) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) if a.num_components() == 1 => a,
        _ => return input.clone(),
    };
    let dims = input.dimensions();
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;
    let mut buf = [0.0f64];
    let rectified: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0].abs()
        })
        .collect();
    let mut data = Vec::with_capacity(n);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let mut sum = 0.0;
                let mut count = 0.0;
                for dz in -1i64..=1 {
                    for dy in -1i64..=1 {
                        for dx in -1i64..=1 {
                            let sx = ix as i64 + dx;
                            let sy = iy as i64 + dy;
                            let sz = iz as i64 + dz;
                            if sx >= 0
                                && sy >= 0
                                && sz >= 0
                                && (sx as usize) < nx
                                && (sy as usize) < ny
                                && (sz as usize) < nz
                            {
                                sum += rectified
                                    [sx as usize + sy as usize * nx + sz as usize * nx * ny];
                                count += 1.0;
                            }
                        }
                    }
                }
                data.push(sum / count);
            }
        }
    }
    ImageData::with_dimensions(dims[0], dims[1], dims[2])
        .with_spacing(input.spacing())
        .with_origin(input.origin())
        .with_point_array(AnyDataArray::F64(DataArray::from_vec(scalars, data, 1)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let img = ImageData::from_function(
            [5, 5, 1],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            "v",
            |x, _, _| x + 1.0,
        );
        let r = image_envelope_detect(&img, "v");
        assert_eq!(r.dimensions(), [5, 5, 1]);
    }

    #[test]
    fn rectifies_and_smooths() {
        let mut img = ImageData::with_dimensions(3, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![-1.0, 3.0, -5.0],
                1,
            )));
        let r = image_envelope_detect(&img, "v");
        let arr = r.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 2.0);
        arr.tuple_as_f64(1, &mut buf);
        assert_eq!(buf[0], 3.0);
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 4.0);
    }
}
