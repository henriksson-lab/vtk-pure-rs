use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute circumradius for each polygon cell that is exactly a triangle.
///
/// R = (a*b*c)/(4*area). Adds "Circumradius" cell data.
/// Also computes the ratio R/r (circumradius/inradius) as a quality metric.
/// Non-triangle polygon cells receive 0.0, matching the triangle-specific
/// definition instead of deriving a value from the first three vertices only.
pub fn circumradius(input: &PolyData) -> PolyData {
    let mut radii = Vec::new();
    let mut ratios = Vec::new();

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            radii.push(0.0);
            ratios.push(0.0);
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        let v1 = input.points.get(cell[1] as usize);
        let v2 = input.points.get(cell[2] as usize);

        let a = dist(v0, v1);
        let b = dist(v1, v2);
        let c = dist(v2, v0);
        let s = (a + b + c) * 0.5;
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

        let cr = if area > 1e-15 {
            a * b * c / (4.0 * area)
        } else {
            0.0
        };
        let ir = if s > 1e-15 { area / s } else { 0.0 };
        let ratio = if ir > 1e-15 { cr / ir } else { 0.0 };

        radii.push(cr);
        ratios.push(ratio);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Circumradius",
            radii,
            1,
        )));
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "CircumInRatio",
            ratios,
            1,
        )));
    pd
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equilateral_circumradius() {
        let h = (3.0f64).sqrt() / 2.0;
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = circumradius(&pd);
        let arr = result.cell_data().get_array("Circumradius").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        // Equilateral triangle circumradius = a/sqrt(3)
        assert!((buf[0] - 1.0 / 3.0f64.sqrt()).abs() < 0.01);
    }

    #[test]
    fn ratio_equilateral() {
        let h = (3.0f64).sqrt() / 2.0;
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = circumradius(&pd);
        let arr = result.cell_data().get_array("CircumInRatio").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!((buf[0] - 2.0).abs() < 0.01); // equilateral R/r = 2
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = circumradius(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn non_triangle_polygon_gets_zero_triangle_metric() {
        let pd = PolyData::from_polygons(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![vec![0, 1, 2, 3]],
        );
        let result = circumradius(&pd);
        let arr = result.cell_data().get_array("Circumradius").unwrap();
        let mut buf = [1.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 0.0);
    }
}
