use crate::data::{AnyDataArray, DataArray, PolyData};

/// Colormap type for scalar-to-color mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMapType {
    Jet,
    Viridis,
    CoolWarm,
}

/// Map a scalar point-data array to RGB colors using a configurable colormap.
///
/// Reads the named array from point data, normalizes values to [0, 1] based on
/// the array's range, then maps through the chosen colormap. The result is a
/// 3-component u8 "Colors" array added to point data.
pub fn scalar_to_color(input: &PolyData, array_name: &str, colormap: ColorMapType) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return input.clone(),
    };

    let n: usize = arr.num_tuples();
    if n == 0 {
        return input.clone();
    }

    // Find scalar range
    let mut min_val: f64 = f64::MAX;
    let mut max_val: f64 = f64::MIN;
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        if buf[0] < min_val {
            min_val = buf[0];
        }
        if buf[0] > max_val {
            max_val = buf[0];
        }
    }

    let range: f64 = max_val - min_val;
    let mut colors: Vec<f64> = Vec::with_capacity(n * 3);

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t: f64 = if range.abs() > 1e-30 {
            (buf[0] - min_val) / range
        } else {
            0.5
        };
        let [r, g, b] = map_color(t, colormap);
        colors.push(r);
        colors.push(g);
        colors.push(b);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    pd
}

/// Map a normalized parameter t in [0, 1] to an RGB color in [0, 1].
fn map_color(t: f64, colormap: ColorMapType) -> [f64; 3] {
    let t: f64 = t.clamp(0.0, 1.0);
    match colormap {
        ColorMapType::Jet => jet(t),
        ColorMapType::Viridis => viridis(t),
        ColorMapType::CoolWarm => cool_warm(t),
    }
}

fn jet(t: f64) -> [f64; 3] {
    // Jet colormap: blue -> cyan -> green -> yellow -> red
    let r: f64 = if t < 0.375 {
        0.0
    } else if t < 0.625 {
        (t - 0.375) / 0.25
    } else if t < 0.875 {
        1.0
    } else {
        1.0 - (t - 0.875) / 0.25
    }
    .clamp(0.0, 1.0);

    let g: f64 = if t < 0.125 {
        0.0
    } else if t < 0.375 {
        (t - 0.125) / 0.25
    } else if t < 0.625 {
        1.0
    } else if t < 0.875 {
        1.0 - (t - 0.625) / 0.25
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    let b: f64 = if t < 0.125 {
        0.5 + t / 0.25
    } else if t < 0.375 {
        1.0
    } else if t < 0.625 {
        1.0 - (t - 0.375) / 0.25
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    [r, g, b]
}

fn viridis(t: f64) -> [f64; 3] {
    // Simplified viridis: dark purple -> blue -> teal -> green -> yellow
    let r: f64 = if t < 0.5 {
        0.267 + t * 0.1
    } else {
        0.267 + 0.05 + (t - 0.5) * 1.4
    }
    .clamp(0.0, 1.0);

    let g: f64 = (0.004 + t * 0.88).clamp(0.0, 1.0);

    let b: f64 = if t < 0.5 {
        0.329 + t * 0.7
    } else {
        0.329 + 0.35 - (t - 0.5) * 1.2
    }
    .clamp(0.0, 1.0);

    [r, g, b]
}

fn cool_warm(t: f64) -> [f64; 3] {
    // Cool (blue) to warm (red) diverging colormap
    let r: f64 = if t < 0.5 {
        0.23 + t * 1.1
    } else {
        0.78 + (t - 0.5) * 0.44
    }
    .clamp(0.0, 1.0);

    let g: f64 = if t < 0.5 {
        0.3 + t * 1.2
    } else {
        0.9 - (t - 0.5) * 1.6
    }
    .clamp(0.0, 1.0);

    let b: f64 = if t < 0.5 {
        0.75 + t * 0.4
    } else {
        0.95 - (t - 0.5) * 1.6
    }
    .clamp(0.0, 1.0);

    [r, g, b]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tri_with_scalars() -> PolyData {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = vec![0.0, 5.0, 10.0];
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "Elevation",
                scalars,
                1,
            )));
        pd
    }

    #[test]
    fn adds_colors_array_jet() {
        let pd = make_tri_with_scalars();
        let result = scalar_to_color(&pd, "Elevation", ColorMapType::Jet);
        let arr = result.point_data().get_array("Colors").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        assert_eq!(arr.num_components(), 3);
    }

    #[test]
    fn colors_vary_across_range() {
        let pd = make_tri_with_scalars();
        let result = scalar_to_color(&pd, "Elevation", ColorMapType::Viridis);
        let arr = result.point_data().get_array("Colors").unwrap();
        let mut c0 = [0.0f64; 3];
        let mut c2 = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut c0);
        arr.tuple_as_f64(2, &mut c2);
        // First and last colors should differ
        let diff: f64 = (c0[0] - c2[0]).abs() + (c0[1] - c2[1]).abs() + (c0[2] - c2[2]).abs();
        assert!(diff > 0.1, "colors should differ, diff = {}", diff);
    }

    #[test]
    fn missing_array_returns_clone() {
        let pd = make_tri_with_scalars();
        let result = scalar_to_color(&pd, "NonExistent", ColorMapType::CoolWarm);
        assert!(result.point_data().get_array("Colors").is_none());
        assert!(result.point_data().get_array("Elevation").is_some());
    }
}
