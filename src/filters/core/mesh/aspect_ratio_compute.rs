use crate::data::{AnyDataArray, DataArray, PolyData};
use crate::filters::core::mesh::aspect_ratio::triangle_aspect_ratio;

/// Compute the per-triangle aspect ratio for each polygon cell.
///
/// The aspect ratio follows VTK/Verdict `tri_aspect_ratio`, which equals 1.0
/// for a perfect equilateral triangle and grows for degenerate triangles.
///
/// Adds an "AspectRatio" scalar array to cell data.
pub fn compute_aspect_ratio(input: &PolyData) -> PolyData {
    let mut ratios: Vec<f64> = Vec::with_capacity(input.polys.num_cells());

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            ratios.push(0.0);
            continue;
        }
        let a_pt = input.points.get(cell[0] as usize);
        let b_pt = input.points.get(cell[1] as usize);
        let c_pt = input.points.get(cell[2] as usize);

        ratios.push(triangle_aspect_ratio(a_pt, b_pt, c_pt));
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
