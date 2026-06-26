use crate::data::{AnyDataArray, DataArray, ImageData};

/// Build an image pyramid (multi-resolution representation).
///
/// Returns a Vec of ImageData at decreasing resolutions (factor 2 each level).
/// Level 0 = original, level 1 = half size, etc.
pub fn image_pyramid(input: &ImageData, scalars: &str, levels: usize) -> Vec<ImageData> {
    let mut pyramid = Vec::with_capacity(levels + 1);
    pyramid.push(input.clone());

    for _ in 0..levels {
        let prev = pyramid.last().unwrap();
        let down = crate::filters::core::image::downsample::image_downsample(prev, scalars, 2);
        if down.dimensions()[0] < 2 || down.dimensions()[1] < 2 {
            break;
        }
        pyramid.push(down);
    }

    pyramid
}

/// Compute Laplacian pyramid (difference between levels).
///
/// Returns differences between consecutive pyramid levels (upsampled).
/// Useful for multi-scale edge detection and image blending.
pub fn laplacian_pyramid(input: &ImageData, scalars: &str, levels: usize) -> Vec<ImageData> {
    let gauss = image_pyramid(input, scalars, levels);
    let mut lap = Vec::new();

    for i in 0..gauss.len() - 1 {
        let arr_fine = match gauss[i].point_data().get_array(scalars) {
            Some(a) => a,
            None => continue,
        };
        let arr_coarse = match gauss[i + 1].point_data().get_array(scalars) {
            Some(a) => a,
            None => continue,
        };

        let dims = gauss[i].dimensions();
        let nx = dims[0] as usize;
        let ny = dims[1] as usize;
        let cdims = gauss[i + 1].dimensions();
        let cnx = cdims[0] as usize;
        let ncomp = arr_fine.num_components();
        if ncomp != arr_coarse.num_components() {
            continue;
        }

        let mut buf_f = vec![0.0f64; ncomp];
        let mut buf_c = vec![0.0f64; ncomp];
        let n = nx * ny;
        let mut diff = Vec::with_capacity(n * ncomp);

        for j in 0..ny {
            for i_x in 0..nx {
                arr_fine.tuple_as_f64(j * nx + i_x, &mut buf_f);
                // Nearest neighbor from coarse level
                let ci = (i_x / 2).min(cnx - 1);
                let cj = (j / 2).min(cdims[1] as usize - 1);
                arr_coarse.tuple_as_f64(cj * cnx + ci, &mut buf_c);
                for c in 0..ncomp {
                    diff.push(buf_f[c] - buf_c[c]);
                }
            }
        }

        let mut img = gauss[i].clone();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "LaplacianLevel",
                diff,
                ncomp,
            )));
        lap.push(img);
    }

    lap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pyramid_levels() {
        let mut img = ImageData::with_dimensions(16, 16, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                (0..256).map(|i| i as f64).collect(),
                1,
            )));

        let pyr = image_pyramid(&img, "v", 3);
        assert!(pyr.len() >= 3);
        assert_eq!(pyr[0].dimensions(), [16, 16, 1]);
        assert_eq!(pyr[1].dimensions(), [8, 8, 1]);
    }

    #[test]
    fn laplacian_levels() {
        let mut img = ImageData::with_dimensions(8, 8, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                (0..64).map(|i| i as f64).collect(),
                1,
            )));

        let lap = laplacian_pyramid(&img, "v", 2);
        assert!(!lap.is_empty());
        assert!(lap[0].point_data().get_array("LaplacianLevel").is_some());
    }

    #[test]
    fn single_level() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![1.0; 16],
                1,
            )));

        let pyr = image_pyramid(&img, "v", 0);
        assert_eq!(pyr.len(), 1);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(4, 4, 1);
        let pyr = image_pyramid(&img, "nope", 3);
        // downsample returns clone when array missing, so pyramid still builds
        assert!(pyr.len() >= 1);
    }

    #[test]
    fn laplacian_preserves_component_count() {
        let mut img = ImageData::with_dimensions(4, 4, 1);
        let values: Vec<f64> = (0..16)
            .flat_map(|i| [i as f64, (100 + i) as f64])
            .collect();
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 2)));

        let lap = laplacian_pyramid(&img, "v", 1);
        let arr = lap[0].point_data().get_array("LaplacianLevel").unwrap();
        assert_eq!(arr.num_components(), 2);
        assert_eq!(arr.num_tuples(), 16);
    }
}
