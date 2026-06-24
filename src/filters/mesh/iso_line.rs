use crate::data::{CellArray, DataArray, Points, PolyData};

/// Extract iso-lines (contour lines) from a scalar field on a mesh surface.
///
/// For each triangle, finds edges where the scalar field crosses `iso_value`
/// and produces line segments connecting those crossing points.
/// Returns a PolyData containing line cells representing the iso-contour.
pub fn iso_lines(input: &PolyData, array_name: &str, iso_value: f64) -> PolyData {
    let n: usize = input.points.len();
    let scalars: Vec<f64> = match input.point_data().get_array(array_name) {
        Some(arr) => {
            let mut vals = vec![0.0f64; n];
            let mut buf = [0.0f64];
            for (i, val) in vals.iter_mut().enumerate() {
                arr.tuple_as_f64(i, &mut buf);
                *val = buf[0];
            }
            vals
        }
        None => return PolyData::new(),
    };

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut out_scalars = DataArray::<f64>::new("iso_value", 1);

    for cell in input.polys.iter() {
        let nc: usize = cell.len();
        if nc < 3 {
            continue;
        }

        let mut crossing_pts: Vec<usize> = Vec::new();

        for i in 0..nc {
            let id0: usize = cell[i] as usize;
            let id1: usize = cell[(i + 1) % nc] as usize;
            let s0: f64 = scalars[id0];
            let s1: f64 = scalars[id1];

            let diff0: f64 = s0 - iso_value;
            let diff1: f64 = s1 - iso_value;

            if diff0 * diff1 < 0.0 {
                let t: f64 = (iso_value - s0) / (s1 - s0);
                let p0 = input.points.get(id0);
                let p1 = input.points.get(id1);
                let idx: usize = out_points.len();
                out_points.push([
                    p0[0] + t * (p1[0] - p0[0]),
                    p0[1] + t * (p1[1] - p0[1]),
                    p0[2] + t * (p1[2] - p0[2]),
                ]);
                out_scalars.push_tuple(&[iso_value]);
                crossing_pts.push(idx);
            }
        }

        // For a triangle crossing, we expect exactly 2 crossing points
        if crossing_pts.len() >= 2 {
            for pair in crossing_pts.chunks(2) {
                if pair.len() == 2 {
                    out_lines.push_cell(&[pair[0] as i64, pair[1] as i64]);
                }
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.point_data_mut().add_array(out_scalars.into());
    pd
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn iso_line_on_triangle() {
        // Triangle with scalar values 0, 0, 2 => iso at 1.0 crosses two edges
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("temp", vec![0.0, 0.0, 2.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = iso_lines(&pd, "temp", 1.0);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn iso_line_no_crossing() {
        // All scalar values above the iso-value
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("f", vec![5.0, 6.0, 7.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = iso_lines(&pd, "f", 1.0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn iso_line_missing_array() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = iso_lines(&pd, "nonexistent", 0.5);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
