//! WarpLens -- apply camera lens distortion to mesh points.

use crate::data::PolyData;

/// Parameters used by VTK's `vtkWarpLens` lens distortion model.
///
/// `principal_point` is in format units, `format_width` and `format_height`
/// are the imager size in the same units, and `image_width`/`image_height`
/// are the pixel dimensions used for coordinate conversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarpLensOptions {
    pub principal_point: [f64; 2],
    pub k1: f64,
    pub k2: f64,
    pub p1: f64,
    pub p2: f64,
    pub format_width: f64,
    pub format_height: f64,
    pub image_width: i32,
    pub image_height: i32,
}

impl Default for WarpLensOptions {
    fn default() -> Self {
        Self {
            principal_point: [0.0, 0.0],
            k1: -1.0e-6,
            k2: 0.0,
            p1: 0.0,
            p2: 0.0,
            format_width: 1.0,
            format_height: 1.0,
            image_width: 1,
            image_height: 1,
        }
    }
}

/// Apply barrel (k1 > 0) or pincushion (k1 < 0) radial lens distortion.
///
/// This is the common `vtkWarpLens` case with only `K1` set. `center` maps to
/// VTK's principal point/legacy center, with default unit image and format
/// dimensions.
pub fn warp_lens(input: &PolyData, center: [f64; 2], k1: f64) -> PolyData {
    warp_lens_with_options(
        input,
        &WarpLensOptions {
            principal_point: center,
            k1,
            ..Default::default()
        },
    )
}

/// Apply VTK's full `vtkWarpLens` distortion equation.
pub fn warp_lens_with_options(input: &PolyData, options: &WarpLensOptions) -> PolyData {
    let mut output = input.clone();
    let n = input.points.len();
    let image_width = options.image_width as f64;
    let image_height = options.image_height as f64;

    if let Some(normals_name) = output
        .point_data()
        .normals()
        .map(|array| array.name().to_string())
    {
        output.point_data_mut().remove_array(&normals_name);
    }

    for i in 0..n {
        let pixel = input.points.get(i);
        let x = pixel[0] / image_width * options.format_width - options.principal_point[0];
        let y = (-pixel[1]) / image_height * options.format_height + options.principal_point[1];

        let r_squared = x * x + y * y;
        let radial = 1.0 + options.k1 * r_squared + options.k2 * r_squared * r_squared;
        let new_x = x * radial + options.p1 * (r_squared + 2.0 * x * x) + 2.0 * options.p2 * x * y;
        let new_y = y * radial + options.p2 * (r_squared + 2.0 * y * y) + 2.0 * options.p1 * x * y;

        output.points.set(
            i,
            [
                (new_x + options.principal_point[0]) / options.format_width * image_width,
                (new_y - options.principal_point[1]) / options.format_height * image_height * -1.0,
                pixel[2],
            ],
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barrel_distortion_moves_outward() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        pd.points.push([1.0, 0.0, 0.0]); // r=1

        let result = warp_lens(&pd, [0.0, 0.0], 0.1);
        let p0 = result.points.get(0);
        let p1 = result.points.get(1);
        // Center should not move
        assert!((p0[0]).abs() < 1e-10);
        // Point at r=1 should move outward with positive k1
        assert!(p1[0] > 1.0);
    }

    #[test]
    fn pincushion_distortion_moves_inward() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 0.0, 0.0]);

        let result = warp_lens(&pd, [0.0, 0.0], -0.1);
        let p = result.points.get(0);
        assert!(p[0] < 1.0);
    }

    #[test]
    fn z_unchanged() {
        let mut pd = PolyData::new();
        pd.points.push([1.0, 1.0, 5.0]);

        let result = warp_lens(&pd, [0.0, 0.0], 0.5);
        let p = result.points.get(0);
        assert_eq!(p[2], 5.0);
    }

    #[test]
    fn vtk_full_lens_equation() {
        let mut pd = PolyData::new();
        pd.points.push([20.0, 30.0, 7.0]);

        let options = WarpLensOptions {
            principal_point: [0.1, -0.2],
            k1: 0.01,
            k2: 0.001,
            p1: 0.02,
            p2: -0.03,
            format_width: 36.0,
            format_height: 24.0,
            image_width: 100,
            image_height: 80,
        };
        let result = warp_lens_with_options(&pd, &options);
        let p = result.points.get(0);

        let x = 20.0 / 100.0 * 36.0 - 0.1;
        let y = -30.0 / 80.0 * 24.0 - 0.2;
        let r2 = x * x + y * y;
        let radial = 1.0 + 0.01 * r2 + 0.001 * r2 * r2;
        let new_x = x * radial + 0.02 * (r2 + 2.0 * x * x) + 2.0 * -0.03 * x * y;
        let new_y = y * radial + -0.03 * (r2 + 2.0 * y * y) + 2.0 * 0.02 * x * y;
        let expected_x = (new_x + 0.1) / 36.0 * 100.0;
        let expected_y = (new_y + 0.2) / 24.0 * 80.0 * -1.0;

        assert!((p[0] - expected_x).abs() < 1e-10);
        assert!((p[1] - expected_y).abs() < 1e-10);
        assert_eq!(p[2], 7.0);
    }
}
