use crate::data::{CellArray, Points, PolyData};

/// Extract iso-distance contours on a mesh from geodesic distance field.
///
/// Given a "distance" scalar on the mesh, extracts contour lines at
/// evenly spaced intervals. Returns line segments.
pub fn geodesic_iso_contours(
    input: &PolyData,
    distance_array: &str,
    num_contours: usize,
) -> PolyData {
    let arr = match input.point_data().get_array(distance_array) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let n = input.points.len();
    let mut buf = [0.0f64];
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut max_d = 0.0f64;
    for &v in &values {
        if v >= 0.0 && v < f64::MAX {
            max_d = max_d.max(v);
        }
    }
    if max_d < 1e-15 || num_contours == 0 {
        return PolyData::new();
    }

    let mut out_pts = Points::<f64>::new();
    let mut out_lines = CellArray::new();

    for ci in 1..=num_contours {
        let iso = max_d * ci as f64 / (num_contours + 1) as f64;

        for cell in input.polys.iter() {
            if cell.len() < 3 {
                continue;
            }
            let ids = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
            let sv = [values[ids[0]], values[ids[1]], values[ids[2]]];

            let mut crossings = Vec::new();
            for k in 0..3 {
                let a = ids[k];
                let b = ids[(k + 1) % 3];
                let sa = sv[k];
                let sb = sv[(k + 1) % 3];
                if sa >= 0.0 && sb >= 0.0 && (sa - iso) * (sb - iso) < 0.0 {
                    let t = (iso - sa) / (sb - sa);
                    let pa = input.points.get(a);
                    let pb = input.points.get(b);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn iso_from_gradient() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "d",
                vec![0.0, 2.0, 1.5, 0.5],
                1,
            )));

        let result = geodesic_iso_contours(&pd, "d", 3);
        assert!(result.lines.num_cells() > 0);
    }

    #[test]
    fn no_contours() {
        let pd = PolyData::new();
        let result = geodesic_iso_contours(&pd, "d", 5);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn uniform_no_crossings() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "d",
                vec![1.0, 1.0, 1.0],
                1,
            )));

        let result = geodesic_iso_contours(&pd, "d", 3);
        assert_eq!(result.lines.num_cells(), 0); // all same value
    }
}
