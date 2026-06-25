use crate::data::{AnyDataArray, DataArray, ImageData};

/// VTK-style anisotropic diffusion on ImageData.
///
/// This follows vtkImageAnisotropicDiffusion2D/3D defaults: faces, edges, and
/// corners are enabled, gradient-magnitude thresholding is disabled, `kappa`
/// is the diffusion threshold, and `dt` is the diffusion factor.
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
    let nx = dims[0];
    let ny = dims[1];
    let nz = dims[2];
    let n = nx * ny * nz;
    let num_components = arr.num_components();

    let mut buf = vec![0.0f64; num_components];
    let mut values: Vec<f64> = (0..n)
        .flat_map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf.clone()
        })
        .collect();

    let spacing = input.spacing();
    for _ in 0..iterations {
        let mut next = values.clone();
        if nz > 1 {
            iterate_3d(
                &values,
                &mut next,
                [nx, ny, nz],
                num_components,
                spacing,
                kappa,
                dt,
            );
        } else {
            iterate_2d(
                &values,
                &mut next,
                [nx, ny],
                num_components,
                spacing,
                kappa,
                dt,
            );
        }
        values = next;
    }

    let mut img = input.clone();
    let mut attrs = crate::data::DataSetAttributes::new();
    for i in 0..input.point_data().num_arrays() {
        let a = input.point_data().get_array_by_index(i).unwrap();
        if a.name() == scalars {
            attrs.add_array(AnyDataArray::F64(DataArray::from_vec(
                scalars,
                values.clone(),
                num_components,
            )));
        } else {
            attrs.add_array(a.clone());
        }
    }
    *img.point_data_mut() = attrs;
    img
}

fn iterate_2d(
    input: &[f64],
    output: &mut [f64],
    dims: [usize; 2],
    num_components: usize,
    spacing: [f64; 3],
    diffusion_threshold: f64,
    diffusion_factor: f64,
) {
    let [nx, ny] = dims;
    let ar0 = spacing[0];
    let ar1 = spacing[1];
    let diagonal = (ar0 * ar0 + ar1 * ar1).sqrt();
    let df0 = 1.0 / ar0;
    let df1 = 1.0 / ar1;
    let df01 = 1.0 / diagonal;
    let scale = diffusion_factor / (2.0 * (df0 + df1) + 4.0 * df01);

    for j in 0..ny {
        for i in 0..nx {
            for component in 0..num_components {
                let center_idx = idx(i, j, 0, nx, ny, num_components, component);
                let center = input[center_idx];
                let mut value = center;

                for dj in -1..=1 {
                    for di in -1..=1 {
                        if di == 0 && dj == 0 {
                            continue;
                        }
                        let ni = i as isize + di;
                        let nj = j as isize + dj;
                        if ni < 0 || ni >= nx as isize || nj < 0 || nj >= ny as isize {
                            continue;
                        }

                        let distance = if di != 0 && dj != 0 {
                            diagonal
                        } else if di != 0 {
                            ar0
                        } else {
                            ar1
                        };
                        diffuse_neighbor(
                            input,
                            &mut value,
                            center,
                            distance * diffusion_threshold,
                            scale / distance,
                            idx(
                                ni as usize,
                                nj as usize,
                                0,
                                nx,
                                ny,
                                num_components,
                                component,
                            ),
                        );
                    }
                }

                output[center_idx] = value;
            }
        }
    }
}

fn iterate_3d(
    input: &[f64],
    output: &mut [f64],
    dims: [usize; 3],
    num_components: usize,
    spacing: [f64; 3],
    diffusion_threshold: f64,
    diffusion_factor: f64,
) {
    let [nx, ny, nz] = dims;
    let ar0 = spacing[0];
    let ar1 = spacing[1];
    let ar2 = spacing[2];
    let df0 = 1.0 / ar0;
    let df1 = 1.0 / ar1;
    let df2 = 1.0 / ar2;
    let df01 = 1.0 / (ar0 * ar0 + ar1 * ar1).sqrt();
    let df02 = 1.0 / (ar0 * ar0 + ar2 * ar2).sqrt();
    let df12 = 1.0 / (ar1 * ar1 + ar2 * ar2).sqrt();
    let df012 = 1.0 / (ar0 * ar0 + ar1 * ar1 + ar2 * ar2).sqrt();
    let scale =
        diffusion_factor / (2.0 * (df0 + df1 + df2) + 4.0 * (df01 + df02 + df12) + 8.0 * df012);

    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                for component in 0..num_components {
                    let center_idx = idx(i, j, k, nx, ny, num_components, component);
                    let center = input[center_idx];
                    let mut value = center;

                    for dk in -1..=1 {
                        for dj in -1..=1 {
                            for di in -1..=1 {
                                if di == 0 && dj == 0 && dk == 0 {
                                    continue;
                                }
                                let ni = i as isize + di;
                                let nj = j as isize + dj;
                                let nk = k as isize + dk;
                                if ni < 0
                                    || ni >= nx as isize
                                    || nj < 0
                                    || nj >= ny as isize
                                    || nk < 0
                                    || nk >= nz as isize
                                {
                                    continue;
                                }

                                let distance = ((di as f64 * ar0).powi(2)
                                    + (dj as f64 * ar1).powi(2)
                                    + (dk as f64 * ar2).powi(2))
                                .sqrt();
                                diffuse_neighbor(
                                    input,
                                    &mut value,
                                    center,
                                    distance * diffusion_threshold,
                                    scale / distance,
                                    idx(
                                        ni as usize,
                                        nj as usize,
                                        nk as usize,
                                        nx,
                                        ny,
                                        num_components,
                                        component,
                                    ),
                                );
                            }
                        }
                    }

                    output[center_idx] = value;
                }
            }
        }
    }
}

fn idx(
    i: usize,
    j: usize,
    k: usize,
    nx: usize,
    ny: usize,
    num_components: usize,
    component: usize,
) -> usize {
    (k * ny * nx + j * nx + i) * num_components + component
}

fn diffuse_neighbor(
    input: &[f64],
    output: &mut f64,
    center: f64,
    threshold: f64,
    factor: f64,
    neighbor_idx: usize,
) {
    let temp = input[neighbor_idx] - center;
    if temp.abs() < threshold {
        *output += temp * factor;
    }
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

        let result = anisotropic_diffusion(&img, "v", 100.0, 0.1, 10);
        let arr = result.point_data().get_array("v").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(2, &mut buf);
        assert!(buf[0] < 50.0);
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
