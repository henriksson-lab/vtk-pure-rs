use crate::data::{AnyDataArray, DataArray, ImageData};

/// Perona-Malik anisotropic diffusion on ImageData.
///
/// Edge-preserving smoothing that diffuses along flat regions but
/// reduces diffusion across edges. `kappa` controls edge sensitivity
/// (higher = smoother), `dt` is time step, `iterations` controls amount.
pub fn anisotropic_diffusion(
    input: &ImageData,
    scalars: &str,
    kappa: f64,
    dt: f64,
    iterations: usize,
) -> ImageData {
    let arr = match input.point_data().get_array(scalars) {
        Some(a) => a,
        None => return input.clone(),
    };

    let dims = input.dimensions();
    let nx = dims[0] as usize;
    let ny = dims[1] as usize;
    let nz = dims[2] as usize;
    let n = nx * ny * nz;

    let mut buf = [0.0f64];
    let mut u: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let k2 = kappa * kappa;
    let idx = |i: usize, j: usize, k: usize| k * ny * nx + j * nx + i;

    for _ in 0..iterations {
        let mut new_u = u.clone();
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let c = u[idx(i, j, k)];
                    let mut flux = 0.0;

                    // 6-connected neighbors
                    let nbrs: [(i64, i64, i64); 6] = [
                        (-1, 0, 0),
                        (1, 0, 0),
                        (0, -1, 0),
                        (0, 1, 0),
                        (0, 0, -1),
                        (0, 0, 1),
                    ];
                    for &(di, dj, dk) in &nbrs {
                        let ni = i as i64 + di;
                        let nj = j as i64 + dj;
                        let nk = k as i64 + dk;
                        if ni >= 0
                            && ni < nx as i64
                            && nj >= 0
                            && nj < ny as i64
                            && nk >= 0
                            && nk < nz as i64
                        {
                            let nv = u[idx(ni as usize, nj as usize, nk as usize)];
                            let diff = nv - c;
                            // Perona-Malik conductance: g = 1/(1 + (|∇I|/κ)²)
                            let g = 1.0 / (1.0 + diff * diff / k2);
                            flux += g * diff;
                        }
                    }
                    new_u[idx(i, j, k)] = c + dt * flux;
                }
            }
        }
        u = new_u;
    }

    let mut img = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                u.clone(),
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
    fn smooths_noise() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![10.0, 10.0, 50.0, 10.0, 10.0],
                1,
            )));

        let result = anisotropic_diffusion(&img, "v", 10.0, 0.1, 10);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] < 50.0); // spike reduced
    }

    #[test]
    fn preserves_strong_edge() {
        let mut img = ImageData::with_dimensions(5, 1, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "v",
                vec![0.0, 0.0, 100.0, 100.0, 100.0],
                1,
            )));

        let result = anisotropic_diffusion(&img, "v", 5.0, 0.1, 5);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        // Edge between 0s and 100s should be somewhat preserved
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[0] < 30.0);
        arr.tuple_as_f64(4, &mut buf);
        assert!(buf[0] > 70.0);
    }

    #[test]
    fn uniform_unchanged() {
        let mut img = ImageData::with_dimensions(3, 3, 1);
        img.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("v", vec![5.0; 9], 1)));

        let result = anisotropic_diffusion(&img, "v", 1.0, 0.1, 10);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        for i in 0..9 {
            arr.tuple_as_f64(i, &mut buf);
            assert!((buf[0] - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn missing_array() {
        let img = ImageData::with_dimensions(3, 1, 1);
        let r = anisotropic_diffusion(&img, "nope", 1.0, 0.1, 5);
        assert_eq!(r.dimensions(), [3, 1, 1]);
    }
}
