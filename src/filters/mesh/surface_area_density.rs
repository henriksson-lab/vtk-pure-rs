use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute surface area density: area per vertex (Voronoi area).
///
/// Each vertex gets 1/3 of the area of each adjacent triangle.
/// Adds "AreaDensity" scalar. Useful for adaptive sampling.
pub fn surface_area_density(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut area = vec![0.0f64; n];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let v0 = input.points.get(cell[0] as usize);
        for i in 1..cell.len() - 1 {
            let v1 = input.points.get(cell[i] as usize);
            let v2 = input.points.get(cell[i + 1] as usize);
            let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
            let cx = e1[1] * e2[2] - e1[2] * e2[1];
            let cy = e1[2] * e2[0] - e1[0] * e2[2];
            let cz = e1[0] * e2[1] - e1[1] * e2[0];
            let a = 0.5 * (cx * cx + cy * cy + cz * cz).sqrt() / 3.0; // 1/3 per vertex
            area[cell[0] as usize] += a;
            area[cell[i] as usize] += a;
            area[cell[i + 1] as usize] += a;
        }
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AreaDensity",
            area,
            1,
        )));
    pd
}

/// Compute the total surface area and area per vertex statistics.
pub fn area_statistics(input: &PolyData) -> (f64, f64, f64) {
    // (total, mean_per_vertex, std_per_vertex)
    let result = surface_area_density(input);
    let arr = match result.point_data().get_array("AreaDensity") {
        Some(a) => a,
        None => return (0.0, 0.0, 0.0),
    };
    let n = arr.num_tuples();
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let mut buf = [0.0f64];
    let mut total = 0.0;
    let mut sum2 = 0.0;
    for i in 0..n {
        arr.tuple_as_f64(i, &mut buf);
        total += buf[0];
        sum2 += buf[0] * buf[0];
    }
    let mean = total / n as f64;
    let var = (sum2 / n as f64 - mean * mean).max(0.0);
    (total, mean, var.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_sums_to_area() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = surface_area_density(&pd);
        let arr = result.point_data().get_array("AreaDensity").unwrap();
        let mut buf = [0.0f64];
        let mut total = 0.0;
        for i in 0..3 {
            arr.tuple_as_f64(i, &mut buf);
            total += buf[0];
        }
        assert!((total - 0.5).abs() < 1e-10); // area of unit right triangle
    }

    #[test]
    fn stats_basic() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let (total, mean, _) = area_statistics(&pd);
        assert!((total - 0.5).abs() < 1e-10);
        assert!(mean > 0.0);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let (total, _, _) = area_statistics(&pd);
        assert_eq!(total, 0.0);
    }
}
