use crate::data::{Points, PolyData};

/// Translate all points of a PolyData by a constant offset vector.
pub fn translate(input: &PolyData, offset: [f64; 3]) -> PolyData {
    let mut out_points = Points::<f64>::new();
    for i in 0..input.points.len() {
        let p = input.points.get(i);
        out_points.push([p[0] + offset[0], p[1] + offset[1], p[2] + offset[2]]);
    }

    let mut pd = input.clone();
    pd.points = out_points;
    pd
}

/// Translate a PolyData so that its bounding-box center sits at the origin.
pub fn translate_to_origin(input: &PolyData) -> PolyData {
    if input.points.len() == 0 {
        return input.clone();
    }

    let mut min_x: f64 = f64::MAX;
    let mut min_y: f64 = f64::MAX;
    let mut min_z: f64 = f64::MAX;
    let mut max_x: f64 = f64::NEG_INFINITY;
    let mut max_y: f64 = f64::NEG_INFINITY;
    let mut max_z: f64 = f64::NEG_INFINITY;

    for i in 0..input.points.len() {
        let p = input.points.get(i);
        if p[0] < min_x {
            min_x = p[0];
        }
        if p[1] < min_y {
            min_y = p[1];
        }
        if p[2] < min_z {
            min_z = p[2];
        }
        if p[0] > max_x {
            max_x = p[0];
        }
        if p[1] > max_y {
            max_y = p[1];
        }
        if p[2] > max_z {
            max_z = p[2];
        }
    }

    let cx: f64 = (min_x + max_x) * 0.5;
    let cy: f64 = (min_y + max_y) * 0.5;
    let cz: f64 = (min_z + max_z) * 0.5;

    translate(input, [-cx, -cy, -cz])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn translate_moves_all_points() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = translate(&pd, [10.0, 20.0, 30.0]);
        assert_eq!(result.points.len(), 3);
        let p0 = result.points.get(0);
        assert!((p0[0] - 10.0).abs() < 1e-10);
        assert!((p0[1] - 20.0).abs() < 1e-10);
        assert!((p0[2] - 30.0).abs() < 1e-10);
    }

    #[test]
    fn translate_preserves_cells() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = translate(&pd, [5.0, 5.0, 5.0]);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn translate_to_origin_centers_mesh() {
        let pd = PolyData::from_triangles(
            vec![[2.0, 4.0, 6.0], [4.0, 4.0, 6.0], [2.0, 6.0, 8.0]],
            vec![[0, 1, 2]],
        );
        let result = translate_to_origin(&pd);
        // Bounding box center was (3,5,7), so now center should be at origin
        let mut min_x: f64 = f64::MAX;
        let mut max_x: f64 = f64::MIN;
        let mut min_y: f64 = f64::MAX;
        let mut max_y: f64 = f64::MIN;
        for i in 0..result.points.len() {
            let p = result.points.get(i);
            if p[0] < min_x {
                min_x = p[0];
            }
            if p[0] > max_x {
                max_x = p[0];
            }
            if p[1] < min_y {
                min_y = p[1];
            }
            if p[1] > max_y {
                max_y = p[1];
            }
        }
        let cx: f64 = (min_x + max_x) * 0.5;
        let cy: f64 = (min_y + max_y) * 0.5;
        assert!(cx.abs() < 1e-10);
        assert!(cy.abs() < 1e-10);
    }

    #[test]
    fn translate_preserves_data_arrays() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "point_id",
                vec![0.0, 1.0, 2.0],
                1,
            )));
        pd.cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "cell_id",
                vec![7.0],
                1,
            )));

        let result = translate(&pd, [1.0, 2.0, 3.0]);

        assert!(result.point_data().get_array("point_id").is_some());
        assert!(result.cell_data().get_array("cell_id").is_some());
    }

    #[test]
    fn translate_to_origin_handles_all_negative_bounds() {
        let pd = PolyData::from_points(vec![[-5.0, -4.0, -3.0], [-3.0, -2.0, -1.0]]);
        let result = translate_to_origin(&pd);

        assert_eq!(result.points.get(0), [-1.0, -1.0, -1.0]);
        assert_eq!(result.points.get(1), [1.0, 1.0, 1.0]);
    }
}
