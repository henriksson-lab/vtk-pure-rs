use crate::data::{CellArray, Points, PolyData};

/// Extract multiple isocontours from a scalar field on a mesh.
///
/// Like `mesh_level_set` but for multiple values at once.
/// Returns a PolyData with line segments for all contours.
pub fn multi_contour_on_mesh(input: &PolyData, array_name: &str, values: &[f64]) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };
    let n = input.points.len();
    let mut buf = [0.0f64];
    let scalars: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut out_pts = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for &iso in values {
        for cell in input.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            let sv = [scalars[ids[0]], scalars[ids[1]], scalars[ids[2]]];

            let mut crossings = Vec::new();
            for k in 0..3 {
                let sa = sv[k];
                let sb = sv[(k + 1) % 3];
                if (sa - iso) * (sb - iso) < 0.0 {
                    let t = (iso - sa) / (sb - sa);
                    let pa = input.points.get(ids[k]);
                    let pb = input.points.get(ids[(k + 1) % 3]);
                    crossings.push([
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ]);
                }
            }

            if crossings.len() == 2 {
                let i0 = out_pts.len() as i64;
                out_pts.push(crossings[0]);
                out_pts.push(crossings[1]);
                out_lines.push_cell(&[i0, i0 + 1]);
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.lines = out_lines;
    pd
}

/// Extract a single contour and compute its total length.
pub fn contour_length(input: &PolyData, array_name: &str, isovalue: f64) -> f64 {
    let contour = multi_contour_on_mesh(input, array_name, &[isovalue]);
    let mut total = 0.0;
    for cell in contour.lines.iter() {
        if cell.len() >= 2 {
            let a = contour.points.get(cell[0] as usize);
            let b = contour.points.get(cell[1] as usize);
            total += ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn multi_contour() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 2.0, 1.5, 0.5],
                1,
            )));

        let result = multi_contour_on_mesh(&pd, "f", &[0.5, 1.0, 1.5]);
        assert!(result.lines.num_cells() > 0);
    }

    #[test]
    fn contour_length_test() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 3.0, 0.0],
                1,
            )));

        let len = contour_length(&pd, "f", 1.0);
        assert!(len > 0.0, "contour length={}", len);
    }

    #[test]
    fn no_crossing() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec("f", vec![5.0; 3], 1)));

        assert_eq!(contour_length(&pd, "f", 0.0), 0.0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        assert_eq!(
            multi_contour_on_mesh(&pd, "nope", &[1.0]).lines.num_cells(),
            0
        );
    }
}
