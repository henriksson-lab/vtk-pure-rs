use crate::data::{AnyDataArray, DataArray, PolyData};

/// Assign vertex colors based on vertex position within the bounding box.
///
/// Maps X to red, Y to green, Z to blue, normalized to [0, 1] within the
/// mesh bounding box. Adds a 3-component "PositionColor" array to point data.
pub fn color_from_position(input: &PolyData) -> PolyData {
    let n: usize = input.points.len();
    if n == 0 {
        return input.clone();
    }

    // Compute bounding box
    let mut min_x: f64 = f64::MAX;
    let mut max_x: f64 = f64::MIN;
    let mut min_y: f64 = f64::MAX;
    let mut max_y: f64 = f64::MIN;
    let mut min_z: f64 = f64::MAX;
    let mut max_z: f64 = f64::MIN;

    for i in 0..n {
        let p = input.points.get(i);
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
        min_z = min_z.min(p[2]);
        max_z = max_z.max(p[2]);
    }

    let range_x: f64 = max_x - min_x;
    let range_y: f64 = max_y - min_y;
    let range_z: f64 = max_z - min_z;

    let mut colors: Vec<f64> = Vec::with_capacity(n * 3);
    for i in 0..n {
        let p = input.points.get(i);
        let r: f64 = if range_x > 1e-20 {
            (p[0] - min_x) / range_x
        } else {
            0.5
        };
        let g: f64 = if range_y > 1e-20 {
            (p[1] - min_y) / range_y
        } else {
            0.5
        };
        let b: f64 = if range_z > 1e-20 {
            (p[2] - min_z) / range_z
        } else {
            0.5
        };
        colors.push(r);
        colors.push(g);
        colors.push(b);
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "PositionColor",
            colors,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_cube_corners() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let result = color_from_position(&pd);
        let arr = result.point_data().get_array("PositionColor").unwrap();
        assert_eq!(arr.num_tuples(), 4);
        assert_eq!(arr.num_components(), 3);

        // Point 0 at origin should be (0, 0, 0)
        let mut val = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut val);
        assert!(val[0].abs() < 1e-10);
        assert!(val[1].abs() < 1e-10);
        assert!(val[2].abs() < 1e-10);

        // Point 1 at (1,0,0) should be (1, 0, 0)
        arr.tuple_as_f64(1, &mut val);
        assert!((val[0] - 1.0).abs() < 1e-10);
        assert!(val[1].abs() < 1e-10);
    }

    #[test]
    fn colors_in_range() {
        let pd = PolyData::from_triangles(
            vec![[-5.0, 10.0, 0.0], [5.0, -10.0, 0.0], [0.0, 0.0, 3.0]],
            vec![[0, 1, 2]],
        );
        let result = color_from_position(&pd);
        let arr = result.point_data().get_array("PositionColor").unwrap();
        let mut val = [0.0f64; 3];
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut val);
            for c in &val {
                assert!(
                    *c >= 0.0 - 1e-10 && *c <= 1.0 + 1e-10,
                    "color out of range: {}",
                    c
                );
            }
        }
    }

    #[test]
    fn empty_mesh() {
        let pd = PolyData::new();
        let result = color_from_position(&pd);
        assert!(result.point_data().get_array("PositionColor").is_none());
    }
}
