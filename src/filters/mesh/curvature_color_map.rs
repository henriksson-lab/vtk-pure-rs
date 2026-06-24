use crate::data::{AnyDataArray, DataArray, PolyData};

/// Map curvature values to RGB colors using a diverging blue-white-red colormap.
///
/// Reads the scalar array named `array_name` from the input's point data, maps
/// values through a diverging colormap (blue for negative, white for zero, red
/// for positive), and adds a 3-component "CurvatureColor" array to the output's
/// point data.
///
/// Values are normalized symmetrically around zero using the maximum absolute
/// value in the array.
pub fn curvature_to_color(input: &PolyData, array_name: &str) -> PolyData {
    let mut output: PolyData = input.clone();

    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return output,
    };

    let n: usize = arr.num_tuples();
    if n == 0 {
        return output;
    }

    // Find max absolute value for symmetric normalization
    let mut max_abs: f64 = 0.0;
    let mut buf = [0.0f64];
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let a: f64 = buf[0].abs();
        if a > max_abs {
            max_abs = a;
        }
    }

    if max_abs < 1e-20 {
        max_abs = 1.0;
    }

    let mut colors: Vec<f64> = Vec::with_capacity(n * 3);

    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        let t: f64 = (buf[0] / max_abs).clamp(-1.0, 1.0);
        let (r, g, b) = diverging_blue_white_red(t);
        colors.push(r);
        colors.push(g);
        colors.push(b);
    }

    let color_arr = DataArray::from_vec("CurvatureColor", colors, 3);
    output
        .point_data_mut()
        .add_array(AnyDataArray::F64(color_arr));
    output
}

/// Diverging colormap: blue (-1) -> white (0) -> red (+1).
/// Returns (r, g, b) in [0, 1].
fn diverging_blue_white_red(t: f64) -> (f64, f64, f64) {
    if t < 0.0 {
        // Blue to white: t in [-1, 0]
        let s: f64 = 1.0 + t; // s in [0, 1], 0=full blue, 1=white
        let r: f64 = s;
        let g: f64 = s;
        let b: f64 = 1.0;
        (r, g, b)
    } else {
        // White to red: t in [0, 1]
        let r: f64 = 1.0;
        let g: f64 = 1.0 - t;
        let b: f64 = 1.0 - t;
        (r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_triangle_with_curvature() -> PolyData {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let curv = DataArray::from_vec("MeanCurvature", vec![-1.0, 0.0, 1.0], 1);
        pd.point_data_mut().add_array(AnyDataArray::F64(curv));
        pd
    }

    #[test]
    fn adds_color_array() {
        let pd = make_triangle_with_curvature();
        let result = curvature_to_color(&pd, "MeanCurvature");
        let arr = result.point_data().get_array("CurvatureColor").unwrap();
        assert_eq!(arr.num_tuples(), 3);
        assert_eq!(arr.num_components(), 3);
    }

    #[test]
    fn negative_curvature_is_blue() {
        let pd = make_triangle_with_curvature();
        let result = curvature_to_color(&pd, "MeanCurvature");
        let arr = result.point_data().get_array("CurvatureColor").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf); // curvature = -1.0
                                       // Should be blue: r~0, g~0, b~1
        assert!(
            buf[2] > 0.9,
            "Blue channel should be high for negative curvature"
        );
        assert!(
            buf[0] < 0.1,
            "Red channel should be low for negative curvature"
        );
    }

    #[test]
    fn positive_curvature_is_red() {
        let pd = make_triangle_with_curvature();
        let result = curvature_to_color(&pd, "MeanCurvature");
        let arr = result.point_data().get_array("CurvatureColor").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(2, &mut buf); // curvature = +1.0
                                       // Should be red: r~1, g~0, b~0
        assert!(
            buf[0] > 0.9,
            "Red channel should be high for positive curvature"
        );
        assert!(
            buf[1] < 0.1,
            "Green channel should be low for positive curvature"
        );
    }
}
