use crate::data::{AnyDataArray, DataArray, PolyData};

/// Assign a solid RGB color to all points as a "Colors" array.
pub fn solid_color(input: &PolyData, r: f64, g: f64, b: f64) -> PolyData {
    let n = input.points.len();
    let mut colors = Vec::with_capacity(n * 3);
    for _ in 0..n {
        colors.push(r);
        colors.push(g);
        colors.push(b);
    }
    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    pd
}

/// Color points by height (Z coordinate) using a simple blue-to-red gradient.
pub fn color_by_height(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;
    for i in 0..n {
        let z = input.points.get(i)[2];
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }
    let range = (max_z - min_z).max(1e-15);

    let mut colors = Vec::with_capacity(n * 3);
    for i in 0..n {
        let t = (input.points.get(i)[2] - min_z) / range;
        colors.push(t); // R: low=0, high=1
        colors.push(0.2); // G: constant
        colors.push(1.0 - t); // B: low=1, high=0
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec("Colors", colors, 3)));
    pd
}

/// Color each cell with a distinct color (cycling through a palette).
pub fn color_by_cell_index(input: &PolyData) -> PolyData {
    let palette: Vec<[f64; 3]> = vec![
        [0.894, 0.102, 0.110],
        [0.216, 0.494, 0.722],
        [0.302, 0.686, 0.290],
        [0.596, 0.306, 0.639],
        [1.000, 0.498, 0.000],
        [1.000, 1.000, 0.200],
        [0.651, 0.337, 0.157],
        [0.969, 0.506, 0.749],
        [0.600, 0.600, 0.600],
    ];

    let n_cells = input.polys.num_cells();
    let mut cell_colors = Vec::with_capacity(n_cells * 3);
    for ci in 0..n_cells {
        let c = &palette[ci % palette.len()];
        cell_colors.push(c[0]);
        cell_colors.push(c[1]);
        cell_colors.push(c[2]);
    }

    let mut pd = input.clone();
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "Colors",
            cell_colors,
            3,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_red() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);

        let result = solid_color(&pd, 1.0, 0.0, 0.0);
        let arr = result.point_data().get_array("Colors").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn height_coloring() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // low
        pd.points.push([0.0, 0.0, 10.0]); // high

        let result = color_by_height(&pd);
        let arr = result.point_data().get_array("Colors").unwrap();
        let mut buf = [0.0f64; 3];
        arr.tuple_as_f64(0, &mut buf);
        assert!(buf[2] > 0.9); // blue at bottom
        arr.tuple_as_f64(1, &mut buf);
        assert!(buf[0] > 0.9); // red at top
    }

    #[test]
    fn cell_colors() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);

        let result = color_by_cell_index(&pd);
        let arr = result.cell_data().get_array("Colors").unwrap();
        assert_eq!(arr.num_tuples(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = color_by_height(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
