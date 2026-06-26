//! Compute spherical harmonics coefficients from an equirectangular RGB image.

use crate::data::{AnyDataArray, DataArray, ImageData, Table};
use crate::types::ScalarType;

/// Compute third-degree spherical harmonics coefficients for a 2D RGB/RGBA image.
///
/// This mirrors `vtkSphericalHarmonics`: the input must be a 2D image with an
/// active point scalar array containing either 3 or 4 components. Alpha, when
/// present, is ignored. The output table has one 9-component column named
/// `"SphericalHarmonics"` and three rows, one row per RGB channel.
pub fn spherical_harmonics(input: &ImageData) -> Result<Table, String> {
    let dimensions = input.dimensions();
    let width = dimensions[0];
    let height = dimensions[1];

    let scalars = input
        .point_data()
        .scalars()
        .ok_or_else(|| "No scalars found in image point data.".to_string())?;

    let num_components = scalars.num_components();
    if (num_components != 3 && num_components != 4) || dimensions[2] > 1 {
        return Err("Only 2D images with RGB or RGBA attributes are supported.".to_string());
    }

    let mut harmonics = [[0.0f64; 9]; 3];
    if width == 0 || height == 0 {
        return Ok(output_table(harmonics));
    }

    let solid_angle = 2.0 * std::f64::consts::PI * std::f64::consts::PI / ((width * height) as f64);
    let mut weight_sum = 0.0f64;
    let mut tuple = vec![0.0f64; num_components];

    for i in 0..height {
        let theta = ((i as f64 + 0.5) / height as f64) * std::f64::consts::PI;
        let ct = theta.cos();
        let st = theta.sin();
        let weight = solid_angle * st;

        for j in 0..width {
            let phi = (((j as f64 + 0.5) / width as f64) * 2.0 - 1.0) * std::f64::consts::PI;
            let cp = phi.cos();
            let sp = phi.sin();

            // VTK/OpenGL coordinates: Y up, so rotate the equirectangular normal.
            let n = [st * cp, -ct, st * sp];
            let basis = [
                0.282095,
                -0.488603 * n[1],
                0.488603 * n[2],
                -0.488603 * n[0],
                1.092548 * n[0] * n[1],
                -1.092548 * n[1] * n[2],
                0.315392 * (3.0 * n[2] * n[2] - 1.0),
                -1.092548 * n[0] * n[2],
                0.546274 * (n[0] * n[0] - n[1] * n[1]),
            ];

            weight_sum += weight;
            scalars.tuple_as_f64(i * width + j, &mut tuple);

            for component in 0..3 {
                let v = normalize_component(tuple[component], scalars.scalar_type());
                for y in 0..9 {
                    harmonics[component][y] += weight * v * basis[y];
                }
            }
        }
    }

    if weight_sum > 0.0 {
        let normalize_factor = 4.0 * std::f64::consts::PI / weight_sum;
        for component in &mut harmonics {
            for coefficient in component {
                *coefficient *= normalize_factor;
            }
        }
    }

    Ok(output_table(harmonics))
}

fn normalize_component(v: f64, scalar_type: ScalarType) -> f64 {
    match scalar_type {
        ScalarType::F32 | ScalarType::F64 => v,
        ScalarType::U8 => (v / u8::MAX as f64).powf(2.2),
        ScalarType::I8 => v / i8::MAX as f64,
        ScalarType::I16 => v / i16::MAX as f64,
        ScalarType::I32 => v / i32::MAX as f64,
        ScalarType::I64 => v / i64::MAX as f64,
        ScalarType::U16 => v / u16::MAX as f64,
        ScalarType::U32 => v / u32::MAX as f64,
        ScalarType::U64 => v / u64::MAX as f64,
    }
}

fn output_table(harmonics: [[f64; 9]; 3]) -> Table {
    let mut values = Vec::with_capacity(27);
    for component in harmonics {
        values.extend_from_slice(&component);
    }

    Table::new().with_column(AnyDataArray::F64(DataArray::from_vec(
        "SphericalHarmonics",
        values,
        9,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_white_image_has_expected_dc_coefficient() {
        let mut image = ImageData::with_dimensions(4, 2, 1);
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0; 4 * 2 * 3],
                3,
            )));
        image.point_data_mut().set_active_scalars("rgb");

        let table = spherical_harmonics(&image).unwrap();
        let harmonics = table.column_by_name("SphericalHarmonics").unwrap();

        assert_eq!(table.num_rows(), 3);
        assert_eq!(harmonics.num_components(), 9);

        let mut row = [0.0f64; 9];
        harmonics.tuple_as_f64(0, &mut row);
        assert!((row[0] - 0.282095 * 4.0 * std::f64::consts::PI).abs() < 1e-10);
        for coefficient in row.iter().take(4).skip(1) {
            assert!(coefficient.abs() < 1e-10);
        }
    }

    #[test]
    fn rejects_polyline_volume_or_non_rgb_input() {
        let mut image = ImageData::with_dimensions(2, 2, 2);
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "rgb",
                vec![1.0; 2 * 2 * 2 * 3],
                3,
            )));
        image.point_data_mut().set_active_scalars("rgb");
        assert!(spherical_harmonics(&image).is_err());

        let mut image = ImageData::with_dimensions(2, 2, 1);
        image
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "gray",
                vec![1.0; 2 * 2],
                1,
            )));
        image.point_data_mut().set_active_scalars("gray");
        assert!(spherical_harmonics(&image).is_err());
    }

    #[test]
    fn unsigned_char_input_is_normalized_like_vtk() {
        let mut image = ImageData::with_dimensions(2, 2, 1);
        image
            .point_data_mut()
            .add_array(AnyDataArray::U8(DataArray::from_vec(
                "rgb",
                vec![255; 2 * 2 * 3],
                3,
            )));
        image.point_data_mut().set_active_scalars("rgb");

        let table = spherical_harmonics(&image).unwrap();
        let harmonics = table.column_by_name("SphericalHarmonics").unwrap();

        let mut row = [0.0f64; 9];
        harmonics.tuple_as_f64(0, &mut row);
        assert!((row[0] - 0.282095 * 4.0 * std::f64::consts::PI).abs() < 1e-10);
    }
}
