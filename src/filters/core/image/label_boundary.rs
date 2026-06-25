use crate::data::{AnyDataArray, DataArray, ImageData};

/// Extract boundaries between labeled regions in a segmented ImageData.
///
/// A voxel is on the boundary if any of its 6 neighbors has a different label.
/// Adds "LabelBoundary" binary array (1=boundary, 0=interior).
pub fn image_label_boundary(input: &ImageData, labels_name: &str) -> ImageData {
    let arr = match input.point_data().get_array(labels_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let values: Vec<i64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as i64
        })
        .collect();

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;
    let mut boundary = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let v = values[idx(i, j, k)];
                let is_boundary = (i > 0 && values[idx(i - 1, j, k)] != v)
                    || (i + 1 < nx && values[idx(i + 1, j, k)] != v)
                    || (j > 0 && values[idx(i, j - 1, k)] != v)
                    || (j + 1 < ny && values[idx(i, j + 1, k)] != v)
                    || (k > 0 && values[idx(i, j, k - 1)] != v)
                    || (k + 1 < nz && values[idx(i, j, k + 1)] != v);
                if is_boundary {
                    boundary[idx(i, j, k)] = 1.0;
                }
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "LabelBoundary",
            boundary,
            1,
        )));
    img
}

/// Count boundary voxels between each pair of labels.
pub fn label_contact_area(input: &ImageData, labels_name: &str) -> Vec<(i64, i64, usize)> {
    let arr = match input.point_data().get_array(labels_name) {
        Some(a) => a,
        None => return vec![],
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let values: Vec<i64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0] as i64
        })
        .collect();

    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;
    let mut contacts: std::collections::HashMap<(i64, i64), usize> =
        std::collections::HashMap::new();

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let v = values[idx(i, j, k)];
                let neighbors = [
                    if i + 1 < nx {
                        Some(values[idx(i + 1, j, k)])
                    } else {
                        None
                    },
                    if j + 1 < ny {
                        Some(values[idx(i, j + 1, k)])
                    } else {
                        None
                    },
                    if k + 1 < nz {
                        Some(values[idx(i, j, k + 1)])
                    } else {
                        None
                    },
                ];
                for nb in &neighbors {
                    if let Some(nv) = nb {
                        if *nv != v {
                            let key = if v < *nv { (v, *nv) } else { (*nv, v) };
                            *contacts.entry(key).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut result: Vec<(i64, i64, usize)> =
        contacts.into_iter().map(|((a, b), c)| (a, b, c)).collect();
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_regions_boundary() {
        let mut img = ImageData::with_dimensions(6, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
                1,
            )));

        let result = image_label_boundary(&img, "l");
        let arr = result.point_data().get_array("LabelBoundary").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert_eq!(buf[0], 1.0); // boundary
        arr.tuple_as_f64(3, &mut buf);
        assert_eq!(buf[0], 1.0); // boundary
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0); // interior
    }

    #[test]
    fn contact_area() {
        let mut img = ImageData::with_dimensions(4, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "l",
                vec![0.0, 0.0, 1.0, 1.0],
                1,
            )));

        let contacts = label_contact_area(&img, "l");
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0], (0, 1, 1)); // one contact face
    }

    #[test]
    fn uniform_no_boundary() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("l", vec![1.0; 9], 1)));

        let result = image_label_boundary(&img, "l");
        let arr = result.point_data().get_array("LabelBoundary").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert_eq!(buf[0], 0.0);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = image_label_boundary(&img, "nope");
        assert!(r.point_data().get_array("LabelBoundary").is_none());
    }
}
