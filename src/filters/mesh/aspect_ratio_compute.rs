use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute the per-triangle aspect ratio for each polygon cell.
///
/// The aspect ratio is defined as `circumradius / (2 * inradius)`, which equals
/// 1.0 for a perfect equilateral triangle and grows for degenerate triangles.
///
/// Adds an "AspectRatio" scalar array to cell data.
pub fn compute_aspect_ratio(input: &PolyData) -> PolyData {
    let mut ratios: Vec<f64> = Vec::with_capacity(input.polys.num_cells());

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            ratios.push(f64::MAX);
            continue;
        }
        let a_pt = input.points.get(cell[0] as usize);
        let b_pt = input.points.get(cell[1] as usize);
        let c_pt = input.points.get(cell[2] as usize);

        let a_len: f64 = dist(b_pt, c_pt);
        let b_len: f64 = dist(a_pt, c_pt);
        let c_len: f64 = dist(a_pt, b_pt);

        let s: f64 = (a_len + b_len + c_len) * 0.5;
        let area: f64 = (s * (s - a_len) * (s - b_len) * (s - c_len))
            .max(0.0)
            .sqrt();

        if area < 1e-30 {
            ratios.push(f64::MAX);
            continue;
        }

        let circumradius: f64 = (a_len * b_len * c_len) / (4.0 * area);
        let inradius: f64 = area / s;

        let ratio: f64 = circumradius / (2.0 * inradius);
        ratios.push(ratio);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "AspectRatio",
            ratios,
            1,
        )));
    pd
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx: f64 = a[0] - b[0];
    let dy: f64 = a[1] - b[1];
    let dz: f64 = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_equilateral() -> PolyData {
        let h: f64 = (3.0_f64).sqrt() / 2.0;
        PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, h, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn equilateral_aspect_ratio_is_one() {
        let pd = make_equilateral();
        let result = compute_aspect_ratio(&pd);
        let arr = result.cell_data().get_array("AspectRatio").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            (buf[0] - 1.0).abs() < 1e-10,
            "expected ~1.0, got {}",
            buf[0]
        );
    }

    #[test]
    fn degenerate_triangle_large_ratio() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 0.001, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = compute_aspect_ratio(&pd);
        let arr = result.cell_data().get_array("AspectRatio").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert!(
            buf[0] > 10.0,
            "degenerate triangle should have large ratio, got {}",
            buf[0]
        );
    }

    #[test]
    fn multiple_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, (3.0_f64).sqrt() / 2.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = compute_aspect_ratio(&pd);
        let arr = result.cell_data().get_array("AspectRatio").unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }
}
