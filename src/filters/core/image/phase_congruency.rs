use crate::data::{AnyDataArray, DataArray, ImageData};

/// Compute structure tensor eigenvalues for an ImageData field.
///
/// The structure tensor captures local gradient orientation. Its eigenvalues
/// indicate: both large = corner, one large = edge, both small = flat.
/// Adds "StructureLambda1" and "StructureLambda2" arrays.
pub fn image_structure_tensor(input: &ImageData, scalars: &str, radius: usize) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;
    let sp = input.spacing();
    if n == 0 || arr.num_tuples() != n || sp[0] == 0.0 || sp[1] == 0.0 {
        return input.clone();
    }
    let r = radius.max(1) as i64;

    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let get = |i: i64, j: i64, k: usize| -> f64 {
        values[k * ny * nx
            + (j.clamp(0, ny as i64 - 1) as usize) * nx
            + (i.clamp(0, nx as i64 - 1) as usize)]
    };

    let mut l1 = vec![0.0f64; n];
    let mut l2 = vec![0.0f64; n];

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ii = i as i64;
                let jj = j as i64;
                // Accumulate structure tensor in neighborhood
                let mut sxx = 0.0;
                let mut sxy = 0.0;
                let mut syy = 0.0;
                for dj in -r..=r {
                    for di in -r..=r {
                        let ni = (ii + di).clamp(0, nx as i64 - 1) as usize;
                        let nj = (jj + dj).clamp(0, ny as i64 - 1) as usize;
                        let gxi = (get(ni as i64 + 1, nj as i64, k)
                            - get(ni as i64 - 1, nj as i64, k))
                            / (2.0 * sp[0]);
                        let gyi = (get(ni as i64, nj as i64 + 1, k)
                            - get(ni as i64, nj as i64 - 1, k))
                            / (2.0 * sp[1]);
                        sxx += gxi * gxi;
                        sxy += gxi * gyi;
                        syy += gyi * gyi;
                    }
                }

                // Eigenvalues of 2x2 symmetric matrix [[sxx,sxy],[sxy,syy]]
                let trace = sxx + syy;
                let det = sxx * syy - sxy * sxy;
                let disc = (trace * trace - 4.0 * det).max(0.0).sqrt();
                let idx = k * ny * nx + j * nx + i;
                l1[idx] = (trace + disc) * 0.5;
                l2[idx] = (trace - disc) * 0.5;
            }
        }
    }

    let mut img = input.clone();
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "StructureLambda1",
            l1,
            1,
        )));
    img.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "StructureLambda2",
            l2,
            1,
        )));
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_detection() {
        let mut img = ImageData::with_dimensions(7, 7, 1);
        img.set_spacing([1.0; 3]);
        let mut values = vec![0.0; 49];
        for j in 0..7 {
            for i in 4..7 {
                values[j * 7 + i] = 100.0;
            }
        }
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", values, 1)));

        let result = image_structure_tensor(&img, "v", 1);
        let l1 = result.point_data().get_array("StructureLambda1").unwrap();
        let mut buf = [0.0f64];
        l1.tuple_as_f64(24, &mut buf); // near edge
        assert!(buf[0] > 0.0);
    }

    #[test]
    fn uniform_zero_structure() {
        let mut img = ImageData::with_dimensions(5, 5, 1);
        img.set_spacing([1.0; 3]);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![5.0; 25],
                1,
            )));

        let result = image_structure_tensor(&img, "v", 1);
        let l1 = result.point_data().get_array("StructureLambda1").unwrap();
        let mut buf = [0.0f64];
        l1.tuple_as_f64(12, &mut buf);
        assert!(buf[0] < 1e-10);
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 3, 1);
        let r = image_structure_tensor(&img, "nope", 1);
        assert!(r.point_data().get_array("StructureLambda1").is_none());
    }
}
